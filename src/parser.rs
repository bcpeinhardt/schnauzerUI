//! The parser takes a list of Schnauzer UI tokens and produces an AST.

use crate::scanner::{Token, TokenType};

use anyhow::{Result, bail};

/// Represents the different kinds of statements in SchnauzerUI
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {

    /// A statement consisting of 1 or more commands.
    /// # Example
    /// ```sui
    /// locate "Get a Quote" and click
    /// ```
    Cmd(CmdStmt),

    /// A statement that executes only if
    /// given command executes
    /// # Example
    /// ```sui
    /// if locate "Confirm" then click
    /// ```
    If(IfStmt),

    /// Create or reassign a variable.
    /// # Example
    /// ```sui
    /// save "test@test.com" as username
    /// ```
    SetVariable(SetVariableStmt),

    /// A Schnauzer UI comment.
    /// Comments are automatically added to
    /// test reports.
    /// # Example
    /// ```sui
    /// # This is a comment
    /// ```
    Comment(String),

    /// Schnauzer UIs "error handling".
    /// Let's a script that encounter an error recover.
    /// 
    /// # Example
    /// ```sui
    /// # Encounter error because of typo
    /// locate "Loign"
    /// 
    /// # Some code that doesn't execute
    /// locate "Dashboard"
    /// 
    /// # Script skips ahead to here
    /// catch-error: screenshot
    /// ```
    CatchErr(CmdStmt),

    /// Change SchnauzerUIs locate command from starting
    /// at the top of the document to starting at a particular element
    /// and radiating outward.
    /// 
    /// # Example
    /// ```sui
    /// under "Navigation" locate "Desired Text" and click
    /// ```
    Under(CmdParam, CmdStmt),

    /// The same as `Under`, but starts the search at the currently 
    /// located element.
    /// 
    /// # Example
    /// ```sui
    /// under-active-element locate "Desired Text" and click
    /// ```
    UnderActiveElement(CmdStmt),

    /// This statement is not meant to be parsed. It is added by the interpreter
    /// as part of try-again logic.
    SetHadErrorFieldToFalse,
}

impl std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Cmd(cs) => write!(f, "{}", cs),
            Stmt::If(is) => write!(f, "{}", is),
            Stmt::SetVariable(sv) => write!(f, "{}", sv),
            Stmt::Comment(s) => write!(f, "{}", s),
            Stmt::CatchErr(cs) => write!(f, "catch-error: {}", cs),
            Stmt::SetHadErrorFieldToFalse => write!(f, ""),
            Stmt::Under(cp, cs) => write!(f, "under {} {}", cp, cs),
            Stmt::UnderActiveElement(cs) => write!(f, "under-active-element {}", cs),
        }
    }
}

/// Set a variable with the given name to the given value
#[derive(Debug, Clone, PartialEq)]
pub struct SetVariableStmt {

    /// The name of the variable
    pub name: String,

    /// The value of the variable
    pub value: String,
}

impl std::fmt::Display for SetVariableStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "save {} as {}", self.name, self.value)
    }
}

/// Conditiionally execute a command statement
#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {

    /// The command to execute as the predicate. If the command
    /// doesn't error, the then_branch executes
    pub condition: Cmd,

    /// The body of the if statement to execute if `condition` succeeds
    pub then_branch: CmdStmt,
}

impl std::fmt::Display for IfStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "if {} then {}", self.condition, self.then_branch)
    }
}

/// A statement made of one or more commands
/// Here's some example command statements 
/// explaining the structure
/// (the `Token` is the and keyword)
/// 
/// ```sui
/// locate "Submit"
/// locate "Submit" and click
/// locate "Submit" and click and screenshot
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct CmdStmt {

    /// The leading command
    pub lhs: Cmd,

    /// The optional trailing command statement
    pub rhs: Option<(Token, Box<CmdStmt>)>,
}

