/// Represents all the types of Schnauzer UI tokens.
#[derive(Debug, Clone)]
pub enum TokenType {
    // Commands
    Locate,
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

    // Literals (the associated string is the string literal)
    String(String),

    // Combinators
    If,
    Then,
    And,

    // Variable (the associated string is the variable name)
    Variable(String),
    Save,
    As,

    // Comment token
    Comment(String),

    // EOF and EOL
    Eof,
    Eol,
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
}

impl std::fmt::Display for TokenType {
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
            TokenType::String(s) => s,
            TokenType::If => "if",
            TokenType::Then => "then",
            TokenType::And => "and",
            TokenType::Variable(v) => v,
            TokenType::Eof => "eof",
            TokenType::Eol => "eol",
            TokenType::Save => "save",
            TokenType::As => "as",
            TokenType::Url => "url",
            TokenType::Comment(s) => s,
            TokenType::Press => "press",
            TokenType::Chill => "chill",
        };

        write!(f, "{}", lexeme)
    }
}

impl Token {
    pub fn error(&self, msg: impl std::fmt::Display) -> String {
        format!(
            "[Line {}]: Error at {}: {}",
            self.line, self.token_type, msg
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

            if stmt.trim().starts_with("#") {
                self.tokens
                    .push(self.token(TokenType::Comment(stmt.to_owned())));
                self.tokens.push(self.token(TokenType::Eol));
                continue;
            }

            for item in stmt.trim().split(' ') {
                if let Some(token) = self.resolve_token(item) {
                    self.tokens.push(token);
                }
            }

            self.tokens.push(self.token(TokenType::Eol));
        }

        // Add an end of file token
        self.tokens.push(self.token(TokenType::Eof));
        self.tokens.clone()
    }

    /// Takes a lexeme (the string representation of a token) and tries to resolve it
    /// to a Schnauzer UI token.
    /// String literals are sometimes passed by the scan function in pieces (due to splitting on whitespace),
    /// so the function returns None while it is in the process of rejoining those string literals.
    pub fn resolve_token(&mut self, lexeme: &str) -> Option<Token> {
        match lexeme {
            // Commands
            "locate" => Some(self.token(TokenType::Locate)),
            "type" => Some(self.token(TokenType::Type)),
            "click" => Some(self.token(TokenType::Click)),
            "refresh" => Some(self.token(TokenType::Refresh)),
            "try-again" => Some(self.token(TokenType::TryAgain)),
            "screenshot" => Some(self.token(TokenType::Screenshot)),
            "catch-error:" => Some(self.token(TokenType::CatchError)),
            "if" => Some(self.token(TokenType::If)),
            "then" => Some(self.token(TokenType::Then)),
            "and" => Some(self.token(TokenType::And)),
            "read-to" => Some(self.token(TokenType::ReadTo)),
            "save" => Some(self.token(TokenType::Save)),
            "as" => Some(self.token(TokenType::As)),
            "url" => Some(self.token(TokenType::Url)),
            "press" => Some(self.token(TokenType::Press)),

            // If we get an entire string literal, stript the quotes and construct the token
            word if word.starts_with("\"")
                && word.ends_with("\"")
                && !self.in_quotes
                && word.len() > 1 =>
            {
                // Strip the quotes
                let word = word
                    .strip_prefix("\"")
                    .unwrap()
                    .strip_suffix("\"")
                    .unwrap()
                    .to_owned();

                Some(self.token(TokenType::String(word)))
            }

            // If we get the first part of a string, switch to string literal building mode
            word if word.starts_with("\"") && !self.in_quotes => {
                self.in_quotes = true;

                // Strip the front quote.
                // This unwrap is safe because we checked the string began with a quote in the match guard,
                let without_prefix_quote = word.to_owned().strip_prefix("\"").unwrap().to_owned();

                // Add the beginning of the literal to the buffer.
                self.string_literal_buffer.push_str(&without_prefix_quote);

                // Add back the whitespace to the string
                self.string_literal_buffer.push(' ');

                None
            }

            // If we get the last part of the string literal
            word if word.ends_with("\"") && self.in_quotes => {
                self.in_quotes = false;

                // Strip the end quote
                // This unwrap is safe because we check that word ends in a quote in the match guard.
                let without_end_quote = word.to_owned().strip_suffix("\"").unwrap().to_owned();

                // Add the end of the literal to the string literal buffer.
                self.string_literal_buffer.push_str(&without_end_quote);

                // Clear the buffer and return the string literal
                let res = self.string_literal_buffer.clone();
                self.string_literal_buffer.clear();
                Some(self.token(TokenType::String(res)))
            }

            // If we get part of the middle of the string literal
            word if self.in_quotes => {
                // Add the part to the string literal buffer
                self.string_literal_buffer.push_str(word);
                // add the whitespace back
                self.string_literal_buffer.push(' ');
                None
            }
            word => Some(self.token(TokenType::Variable(word.to_owned()))),
        }
    }

    fn token(&self, tt: TokenType) -> Token {
        Token {
            token_type: tt,
            line: self.line,
        }
    }
}
