use parser::Parser;
use scanner::{Scanner, Token};

pub mod parser;
pub mod scanner;

pub fn run(code: String) {
    let mut scanner = Scanner::from_src(code);
    let tokens = scanner.scan();

    let stmts = Parser::new().parse(tokens);
    for stmt in stmts.iter() {
        println!("{}", stmt);
    }
}
