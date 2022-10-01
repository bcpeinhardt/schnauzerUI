pub mod interpreter;
pub mod parser;
pub mod scanner;
pub mod environment;

use interpreter::Interpreter;
use parser::Parser;
use scanner::{Scanner, Token};
use thirtyfour::prelude::WebDriverResult;

pub async fn run(code: String) -> WebDriverResult<bool> {
    let mut scanner = Scanner::from_src(code);
    let tokens = scanner.scan();

    let stmts = Parser::new().parse(tokens);

    let mut interpreter = Interpreter::new(stmts).await?;
    interpreter.interpret().await
}