impl std::fmt::Display for CmdStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.rhs {
            Some((_, stmt)) => write!(f, "{} and {}", self.lhs, stmt),
            None => write!(f, "{}", self.lhs),
        }
    }
}

/// Represents the different commands in SchnauzerUI
#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    /// Command for resolving a locator to a web element.
    /// Also scrolls the element into view.
    /// The associated string is the locator.
    Locate(CmdParam),

    /// Command for resolving a locator to a web element.
    /// Does not scroll the element into view.
    /// The associated string is the locator.
    LocateNoScroll(CmdParam),

    /// Command for typing text into some web element.
    /// The associated string is the provided text.
    Type(CmdParam),

    /// Command for clicking a web element.
    Click,

    /// Command for refreshing the WebDriver.
    Refresh,

    /// The try again command lets the process know to start over after the last error handling line.
    TryAgain,

    /// Command for taking a screenshot
    Screenshot,

    /// Command for reading the text of a webelement to a variable.
    /// Associated string is the variable name.
    ReadTo(String),

    /// Navigate the driver to the provided URL.
    Url(CmdParam),

    /// Parses the cmd param as a key to press.
    /// Todo: Need a better strategy for handling keyboard input
    Press(CmdParam),

    /// Pauses test execution for the provided number of seconds
    Chill(CmdParam),

    /// Command for selecting an option on a select element.
    /// The associated String is the option text.
    Select(CmdParam),

    /// Command for simulating drag and drop behavior with JavaScript.
    /// The associated String is the locator for the target element.
    DragTo(CmdParam),

    /// Command for uploading a file. Associated text is the path
    /// to the file to upload.
    Upload(CmdParam),

    /// Command for accepting a browser alert window.
    AcceptAlert,

    /// Command for dismissing a browser alert window.
    DismissAlert,
}

impl std::fmt::Display for Cmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cmd::Locate(cp) => write!(f, "locate {}", cp),
            Cmd::Type(cp) => write!(f, "type {}", cp),
            Cmd::Click => write!(f, "click"),
            Cmd::Refresh => write!(f, "refresh"),
            Cmd::TryAgain => write!(f, "try-again"),
            Cmd::Screenshot => write!(f, "screenshot"),
            Cmd::ReadTo(cp) => write!(f, "read-to {}", cp),
            Cmd::Url(cp) => write!(f, "url {}", cp),
            Cmd::Press(cp) => write!(f, "press {}", cp),
            Cmd::Chill(cp) => write!(f, "chill {}", cp),
            Cmd::LocateNoScroll(cp) => write!(f, "locate-no-scroll {}", cp),
            Cmd::Select(cp) => write!(f, "select {}", cp),
            Cmd::DragTo(cp) => write!(f, "drag-to {}", cp),
            Cmd::Upload(cp) => write!(f, "upload {}", cp),
            Cmd::AcceptAlert => write!(f, "accept-alert"),
            Cmd::DismissAlert => write!(f, "dismiss-alert"),
        }
    }
}

/// Represents the kinds of parameters a SchnauzerUI command can have
#[derive(Debug, Clone, PartialEq)]
pub enum CmdParam {

    /// A string literal surrounded by double quotes
    String(String),

    /// A variable
    Variable(String),
}

impl std::fmt::Display for CmdParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CmdParam::String(s) => write!(f, "\"{}\"", s),
            CmdParam::Variable(v) => write!(f, "{}", v),
        }
    }
}

impl TryFrom<Token> for CmdParam {
    type Error = anyhow::Error;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.token_type {
            TokenType::String(s) => Ok(Self::String(s)),
            TokenType::Variable(v) => Ok(Self::Variable(v)),
            _ => bail!("Invalid Input"),
        }
    }
}

/// The Parser is responsible for transforming a list of SchnauzerUI tokens
/// in an AST.
#[derive(Debug)]
pub struct Parser {

    /// A buffer for collecting the built up statements.
    stmts: Vec<Stmt>,

    /// Tracks the current line for error reporting.
    curr_line: Vec<Token>,

