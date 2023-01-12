use std::{collections::HashMap, path::PathBuf};

use clap::{ArgGroup, Parser};
use promptly::{prompt, prompt_default};

use schnauzer_ui::{
    datatable::read_csv, install_drivers, interpreter::Interpreter, new_driver, parser::Stmt, run,
    scanner::Scanner, with_drivers_running, SupportedBrowser, WebDriverConfig,
};

/// SchnauzerUI is a DSL for automated web UI testing.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(
    ArgGroup::new("script_path")
        .required(true)
        .args(["input_filepath", "repl"])))]
struct Cli {
    /// Path to a SchnauzerUI .sui file to run
    #[arg(short = 'f', long)]
    input_filepath: Option<PathBuf>,

    /// Run SchnauzerUI in a REPL.
    #[arg(short = 'i', long)]
    repl: bool,

    /// When --filepath passed, path to a directory for logs and screenshots.
    /// When --repl passed, path to a directory for the script to record the repl interactions.
    #[arg(short, long)]
    output_dir: Option<PathBuf>,

    /// Whether or not to display the browsers while the tests are running.
    #[arg(short = 'z', long)]
    headless: bool,

    /// Which browser to use. Supports "firefox" or "chrome".
    /// Defaults to chrome.
    #[arg(short, long, default_value_t = String::from("chrome"))]
    browser: String,

    /// Path to an excel file which holds variable values for test runs
    #[arg(short = 'x', long)]
    datatable: Option<PathBuf>,

    /// Highlight elements which are located to more clearly demonstrate process
    #[arg(long)]
    demo: bool,

    /// "Bring Your Own Drivers". Turns off driver management so you can run against external processes.
    #[arg(long)]
    byod: bool,

    #[arg(long)]
    override_port: Option<usize>,
}

fn main() {
    // Parse cli options
    let cli = Cli::parse();

    if cli.byod {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                start(cli).await;
            });
    } else {
        install_drivers();
        with_drivers_running(|| {
            // This sleep is here because the geckodriver and chromedriver actually
            // take a moment to fully register after starting up.
            // The release build is so fast the webdriver cant start because the drivers
            // aren't initialized. This gives them 5 seconds to start up.
            std::thread::sleep(std::time::Duration::from_secs(5));

            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    start(cli).await;
                });
        });
    }
}

async fn start(
    Cli {
        input_filepath,
        repl,
        mut output_dir,
        headless,
        browser,
        datatable,
        demo,
        byod: _,
        override_port,
    }: Cli,
) {
    // Resolve browser to a supported browser
    let browser = match browser.as_str() {
        "chrome" => SupportedBrowser::Chrome,
        "firefox" => SupportedBrowser::FireFox,
        _ => {
            eprintln!(
                "Unsupported browser: {}, currently SchnauzerUI supports 'firefox' and 'chrome'",
                browser
            );
            return;
        }
    };

    let port = override_port.unwrap_or_else(|| match browser {
        SupportedBrowser::FireFox => 4444,
        SupportedBrowser::Chrome => 9515,
    });

    // Verify that the passed --output-dir could be a directory (a '.' would indicate a file instead)
    if let Some(ref mut output) = output_dir {
        if looks_like_file(output) {
            eprintln!(
                "Usage: output_dir flag must be a directory, but received {}",
                output.display()
            );
            return;
        }
        // If it can be a directory, create it along with any required parent folders.
        std::fs::create_dir_all(output.clone())
            .expect(&format!("Could not create directory: {}", output.display()));
    }

    // If a path to a datatable was provided, read in the datatable as a csv.
    let dt = datatable.map(|path| read_csv(path));

    // Combine webdriver related arguments into a config object
    let driver_config = WebDriverConfig {
        port,
        headless,
        browser,
    };

    // Delegate based on provided cli arguments
    match (input_filepath, repl) {
        // They provided a filepath, so verify it's a file and just run the given file
        (Some(filepath), false) => {
            if !filepath.is_file() {
                eprintln!(
                    "Usage: filepath flag must be a file, but received {}",
                    filepath.display()
                );
                return;
            }

            // the output directory should default to the directory of the input file
            let output = output_dir
                .or(filepath.parent().map(|f| f.to_path_buf()))
                .unwrap_or(".".into());
            run_file(filepath, output, driver_config, dt, demo).await
        }

        // They provided the repl flag, so run in repl mode.
        // The output directory should default to the current directory.
        (None, true) => {
            if let Err(e) = repl_loop(output_dir.unwrap_or(".".into()), driver_config, demo).await {
                eprintln!("REPL encountered an error: {}", e);
            }
        }

        // This represents an unreachable combination of cli arguments.
        _ => unreachable!(),
    }
}

