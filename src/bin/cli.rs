use std::path::PathBuf;

use schnauzer_ui::{
    datatable::{preprocess, read_csv},
    interpreter::Interpreter,
    parser::Stmt,
    scanner::Scanner,
    test_report::SuiReport,
    webdriver::{new_driver, SupportedBrowser, WebDriverConfig},
};

use anyhow::{bail, Context, Result};
use camino::Utf8PathBuf;
use clap::Parser;
use promptly::{prompt, prompt_default};

/// SchnauzerUI is a DSL for automated web UI testing.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to a SchnauzerUI .sui file to run
    #[arg(short = 'f', long)]
    input_filepath: Option<Utf8PathBuf>,

    /// Path to an excel file which holds variable values for test runs
    #[arg(short = 'x', long)]
    datatable: Option<Utf8PathBuf>,

    /// When --filepath or -f passed, path to a directory for logs and screenshots.
    /// When in repl mode, path to the directory where the script will be saved.
    #[arg(short, long, default_value_t = Utf8PathBuf::from("."))]
    output_directory: Utf8PathBuf,

    /// Whether or not to display the browsers while the tests are running.
    #[arg(short = 'z', long)]
    headless: bool,

    /// Specify the browser to run the tests on.
    #[arg(short, long, default_value_t = SupportedBrowser::Firefox)]
    browser: SupportedBrowser,

    /// Highlight elements which are located to more clearly demonstrate process
    #[arg(long, short)]
    demo: bool,

    /// The port your webdriver compliant process is running on
    #[arg(long, short, default_value_t = 4444)]
    port: usize,
}

fn main() {
    // Parse cli options
    let cli = Cli::parse();

    // This is equivalent to #[tokio::main], but we dont
    // want to enter async land until we've called
    // `Cli::parse()` on our clap cli input.
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            if let Err(e) = start(cli).await {
                eprintln!("{}", e);
            }
        });
}

/// Entrypoint for the program after CLI args are parsed.
async fn start(
    Cli {
        input_filepath,
        datatable,
        output_directory,
        headless,
        browser,
        demo,
        port,
    }: Cli,
) -> Result<()> {
    // Create the provided output directory.
    if std::fs::create_dir_all(&output_directory).is_err() {
        bail!("Could not create output directory: {}", output_directory);
    }

    // Combine webdriver related arguments into a config object
    let driver_config = WebDriverConfig {
        port,
        headless,
        browser,
    };

    // Delegate based on provided cli arguments
    match input_filepath {
        // They provided a filepath, so verify it's a file and just run the given file
        Some(filepath) => {
            if !filepath.is_file() {
                bail!(
                    "Usage: filepath flag must be a file, but received {}",
                    filepath
                );
            }

            FileRunner {
                input_filepath: filepath,
                datatable,
                output_directory,
                driver_config,
                demo,
            }
            .run()
            .await?;
        }

        // They did not provide a filepath, so run in REPL mode
        None => {
            ReplRunner::new(output_directory, driver_config, demo)
                .await?
                .run()
                .await?;
        }
    }

    Ok(())
}

struct FileRunner {
    input_filepath: Utf8PathBuf,
    datatable: Option<Utf8PathBuf>,
    output_directory: Utf8PathBuf,
    driver_config: WebDriverConfig,
    demo: bool,
}

impl FileRunner {
    pub async fn run(self) -> Result<()> {
        let tokens = Scanner::from_src(self.process_input_file()?).scan();
        let stmts = schnauzer_ui::parser::Parser::new().parse(tokens);
        let interpreter = Interpreter::new(
            new_driver(self.driver_config).await?,
            stmts,
            self.demo,
            SuiReport::new(self.get_filename_for_report()?, self.output_directory),
        );
        interpreter.interpret(true).await?.write_report()
    }

    fn process_input_file(&self) -> Result<String> {
        let sui_code = self.read_input_file()?;
        self.expand_datatable_into_script(sui_code)
    }

    fn read_input_file(&self) -> Result<String> {
        std::fs::read_to_string(&self.input_filepath)
            .with_context(|| format!("Errored reading file {}", self.input_filepath))
    }

    fn get_filename_for_report(&self) -> Result<String> {
        self.input_filepath
            .file_stem()
            .map(|filename| filename.into())
            .with_context(|| "Could not get file name")
    }

