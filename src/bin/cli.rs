use schnauzer_ui::run;

use log::LevelFilter;

#[tokio::main]
async fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() == 2 {
        let filepath = args.get(1).unwrap().to_owned();
        let code = std::fs::read_to_string(filepath.clone())
            .expect(&format!("Errored reading file {}", filepath));
        simple_logging::log_to_file(format!("./{}.log", filepath), LevelFilter::Info).expect("Failed to init logging");
        run(code).await.expect("Oh no!");
    } else {
        println!("Usage: ./schui <filepath>");
    }
}
