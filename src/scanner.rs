//! The purpose of the scanner is to transform a list of characters into a list of tokens.

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Commands
    Locate,
    Type,
    Click,
    Report,
    Refresh,
    TryAgain,
    Screenshot,
    HadError,

    // Literals
    String(String),

    // Combinators
    If,
    Then,
    And,


    // Variable
    Variable(String),

    // EOF
    Eof
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize
}

pub struct Scanner {
    src: String,
    tokens: Vec<Token>,
    line: usize,
}

impl Scanner { 
    pub fn new(src: String) -> Self {
        Self {
            src,
            tokens: vec![],
            line: 0,
        }
    }

    pub fn scan(&mut self) -> Vec<Token> {
        for line in self.src.lines() {
            self.line += 1;

            // Replace whitespace between quotes with ~
            let mut buf = String::new();
            let mut currently_between_quotes = false;
            for c in line.chars() {
                if c == '"' {
                    currently_between_quotes = !currently_between_quotes;
                }

                if currently_between_quotes && c == ' ' { 
                    buf.push('~');
                } else {
                    buf.push(c);
                }
            }
            
            for item in buf.split_whitespace() {
                let token = self.resolve_token(item);
                self.tokens.push(token);
            }
        }

        self.tokens.push(self.token(TokenType::Eof));
        self.tokens.clone()
    }

    pub fn resolve_token(&self, lexeme: &str) -> Token {
        match lexeme {
            "locate" => self.token(TokenType::Locate),
            "type" => self.token(TokenType::Type),
            "click" => self.token(TokenType::Click),
            "report" => self.token(TokenType::Report),
            "refresh" => self.token(TokenType::Refresh),
            "try-again" => self.token(TokenType::TryAgain),
            "screenshot" => self.token(TokenType::Screenshot),
            "had-error" => self.token(TokenType::HadError),
            "if" => self.token(TokenType::If),
            "then" => self.token(TokenType::Then),
            "and" => self.token(TokenType::And),
            word if word.starts_with("\"") && word.ends_with("\"") => {

                // Strip quotes
                let mut word: String = word.chars().skip(1).take(word.len() - 2).collect();

                // Re-add the previously removed whitespace
                word = word.replace("~", " ");

                // Returna string token
                let tt = TokenType::String(word);
                self.token(tt)
            },
            word => self.token(TokenType::Variable(word.to_owned()))
        }
    }

    pub fn token(&self, tt: TokenType) -> Token {
        Token { token_type: tt, line: self.line }
    }
}