    fn expand_datatable_into_script(&self, sui_code: String) -> Result<String> {
        if let Some(ref dt_path) = self.datatable {
            let dt = read_csv(dt_path)?;
            Ok(preprocess(sui_code, dt))
        } else {
            Ok(sui_code)
        }
    }
}

struct ReplRunner {
    output_filepath: Utf8PathBuf,
    script_buffer: String,
    interpreter: Interpreter,
}

impl ReplRunner {
    pub async fn new(
        output_filepath: Utf8PathBuf,
        driver_config: WebDriverConfig,
        is_demo: bool,
    ) -> Result<Self> {
        let driver = new_driver(driver_config).await?;
        Ok(Self {
            // Passed in
            output_filepath,

            // Initializers
            script_buffer: String::new(),
            interpreter: Interpreter::new(driver, vec![], is_demo, SuiReport::non_writeable()),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        if let Err(e) = self.run_loop().await {
            bail!("REPL encountered an error: {}", e);
        }
        Ok(())
    }

    pub async fn run_loop(&mut self) -> Result<()> {
        let script_name = Self::prompt_for_script_name()?;

        // Handle starting from an existing sui script
        if let Some(start_path) = Self::prompt_for_start_script()? {
            let code = std::fs::read_to_string(start_path)
                .with_context(|| "Error reading in start file code")?;
            let tokens = Scanner::from_src(code).scan();
            let stmts = schnauzer_ui::parser::Parser::new().parse(tokens);
            self.execute_starting_script(stmts).await?;
        }

        loop {
            let code = Self::prompt_for_statement()?;
            // Check if the user wants to exit
            if code == "exit" {
                self.interpreter
                    .driver
                    .close_window()
                    .await
                    .with_context(|| "Error closing browser window")?;
                break;
            }
            let tokens = Scanner::from_src(code).scan();
            let stmts = schnauzer_ui::parser::Parser::new().parse(tokens);
            for stmt in stmts.iter() {
                if let Err(e) = self.interpreter.execute_stmt(stmt.clone()).await {
                    eprintln!("The statement {} resulted in an error: {}", stmt, e.0);
                }
                if Self::prompt_save_statement()? {
                    self.push_statement_to_script_buffer(stmt);
                }
            }
        }

        if Self::prompt_to_save_the_script()? {
            self.write_script_to_file(script_name)?;
        }

        Ok(())
    }

    fn prompt_for_statement() -> Result<String> {
        prompt("sui_command or \"exit\"").context("Error reading in line")
    }

    fn prompt_save_statement() -> Result<bool> {
        prompt_default("Save this statement?", true).context("Error reading in line")
    }

    fn prompt_to_save_the_script() -> Result<bool> {
        prompt_default("Save this test run as a SchnauzerUI script?", true)
            .context("Error saving the test run as a SchnauzerUI script.")
    }

    async fn execute_starting_script(&mut self, statements: Vec<Stmt>) -> Result<()> {
        for stmt in statements.into_iter() {
            self.push_statement_to_script_buffer(&stmt);
            self.execute_startup_statement(stmt).await?;
        }
        Ok(())
    }

    fn write_script_to_file(&self, script_name: String) -> Result<()> {
        std::fs::write(
            self.output_filepath
                .with_file_name(&script_name)
                .with_extension("sui"),
            self.script_buffer.clone(),
        )
        .context("Error writing script to output file")
    }

    async fn execute_startup_statement(&mut self, stmt: Stmt) -> Result<()> {
        if let Err(e) = self.interpreter.execute_stmt(stmt).await {
            bail!(
                "Warning: Error encountered while running start script: {}",
                e.0
            );
        }
        Ok(())
    }

    fn push_statement_to_script_buffer(&mut self, stmt: &Stmt) {
        if let Stmt::Comment(_) = stmt {
            self.script_buffer.push('\n');
        }
        self.script_buffer.push_str(&format!("{}", stmt));
        self.script_buffer.push('\n');
    }

    fn prompt_for_script_name() -> Result<String> {
        prompt_default("What is the name of this test?", "test".to_owned())
            .with_context(|| "Error reading script name")
    }

    fn prompt_for_start_script() -> Result<Option<PathBuf>> {
        let use_start_script: bool =
            prompt("Do you want to start from an existing script".to_owned())
                .with_context(|| "Error prompting start script")?;

        if use_start_script {
            let start_script = prompt("Please provide the path to the script".to_owned())
                .with_context(|| "Error reading in file")?;
            Ok(Some(start_script))
        } else {
            Ok(None)
        }
    }
}
