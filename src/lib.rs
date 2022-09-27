use scanner::{Scanner, Token};

pub mod scanner;

pub fn run(code: String) {
    let mut scanner = Scanner::new(code);
    let tokens: Vec<Token> = scanner.scan();
    for token in tokens.iter() {
        println!("{:?}", token);
    }
}
