use std::fmt::Display;

/// Represents all the types of Schnauzer UI tokens.
#[derive(Debug, Clone)]
pub enum TokenType {
    Locate,
    LocateNoScroll,
    Type,
    Click,
    Refresh,
    TryAgain,
    Screenshot,
    CatchError,
    ReadTo,
    Url,
    Press,
    Chill,
    Select,
    DragTo,
    Upload,
    AcceptAlert,
    DismissAlert,
    Under,
    UnderActiveElement,
    StringLiteral,
    If,
    Then,
    And,
    Variable,
    Save,
    As,
    Comment,
    Eof,
    Eol,
}

/// We are implementing display for `TokenType`, but this is just for printing error messages.
/// The token_type doesn't have enough information to display say, a string literal.
impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lexeme = match self {
            TokenType::Locate => "locate",
            TokenType::Type => "type",
            TokenType::Click => "click",
            TokenType::Refresh => "refresh",
            TokenType::TryAgain => "try-again",
            TokenType::Screenshot => "screenshot",
            TokenType::CatchError => "catch-error:",
            TokenType::ReadTo => "read-to",
            TokenType::StringLiteral => "quoted text",
            TokenType::If => "if",
            TokenType::Then => "then",
            TokenType::And => "and",
            TokenType::Variable => "a variable",
            TokenType::Eof => "eof",
            TokenType::Eol => "eol",
            TokenType::Save => "save",
            TokenType::As => "as",
            TokenType::Url => "url",
            TokenType::Comment => "a comment",
            TokenType::Press => "press",
            TokenType::Chill => "chill",
            TokenType::LocateNoScroll => "locate-no-scroll",
            TokenType::Select => "select",
            TokenType::DragTo => "drag-to",
            TokenType::Upload => "upload",
            TokenType::AcceptAlert => "accept-alert",
            TokenType::DismissAlert => "dismiss-alert",
            TokenType::Under => "under",
            TokenType::UnderActiveElement => "under-active-element",
        };

        write!(f, "{}", lexeme)
    }
}

impl PartialEq for TokenType {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

/// Represents a Schnauzer UI Token
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The type of the Token
    pub token_type: TokenType,

    /// The line the token was found on (for error reporting)
    pub line: usize,

    /// The String representation on the token
    pub lexeme: String,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lexeme)
    }
}

impl Token {
    /// Formatted error string for producing a parse error at a given token.
    pub fn error(&self, msg: impl std::fmt::Display) -> String {
        format!(
            "[Line {}]: Error at token \"{}\": {}",
            self.line, self.lexeme, msg
        )
    }
}

/// The purpose of the scanner is to transform a list of characters into a list of tokens.
/// # Example
/// ```
/// use schnauzer_ui::scanner::*;
///
/// let src = "locate \"username\" and type \"test@test.com\"";
/// let tokens: Vec<Token> = Scanner::from_src(src.to_owned()).scan();
/// ```
#[derive(Debug)]
pub struct Scanner {
    /// The source code as a String
    src: String,

    /// A buffer for collecting tokens as we scan the source code.
    tokens: Vec<Token>,

    /// The current line number in the source code.
    line: usize,

    /// Used to collect the pieces of a string literal that may have been split by whitespace
    string_literal_buffer: String,

    /// Keeps track of whether we are currently scanning between quotes
    in_quotes: bool,
}

impl Scanner {
    /// Constructor
    pub fn from_src(src: String) -> Self {
        Self {
            src,
            tokens: vec![],
            line: 0,
            string_literal_buffer: String::new(),
            in_quotes: false,
        }
    }

    /// Produces a vector of tokens from the provided source code.
    pub fn scan(&mut self) -> Vec<Token> {
        // Process a line at a time
        for stmt in self.src.clone().lines() {
            // Increment tracking for the current line of the source code
            self.line += 1;

            // Skip whitespace
            if stmt.trim().is_empty() {
                continue;
            }

            // Comments
            if stmt.trim().starts_with('#') {
                self.add_token(TokenType::Comment, stmt.to_owned());
                self.add_token(TokenType::Eol, "EOL".into());
                continue;
            }

            // Regular tokens
            for item in stmt.trim().split(' ') {
                if let Some(token) = self.resolve_token(item) {
                    self.tokens.push(token);
                }
            }

            // End of line token
            self.add_token(TokenType::Eol, "EOL".into());
        }

        // Add an end of file token
        self.add_token(TokenType::Eof, "EOF".into());
        self.tokens.clone()
    }

