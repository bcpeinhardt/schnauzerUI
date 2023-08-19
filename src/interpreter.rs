//! The interpreter is responsible for executing Schnauzer UI stmts. It translates Schnauzer UI 
//! statements into thirtyfour queries.

use anyhow::{bail, Context, Result};
use async_recursion::async_recursion;
use camino::Utf8PathBuf;
use thirtyfour::{components::SelectElement, prelude::*};

use crate::{
    environment::Environment,
    parser::{Cmd, CmdParam, CmdStmt, IfStmt, SetVariableStmt, Stmt},
    test_report::{ExecutedStmt, SuiReport},
};

/// The interpreter is responsible for executing Schnauzer UI stmts. It translates Schnauzer UI 
/// statements into thirtyfour queries.
#[derive(Debug)]
pub struct Interpreter {
    /// Each interpreter has it's own browser window for executing scripts
    pub driver: WebDriver,

    /// The statements for the interpreter to execute
    stmts: Vec<Stmt>,

    /// Each interpreter gets an environment for storing variables
    environment: Environment,

    /// The locate command brings an element into focus. That element is stored here. Subsequent commands are performed
    /// against this element.
    current_element: Option<WebElement>,

    /// The last locator used to locate an element. Stored
    /// to re-execute locate command when necessary (like for a stale element)
    last_used_locator: Option<String>,

    /// The had error field tracks whether or not the script encountered an error, and is used to move between catch-error: statements.
    had_error: bool,

    /// We store the statements that we encounter since the last catch-error stmt in order for
    /// the try-again command to be able to re-execute them.
    statements_since_last_error_handling: Vec<Stmt>,

    /// The progress of the program is stored into a buffer to optionally be written to a file
    pub reporter: SuiReport,

    /// A buffer for storing png bytes of screenshots taken during testing
    screenshot_buffer: Vec<Vec<u8>>,

    /// Denotes whether the program is in "demo" mode
    is_demo: bool,

    /// Base for when the under command is used
    under_element: Option<WebElement>,
}

impl Interpreter {
    /// Constructor for the Interpreter. Registers a webdriver against a standalone selenium grid running at port 4444.
    pub fn new(driver: WebDriver, stmts: Vec<Stmt>, is_demo: bool, reporter: SuiReport) -> Self {
        let stmts = stmts.into_iter().rev().collect();

        Self {
            // Provided fields
            driver,
            stmts,
            is_demo,
            reporter,

            // Initializers
            environment: Environment::new(),
            current_element: None,
            had_error: false,
            statements_since_last_error_handling: vec![],
            screenshot_buffer: vec![],
            last_used_locator: None,
            under_element: None,
        }
    }

    /// "Reset" the interpreter to reuse it.
    fn reset(&mut self) {
        self.current_element = None;
        self.had_error = false;
        self.statements_since_last_error_handling.clear();
    }

    /// Executes a list of stmts. Returns a boolean indication of whether or not there was an early return.
    pub async fn interpret(mut self, close_driver: bool) -> Result<SuiReport> {
        self.reset();

        while let Some(stmt) = self.stmts.pop() {
            match self.execute_stmt(stmt.clone()).await {
                Ok(()) => {
                    self.reporter.add_statement(ExecutedStmt {
                        text: stmt.to_string(),
                        error: None,
                        screenshots: std::mem::take(&mut self.screenshot_buffer),
                    });
                }
                Err(e) => {
                    // report the error
                    self.reporter.add_statement(ExecutedStmt {
                        text: stmt.to_string(),
                        error: Some(e.to_string()),
                        screenshots: std::mem::take(&mut self.screenshot_buffer),
                    });

                    match self.had_error {
                        true => break,
                        false => self.had_error = true,
                    }
                }
            }
        }

        // We completed the entire script.
        if close_driver {
            self.driver.close_window().await?;
        }

        // If had_error is still true when we exit, it means we had to do an early exit
        self.reporter.set_exited_early(self.had_error);
        Ok(self.reporter)
    }

    /// Takes a webelement, attempts to scroll the element into view, and then sets
    /// the element as currently in focus. Subsequent commands will be executed against this element.
    async fn set_curr_elem(
        &mut self,
        elem: WebElement,
        scroll_into_view: bool,
    ) -> Result<WebElement> {
        // Scroll the element into view if specified, but don't fail on an error
        // as this can error falsely for thing like chat windows
        if scroll_into_view {
            let _ = elem.scroll_into_view().await;
        }

        // Give the located element a purple border if in demo mode
        if self.is_demo {
            let _ = self.driver
                .execute(
                    r#"
            arguments[0].style.border = '5px solid purple';
            "#,
                    vec![elem.to_json().context("Error jsonifying element")?],
                )
                .await
                .context("Error highlighting element")?;

            // Remove the border from the previously located element
            if let Some(ref curr_elem) = self.current_element {
                // For now we are explicitly ignoring the error, because if the un-highlight fails
                // it could simply be that the element has become stale.
                let _ = self
                    .driver
                    .execute(
                        r#"
            arguments[0].style.border = 'none';
            "#,
                        vec![curr_elem.to_json().context("Error jsonifying element")?],
                    )
                    .await;
            }
        }

        // Set the current element
        self.current_element = Some(elem.clone());
        Ok(elem)
    }

