//! Schnauzer UI is a human readable DSL for performing automated UI testing in the browser.
//! The main goal of SchnauzerUI is to increase stakeholder visibility and participation in 
//! Quality Assurance testing. Rather than providing a shim to underling code written by 
//! a QA engineer (see [Cucumber](https://cucumber.io/)), Schnauzer UI is the only source of truth for a 
//! test's execution. In this way, Schnauzer UI aims to provide a test report you can trust.
//! 
//! Let's look at an example:
//! ```SchnauzerUI
//! # Type in username (located by labels)
//! locate "Username" and type "test@test.com"
//! 
//! # Type in password (located by placeholder)
//! locate "Password" and type "Password123!"
//! 
//! # Click the submit button (located by element text)
//! locate "Submit" and click 
//! ```
//! 
//! A Schnauzer UI script is composed of commands that execute on top of running Selenium webdrivers.
//! 
//! A `#` creates a comment. Comments in SchnauzerUI are automatically added to test reports.
//! 
//! The `locate` command locates a WebElement in the most straightforward way possible. It begins with 
//! aspects of the element that are __visible to the user__ (text, placeholders, adjacent labels, etc.). This is important for a few reasons:
//! 
//! 1. QA testers rarely need to go digging around in HTML to write tests.
//! 2. Tests are more likely to survive a change in technology (for example, migrating JavaScript frameworks).
//! 3. Tests are more representative of user experience (The user doesn't care about test_ids, they do care about placeholders).
//! 
//! Then, the locate command can default to more technology specific locators, in order to allow flexibility in 
//! test authoring (precedence to be determined)
//! 
//! # Installation
//! 
//! Schnauzer UI is a binary, but we are not doing pre-built binaries at this time.
//! To install it, make sure you have Cargo and Rust installed on your system,
//! then run `cargo install --git https://github.com/bcpeinhardt/schnauzerUI`.
//! 
//! Schnauzer UI is __also__ a Rust library, which can be used in other projects. 

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
