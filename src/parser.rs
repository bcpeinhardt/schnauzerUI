use crate::scanner::{Token, TokenType};

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub lhs: Cmd,
    pub rhs: Option<(Token, Box<Stmt>)>
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {

    /// Command for resolving a locator to a web element.
    /// The associated string is the provided locator argument.
    Locate(Token),

    /// Command for typing text into some web element.
    /// The associated string is the provided text.
    Type(Token),

    /// Command for clicking a web element.
    Click,

    /// Command for printing to a report log.
    Report(Token),

    /// Command for refreshing the WebDriver.
    Refresh,

    /// The try again command lets the process know to start over after the last error handling line.
    TryAgain,

    /// Command for taking a screenshot
    Screenshot,

    /// Command that evaluates to a boolean for if there is an unhandled error.
    HadError,

    /// Command for reading the text of a webelemnt to a variable
    /// Associated string is the variable name
    ReadTo(Token),
}

pub struct Parser {
    stmts: Vec<Stmt>,
    curr_line: Vec<Token>,
    curr_token: usize,
}

impl Parser {

    pub fn new() -> Self {
        Self {
            stmts: vec![],
            curr_line: vec![],
            curr_token: 0
        }
    }

    pub fn parse(&mut self, tokens: Vec<Token>) -> Vec<Stmt> {
        for line in tokens.split(|t| t.token_type == TokenType::Eol) {
            self.curr_line = line.to_vec();
            if self.curr_line.get(self.curr_token).unwrap().token_type == TokenType::Eol { 
                break;
            }
            match self.parse_stmt() {
                Ok(stmt) => self.stmts.push(stmt),
                Err(e) => { eprintln!("Whoah there partner!") }
            }
            self.curr_token = 0;
        }
        let stmts = self.stmts.clone();
        self.stmts.clear();
        stmts
    }

    /// Parses a statement
    /// locate "Submit" and click
    pub fn parse_stmt(&mut self) -> Result<Stmt, ()> {
        let lhs = self.parse_cmd()?;
        if let Some(and_token) = self.advance_on(TokenType::And) {
            let rhs = self.parse_stmt()?;
            Ok(Stmt { lhs, rhs: Some((and_token, Box::new(rhs))) })
        } else {
            Ok(Stmt { lhs, rhs: None })
        }
    }

    pub fn parse_cmd(&mut self) -> Result<Cmd, ()> {
        if let Some(_) = self.advance_on(TokenType::Locate) {
            let locator = self.advance_on_or_err(TokenType::String)?;
            Ok(Cmd::Locate(locator))
        } else if let Some(_) = self.advance_on(TokenType::Type) {
            let txt = self.advance_on_or_err(TokenType::String)?;
            Ok(Cmd::Type(txt))
        }else if let Some(_) = self.advance_on(TokenType::Report) {
            let log_msg = self.advance_on_or_err(TokenType::String)?;
            Ok(Cmd::Report(log_msg))
        } else if let Some(_) = self.advance_on(TokenType::ReadTo){
            let var = self.advance_on_or_err(TokenType::Variable)?;
            Ok(Cmd::ReadTo(var))
        } else if let Some(token) = self.advance_on_any() {
            match token.token_type {
                TokenType::Click => Ok(Cmd::Click),
                TokenType::Refresh => Ok(Cmd::Refresh),
                TokenType::TryAgain => Ok(Cmd::TryAgain),
                TokenType::Screenshot => Ok(Cmd::Screenshot),
                TokenType::HadError => Ok(Cmd::HadError),
                _ => {
                    Err(())
                }
            }
        } else {
            Err(())
        }
    } 

    fn advance_on(&mut self, tt: TokenType) -> Option<Token> {
        if let Some(token) = self.curr_line.get(self.curr_token) {
            if token.token_type == tt {
                self.curr_token += 1;
                return Some(token.clone());
            }
        }
        None
    }

    fn advance_on_any(&mut self) -> Option<Token> {
        self.curr_token += 1;
        self.curr_line.get(self.curr_token - 1).map(|t| t.clone())
    }

    fn advance_on_or_err(&mut self, tt: TokenType) -> Result<Token, ()> {
        if let Some(token) = self.curr_line.get(self.curr_token) {
            if token.token_type == tt {
                self.curr_token += 1;
                return Ok(token.clone());
            }
        }
        Err(())
    }
}