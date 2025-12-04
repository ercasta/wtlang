// Token types for the WTLang lexer
use crate::errors::{ErrorCode, DiagnosticBag, Location};

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
    Where,
    By,
    Asc,
    Desc,
    Key,
    Ref,
    
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
    diagnostics: DiagnosticBag,
    source: String,  // Keep source for context in error messages
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            diagnostics: DiagnosticBag::new(),
            source: input.to_string(),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, DiagnosticBag> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            self.skip_whitespace();
            if self.is_at_end() {
                break;
            }
            
            match self.next_token() {
                Ok(token) => tokens.push(token),
                Err(_) => {
                    // Error already added to diagnostics, continue to find more errors
                    self.advance(); // Skip the problematic character
                }
            }
        }
        
        tokens.push(Token::new(TokenType::Eof, self.line, self.column));
        
        if self.diagnostics.has_errors() {
            Err(self.diagnostics.clone())
        } else {
            Ok(tokens)
        }
    }
    
    fn add_error(&mut self, code: ErrorCode, message: String, line: usize, column: usize) {
        let location = Location::new(line, column);
        self.diagnostics.add_error(code, message, location);
    }

    fn next_token(&mut self) -> Result<Token, ()> {
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
                    self.add_error(
                        ErrorCode::E1003,
                        "Unexpected character '&', did you mean '&&'?".to_string(),
                        start_line,
                        start_column
                    );
                    return Err(());
                }
            },
            
            '|' => {
                self.advance();
                if self.current_char() == '|' {
                    self.advance();
                    TokenType::Or
                } else {
                    self.add_error(
                        ErrorCode::E1003,
                        "Unexpected character '|', did you mean '||'?".to_string(),
                        start_line,
                        start_column
                    );
                    return Err(());
                }
            },
            
            _ => {
                self.add_error(
                    ErrorCode::E1003,
                    format!("Invalid character '{}'", ch),
                    start_line,
                    start_column
                );
                return Err(());
            }
        };
        
        Ok(Token::new(token_type, start_line, start_column))
    }

    fn read_string(&mut self) -> Result<Token, ()> {
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
            self.add_error(
                ErrorCode::E1001,
                "Unterminated string literal".to_string(),
                start_line,
                start_column
            );
            return Err(());
        }
        
        self.advance(); // Skip closing quote
        Ok(Token::new(TokenType::StringLiteral(value), start_line, start_column))
    }

    fn read_number(&mut self) -> Result<Token, ()> {
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
            match value.parse::<f64>() {
                Ok(num) => Ok(Token::new(TokenType::FloatLiteral(num), start_line, start_column)),
                Err(_) => {
                    self.add_error(
                        ErrorCode::E1002,
                        format!("Invalid float '{}'", value),
                        start_line,
                        start_column
                    );
                    Err(())
                }
            }
        } else {
            match value.parse::<i64>() {
                Ok(num) => Ok(Token::new(TokenType::IntLiteral(num), start_line, start_column)),
                Err(_) => {
                    self.add_error(
                        ErrorCode::E1002,
                        format!("Invalid integer '{}'", value),
                        start_line,
                        start_column
                    );
                    Err(())
                }
            }
        }
    }

    fn read_identifier(&mut self) -> Result<Token, ()> {
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
            "where" => TokenType::Where,
            "by" => TokenType::By,
            "asc" => TokenType::Asc,
            "desc" => TokenType::Desc,
            "key" => TokenType::Key,
            "ref" => TokenType::Ref,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("page table from display button section");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Page);
        assert_eq!(tokens[1].token_type, TokenType::Table);
        assert_eq!(tokens[2].token_type, TokenType::From);
        assert_eq!(tokens[3].token_type, TokenType::Identifier("display".to_string()));
        assert_eq!(tokens[4].token_type, TokenType::Button);
        assert_eq!(tokens[5].token_type, TokenType::Section);
    }

    #[test]
    fn test_type_keywords() {
        let mut lexer = Lexer::new("number string bool date");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[1].token_type, TokenType::String);
        assert_eq!(tokens[2].token_type, TokenType::Bool);
        assert_eq!(tokens[3].token_type, TokenType::Date);
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("myVar my_var MyClass _private");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Identifier("myVar".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::Identifier("my_var".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Identifier("MyClass".to_string()));
        assert_eq!(tokens[3].token_type, TokenType::Identifier("_private".to_string()));
    }

    #[test]
    fn test_integer_literals() {
        let mut lexer = Lexer::new("0 42 1000");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::IntLiteral(0));
        assert_eq!(tokens[1].token_type, TokenType::IntLiteral(42));
        assert_eq!(tokens[2].token_type, TokenType::IntLiteral(1000));
    }

    #[test]
    fn test_float_literals() {
        let mut lexer = Lexer::new("3.14 0.5 10.0");
        let tokens = lexer.tokenize().unwrap();
        
        match tokens[0].token_type {
            TokenType::FloatLiteral(val) => assert!((val - 3.14).abs() < 0.001),
            _ => panic!("Expected float literal"),
        }
        match tokens[1].token_type {
            TokenType::FloatLiteral(val) => assert!((val - 0.5).abs() < 0.001),
            _ => panic!("Expected float literal"),
        }
    }

    #[test]
    fn test_string_literals() {
        let mut lexer = Lexer::new(r#""Hello, World!" "test" """#);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::StringLiteral("Hello, World!".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::StringLiteral("test".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::StringLiteral("".to_string()));
    }

    #[test]
    fn test_boolean_literals() {
        let mut lexer = Lexer::new("true false");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::BoolLiteral(true));
        assert_eq!(tokens[1].token_type, TokenType::BoolLiteral(false));
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / % == != < <= > >= = ->");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Plus);
        assert_eq!(tokens[1].token_type, TokenType::Minus);
        assert_eq!(tokens[2].token_type, TokenType::Star);
        assert_eq!(tokens[3].token_type, TokenType::Slash);
        assert_eq!(tokens[4].token_type, TokenType::Percent);
        assert_eq!(tokens[5].token_type, TokenType::Equals);
        assert_eq!(tokens[6].token_type, TokenType::NotEquals);
        assert_eq!(tokens[7].token_type, TokenType::LessThan);
        assert_eq!(tokens[8].token_type, TokenType::LessThanEquals);
        assert_eq!(tokens[9].token_type, TokenType::GreaterThan);
        assert_eq!(tokens[10].token_type, TokenType::GreaterThanEquals);
        assert_eq!(tokens[11].token_type, TokenType::Assign);
        assert_eq!(tokens[12].token_type, TokenType::Arrow);
    }

    #[test]
    fn test_delimiters() {
        let mut lexer = Lexer::new("( ) { } [ ] , : ; .");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::LeftParen);
        assert_eq!(tokens[1].token_type, TokenType::RightParen);
        assert_eq!(tokens[2].token_type, TokenType::LeftBrace);
        assert_eq!(tokens[3].token_type, TokenType::RightBrace);
        assert_eq!(tokens[4].token_type, TokenType::LeftBracket);
        assert_eq!(tokens[5].token_type, TokenType::RightBracket);
        assert_eq!(tokens[6].token_type, TokenType::Comma);
        assert_eq!(tokens[7].token_type, TokenType::Colon);
        assert_eq!(tokens[8].token_type, TokenType::Semicolon);
        assert_eq!(tokens[9].token_type, TokenType::Dot);
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("page // this is a comment\ntable");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Page);
        assert_eq!(tokens[1].token_type, TokenType::Table);
        assert_eq!(tokens.len(), 3); // page, table, EOF
    }

    #[test]
    fn test_position_tracking() {
        let mut lexer = Lexer::new("page\ntable");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);
        assert_eq!(tokens[1].line, 2);
        assert_eq!(tokens[1].column, 1);
    }

    #[test]
    fn test_unterminated_string() {
        let mut lexer = Lexer::new(r#""unterminated"#);
        let result = lexer.tokenize();
        
        assert!(result.is_err());
        let diag = result.unwrap_err();
        assert_eq!(diag.diagnostics().len(), 1);
        assert!(diag.format_all().contains("Unterminated string"));
    }

    #[test]
    fn test_complex_expression() {
        let mut lexer = Lexer::new("let x: number = 42 + 3.14");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Let);
        assert_eq!(tokens[1].token_type, TokenType::Identifier("x".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Colon);
        assert_eq!(tokens[3].token_type, TokenType::Number);
        assert_eq!(tokens[4].token_type, TokenType::Assign);
        assert_eq!(tokens[5].token_type, TokenType::IntLiteral(42));
        assert_eq!(tokens[6].token_type, TokenType::Plus);
        match tokens[7].token_type {
            TokenType::FloatLiteral(val) => assert!((val - 3.14).abs() < 0.001),
            _ => panic!("Expected float literal"),
        }
    }

    #[test]
    fn test_function_definition() {
        let mut lexer = Lexer::new("function add(x: number, y: number) -> number");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Function);
        assert_eq!(tokens[1].token_type, TokenType::Identifier("add".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::LeftParen);
        assert_eq!(tokens[3].token_type, TokenType::Identifier("x".to_string()));
        assert_eq!(tokens[4].token_type, TokenType::Colon);
        assert_eq!(tokens[5].token_type, TokenType::Number);
    }

    #[test]
    fn test_whitespace_handling() {
        let mut lexer = Lexer::new("  \t\n  page  \n\t  table  ");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Page);
        assert_eq!(tokens[1].token_type, TokenType::Table);
        assert_eq!(tokens.len(), 3); // page, table, EOF
    }

    #[test]
    fn test_filter_keywords() {
        let mut lexer = Lexer::new("filter single multi");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Filter);
        assert_eq!(tokens[1].token_type, TokenType::Single);
        assert_eq!(tokens[2].token_type, TokenType::Multi);
    }

    #[test]
    fn test_control_flow_keywords() {
        let mut lexer = Lexer::new("if else forall in return");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::If);
        assert_eq!(tokens[1].token_type, TokenType::Else);
        assert_eq!(tokens[2].token_type, TokenType::Forall);
        assert_eq!(tokens[3].token_type, TokenType::In);
        assert_eq!(tokens[4].token_type, TokenType::Return);
    }

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Eof);
    }

    #[test]
    fn test_only_whitespace() {
        let mut lexer = Lexer::new("   \n\t  \n  ");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Eof);
    }
}