    /// Takes a lexeme (the string representation of a token) and tries to resolve it
    /// to a Schnauzer UI token.
    /// String literals are sometimes passed by the scan function in pieces (due to splitting on whitespace),
    /// so the function returns None while it is in the process of rejoining those string literals.
    fn resolve_token(&mut self, lexeme: &str) -> Option<Token> {
        match lexeme {
            // Commands
            "locate" if !self.in_quotes => Some(self.token(TokenType::Locate, "locate".into())),
            "type" if !self.in_quotes => Some(self.token(TokenType::Type, "type".into())),
            "click" if !self.in_quotes => Some(self.token(TokenType::Click, "click".into())),
            "refresh" if !self.in_quotes => Some(self.token(TokenType::Refresh, "refresh".into())),
            "try-again" if !self.in_quotes => {
                Some(self.token(TokenType::TryAgain, "try-again".into()))
            }
            "screenshot" if !self.in_quotes => {
                Some(self.token(TokenType::Screenshot, "screenshot".into()))
            }
            "catch-error:" if !self.in_quotes => {
                Some(self.token(TokenType::CatchError, "catch-error".into()))
            }
            "if" if !self.in_quotes => Some(self.token(TokenType::If, "if".into())),
            "then" if !self.in_quotes => Some(self.token(TokenType::Then, "then".into())),
            "and" if !self.in_quotes => Some(self.token(TokenType::And, "and".into())),
            "read-to" if !self.in_quotes => Some(self.token(TokenType::ReadTo, "read-to".into())),
            "save" if !self.in_quotes => Some(self.token(TokenType::Save, "save".into())),
            "as" if !self.in_quotes => Some(self.token(TokenType::As, "as".into())),
            "url" if !self.in_quotes => Some(self.token(TokenType::Url, "url".into())),
            "press" if !self.in_quotes => Some(self.token(TokenType::Press, "press".into())),
            "chill" if !self.in_quotes => Some(self.token(TokenType::Chill, "chill".into())),
            "locate-no-scroll" if !self.in_quotes => {
                Some(self.token(TokenType::LocateNoScroll, "locate-no-scroll".into()))
            }
            "select" if !self.in_quotes => Some(self.token(TokenType::Select, "select".into())),
            "drag-to" if !self.in_quotes => Some(self.token(TokenType::DragTo, "drag-to".into())),
            "upload" if !self.in_quotes => Some(self.token(TokenType::Upload, "upload".into())),
            "accept-alert" if !self.in_quotes => {
                Some(self.token(TokenType::AcceptAlert, "accept-alert".into()))
            }
            "dismiss-alert" if !self.in_quotes => {
                Some(self.token(TokenType::DismissAlert, "dismiss-alert".into()))
            }
            "under" if !self.in_quotes => Some(self.token(TokenType::Under, "under".into())),
            "under-active-element" if !self.in_quotes => {
                Some(self.token(TokenType::UnderActiveElement, "under-active-element".into()))
            }
            // If we get an entire string literal, stript the quotes and construct the token
            word if word.starts_with('\"')
                && word.ends_with('\"')
                && !self.in_quotes
                && word.len() > 1 =>
            {
                // Strip the quotes
                let word = word
                    .strip_prefix('\"')
                    .unwrap()
                    .strip_suffix('\"')
                    .unwrap()
                    .to_owned();

                Some(self.token(TokenType::StringLiteral, word))
            }

            // If we get the first part of a string, switch to string literal building mode
            word if word.starts_with('\"') && !self.in_quotes => {
                self.in_quotes = true;

                // Strip the front quote.
                // This unwrap is safe because we checked the string began with a quote in the match guard,
                let without_prefix_quote = word.to_owned().strip_prefix('\"').unwrap().to_owned();

                // Add the beginning of the literal to the buffer.
                self.string_literal_buffer.push_str(&without_prefix_quote);

                // Add back the whitespace to the string
                self.string_literal_buffer.push(' ');

                None
            }

            // If we get the last part of the string literal
            word if word.ends_with('\"') && self.in_quotes => {
                self.in_quotes = false;

                // Strip the end quote
                // This unwrap is safe because we check that word ends in a quote in the match guard.
                let without_end_quote = word.to_owned().strip_suffix('\"').unwrap().to_owned();

                // Add the end of the literal to the string literal buffer.
                self.string_literal_buffer.push_str(&without_end_quote);

                // Clear the buffer and return the string literal
                let res = self.string_literal_buffer.clone();
                self.string_literal_buffer.clear();
                Some(self.token(TokenType::StringLiteral, res))
            }

            // If we get part of the middle of the string literal
            word if self.in_quotes => {
                // Add the part to the string literal buffer
                self.string_literal_buffer.push_str(word);
                // add the whitespace back
                self.string_literal_buffer.push(' ');
                None
            }

            // If it's not a string literal or a keyword, it's a variable.
            word => Some(self.token(TokenType::Variable, word.into())),
        }
    }

    fn add_token(&mut self, tt: TokenType, lexeme: String) {
        self.tokens.push(self.token(tt, lexeme));
    }

    fn token(&self, tt: TokenType, lexeme: String) -> Token {
        Token {
            token_type: tt,
            line: self.line,
            lexeme,
        }
    }
}
