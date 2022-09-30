pub mod interpreter;
pub mod parser;
pub mod scanner;

use interpreter::Interpreter;
use parser::Parser;
use scanner::{Scanner, Token};
use thirtyfour::prelude::WebDriverResult;

pub async fn run(code: String) -> WebDriverResult<()> {
    let mut scanner = Scanner::from_src(code);
    let tokens = scanner.scan();

    let stmts = Parser::new().parse(tokens);

    let mut interpreter = Interpreter::new().await?;
    interpreter.interpret(stmts).await;

    Ok(())
}
