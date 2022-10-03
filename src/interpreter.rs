use std::time::Duration;

use async_recursion::async_recursion;
use thirtyfour::prelude::*;

use crate::{
    environment::Environment,
    parser::{Cmd, CmdParam, CmdStmt, IfStmt, SetVariableStmt, Stmt},
};

/// Represent the Severity of an error within the interpreter (i.e. how to respond to an error).
/// On a Recoverable error, the script will go to the next catch-error: stmt.
/// On an Exit error, the interpret method will early return.
pub enum Severity {
    Exit,
    Recoverable,
}

/// Type alias for errors in the interpreter.
pub type RuntimeResult<T, E> = Result<T, (E, Severity)>;

/// The interpreter is responsible for executing Schnauzer UI stmts against a running selenium grid.
pub struct Interpreter {
    /// Each interpreter has it's own browser window for executing scripts
    driver: WebDriver,

    /// The statements for the interpreter to execute
    stmts: Vec<Stmt>,

    environment: Environment,

    /// The locate command brings an element into focus. That element is stored here. Subsequents commands are performed
    /// against this element.
    curr_elem: Option<WebElement>,

    /// The had error field tracks whether or not the script encountered an error, and is used to move between catch-error: statements.
    had_error: bool,

    /// We store the statements that we encounter since the last catch-error stmt in order for
    /// the try-again command to be able to re-execute them.
    stmts_since_last_error_handling: Vec<Stmt>,

    /// The tried again field stores whether or not we are in try-again mode. It is used to cause an early return
    /// in the case that we encounter an error while in try-again mode.
    tried_again: bool,

    pub log_buffer: String,

    pub screenshot_buffer: Vec<Vec<u8>>,
}

impl Interpreter {
    /// Constructor for the Interpreter. Registers a webdriver against a standalone selenium grid running at port 4444.
    pub async fn new(stmts: Vec<Stmt>) -> WebDriverResult<Self> {
        let caps = DesiredCapabilities::firefox();
        let driver = WebDriver::new("http://localhost:4444", caps).await?;
        let stmts = stmts.into_iter().rev().collect();

        Ok(Self {
            driver,
            stmts,
            environment: Environment::new(),
            curr_elem: None,
            had_error: false,
            stmts_since_last_error_handling: vec![],
            tried_again: false,
            log_buffer: String::new(),
            screenshot_buffer: vec![]
        })
    }

    fn log_cmd(&mut self, msg: &str) {
        self.log_buffer.push_str(&format!("Info: {}", msg));
        self.log_buffer.push_str("\n");
    }

    fn log_err(&mut self, msg: &str) {
        self.log_buffer.push_str(&format!("Error: {}", msg));
        self.log_buffer.push_str("\n");
    }

    /// Executes a list of stmts. Returns a boolean indication of whether or not there was an early return.
    pub async fn interpret(&mut self) -> WebDriverResult<bool> {
        while let Some(stmt) = self.stmts.pop() {
            match self.execute_stmt(stmt).await {
                Ok(_) => { /* Just keep swimming */ }
                Err((e, sev)) => match sev {
                    Severity::Exit => {
                        self.driver.close_window().await?;
                        return Ok(true);
                    }
                    Severity::Recoverable => {
                        self.log_err(&e);
                        self.had_error = true;
                    }
                },
            }
        }

        // We completed the entire script.
        self.driver.close_window().await?;

        // Return whether or not we exited the program while inn error mode.
        Ok(self.had_error)
    }

    /// Produces an error with the appropriate severity based on
    /// whether we are currently trying to execute stmts again.
    fn error(&self, msg: &str) -> (String, Severity) {
        if self.tried_again {
            (msg.to_owned(), Severity::Exit)
        } else {
            (msg.to_owned(), Severity::Recoverable)
        }
    }

    /// Takes a webelement, attempts to scroll the element into view, and then sets
    /// the element as currently in focus. Subsequent commands will be executed against this element.
    async fn set_curr_elem(&mut self, elem: WebElement) -> RuntimeResult<(), String> {
        elem.scroll_into_view()
            .await
            .map_err(|_| self.error("Error scrolling web element into view"))?;
        self.curr_elem = Some(elem);
        Ok(())
    }

    /// Returns a reference to the current element for performing operations on, or an
    /// error if there is no current element.
    fn get_curr_elem(&self) -> RuntimeResult<&WebElement, String> {
        self.curr_elem
            .as_ref()
            .ok_or(self.error("No element currently located. Try using the locate command"))
    }

