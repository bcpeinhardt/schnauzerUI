use crate::scanner::{Token, TokenType};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Cmd(CmdStmt),
    If(IfStmt),
    SetVariable(SetVariableStmt),
    Comment(String),
    CatchErr(CmdStmt),

    /// This statement is not meant to be parsed. It is added by the interpreter
    /// as part of try-again logic.
    SetTryAgainFieldToFalse,
}

impl std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Cmd(cs) => write!(f, "{}", cs),
            Stmt::If(is) => write!(f, "{}", is),
            Stmt::SetVariable(sv) => write!(f, "{}", sv),
            Stmt::Comment(s) => write!(f, "{}", s),
            Stmt::CatchErr(cs) => write!(f, "catch-error: {}", cs),
            Stmt::SetTryAgainFieldToFalse => write!(f, ""),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetVariableStmt {
    pub variable_name: String,
    pub value: String,
}

impl std::fmt::Display for SetVariableStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "save {} as {}", self.variable_name, self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub condition: Cmd,
    pub then_branch: CmdStmt,
}

impl std::fmt::Display for IfStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "if {} then {}", self.condition, self.then_branch)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CmdStmt {
    pub lhs: Cmd,
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

/// TODO: Add new tab cmd.
#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    /// Command for resolving a locator to a web element.
    /// The associated string is the provided locator argument.
    Locate(CmdParam),

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

    /// Command for reading the text of a webelemnt to a variable
    /// Associated string is the variable name
    ReadTo(String),

    /// Navigate the driver to the provided URL
    Url(CmdParam),

    /// Parses the cmd param as a key to press.
    /// Todo: Need a better strategy for handling keyboard input
    Press(CmdParam),

    /// Pauses test execution for the provided number of seconds
    Chill(CmdParam),
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
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CmdParam {
    String(String),
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
    type Error = String;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.token_type {
            TokenType::String(s) => Ok(Self::String(s)),
            TokenType::Variable(v) => Ok(Self::Variable(v)),
            _ => Err("Invalid input".to_owned()),
        }
    }
}

