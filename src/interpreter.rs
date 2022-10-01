use std::time::Duration;

use async_recursion::async_recursion;
use log::{error, info};
use thirtyfour::prelude::*;

use crate::{parser::{Cmd, CmdParam, CmdStmt, Stmt, SetVariableStmt, IfStmt}, environment::Environment};

pub enum Severity {
    Exit,
    Recoverable
}
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

    /// We maintain a count of screenshots taken during a test run for generating filenames for
    /// said screenshots.
    screenshot_counter: usize,

    /// The had error field tracks whether or not the script encountered an error, and is used to move between catch-error: statements.
    had_error: bool,

    /// We store the statements that we encounter since the last catch-error stmt in order for
    /// the try-again command to be able to re-execute them.
    stmts_since_last_error_handling: Vec<Stmt>,

    /// The tried again field stores whether or not we are in try-again mode. It is used to cause an early return
    /// in the case that we encounter an error while in try-again mode.
    tried_again: bool,
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
            screenshot_counter: 1,
            had_error: false,
            stmts_since_last_error_handling: vec![],
            tried_again: false,
        })
    }

    /// Executes a list of stmts. Returns a boolean indication of whether or not there was an early return.
    pub async fn interpret(&mut self) -> WebDriverResult<bool> {

        while let Some(stmt) = self.stmts.pop() {

            match self.execute_stmt(stmt).await {
                Ok(_) => { /* Just keep swimming */ },
                Err((e, sev)) => {
                    match sev {
                        Severity::Exit => {
                            self.driver.close_window().await?;
                            return Ok(true);
                        },
                        Severity::Recoverable => {
                            error!("{}", e);
                            self.had_error = true;
                        },
                    }
                },
            }
        }

        // We completed the entire script.
        self.driver.close_window().await?;
        Ok(false)
    }

    /// Produces an error witht he appropriate severity based on
    /// whether we are currently trying to execute stmts again.
    pub fn error(&self, msg: &str) -> (String, Severity) {
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
        self.curr_elem.as_ref()
            .ok_or(self.error("No element currently located. Try using the locate command"))
    }

    pub async fn execute_stmt(&mut self, stmt: Stmt) -> RuntimeResult<(), String> {
        self.stmts_since_last_error_handling.push(stmt.clone());
        if !self.had_error {
            match stmt {
                Stmt::Cmd(cs) => self.execute_cmd_stmt(cs).await,
                Stmt::If(is) => self.execute_if_stmt(is).await,
                Stmt::SetVariable(sv) => {
                    self.set_variable(sv);
                    Ok(())
                },
                Stmt::Comment(s) => {
                    info!("{}", s);
                    Ok(())
                }
                Stmt::CatchErr(_) => {
                    self.stmts_since_last_error_handling.clear();
                    Ok(())
                }
                Stmt::SetTryAgainFieldToFalse => {
                    self.tried_again = false;
                    Ok(())
                },
            }
        } else {
            match stmt {
                Stmt::CatchErr(cs) => {
                    self.execute_cmd_stmt(cs).await?;
                    self.had_error = false;
                    Ok(())
                }
                stmt => {
                    self.stmts_since_last_error_handling.push(stmt);
                    Ok(())
                }
            }
        }
    }

    pub fn set_variable(&mut self, SetVariableStmt { variable_name, value}: SetVariableStmt) {
        self.environment.set_variable(variable_name, value);
    }

    pub async fn execute_if_stmt(&mut self, IfStmt { condition, then_branch }: IfStmt) -> RuntimeResult<(), String> {
        if self.execute_cmd(condition).await.is_ok() {
            self.execute_cmd_stmt(then_branch).await
        } else {
            Ok(())
        }
    }


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
        let _ = match cmd {
            Cmd::Locate(locator) => self.locate(locator).await?,
            Cmd::Type(txt) => self.type_into_elem(txt).await?,
            Cmd::Click => self.click().await?,
            Cmd::Refresh => self.refresh().await?,
            Cmd::TryAgain => {
                self.try_again();
            },
            Cmd::Screenshot => self.screenshot().await?,
            Cmd::ReadTo(cp) => self.read_to(cp).await?,
            Cmd::Url(url) => self.url_cmd(url).await?,
        };
        Ok(())
    }

    pub async fn read_to(&mut self, name: String) -> RuntimeResult<(), String> {
        let txt = self.get_curr_elem()?.text().await.map_err(|_| self.error("Error getting text from element"))?;
        self.environment.set_variable(name, txt);
        Ok(())
    }

    pub fn try_again(&mut self) {
        self.tried_again = true;
        self.stmts.push(Stmt::SetTryAgainFieldToFalse);
        self.stmts.append(&mut self.stmts_since_last_error_handling.clone());
        self.stmts_since_last_error_handling.clear();
    }

    pub async fn screenshot(&mut self) -> RuntimeResult<(), String> {
        let path_string = format!("./screenshot_{}.jpg", self.screenshot_counter);
        info!("Screenshot: {}", path_string);
        let path = std::path::Path::new(&path_string);
        self.screenshot_counter += 1;
        self.driver
            .screenshot(path)
            .await
            .map_err(|_| self.error("Error taking screenshot."))
    }

    pub async fn refresh(&mut self) -> RuntimeResult<(), String> {
        self.driver
            .refresh()
            .await
            .map_err(|_| self.error("Error refreshing page"))
    }

    pub async fn click(&mut self) -> RuntimeResult<(), String> {
        self.driver
            .action_chain()
            .move_to_element_center(self.get_curr_elem()?)
            .click()
            .perform()
            .await
            .map_err(|_| self.error("Error clicking element"))
    }

    pub fn resolve(&self, cmd_param: CmdParam) -> RuntimeResult<String, String> {
        match cmd_param {
            CmdParam::String(s) => Ok(s),
            CmdParam::Variable(v) => self.environment.get_variable(v).ok_or(self.error("Variable is not yet defined")),
        }
    }

    pub async fn type_into_elem(&mut self, cmd_param: CmdParam) -> RuntimeResult<(), String> {
        let txt = self.resolve(cmd_param)?;
        self.get_curr_elem()?
            .send_keys(txt)
            .await
            .map_err(|_| self.error("Error typing into element"))
    }

    pub async fn url_cmd(&mut self, url: CmdParam) -> RuntimeResult<(), String> {
        let url = self.resolve(url)?;
        self.driver
            .goto(url)
            .await
            .map_err(|_| self.error("Error navigating to page."))
    }

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
        }

        Err(self.error("Could not locate the element"))
    }
}
