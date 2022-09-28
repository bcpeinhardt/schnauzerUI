use parser::Parser;
use scanner::{Scanner, Token};

pub mod scanner;
pub mod parser;

pub fn run(code: String) {
    let mut scanner = Scanner::from_src(code);
    let tokens = scanner.scan();
    let ast = Parser::new().parse(tokens);
    println!("{:?}", ast);
}