/// Reads in the contents of the input file and runs it as scnahuzer ui code.
async fn run_file(
    input_filepath: PathBuf,
    output_filepath: PathBuf,
    driver_config: WebDriverConfig,
    dt: Option<Vec<HashMap<String, String>>>,
    is_demo: bool,
) {
    // Read in the file
    let code = std::fs::read_to_string(input_filepath.clone()).expect(&format!(
        "Errored reading file {}",
        input_filepath.display()
    ));

    let file_name = get_filename_as_string(&input_filepath);

    // Create a driver
    let driver = new_driver(driver_config)
        .await
        .expect("Could not launch driver");

    // Run the code
    run(code, output_filepath, file_name, driver, dt, is_demo)
        .await
        .expect("Oh no!");
}

async fn repl_loop(
    output_filepath: PathBuf,
    driver_config: WebDriverConfig,
    is_demo: bool,
) -> Result<(), &'static str> {
    let driver = new_driver(driver_config)
        .await
        .map_err(|_| "Error starting interpreter and/or browser")?;
    let mut interpreter = Interpreter::new(driver, vec![], is_demo, None);

    let mut script_buffer = String::new();

    let script_name: String = prompt_default("What is the name of this test?", "test".to_owned())
        .map_err(|_| "Error reading script name")?;

    let use_start_script: bool = prompt("Do you want to start from an existing script".to_owned())
        .map_err(|_| "Error prompting start script")?;
    let start_script: Option<PathBuf> = use_start_script.then(|| {
        prompt("Please provide the path to the script".to_owned()).expect("Error reading in file")
    });

    if let Some(start_path) = start_script {
        let code =
            std::fs::read_to_string(start_path).map_err(|_| "Error reading in start file code")?;

        // Scan and parse the code
        let mut scanner = Scanner::from_src(code);
        let tokens = scanner.scan();
        let stmts = schnauzer_ui::parser::Parser::new().parse(tokens);

        for stmt in stmts.into_iter() {
            match stmt {
                Stmt::Comment(_) => {
                    script_buffer.push('\n');
                }
                _ => {}
            }
            script_buffer.push_str(&format!("{}", stmt));
            script_buffer.push('\n');
            match interpreter.execute_stmt(stmt).await {
                Ok(_) => {}
                Err(_) => {
                    println!("Warning: Error encountered while running start script.");
                }
            }
        }
    }

    loop {
        // Prompt for a schnauzer_ui statement
        let code: String = prompt("Enter a command").map_err(|_| "Error reading in line")?;

        // Check if the user wants to exit
        if code == "exit" {
            interpreter
                .driver
                .close_window()
                .await
                .map_err(|_| "Error closing browser window")?;
            break;
        }

        // Scan and parse the code
        let mut scanner = Scanner::from_src(code);
        let tokens = scanner.scan();
        let stmts = schnauzer_ui::parser::Parser::new().parse(tokens);

        for stmt in stmts.into_iter() {
            match interpreter.execute_stmt(stmt.clone()).await {
                Ok(_) => {
                    // Prompt the user if they want to save the statement
                    let save_stmt: bool = prompt_default("Save this statement?", true)
                        .map_err(|_| "Error reading in line")?;
                    if save_stmt {
                        script_buffer.push_str(&format!("{}", stmt));
                        script_buffer.push('\n');
                        match stmt {
                            Stmt::Comment(_) => {
                                script_buffer.push('\n');
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    eprintln!("The statement {} resulted in an error: {}", stmt, e.0);
                    let save_stmt: bool = prompt_default("Save this statement anyway?", false)
                        .map_err(|_| "Error reading in line")?;
                    if save_stmt {
                        script_buffer.push_str(&format!("{}", stmt));
                        script_buffer.push('\n');
                        match stmt {
                            Stmt::Comment(_) => {}
                            _ => {
                                script_buffer.push('\n');
                            }
                        }
                    }
                }
            }
        }
    }

    // Prompt the user if the want to save the script
    let save_script: bool = prompt_default("Save this test run as a SchnauzerUI script?", true)
        .map_err(|_| "Error saving the test run as a SchnauzerUI script.")?;

    // If they want to save the script, write the script buffer to the output path.
    if save_script {
        std::fs::write(
            output_filepath
                .with_file_name(&script_name)
                .with_extension("sui"),
            script_buffer,
        )
        .expect("Error writing script to output file");
    }

    Ok(())
}

// Helpers ---------------------

fn get_filename_as_string(path: &PathBuf) -> String {
    path.file_stem()
        .expect("Could not get file name")
        .to_string_lossy()
        .to_string()
}

fn looks_like_file(path: &PathBuf) -> bool {
    path.file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or("".to_owned())
        .contains(".")
}
