use std::time::Duration;

use async_recursion::async_recursion;
use log::{info, error};
use thirtyfour::prelude::*;

use crate::parser::{Cmd, CmdParam, CmdStmt, Stmt};

pub struct Interpreter {
    driver: WebDriver,
    curr_elem: Option<WebElement>,
    screenshot_counter: usize,
    had_error: bool,
    stmts_since_last_error_handling: Vec<Stmt>,
    tried_again: bool,
}

impl Interpreter {
    pub async fn new() -> WebDriverResult<Self> {

        let caps = DesiredCapabilities::firefox();
        let driver = WebDriver::new("http://localhost:4444", caps).await?;

        Ok(Self {
            driver,
            curr_elem: None,
            screenshot_counter: 1,
            had_error: false,
            stmts_since_last_error_handling: vec![],
            tried_again: false
        })
    }

    async fn set_curr_elem(&mut self, elem: WebElement) -> Result<(), String> {
        // Scroll the web element into view
        elem.scroll_into_view().await.map_err(|_| "Error scrolling web element into view")?;
        self.curr_elem = Some(elem);
        Ok(())
    }

    fn get_curr_elem(&self) -> Result<WebElement, String> { 
        self.curr_elem.clone().ok_or("No element currently in focus. Try using the locate command".to_owned())
    }

    pub async fn interpret(&mut self, stmts: Vec<Stmt>) {
        for stmt in stmts.into_iter() {
            if self.execute_stmt(stmt).await {
                self.driver.close_window().await.expect("Error closing driver window");
                return;
            }
        }
    }

    pub async fn execute_stmt(&mut self, stmt: Stmt) -> bool {
        self.stmts_since_last_error_handling.push(stmt.clone());
            if !self.had_error {
                match stmt {
                    Stmt::Cmd(cs) => {
                        self.execute_cmd_stmt(cs).await
                    }
                    Stmt::If(_) => todo!(),
                    Stmt::SetVariable(_) => todo!(),
                    Stmt::Comment(s) => {
                        info!("{}", s);
                        false
                    },
                    Stmt::CatchErr(_) => {
                        self.stmts_since_last_error_handling.clear();
                        false
                    },
                }
            } else {
                match stmt {
                    Stmt::CatchErr(cs) => {
                        self.had_error = false;
                        if self.execute_cmd_stmt(cs).await {
                            true
                        } else {
                            self.stmts_since_last_error_handling.clear();
                            false
                        }
                        
                    },
                    stmt => {
                        self.stmts_since_last_error_handling.push(stmt);
                        false
                    }
                }
            }
    }

    #[async_recursion]
    pub async fn execute_cmd_stmt(&mut self, cs: CmdStmt) -> bool {
        match self.execute_cmd(cs.lhs).await {
            Err(s) => {
                self.had_error = true;
                error!("{}", s);
                false
            },
            Ok(true) => true,
            Ok(false) => {
                if let Some((_, rhs)) = cs.rhs {
                    self.execute_cmd_stmt(*rhs).await
                } else {
                    false
                }
            }
        }
    }

    pub async fn execute_cmd(&mut self, cmd: Cmd) -> Result<bool, String> {
        match cmd {
            Cmd::Locate(locator) => self.locate(locator).await.map(|_| false),
            Cmd::Type(txt) => self.type_into_elem(txt).await.map(|_| false),
            Cmd::Click => self.click().await.map(|_| false),
            Cmd::Refresh => self.refresh().await.map(|_| false),
            Cmd::TryAgain => self.try_again().await,
            Cmd::Screenshot => self.screenshot().await.map(|_| false),
            Cmd::ReadTo(_) => todo!(),
            Cmd::Url(url) => self.url_cmd(url).await.map(|_| false),
        }
    }

    pub async fn try_again(&mut self) -> Result<bool, String> {
        if self.tried_again {
            return Ok(true);
        }
        self.tried_again = true;
        let stmts = self.stmts_since_last_error_handling.clone();
        self.stmts_since_last_error_handling.clear();
        self.interpret(stmts).await;
        self.tried_again = false;
        Ok(false)
    }

    pub async fn screenshot(&mut self) -> Result<(), String> {
        let path_string = format!("./screenshot_{}.jpg", self.screenshot_counter);
        info!("Screenshot: {}", path_string);
        let path = std::path::Path::new(&path_string);
        self.screenshot_counter += 1;
        self.driver.screenshot(path).await.map_err(|_| "Error taking screenshot.".to_owned())
    }

    pub async fn refresh(&mut self) -> Result<(), String> {
        self.driver.refresh().await.map_err(|_| "Error refreshing page".to_owned())
    }

    pub async fn click(&mut self) -> Result<(), String> {
        self.driver.action_chain().move_to_element_center(&self.get_curr_elem()?).click().perform().await.map_err(|_| "Error clicking element".to_owned())
    }

    pub fn resolve(&self, cmd_param: CmdParam) -> Result<String, String> {
        match cmd_param {
            CmdParam::String(s) => Ok(s),
            CmdParam::Variable(_) => todo!(),
        }
    }

    pub async fn type_into_elem(&mut self, cmd_param: CmdParam) -> Result<(), String> {
        let txt = self.resolve(cmd_param)?;
        self.get_curr_elem()?.send_keys(txt).await.map_err(|_| "Error typing into element".to_owned())
    }

    pub async fn url_cmd(&mut self, url: CmdParam) -> Result<(), String> {
        let url = self.resolve(url)?;
        self.driver.goto(url).await.map_err(|_| "Error navigating to page.".to_owned())
    }

    pub async fn locate(&mut self, locator: CmdParam) -> Result<(), String> {
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
            }

            Err("Could not locate the element".to_owned())
        
    }
}
