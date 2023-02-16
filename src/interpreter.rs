use std::path::PathBuf;

use async_recursion::async_recursion;
use futures::TryFutureExt;
use thirtyfour::{components::SelectElement, prelude::*};

use crate::{
    environment::Environment,
    parser::{Cmd, CmdParam, CmdStmt, IfStmt, SetVariableStmt, Stmt},
    test_report::{ExecutedStmt, Report},
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
    pub driver: WebDriver,

    /// The statements for the interpreter to execute
    stmts: Vec<Stmt>,

    /// Each interpreter gets an environment for storing variables
    environment: Environment,

    /// The locate command brings an element into focus. That element is stored here. Subsequents commands are performed
    /// against this element.
    curr_elem: Option<WebElement>,

    /// The last locator used to locate an element. Stored
    /// to re-execute locate command when necessary (like for a stale element)
    locator: Option<String>,

    /// The had error field tracks whether or not the script encountered an error, and is used to move between catch-error: statements.
    had_error: bool,

    /// We store the statements that we encounter since the last catch-error stmt in order for
    /// the try-again command to be able to re-execute them.
    stmts_since_last_error_handling: Vec<Stmt>,

    /// The tried again field stores whether or not we are in try-again mode. It is used to cause an early return
    /// in the case that we encounter an error while in try-again mode.
    tried_again: bool,

    /// The progress of the program is stored into a buffer to optionally be written to a file
    pub reporter: Option<Report>,
    pub screenshot_buf: Vec<Vec<u8>>,

    /// Denotes whether the program is in "demo" mode
    is_demo: bool,

    /// Base for when the under command is used
    under_element: Option<WebElement>,
}

impl Interpreter {
    /// Constructor for the Interpreter. Registers a webdriver against a standalone selenium grid running at port 4444.
    pub fn new(
        driver: WebDriver,
        stmts: Vec<Stmt>,
        is_demo: bool,
        reporter: Option<Report>,
    ) -> Self {
        let stmts = stmts.into_iter().rev().collect();

        Self {
            driver,
            stmts,
            environment: Environment::new(),
            curr_elem: None,
            had_error: false,
            stmts_since_last_error_handling: vec![],
            tried_again: false,
            reporter,
            screenshot_buf: vec![],
            is_demo,
            locator: None,
            under_element: None,
        }
    }

