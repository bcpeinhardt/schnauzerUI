use std::path::{Path, PathBuf};

use clap::{ArgGroup, Parser};
use futures::future::join_all;
use promptly::{prompt, prompt_default, prompt_opt};
use walkdir::WalkDir;

use schnauzer_ui::{interpreter::Interpreter, parser::Stmt, run, scanner::Scanner};

/// SchnauzerUI is a DSL for automated web UI testing.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(
    ArgGroup::new("script_path")
        .required(true)
        .args(["input_dir", "filepath", "repl"])))]
struct Cli {
    /// Path to a directory of scripts to run
    #[arg(short, long)]
    input_dir: Option<PathBuf>,

    /// Path to a SchnauzerUI .sui file to run
    #[arg(short, long)]
    filepath: Option<PathBuf>,

    /// Run SchnauzerUI in a REPL.
    #[arg(long)]
    repl: bool,

    /// When --dir or --filepath passed, path to a directory for logs and screenshots.
    /// When --repl passed, path to a directory for the script to record the repl interactions.
    #[arg(short, long)]
    output_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    // Destructure the cli arguments passed
    let Cli {
        input_dir,
        filepath,
        repl,
        output_dir,
    } = Cli::parse();

    // Verify that the passed --output-dir could be a directory (a . would indicate a file instead)
    let looks_like_file = |path: &PathBuf| {
        path.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or("".to_owned())
            .contains(".")
    };

    if let Some(ref output) = output_dir {
        if looks_like_file(output) {
            eprintln!(
                "Usage: output_dir flag must be a directory, but received {}",
                output.display()
            );
            return;
        }
        // If it can be a directory, create it along with any required parent folders.
        std::fs::create_dir_all(output.clone()).expect("Could not create directory for logs");
    }

    // Delegate based on provided cli arguments
    match (input_dir, filepath, repl) {
        // They provided a directory, so verify it's a directory and run all the .sui files in the directory
        (Some(dir), None, false) => {
            if !dir.is_dir() {
                eprintln!(
                    "Usage: dir flag must be a directory, but received {}",
                    dir.display()
                );
                return;
            }
            run_dir(dir, output_dir).await
        }

        // They provided a filepath, so verify it's a file and just run the given file
        // the output directory should default to the directory of the input file.
        (None, Some(filepath), false) => {
            if !filepath.is_file() {
                eprintln!(
                    "Usage: filepath flag must be a file, but received {}",
                    filepath.display()
                );
                return;
            }

            let output = output_dir
                .or(filepath.parent().map(|f| f.to_path_buf()))
                .unwrap_or(".".into());
            run_file(filepath, output).await
        }

        // They provided the repl flag, so run in repl mode.
        // The output directory should default to the current directory.
        (None, None, true) => run_repl(output_dir.unwrap_or(".".into())).await,

        // This represents an unreachable combination of cli arguments.
        _ => unreachable!(),
    }
}

/// Reads in the contents of the input file and runs it as scnahuzer ui code.
async fn run_file(input_filepath: PathBuf, output_filepath: PathBuf) {
    // Read in the file
    let code = std::fs::read_to_string(input_filepath.clone()).expect(&format!(
        "Errored reading file {}",
        input_filepath.display()
    ));

    let file_name = input_filepath
        .file_stem()
        .expect("Could not get file name")
        .to_string_lossy()
        .to_string();

    // This unwrap is safe because we validate that the input_filepath
    // has a path component in the main function.
    run(code, output_filepath, file_name).await.expect("Oh no!");
}

/// Walks a directory and runs every sui file it finds as schnauzer ui code.
/// Scripts run concurrently in different threads.
/// The output directory should default to the directory of the currently running script.
async fn run_dir(directory: PathBuf, output_dir: Option<PathBuf>) {
    let mut tests = Vec::new();
    for entry in WalkDir::new(&directory)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let op = output_dir.clone();
        tests.push(tokio::spawn(async move {
            if let Some(Some("sui")) = entry.path().extension().map(|os_str| os_str.to_str()) {
                let op = op.clone().or(entry
                    .clone()
                    .path()
                    .parent()
                    .map(|p| p.to_path_buf()))
                    .unwrap_or(".".into());
                run_file(entry.into_path(), op).await;
            }
        }));
    }
    join_all(tests).await;
}

async fn run_repl(output_filepath: PathBuf) {
    repl_loop(output_filepath).await.unwrap();
}

async fn repl_loop(output_filepath: PathBuf) -> Result<(), &'static str> {
    let mut interpreter = Interpreter::new(vec![])
        .await
        .map_err(|_| "Error starting interpreter and/or browser")?;
    let mut script_buffer = String::new();

    let script_name: String = prompt_default("What is the name of this test?", "test".to_owned())
        .map_err(|_| "Error reading script name")?;
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
                            Stmt::Comment(_) => {}
                            _ => {
                                script_buffer.push('\n');
                            }
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