    /// Returns a reference to the current element for performing operations on, or an
    /// error if there is no current element.
    async fn get_curr_elem(&mut self) -> Result<&WebElement> {
        if let Some(elem) = self.current_element.as_ref() {
            if !elem
                .is_present()
                .await
                .context("Error checking if element is present")?
            {
                // Element is stale, so replay the last locate command. Helps with pages which are highly dynamic
                // for a few moments during the loading.
                if let Some(locator) = self.last_used_locator.clone() {
                    let _ = self.locate(CmdParam::String(locator), false).await?;
                }
            }
        }

        self.current_element
            .as_ref()
            .context("No element currently located. Try using the locate command")
    }

    /// Executes a single SchnauzerUI statement.
    pub async fn execute_stmt(&mut self, stmt: Stmt) -> Result<()> {
        // Add the statement to the list of stmts since the last catch-error stmt was encountered.
        // Used by the try-again command to re-execute on an error.
        self.statements_since_last_error_handling.push(stmt.clone());

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
                    self.statements_since_last_error_handling.clear();
                    Ok(())
                }
                Stmt::SetHadErrorFieldToFalse => {
                    // This command was inserted by the interpreter as part of executing try-again.
                    // Reaching this command means the second attempt passed without erroring,
                    // so we go back to normal execution mode.
                    self.had_error = false;
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
                        .context("Error getting active element.")?;
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
                    self.statements_since_last_error_handling.push(stmt);
                    Ok(())
                }
            }
        }
    }

    /// Sets the value of a variable.
    fn set_variable(
        &mut self,
        SetVariableStmt {
            name: variable_name,
            value,
        }: SetVariableStmt,
    ) {
        self.environment.set_variable(variable_name, value);
    }

    /// Tries to retrieve the value of a variable.
    fn get_variable(&self, name: &str) -> Result<String> {
        self.environment
            .get_variable(name)
            .context("Variable is not yet defined")
    }

    /// Takes a cmd_param and tries to resolve it to a string. If it's a user provided String literal, just
    /// returns the value of the string. If it's a variable name, tries to retrieve the variable
    /// from the interpreters environment.
    fn resolve(&self, cmd_param: CmdParam) -> Result<String> {
        match cmd_param {
            CmdParam::String(s) => Ok(s),
            CmdParam::Variable(v) => self.get_variable(&v),
        }
    }

    /// If the provided condition does not fail, executes the following cmd_stmt.
    /// Note: Our grammar does not accomodate nested if statements.
    async fn execute_if_stmt(
        &mut self,
        IfStmt {
            condition,
            then_branch,
        }: IfStmt,
    ) -> Result<()> {
        if self.execute_cmd(condition).await.is_ok() {
            self.execute_cmd_stmt(then_branch).await
        } else {
            Ok(())
        }
    }

    /// Execute each cmd until there are no more combining `and` tokens.
    /// Fail early if one command fails.
    #[async_recursion]
    async fn execute_cmd_stmt(&mut self, cs: CmdStmt) -> Result<()> {
        self.execute_cmd(cs.lhs).await?;
        if let Some((_, rhs)) = cs.rhs {
            self.execute_cmd_stmt(*rhs).await
        } else {
            Ok(())
        }
    }

    /// Execute a single Schnauzer UI command
    async fn execute_cmd(&mut self, cmd: Cmd) -> Result<()> {
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
            Cmd::AcceptAlert => self
                .driver
                .accept_alert()
                .await
                .context("Error accepting alert"),
            Cmd::DismissAlert => self
                .driver
                .dismiss_alert()
                .await
                .context("Error dismissing alert"),
        }
    }

    // Very often a user will locate an html label element and then
    // specify a command intending to interact with it's associated
    // input element. We'll try to aid this convenient behavior by dynamically
    // swapping out the label for its input as the current element.
    // A label/input pair with the matching for/id attributes respectively,
    // or a label/input pair where the label element contains the input element or directly precedes it,
    // will be swapped.
    async fn resolve_label(&mut self) -> Result<()> {
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
            if let Ok(input) = self
                .get_curr_elem()
                .await?
                .query(By::Tag("input"))
                .or(By::Tag("textarea"))
                .or(By::Tag("select"))
                .nowait()
                .first()
                .await
            {
                let _ = self.set_curr_elem(input, false).await?;
                return Ok(());
            }

            // Get the for attribute
            let for_attr = self
                .get_curr_elem()
                .await?
                .attr("for")
                .await?;

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
                    let _ = self.set_curr_elem(target, false).await?;
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
                let _ = self.set_curr_elem(elm, false).await?;
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
                        let _ = self.set_curr_elem(parent, false).await?;
                        match self
                            .get_curr_elem()
                            .await?
                            .query(By::Tag("input"))
                            .or(By::Tag("textarea"))
                            .or(By::XPath("select"))
                            .nowait()
                            .first()
                            .await
                            .ok()
                        {
                            Some(elm) => {
                                let _ = self.set_curr_elem(elm, false).await?;
                                return Ok(());
                            }
                            None => continue,
                        }
                    }
                    Err(_) => break, // the resolve failed but we'll keep going
                }
            }
        }

        Ok(())
    }

    /// Upload a file.
    async fn upload(&mut self, cp: CmdParam) -> Result<()> {
        // Uploading to a file input is the same as typing keys into it,
        // but our users shouldn't have to know that.
        let path = Utf8PathBuf::from(self.resolve(cp)?).canonicalize_utf8()?;

        self.get_curr_elem()
            .await?
            .send_keys(path)
            .await
            .context("Error uploading file")
    }

    /// Drag the currently located element to another (simulated with js)
    async fn drag_to(&mut self, cp: CmdParam) -> Result<()> {
        let current = self.get_curr_elem().await?.clone();
        let _ = self.locate(cp, false).await?;
        current
            .js_drag_to(self.get_curr_elem().await?)
            .await
            .context("Error dragging element.")
    }

    /// Select an option from a select element.
    async fn select(&mut self, cp: CmdParam) -> Result<()> {
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
                .context("Error getting parent select. Try locating the select element directly")?;
            let _ = self.set_curr_elem(parent_select, false).await?;
        }

        // Try to create a select element from the current located element
        let select_elm = SelectElement::new(self.get_curr_elem().await?)
            .await
            .context("Element is not a <select> element")?;

        // Try to select the element by text
        select_elm
            .select_by_visible_text(&option_text)
            .await
            .context(format!("Could not select text {}", option_text))
    }

    /// Wait a given number of seconds.
    async fn chill(&mut self, cp: CmdParam) -> Result<()> {
        let time_to_wait = match self.resolve(cp)?.parse::<u64>() {
            Ok(time) => time,
            _ => bail!("Could not parse time to wait as integer."),
        };

        tokio::time::sleep(tokio::time::Duration::from_secs(time_to_wait)).await;

        Ok(())
    }

    /// Simulate keyboard input.
    async fn press(&mut self, cp: CmdParam) -> Result<()> {
        let key_to_press = match self.resolve(cp)?.as_ref() {
            "Enter" => Key::Enter,
            _ => bail!("Unsupported Key"),
        };
        self.get_curr_elem()
            .await?
            .send_keys("" + &key_to_press)
            .await
            .context("Error pressing key. Make sure you have an element in focus first.")
    }

    /// Reads the text of the currently located element to a variable.
    async fn read_to(&mut self, name: String) -> Result<()> {
        let txt = self
            .get_curr_elem()
            .await?
            .text()
            .await
            .context("Error getting text from element")?;
        self.environment.set_variable(name, txt);
        Ok(())
    }

    /// Re-executes the commands since the last catch-error stmt.
    fn try_again(&mut self) {
        self.stmts.push(Stmt::SetHadErrorFieldToFalse);

        // This would be more efficient with some kind of mem_swap type function.
        self.stmts
            .append(&mut self.statements_since_last_error_handling.clone());
        self.statements_since_last_error_handling.clear();
    }

    /// Takes a screenshot of the page.
    async fn screenshot(&mut self) -> Result<()> {
        let ss = self
            .driver
            .screenshot_as_png()
            .await
            .context("Error taking screenshot.")?;
        self.screenshot_buffer.push(ss);
        Ok(())
    }

    /// Refreshes the webpage
    async fn refresh(&mut self) -> Result<()> {
        self.driver.refresh().await.context("Error refreshing page")
    }

    /// Tries to click on the currently located web element.
    async fn click(&mut self) -> Result<()> {
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
            .context("Error clicking element")
    }

    /// Tries to type into the current element
    async fn type_into_elem(&mut self, cmd_param: CmdParam) -> Result<()> {
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
            .context("Could not locate active element")?;

        let _ = active_elm.clear().await.context("Error clearing element");

        // Type into the element
        active_elm
            .send_keys(txt)
            .await
            .context("Error typing into element")
    }

    /// Navigates to the provided url.
    async fn url_cmd(&mut self, url: CmdParam) -> Result<()> {
        let url = self.resolve(url)?;
        self.driver
            .goto(url)
            .await
            .context("Error navigating to page.")
    }

    /// Attempt to locate an element on the page, testing the locator in the following precedence
    /// (placeholder, preceding label, text, id, name, title, class, xpath)
    #[async_recursion]
    async fn locate(
        &mut self,
        locator: CmdParam,
        scroll_into_view: bool,
    ) -> Result<WebElement> {
        let locator = self.resolve(locator)?;

        // Store the locator in case we need to re-execute locate command (stale element, etc.)
        self.last_used_locator = Some(locator.clone());

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

            // Try to find the element by any related contents whatsoever.
            if let Ok(containing_list) = self
                .driver
                .query(By::XPath(&format!("//*[contains(., '{}')]", locator)))
                .and_displayed()
                .nowait()
                .all_from_selector()
                .await
            {
                if let Some(elm) = containing_list.last() {
                    return self.set_curr_elem(elm.to_owned(), scroll_into_view).await;
                }
            }
        }

        bail!("Could not locate the element")
    }
}