    /// Tracks the current index
    index: usize,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {

    /// Creates a new parser.
    pub fn new() -> Self {
        Self {
            stmts: vec![],
            curr_line: vec![],
            index: 0,
        }
    }

    /// Transform a list of tokens into a list of statements.
    pub fn parse(&mut self, tokens: Vec<Token>) -> Vec<Stmt> {
        // A token list passed to the parse should always end in an Eof token.
        // The unwrap is safe because we checked the len > 0.
        assert!(!tokens.is_empty() && tokens.last().unwrap().token_type == TokenType::Eof);

        for line in tokens.split(|t| t.token_type == TokenType::Eol) {
            self.curr_line = line.to_vec();
            if self.current_token().unwrap().token_type == TokenType::Eof {
                break;
            }
            match self.parse_stmt() {
                Ok(stmt) => self.stmts.push(stmt),
                Err(e) => {
                    eprintln!("{}", e)
                }
            }
            self.index = 0;
        }
        let stmts = self.stmts.clone();
        self.stmts.clear();
        stmts
    }

    /// Parse a single Schnauzer UI statement
    pub fn parse_stmt(&mut self) -> Result<Stmt> {
        if self.advance_on(TokenType::If).is_ok() {
            self.parse_if_stmt().map(Stmt::If)
        } else if self.advance_on(TokenType::Under).is_ok() {
            let cp = self.parse_cmd_param()?;
            let cs = self.parse_cmd_stmt()?;
            Ok(Stmt::Under(cp, cs))
        } else if self.advance_on(TokenType::UnderActiveElement).is_ok() {
            let cs = self.parse_cmd_stmt()?;
            Ok(Stmt::UnderActiveElement(cs))
        } else if let Ok(Token {
            token_type: TokenType::Comment(s),
            ..
        }) = self.advance_on(TokenType::Comment("n/a".to_owned()))
        {
            Ok(Stmt::Comment(s))
        } else if self.advance_on(TokenType::CatchError).is_ok() {
            let stmt = self.parse_cmd_stmt()?;
            Ok(Stmt::CatchErr(stmt))
        } else if self.advance_on(TokenType::Save).is_ok() {
            let value = self
                .advance_on(TokenType::String("n/a".to_owned()))?;
            let _as_token = self
                .advance_on(TokenType::As)?;
            let variable_name = self
                .advance_on(TokenType::Variable("n/a".to_owned()))?;

            match (variable_name, value) {
                (
                    Token {
                        token_type: TokenType::Variable(variable_name),
                        ..
                    },
                    Token {
                        token_type: TokenType::String(value),
                        ..
                    },
                ) => Ok(Stmt::SetVariable(SetVariableStmt {
                    name: variable_name,
                    value,
                })),
                _ => bail!("Error"),
            }
        } else {
            self.parse_cmd_stmt().map(Stmt::Cmd)
        }
    }

    /// Parse an if statement
    fn parse_if_stmt(&mut self) -> Result<IfStmt> {
        let condition = self.parse_cmd()?;
        let _then_token = self
            .advance_on(TokenType::Then)?;
        let then_branch = self.parse_cmd_stmt()?;
        Ok(IfStmt {
            condition,
            then_branch,
        })
    }

    /// Parses a command statement
    fn parse_cmd_stmt(&mut self) -> Result<CmdStmt> {
        let lhs = self.parse_cmd()?;
        if let Ok(and_token) = self.advance_on(TokenType::And) {
            let rhs = self.parse_cmd_stmt()?;
            Ok(CmdStmt {
                lhs,
                rhs: Some((and_token, Box::new(rhs))),
            })
        } else {
            Ok(CmdStmt { lhs, rhs: None })
        }
    }

    /// Parse a `CmdParam`, the type representing what can be passed to a SchnauzerUI command
    /// as an argument.
    fn parse_cmd_param(&mut self) -> Result<CmdParam> {
        self.advance_on_any_of(vec![
            TokenType::String("n/a".to_owned()),
            TokenType::Variable("n/a".to_owned()),
        ])?
        .try_into()
    }

    /// Parse a single SchnauzerUI command.
    fn parse_cmd(&mut self) -> Result<Cmd> {
        if self.advance_on(TokenType::Locate).is_ok() {
            self.parse_cmd_param().map(Cmd::Locate)
        } else if self.advance_on(TokenType::LocateNoScroll).is_ok() {
            self.parse_cmd_param().map(Cmd::LocateNoScroll)
        } else if self.advance_on(TokenType::Type).is_ok() {
            self.parse_cmd_param().map(Cmd::Type)
        } else if self.advance_on(TokenType::ReadTo).is_ok() {
            let var = self
                .advance_on(TokenType::Variable("n/a".to_owned()))?;

            match var {
                Token {
                    token_type: TokenType::Variable(v),
                    ..
                } => Ok(Cmd::ReadTo(v)),
                _ => bail!("Expected variable"),
            }
        } else if self.advance_on(TokenType::Url).is_ok() {
            self.parse_cmd_param().map(Cmd::Url)
        } else if self.advance_on(TokenType::Press).is_ok() {
            self.parse_cmd_param().map(Cmd::Press)
        } else if self.advance_on(TokenType::Chill).is_ok() {
            self.parse_cmd_param().map(Cmd::Chill)
        } else if self.advance_on(TokenType::Select).is_ok() {
            self.parse_cmd_param().map(Cmd::Select)
        } else if self.advance_on(TokenType::DragTo).is_ok() {
            self.parse_cmd_param().map(Cmd::DragTo)
        } else if self.advance_on(TokenType::Upload).is_ok() {
            self.parse_cmd_param().map(Cmd::Upload)
        } else {
            let token = self.advance_on_any();
            match token.token_type {
                TokenType::Click => Ok(Cmd::Click),
                TokenType::Refresh => Ok(Cmd::Refresh),
                TokenType::TryAgain => Ok(Cmd::TryAgain),
                TokenType::Screenshot => Ok(Cmd::Screenshot),
                TokenType::AcceptAlert => Ok(Cmd::AcceptAlert),
                TokenType::DismissAlert => Ok(Cmd::DismissAlert),
                _ => bail!("Expected Command"),
            }
        }
    }

    /// If the current token is the type of token you're looking for,
    /// consume it. Otherwise, return nothing and do not increment.
    fn advance_on(&mut self, tt: TokenType) -> Result<Token> {
        if let Some(token) = self.current_token() {
            if token.token_type == tt {
                self.index += 1;
                return Ok(token);
            } else {
                bail!(token.error(format!("Expected `{}` token", tt)))
            }
        } else {
            // I'm not sure what to do here. Think I need to rethink these helpers
            bail!("Fix this later")
        }
    }

    /// Same as `advance_on`, but lets you specify a list of acceptable token types.
    fn advance_on_any_of(&mut self, tts: Vec<TokenType>) -> Result<Token> {
        for tt in tts.clone().into_iter() {
            if let Ok(t) = self.advance_on(tt) {
                return Ok(t);
            }
        }

        // Todo: Move this out to a display impl
        let length = tts.len();
        let tts = tts.into_iter().enumerate().fold(String::new(), |mut buf, (i, token_type) | {
            buf.push_str(&token_type.to_string());
            if i != length {
                buf.push_str(", ");
            }
            buf
        });
        bail!(format!("Expected one of the following tokens: {}", tts))
    }

    /// Advance on any token.
    fn advance_on_any(&mut self) -> Token {
        self.index += 1;
        self.curr_line.get(self.index - 1).cloned().unwrap()
    }

    /// Return a copy of the current token for inspection
    fn current_token(&self) -> Option<Token> {
        self.curr_line.get(self.index).cloned()
    }

    /// Return the previous token if there is one
    /// # Panics
    fn previous_token(&self) -> Option<Token> {
        self.curr_line.get(self.index - 1).cloned()
    }
}
