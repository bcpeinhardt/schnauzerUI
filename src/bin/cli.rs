use schnauzer_ui::run;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() == 2 {
        let filepath = args.get(1).unwrap().to_owned();
        let code = std::fs::read_to_string(filepath.clone())
            .expect(&format!("Errored reading file {}", filepath));
        run(code);
    } else {
        println!("Usage: ./schui <filepath>");
    }
}