    /// Executes a single SchnauzerUI statement.
    pub async fn execute_stmt(&mut self, stmt: Stmt) -> RuntimeResult<(), String> {
        // Add the statement to the list of stmts since the last catch-error stmt was encountered.
        // Used by the try-again commadn to re-execute on an error.
        self.stmts_since_last_error_handling.push(stmt.clone());

        if !self.had_error {
            // Normal Execution
            match stmt {
                Stmt::Cmd(cs) => self.execute_cmd_stmt(cs).await,
                Stmt::If(is) => self.execute_if_stmt(is).await,
                Stmt::SetVariable(sv) => {
                    self.set_variable(sv);
                    Ok(())
                }
                Stmt::Comment(s) => {
                    // Comments are simply added to the report log.
                    self.log_cmd(&s);
                    Ok(())
                }
                Stmt::CatchErr(_) => {
                    // If we hit a catch-error stmt but no error occured, we dont do anything.
                    // Clear statements since last error so try-again command doesnt re-execute the entire script.
                    self.stmts_since_last_error_handling.clear();
                    Ok(())
                }
                Stmt::SetTryAgainFieldToFalse => {
                    // This command was inserted by the interpreter as part of executing try-again.
                    // Reaching this command means the second attempt passed without erroring,
                    // so we go back to normal execution mode.
                    self.tried_again = false;
                    Ok(())
                }
            }
        } else {
            // Syncronizing after an error.
            match stmt {
                Stmt::CatchErr(cs) => {
                    // Execute the commands on the catch-error line.
                    self.execute_cmd_stmt(cs).await?;

                    // Exit error mode and continue normal operation.
                    self.had_error = false;
                    Ok(())
                }
                stmt => {
                    // Read in the rest of the stmts until catch-error for possible re-execution.
                    self.stmts_since_last_error_handling.push(stmt);
                    Ok(())
                }
            }
        }
    }

    /// Sets the value of a variable.
    pub fn set_variable(
        &mut self,
        SetVariableStmt {
            variable_name,
            value,
        }: SetVariableStmt,
    ) {
        self.environment.set_variable(variable_name, value);
    }

    /// Tries to retrieve the value of a variable.
    pub fn get_variable(&self, name: &str) -> RuntimeResult<String, String> {
        self.environment
            .get_variable(name)
            .ok_or(self.error("Variable is not yet defined"))
    }

    /// Takes a cmd_param and tries to resolve it to a string. If it's a user provided String literal, just
    /// returns the value of the string. If it's a variable name, tries to retrieve the variable
    /// from the interpreters environment.
    pub fn resolve(&self, cmd_param: CmdParam) -> RuntimeResult<String, String> {
        match cmd_param {
            CmdParam::String(s) => Ok(s),
            CmdParam::Variable(v) => self.get_variable(&v),
        }
    }

    /// If the provided condition does not fail, executes the following cmd_stmt.
    /// Note: Our grammar does not accomodate nested if statements.
    pub async fn execute_if_stmt(
        &mut self,
        IfStmt {
            condition,
            then_branch,
        }: IfStmt,
    ) -> RuntimeResult<(), String> {
        if self.execute_cmd(condition).await.is_ok() {
            self.execute_cmd_stmt(then_branch).await
        } else {
            Ok(())
        }
    }

    /// Execute each cmd until there are no more combining `and` tokens.
    /// Fail early if one command fails.
    #[async_recursion]
    pub async fn execute_cmd_stmt(&mut self, cs: CmdStmt) -> RuntimeResult<(), String> {
        self.execute_cmd(cs.lhs).await?;
        if let Some((_, rhs)) = cs.rhs {
            self.execute_cmd_stmt(*rhs).await
        } else {
            Ok(())
        }
    }

    pub async fn execute_cmd(&mut self, cmd: Cmd) -> RuntimeResult<(), String> {
        match cmd {
            Cmd::Locate(locator) => self.locate(locator).await,
            Cmd::Type(txt) => self.type_into_elem(txt).await,
            Cmd::Click => self.click().await,
            Cmd::Refresh => self.refresh().await,
            Cmd::TryAgain => {
                self.try_again();
                Ok(())
            }
            Cmd::Screenshot => self.screenshot().await,
            Cmd::ReadTo(cp) => self.read_to(cp).await,
            Cmd::Url(url) => self.url_cmd(url).await,
        }
    }