pub struct Parser {
    stmts: Vec<Stmt>,
    curr_line: Vec<Token>,
    index: usize,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            stmts: vec![],
            curr_line: vec![],
            index: 0,
        }
    }

    pub fn parse(&mut self, tokens: Vec<Token>) -> Vec<Stmt> {
        // A token list passed to the parse should always end in an Eof token.
        // The unwrap is safe because we checked the len > 0.
        assert!(tokens.len() > 0 && tokens.last().unwrap().token_type == TokenType::Eof);

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

    pub fn parse_stmt(&mut self) -> Result<Stmt, String> {
        if self.advance_on(TokenType::If).is_some() {
            self.parse_if_stmt().map(|is| Stmt::If(is))
        } else if let Some(Token {
            token_type: TokenType::Comment(s),
            ..
        }) = self.advance_on(TokenType::Comment("n/a".to_owned()))
        {
            Ok(Stmt::Comment(s))
        } else if self.advance_on(TokenType::CatchError).is_some() {
            let stmt = self.parse_cmd_stmt()?;
            Ok(Stmt::CatchErr(stmt))
        } else if self.advance_on(TokenType::Save).is_some() {
            let value = self
                .advance_on(TokenType::String("n/a".to_owned()))
                .ok_or(self.error("Expected some txt"))?;
            let _as_token = self
                .advance_on(TokenType::As)
                .ok_or(self.error("Expected `as`"))?;
            let variable_name = self
                .advance_on(TokenType::Variable("n/a".to_owned()))
                .ok_or(self.error("Expected a variable name"))?;

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
                    variable_name,
                    value,
                })),
                _ => Err(self.error("Error")),
            }
        } else {
            self.parse_cmd_stmt().map(|cs| Stmt::Cmd(cs))
        }
    }

    pub fn parse_if_stmt(&mut self) -> Result<IfStmt, String> {
        let condition = self.parse_cmd()?;
        let _then_token = self
            .advance_on(TokenType::Then)
            .ok_or(self.error("Expected keyword `then`"))?;
        let then_branch = self.parse_cmd_stmt()?;
        Ok(IfStmt {
            condition,
            then_branch,
        })
    }

    /// Parses a statement
    /// Ex. locate "Submit" and click
    pub fn parse_cmd_stmt(&mut self) -> Result<CmdStmt, String> {
        let lhs = self.parse_cmd()?;
        if let Some(and_token) = self.advance_on(TokenType::And) {
            let rhs = self.parse_cmd_stmt()?;
            Ok(CmdStmt {
                lhs,
                rhs: Some((and_token, Box::new(rhs))),
            })
        } else {
            Ok(CmdStmt { lhs, rhs: None })
        }
    }

    pub fn parse_cmd_param(&mut self) -> Result<CmdParam, String> {
        self.advance_on_any_of(vec![
            TokenType::String("n/a".to_owned()),
            TokenType::Variable("n/a".to_owned()),
        ])
        .ok_or(self.error("Expected variable or text"))?
        .try_into()
    }

    pub fn parse_cmd(&mut self) -> Result<Cmd, String> {
        if self.advance_on(TokenType::Locate).is_some() {
            self.parse_cmd_param().map(|cp| Cmd::Locate(cp))
        } else if self.advance_on(TokenType::LocateNoScroll).is_some() {
            self.parse_cmd_param().map(|cp| Cmd::LocateNoScroll(cp))
        } else if self.advance_on(TokenType::Type).is_some() {
            self.parse_cmd_param().map(|cp| Cmd::Type(cp))
        } else if self.advance_on(TokenType::ReadTo).is_some() {
            let var = self
                .advance_on(TokenType::Variable("n/a".to_owned()))
                .ok_or(self.error("Expected Variable"))?;

            match var {
                Token {
                    token_type: TokenType::Variable(v),
                    ..
                } => Ok(Cmd::ReadTo(v)),
                _ => Err(self.error("Expected Variable")),
            }
        } else if self.advance_on(TokenType::Url).is_some() {
            self.parse_cmd_param().map(|cp| Cmd::Url(cp))
        } else if self.advance_on(TokenType::Press).is_some() {
            self.parse_cmd_param().map(|cp| Cmd::Press(cp))
        } else if self.advance_on(TokenType::Chill).is_some() {
            self.parse_cmd_param().map(|cp| Cmd::Chill(cp))
        } else {
            let token = self.advance_on_any();
            match token.token_type {
                TokenType::Click => Ok(Cmd::Click),
                TokenType::Refresh => Ok(Cmd::Refresh),
                TokenType::TryAgain => Ok(Cmd::TryAgain),
                TokenType::Screenshot => Ok(Cmd::Screenshot),
                _ => Err(token.error("Expected command")),
            }
        }
    }

    fn advance_on(&mut self, tt: TokenType) -> Option<Token> {
        if let Some(token) = self.current_token() {
            if token.token_type == tt {
                self.index += 1;
                return Some(token);
            }
        }
        None
    }

    fn advance_on_any_of(&mut self, tts: Vec<TokenType>) -> Option<Token> {
        for tt in tts.into_iter() {
            if let Some(t) = self.advance_on(tt) {
                return Some(t);
            }
        }
        None
    }

    fn advance_on_any(&mut self) -> Token {
        self.index += 1;
        self.curr_line
            .get(self.index - 1)
            .map(|t| t.clone())
            .unwrap()
    }

    fn error(&self, msg: &str) -> String {
        self.current_token()
            .map(|t| t.error(msg))
            .unwrap_or(self.previous_token().error(msg))
    }

    fn current_token(&self) -> Option<Token> {
        self.curr_line.get(self.index).map(|t| t.clone())
    }

    /// # Panics
    fn previous_token(&self) -> Token {
        self.curr_line.get(self.index - 1).unwrap().clone()
    }
}
