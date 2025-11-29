// Token types for the WTLang lexer
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Page,
    Table,
    Title,
    Subtitle,
    Button,
    Section,
    Text,
    Let,
    Function,
    External,
    From,
    Import,
    Test,
    Mock,
    Assert,
    If,
    Else,
    Forall,
    In,
    Return,
    Filter,
    Single,
    Multi,
    
    // Types
    Int,
    Float,
    String,
    Date,
    Currency,
    Bool,
    Number,     // Alias for Float
    
    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    
    // Identifiers
    Identifier(String),
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equals,
    NotEquals,
    LessThan,
    LessThanEquals,
    GreaterThan,
    GreaterThanEquals,
    And,
    Or,
    Not,
    Arrow,          // ->
    FatArrow,       // =>
    Assign,         // =
    
    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    Semicolon,
    Dot,
    Underscore,
    
    // Special
    Eof,
    Newline,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize) -> Self {
        Token { token_type, line, column }
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            self.skip_whitespace();
            if self.is_at_end() {
                break;
            }
            
            let token = self.next_token()?;
            tokens.push(token);
        }
        
        tokens.push(Token::new(TokenType::Eof, self.line, self.column));
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, String> {
        let start_line = self.line;
        let start_column = self.column;
        
        let ch = self.current_char();
        
        // Single-line comments
        if ch == '/' && self.peek() == Some('/') {
            self.skip_comment();
            self.skip_whitespace();  // Skip whitespace after comment
            return self.next_token();
        }
        
        // String literals
        if ch == '"' {
            return self.read_string();
        }
        
        // Numbers
        if ch.is_ascii_digit() {
            return self.read_number();
        }
        
        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            return self.read_identifier();
        }
        
        // Operators and delimiters
        let token_type = match ch {
            '+' => { self.advance(); TokenType::Plus },
            '*' => { self.advance(); TokenType::Star },
            '/' => { self.advance(); TokenType::Slash },
            '%' => { self.advance(); TokenType::Percent },
            '(' => { self.advance(); TokenType::LeftParen },
            ')' => { self.advance(); TokenType::RightParen },
            '{' => { self.advance(); TokenType::LeftBrace },
            '}' => { self.advance(); TokenType::RightBrace },
            '[' => { self.advance(); TokenType::LeftBracket },
            ']' => { self.advance(); TokenType::RightBracket },
            ',' => { self.advance(); TokenType::Comma },
            ':' => { self.advance(); TokenType::Colon },
            ';' => { self.advance(); TokenType::Semicolon },
            '.' => { self.advance(); TokenType::Dot },
            '_' => { self.advance(); TokenType::Underscore },
            
            '-' => {
                self.advance();
                if self.current_char() == '>' {
                    self.advance();
                    TokenType::Arrow
                } else {
                    TokenType::Minus
                }
            },
            
            '=' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::Equals
                } else if self.current_char() == '>' {
                    self.advance();
                    TokenType::FatArrow
                } else {
                    TokenType::Assign
                }
            },
            
            '!' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::NotEquals
                } else {
                    TokenType::Not
                }
            },
            
            '<' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::LessThanEquals
                } else {
                    TokenType::LessThan
                }
            },
            
            '>' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::GreaterThanEquals
                } else {
                    TokenType::GreaterThan
                }
            },
            
            '&' => {
                self.advance();
                if self.current_char() == '&' {
                    self.advance();
                    TokenType::And
                } else {
                    return Err(format!("Unexpected character '&' at line {}, column {}", start_line, start_column));
                }
            },
            
            '|' => {
                self.advance();
                if self.current_char() == '|' {
                    self.advance();
                    TokenType::Or
                } else {
                    return Err(format!("Unexpected character '|' at line {}, column {}", start_line, start_column));
                }
            },
            
            _ => {
                return Err(format!("Unexpected character '{}' at line {}, column {}", ch, start_line, start_column));
            }
        };
        
        Ok(Token::new(token_type, start_line, start_column))
    }

    fn read_string(&mut self) -> Result<Token, String> {
        let start_line = self.line;
        let start_column = self.column;
        
        self.advance(); // Skip opening quote
        let mut value = String::new();
        
        while !self.is_at_end() && self.current_char() != '"' {
            if self.current_char() == '\\' {
                self.advance();
                if !self.is_at_end() {
                    let escaped = match self.current_char() {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        _ => self.current_char(),
                    };
                    value.push(escaped);
                    self.advance();
                }
            } else {
                value.push(self.current_char());
                self.advance();
            }
        }
        
        if self.is_at_end() {
            return Err(format!("Unterminated string at line {}, column {}", start_line, start_column));
        }
        
        self.advance(); // Skip closing quote
        Ok(Token::new(TokenType::StringLiteral(value), start_line, start_column))
    }

    fn read_number(&mut self) -> Result<Token, String> {
        let start_line = self.line;
        let start_column = self.column;
        
        let mut value = String::new();
        let mut is_float = false;
        
        while !self.is_at_end() && (self.current_char().is_ascii_digit() || self.current_char() == '.') {
            if self.current_char() == '.' {
                if is_float {
                    break; // Second dot, stop here
                }
                is_float = true;
            }
            value.push(self.current_char());
            self.advance();
        }
        
        if is_float {
            let num = value.parse::<f64>()
                .map_err(|_| format!("Invalid float '{}' at line {}, column {}", value, start_line, start_column))?;
            Ok(Token::new(TokenType::FloatLiteral(num), start_line, start_column))
        } else {
            let num = value.parse::<i64>()
                .map_err(|_| format!("Invalid integer '{}' at line {}, column {}", value, start_line, start_column))?;
            Ok(Token::new(TokenType::IntLiteral(num), start_line, start_column))
        }
    }

    fn read_identifier(&mut self) -> Result<Token, String> {
        let start_line = self.line;
        let start_column = self.column;
        
        let mut value = String::new();
        
        while !self.is_at_end() && (self.current_char().is_alphanumeric() || self.current_char() == '_') {
            value.push(self.current_char());
            self.advance();
        }
        
        let token_type = match value.as_str() {
            "page" => TokenType::Page,
            "table" => TokenType::Table,
            "title" => TokenType::Title,
            "subtitle" => TokenType::Subtitle,
            "button" => TokenType::Button,
            "section" => TokenType::Section,
            "text" => TokenType::Text,
            "let" => TokenType::Let,
            "function" => TokenType::Function,
            "external" => TokenType::External,
            "from" => TokenType::From,
            "import" => TokenType::Import,
            "test" => TokenType::Test,
            "mock" => TokenType::Mock,
            "assert" => TokenType::Assert,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "forall" => TokenType::Forall,
            "in" => TokenType::In,
            "return" => TokenType::Return,
            "filter" => TokenType::Filter,
            "single" => TokenType::Single,
            "multi" => TokenType::Multi,
            "int" => TokenType::Int,
            "float" => TokenType::Float,
            "string" => TokenType::String,
            "date" => TokenType::Date,
            "currency" => TokenType::Currency,
            "bool" => TokenType::Bool,
            "number" => TokenType::Number,
            "true" => TokenType::BoolLiteral(true),
            "false" => TokenType::BoolLiteral(false),
            _ => TokenType::Identifier(value),
        };
        
        Ok(Token::new(token_type, start_line, start_column))
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            let ch = self.current_char();
            if ch == ' ' || ch == '\t' || ch == '\r' || ch == '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        while !self.is_at_end() && self.current_char() != '\n' {
            self.advance();
        }
    }

    fn current_char(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.position]
        }
    }

    fn peek(&self) -> Option<char> {
        if self.position + 1 < self.input.len() {
            Some(self.input[self.position + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            if self.input[self.position] == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}