    /// Reads the text of the currently located element to a variable.
    pub async fn read_to(&mut self, name: String) -> RuntimeResult<(), String> {
        let txt = self
            .get_curr_elem()?
            .text()
            .await
            .map_err(|_| self.error("Error getting text from element"))?;
        self.environment.set_variable(name, txt);
        Ok(())
    }

    /// Re-executes the commands since the last catch-error stmt.
    pub fn try_again(&mut self) {
        self.tried_again = true;
        self.stmts.push(Stmt::SetTryAgainFieldToFalse);
        self.stmts
            .append(&mut self.stmts_since_last_error_handling.clone());
        self.stmts_since_last_error_handling.clear();
    }

    /// Takes a screenshot of the page.
    pub async fn screenshot(&mut self) -> RuntimeResult<(), String> {
        self.log_cmd(&format!("Taking a screenshot"));
        let ss = self.driver
            .screenshot_as_png()
            .await
            .map_err(|_| self.error("Error taking screenshot."))?;
        self.screenshot_buffer.push(ss);
        Ok(())
    }

    /// Refreshes the webpage
    pub async fn refresh(&mut self) -> RuntimeResult<(), String> {
        self.driver
            .refresh()
            .await
            .map_err(|_| self.error("Error refreshing page"))
    }

    /// Tries to click on the currently located web element.
    pub async fn click(&mut self) -> RuntimeResult<(), String> {
        self.driver
            .action_chain()
            .move_to_element_center(self.get_curr_elem()?)
            .click()
            .perform()
            .await
            .map_err(|_| self.error("Error clicking element"))
    }

    /// Tries to type into the current element
    pub async fn type_into_elem(&mut self, cmd_param: CmdParam) -> RuntimeResult<(), String> {
        let txt = self.resolve(cmd_param)?;
        self.get_curr_elem()?
            .send_keys(txt)
            .await
            .map_err(|_| self.error("Error typing into element"))
    }

    /// Navigates to the provided url.
    pub async fn url_cmd(&mut self, url: CmdParam) -> RuntimeResult<(), String> {
        let url = self.resolve(url)?;
        self.driver
            .goto(url)
            .await
            .map_err(|_| self.error("Error navigating to page."))
    }

    /// Attempt to locate an element on the page, testing the locator in the following precedence
    /// (placeholder, preceding label, text, id, name, title, class, xpath)
    pub async fn locate(&mut self, locator: CmdParam) -> RuntimeResult<(), String> {
        let locator = self.resolve(locator)?;
        for wait in [0, 5, 10] {
            // Locate an element by its placeholder
            if let Ok(found_elem) = self
                .driver
                .query(By::XPath(&format!("//input[@placeholder='{}']", locator)))
                .wait(Duration::from_secs(wait), Duration::from_secs(1))
                .single()
                .await
            {
                return self.set_curr_elem(found_elem).await;
            }

            // Locate an input element by a preceding label
            let label_locator = format!("//label[text()='{}']/../input", locator);
            if let Ok(found_elem) = self
                .driver
                .query(By::XPath(&label_locator))
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem).await;
            }

            // Try to find the element by its text
            if let Ok(found_elem) = self
                .driver
                .query(By::XPath(&format!("//*[text()='{}']", locator)))
                .nowait()
                .single()
                .await
            {
                return self.set_curr_elem(found_elem).await;
            }

            // Try to find an element by it's id
            if let Ok(found_elem) = self.driver.query(By::Id(&locator)).nowait().single().await {
                return self.set_curr_elem(found_elem).await;
            }

            // Try to find an element by it's name
            if let Ok(found_elem) = self
                .driver
                .query(By::Name(&locator))
                .nowait()
                .single()
                .await
            {
                return self.set_curr_elem(found_elem).await;
            }

            // Try to find an element by it's title
            if let Ok(found_elem) = self
                .driver
                .query(By::XPath(&format!("//*[@title='{}']", locator)))
                .nowait()
                .single()
                .await
            {
                return self.set_curr_elem(found_elem).await;
            }

            // Try to find an element by it's class
            if let Ok(found_elem) = self
                .driver
                .query(By::ClassName(&locator))
                .nowait()
                .single()
                .await
            {
                return self.set_curr_elem(found_elem).await;
            }

            // Try to find an element by xpath
            if let Ok(found_elem) = self
                .driver
                .query(By::XPath(&locator))
                .nowait()
                .single()
                .await
            {
                return self.set_curr_elem(found_elem).await;
            }
        }

        Err(self.error("Could not locate the element"))
    }
}
