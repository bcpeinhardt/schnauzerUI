use clap::{ArgGroup, Parser};
use futures::future::join_all;
use walkdir::WalkDir;

use schnauzer_ui::run;

/// SchnauzerUI is a DSL for automated web UI testing.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(
    ArgGroup::new("script_path")
        .required(true)
        .args(["dir", "filepath", "repl"])))]
struct Cli {
    /// Path to a directory of scripts to run in place
    #[arg(short, long)]
    dir: Option<String>,

    /// Path to a SchnauzerUI .sui file
    #[arg(short, long)]
    filepath: Option<String>,

    /// Run SchnauzerUI in a REPL.
    #[arg(long)]
    repl: bool,

    /// Path to a directory for logs and screenshots
    #[arg(short, long)]
    output: Option<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match (cli.dir, cli.filepath, cli.repl) {
        (Some(dir), None, false) => run_dir(dir, cli.output).await,
        (None, Some(filepath), false) => run_file(filepath, cli.output).await,
        (None, None, true) => run_repl(cli.output).await,
        _ => unreachable!(),
    }
}

async fn run_file(filepath: String, output_filepath: Option<String>) {
    // Read in the file
    let code = std::fs::read_to_string(filepath.clone())
        .expect(&format!("Errored reading file {}", filepath));

    // Get the current operating directory by splitting off the filename
    let len = filepath.split(|c| c == '/' || c == '\\').count();
    let path = filepath
        .split(|c| c == '/' || c == '\\' )
        .enumerate()
        .filter(|(i, _)| *i != len - 1)
        .map(|(_, txt)| txt)
        .collect::<Vec<&str>>()
        .join("/");

    if let Some(op) = output_filepath {

        // Create the appropriate directory for logging if it doesn't exist
        std::fs::create_dir_all(format!("{}/{}", path, op))
            .expect("Could not create directory for logs");

        // Init logging to the specified path
        let log_file_name = filepath.split(|c| c == '/' || c == '\\' ).last().unwrap().split(".").filter(|ext| *ext != "sui").collect::<Vec<&str>>().join(".");
        let log_path = format!("{}/{}/{}.log", path, op, log_file_name);
        run(code, log_path).await.expect("Oh no!");
    } else {
        let log_file_name = filepath.split(|c| c == '/' || c == '\\').last().unwrap().split(".").filter(|ext| *ext != "sui").collect::<Vec<&str>>().join(".");
        let log_path = format!("{}/{}.log", path, log_file_name);
        run(code, log_path).await.expect("Oh no!");
    }
}

async fn run_dir(directory: String, output_filepath: Option<String>) {
    let mut tests = Vec::new();
    for entry in WalkDir::new(&directory).follow_links(true).into_iter().filter_map(|e| e.ok()) {
        let op_clone = output_filepath.clone();
        tests.push(tokio::spawn(async move { 
            let entry = entry.path().to_str().expect("Invalid directory provided.");
            if entry.contains(".sui") {
                run_file(entry.to_owned(), op_clone).await;
            }
        }));
    }
    join_all(tests).await;
}

async fn run_repl(output_filepath: Option<String>) {}