    /// Executes a list of stmts. Returns a boolean indication of whether or not there was an early return.
    pub async fn interpret(&mut self, close_driver: bool) -> WebDriverResult<bool> {
        // Reset in case the interpreter is being reused
        self.curr_elem = None;
        self.had_error = false;
        self.stmts_since_last_error_handling.clear();
        self.tried_again = false;

        while let Some(stmt) = self.stmts.pop() {
            match self.execute_stmt(stmt.clone()).await {
                Ok(()) => {
                    if let Some(ref mut reporter) = self.reporter {
                        reporter.add_stmt(ExecutedStmt {
                            text: stmt.to_string(),
                            error: None,
                            screenshots: std::mem::replace(&mut self.screenshot_buf, vec![]),
                        });
                    }
                }
                Err((e, sev)) => {
                    if let Some(ref mut reporter) = self.reporter {
                        // report the error
                        reporter.add_stmt(ExecutedStmt {
                            text: stmt.to_string(),
                            error: Some(e),
                            screenshots: std::mem::replace(&mut self.screenshot_buf, vec![]),
                        });
                    }

                    match sev {
                        Severity::Exit => {
                            break;
                        }
                        Severity::Recoverable => {
                            self.had_error = true;
                        }
                    }
                }
            }
        }

        // We completed the entire script.
        if close_driver {
            self.driver.close_window().await?;
        }

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
    async fn set_curr_elem(
        &mut self,
        elem: WebElement,
        scroll_into_view: bool,
    ) -> RuntimeResult<WebElement, String> {
        // Scroll the element into view if specified, but don't fail on an error
        // as this can error falsely for thing like chat windows
        if scroll_into_view {
            let _ = elem.scroll_into_view().await;
        }

        // Give the located element a purple border if in demo mode
        if self.is_demo {
            self.driver
                .execute(
                    r#"
            arguments[0].style.border = '5px solid purple';
            "#,
                    vec![elem
                        .to_json()
                        .map_err(|_| self.error("Error jsonifying element"))?],
                )
                .await
                .map_err(|_| self.error("Error highlighting element"))?;

            // Remove the border from the previously located element
            if let Some(ref curr_elem) = self.curr_elem {
                // For now we are explicitly ignoring the error, because if the un-highlight fails
                // it could simply be that the element has become stale.
                let _ = self
                    .driver
                    .execute(
                        r#"
            arguments[0].style.border = 'none';
            "#,
                        vec![curr_elem
                            .to_json()
                            .map_err(|_| self.error("Error jsonifying element"))?],
                    )
                    .await;
            }
        }

        // Set the current element
        self.curr_elem = Some(elem.clone());
        Ok(elem)
    }

    /// Returns a reference to the current element for performing operations on, or an
    /// error if there is no current element.
    async fn get_curr_elem(&mut self) -> RuntimeResult<&WebElement, String> {
        if let Some(elem) = self.curr_elem.as_ref() {
            if !elem
                .is_present()
                .await
                .map_err(|_| self.error("Error checking if element is present"))?
            {
                // Element is stale, so replay the last locate command. Helps with pages which are highly dynamic
                // for a few moments during the loading.
                if let Some(locator) = self.locator.clone() {
                    self.locate(CmdParam::String(locator), false).await?;
                }
            }
        }

        self.curr_elem
            .as_ref()
            .ok_or(self.error("No element currently located. Try using the locate command"))
    }

    /// Executes a single SchnauzerUI statement.
    pub async fn execute_stmt(&mut self, stmt: Stmt) -> RuntimeResult<(), String> {
        // Add the statement to the list of stmts since the last catch-error stmt was encountered.
        // Used by the try-again command to re-execute on an error.
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
                Stmt::Comment(_) => {
                    // Comments are simply added to the report log, so we just ignore them
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
                Stmt::Under(cp, cs) => {
                    self.under_element = Some(self.locate(cp, true).await?);
                    self.execute_cmd_stmt(cs).await?;
                    self.under_element = None;
                    Ok(())
                }
                Stmt::UnderActiveElement(cs) => {
                    let active_elm = self
                        .driver
                        .active_element()
                        .await
                        .map_err(|_| self.error("Error getting active element."))?;
                    self.under_element = Some(active_elm);
                    self.execute_cmd_stmt(cs).await?;
                    self.under_element = None;
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
        // Adding a default wait of 1 second between commands because it just mimics human timing a lot
        // better. Will add a flag to turn this off.
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        match cmd {
            Cmd::Locate(locator) => self.locate(locator, true).await.map(|_| ()),
            Cmd::LocateNoScroll(locator) => self.locate(locator, false).await.map(|_| ()),
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
            Cmd::Press(cp) => self.press(cp).await,
            Cmd::Chill(cp) => self.chill(cp).await,
            Cmd::Select(cp) => self.select(cp).await,
            Cmd::DragTo(cp) => self.drag_to(cp).await,
            Cmd::Upload(cp) => self.upload(cp).await,
            Cmd::AcceptAlert => {
                self.driver
                    .accept_alert()
                    .map_err(|_| self.error("Error accepting alert"))
                    .await
            }
            Cmd::DismissAlert => {
                self.driver
                    .dismiss_alert()
                    .map_err(|_| self.error("Error dismissing alert"))
                    .await
            }
        }
    }

    // Very often a user will locate an html label element and then
    // specify a command intending to interact with it's associated
    // input element. We'll try to aid this convenient behavior by dynamically
    // swapping out the label for its input as the current element.
    // A label/input pair with the matching for/id attributes respectively,
    // or a label/input pair where the label element contains the input element or directly precedes it,
    // will be swapped.
    async fn resolve_label(&mut self) -> RuntimeResult<(), String> {
        // Label with correct for attribute
        if self
            .get_curr_elem()
            .await?
            .tag_name()
            .await
            .unwrap_or("ignore_error".to_owned())
            == "label"
        {
            // Label contains input or textarea
            if let Some(input) = self
                .get_curr_elem()
                .await?
                .query(By::Tag("input"))
                .or(By::Tag("textarea"))
                .or(By::Tag("select"))
                .nowait()
                .first()
                .await
                .ok()
            {
                self.set_curr_elem(input, false).await?;
                return Ok(());
            }

            // Get the for attribute
            let for_attr = self
                .get_curr_elem()
                .await?
                .attr("for")
                .await
                .map_err(|_| self.error("Unknown error"))?;

            // Try to find the input element with the corresponding id or name attribute
            if let Some(for_attr) = for_attr {
                // Try to find the element
                let label_target = self
                    .driver
                    .query(By::Id(&for_attr))
                    .or(By::Name(&for_attr))
                    .nowait()
                    .first()
                    .await
                    .ok();

                // If we found an associated element, swap into current element
                if let Some(target) = label_target {
                    self.set_curr_elem(target, false).await?;
                    return Ok(());
                }
            }

            // If the label doesn't contain the input or have an associated for attribute
            // leading to the input, then check to see if there is an input element right
            // after the label
            let following_input = self
                .get_curr_elem()
                .await?
                .query(By::XPath("./following-sibling::input"))
                .or(By::XPath("./following-sibling::textarea"))
                .or(By::XPath("./following-sibling::select"))
                .nowait()
                .first()
                .await
                .ok();

            if let Some(elm) = following_input {
                self.set_curr_elem(elm, false).await?;
                return Ok(());
            }

            // If none of this works, perform a recursive search for the input element
            // (like the "under" command but specifically for an input)
            // Limit to 5 elements of depth b/c anything further is probably a bug.
            // To do a full recursive search, users can use 
            // `under "<label-text>" locate "input" and type "some text"`
            for _ in 0..5 {
                match self.get_curr_elem().await?.parent().await {
                    Ok(parent) => {
                        self.set_curr_elem(parent, false).await?;
                        match self.get_curr_elem().await?.query(By::Tag("input"))
                        .or(By::Tag("textarea"))
                        .or(By::XPath("select"))
                        .nowait().first().await.ok() {
                            Some(elm) => {
                                self.set_curr_elem(elm, false).await?;
                                return Ok(());
                            },
                            None => continue,
                        }
                    },
                    Err(_) => break, // the resolve failed but we'll keep going
                }
            }
        }

        Ok(())
    }

    pub async fn upload(&mut self, cp: CmdParam) -> RuntimeResult<(), String> {
        // Uploading to a file input is the same as typing keys into it,
        // but our users shouldn't have to know that.

        let path_str = self.resolve(cp)?;
        let path = PathBuf::from(path_str);
        let abs_path = path
            .canonicalize()
            .map_err(|_| self.error("Error resolving path to file"))?;
        let abs_path_str = abs_path
            .to_str()
            .ok_or(self.error("Error converting absolute path to string"))?;

        self.get_curr_elem()
            .await?
            .send_keys(abs_path_str)
            .await
            .map_err(|_| self.error("Error uploading file"))
    }

    pub async fn drag_to(&mut self, cp: CmdParam) -> RuntimeResult<(), String> {
        let current = self.get_curr_elem().await?.clone();
        self.locate(cp, false).await?;
        current
            .js_drag_to(self.get_curr_elem().await?)
            .await
            .map_err(|_| self.error("Error dragging element."))
    }

    pub async fn select(&mut self, cp: CmdParam) -> RuntimeResult<(), String> {
        let option_text = self.resolve(cp)?;

        self.resolve_label().await?;

        // Sometimes, a Select element's only visible text on the page
        // is it's default option. Many users may try to locate
        // the select element based on that text and have to dive into the html
        // before realizing they aren't locating the select element. To prevent
        // this, when select is called, if the currently selected element is an option,
        // we first change it to the parent select containing it.
        if self
            .get_curr_elem()
            .await?
            .tag_name()
            .await
            .unwrap_or("ignore error".to_owned())
            == "option"
        {
            let parent_select = self
                .get_curr_elem()
                .await?
                .query(By::XPath("./.."))
                .first()
                .await
                .map_err(|_| {
                    self.error(
                        "Error getting parent select. Try locating the select element directly",
                    )
                })?;
            self.set_curr_elem(parent_select, false).await?;
        }

        // Try to create a select element from the current located element
        let select_elm = SelectElement::new(self.get_curr_elem().await?)
            .await
            .map_err(|_| self.error("Element is not a <select> element"))?;

        // Try to select the element by text
        select_elm
            .select_by_visible_text(&option_text)
            .await
            .map_err(|_| self.error(&format!("Could not select text {}", option_text)))
    }

    pub async fn chill(&mut self, cp: CmdParam) -> RuntimeResult<(), String> {
        let time_to_wait = match self.resolve(cp)?.parse::<u64>() {
            Ok(time) => time,
            _ => return Err(self.error("Could not parse time to wait as integer.")),
        };

        tokio::time::sleep(tokio::time::Duration::from_secs(time_to_wait)).await;

        Ok(())
    }

    pub async fn press(&mut self, cp: CmdParam) -> RuntimeResult<(), String> {
        let key_to_press = match self.resolve(cp)?.as_ref() {
            "Enter" => Key::Enter,
            _ => return Err(self.error("Unsupported Key")),
        };
        self.get_curr_elem()
            .await?
            .send_keys("" + &key_to_press)
            .await
            .map_err(|_| {
                self.error("Error pressing key. Make sure you have an element in focus first.")
            })
    }

    /// Reads the text of the currently located element to a variable.
    pub async fn read_to(&mut self, name: String) -> RuntimeResult<(), String> {
        let txt = self
            .get_curr_elem()
            .await?
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

        // This would be more efficient with some kind of mem_swap type function.
        self.stmts
            .append(&mut self.stmts_since_last_error_handling.clone());
        self.stmts_since_last_error_handling.clear();
    }

    /// Takes a screenshot of the page.
    pub async fn screenshot(&mut self) -> RuntimeResult<(), String> {
        let ss = self
            .driver
            .screenshot_as_png()
            .await
            .map_err(|_| self.error("Error taking screenshot."))?;
        self.screenshot_buf.push(ss);
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
        self.resolve_label().await?;

        // We need to wait for the element to be clickable by default,
        // but also account for weird htmls structures. So, we'll
        // wait for the element to be clickable, but ignore the error if
        // there is one.
        let _ = self.get_curr_elem().await?.wait_until().clickable().await;

        self.driver
            .action_chain()
            .move_to_element_center(self.get_curr_elem().await?)
            .click()
            .perform()
            .await
            .map_err(|e| self.error(&format!("Error clicking element: {}", e)))
    }

    /// Tries to type into the current element
    pub async fn type_into_elem(&mut self, cmd_param: CmdParam) -> RuntimeResult<(), String> {
        let txt = self.resolve(cmd_param)?;

        self.resolve_label().await?;

        // Instead of typing into the located element,
        // we'll click the located element, then type
        // into the "active" element. This will a help
        // a lot with custom popup typing interactions.

        // Click the current element
        self.click().await?;

        // Wait a second in case some javascript needs to happen
        // for fancy components
        std::thread::sleep(std::time::Duration::from_secs(1));

        // Get the active element
        let active_elm = self
            .driver
            .active_element()
            .await
            .map_err(|_| self.error("Could not locate active element"))?;

        active_elm.clear().await.map_err(|_| self.error("Error clearing element"))?;

        // Type into the element
        active_elm
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
    #[async_recursion]
    pub async fn locate(
        &mut self,
        locator: CmdParam,
        scroll_into_view: bool,
    ) -> RuntimeResult<WebElement, String> {
        let locator = self.resolve(locator)?;

        // Store the locator in case we need to re-execute locate command (stale element, etc.)
        self.locator = Some(locator.clone());

        // If we're in a state of "under", search from the base element
        if let Some(ref base_elem) = self.under_element {
            // Locate an input element by its placeholder
            if let Ok(found_elem) = base_elem
                .query(By::XPath(&format!(".//input[@placeholder='{}']", locator)))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find the element by partial placeholder
            if let Ok(found_elem) = base_elem
                .query(By::XPath(&format!(
                    ".//input[contains(@placeholder, '{}')]",
                    locator
                )))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find the element by its text
            if let Ok(found_elem) = base_elem
                .query(By::XPath(&format!(".//*[text()='{}']", locator)))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find the element by partial text
            if let Ok(found_elem) = base_elem
                .query(By::XPath(&format!(".//*[contains(text(), '{}')]", locator)))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find an element by it's title
            if let Ok(found_elem) = base_elem
                .query(By::XPath(&format!(".//*[@title='{}']", locator)))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to locate by aria-label
            if let Ok(found_elem) = base_elem
                .query(By::XPath(&format!(".//*[@aria-label='{}']", locator)))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find an element by it's id
            if let Ok(found_elem) = base_elem
                .query(By::XPath(&format!(".//*[@id='{}']", locator)))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find an element by it's name
            if let Ok(found_elem) = base_elem
                .query(By::Name(&locator))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find an element by it's class
            if let Ok(found_elem) = base_elem
                .query(By::ClassName(&locator))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find an element by tag name
            if let Ok(found_elem) = base_elem
                .query(By::Tag(&locator))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find an element by xpath
            if let Ok(found_elem) = base_elem
                .query(By::XPath(&format!(".{}", locator)))
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // If we don't find it under the under elem,
            // go up one
            self.under_element = base_elem.parent().await.ok();
            return self
                .locate(CmdParam::String(locator), scroll_into_view)
                .await;
        }

        // Regular queries
        for wait in [0, 5, 10, 20, 30] {
            std::thread::sleep(std::time::Duration::from_secs(wait));

            // Locate an input element by its placeholder
            if let Ok(found_elem) = self
                .driver
                .query(By::XPath(&format!("//input[@placeholder='{}']", locator)))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find the element by partial placeholder
            if let Ok(found_elem) = self
                .driver
                .query(By::XPath(&format!(
                    "//input[contains(@placeholder, '{}')]",
                    locator
                )))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find the element by its text
            if let Ok(found_elem) = self
                .driver
                .query(By::XPath(&format!("//*[text()='{}']", locator)))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find the element by partial text
            if let Ok(found_elem) = self
                .driver
                .query(By::XPath(&format!("//*[contains(text(), '{}')]", locator)))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find an element by it's title
            if let Ok(found_elem) = self
                .driver
                .query(By::XPath(&format!("//*[@title='{}']", locator)))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to locate by aria-label
            if let Ok(found_elem) = self
                .driver
                .query(By::XPath(&format!("//*[@aria-label='{}']", locator)))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find an element by it's id
            if let Ok(found_elem) = self
                .driver
                .query(By::Id(&locator))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find an element by it's name
            if let Ok(found_elem) = self
                .driver
                .query(By::Name(&locator))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find an element by it's class
            if let Ok(found_elem) = self
                .driver
                .query(By::ClassName(&locator))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find an element by tag name
            if let Ok(found_elem) = self
                .driver
                .query(By::Tag(&locator))
                .and_displayed()
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }

            // Try to find an element by xpath
            if let Ok(found_elem) = self
                .driver
                .query(By::XPath(&locator))
                .nowait()
                .first()
                .await
            {
                return self.set_curr_elem(found_elem, scroll_into_view).await;
            }
        }

        Err(self.error("Could not locate the element"))
    }
}
