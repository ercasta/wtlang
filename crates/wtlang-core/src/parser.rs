// Parser for WTLang
use crate::ast::*;
use crate::lexer::{Token, TokenType};
use crate::errors::{ErrorCode, DiagnosticBag, Location};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    diagnostics: DiagnosticBag,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { 
            tokens, 
            current: 0,
            diagnostics: DiagnosticBag::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Program, DiagnosticBag> {
        let mut items = Vec::new();
        
        while !self.is_at_end() {
            match self.parse_program_item() {
                Ok(item) => items.push(item),
                Err(_) => {
                    // Error already added to diagnostics
                    // Try to recover by skipping to next top-level item
                    self.synchronize();
                }
            }
        }
        
        if self.diagnostics.has_errors() {
            Err(self.diagnostics.clone())
        } else {
            Ok(Program { items })
        }
    }
    
    fn synchronize(&mut self) {
        // Skip tokens until we find a likely start of a new item
        while !self.is_at_end() {
            if matches!(
                self.peek().token_type,
                TokenType::Page | TokenType::Table | TokenType::Function | TokenType::External | TokenType::Test
            ) {
                return;
            }
            self.advance();
        }
    }
    
    fn add_error(&mut self, code: ErrorCode, message: String) {
        let token = self.peek();
        let location = Location::new(token.line, token.column);
        self.diagnostics.add_error(code, message, location);
    }

    fn parse_program_item(&mut self) -> Result<ProgramItem, ()> {
        match &self.peek().token_type {
            TokenType::Table => Ok(ProgramItem::TableDef(self.parse_table_def()?)),
            TokenType::Page => Ok(ProgramItem::Page(self.parse_page()?)),
            TokenType::Function => Ok(ProgramItem::FunctionDef(self.parse_function_def()?)),
            TokenType::External => Ok(ProgramItem::ExternalFunction(self.parse_external_function()?)),
            TokenType::Test => Ok(ProgramItem::Test(self.parse_test()?)),
            _ => {
                self.add_error(
                    ErrorCode::E2001,
                    format!("Expected table, page, function, external, or test, got {:?}", self.peek().token_type)
                );
                Err(())
            }
        }
    }

    fn parse_table_def(&mut self) -> Result<TableDef, ()> {
        self.expect(TokenType::Table)?;
        let name = self.expect_identifier()?;
        self.expect(TokenType::LeftBrace)?;
        
        let mut fields = Vec::new();
        while !self.check(&TokenType::RightBrace) {
            fields.push(self.parse_field()?);
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(TableDef { name, fields })
    }

    fn parse_field(&mut self) -> Result<Field, ()> {
        let name = self.expect_identifier()?;
        self.expect(TokenType::Colon)?;
        let field_type = self.parse_type()?;
        
        let mut constraints = Vec::new();
        if self.check(&TokenType::LeftBracket) {
            self.advance();
            constraints = self.parse_constraints()?;
            self.expect(TokenType::RightBracket)?;
        }
        
        // Consume optional trailing comma
        if self.check(&TokenType::Comma) {
            self.advance();
        }
        
        Ok(Field { name, field_type, constraints })
    }

    fn parse_type(&mut self) -> Result<Type, ()> {
        let token = self.advance().clone();
        match &token.token_type {
            TokenType::Int => Ok(Type::Int),
            TokenType::Float => Ok(Type::Float),
            TokenType::Number => Ok(Type::Float),  // number is alias for float
            TokenType::String => Ok(Type::String),
            TokenType::Text => Ok(Type::String),  // text keyword also valid as type
            TokenType::Date => Ok(Type::Date),
            TokenType::Currency => Ok(Type::Currency),
            TokenType::Bool => Ok(Type::Bool),
            TokenType::Filter => Ok(Type::Filter),  // filter type
            TokenType::Ref => {
                // ref TableName
                let table_name = self.expect_identifier()?;
                Ok(Type::Ref(table_name))
            }
            _ => {
                self.add_error(
                    ErrorCode::E2003,
                    format!("Expected type, got {:?}", token.token_type)
                );
                Err(())
            }
        }
    }

    fn parse_constraints(&mut self) -> Result<Vec<Constraint>, ()> {
        let mut constraints = Vec::new();
        
        loop {
            // Check for keyword constraints first
            let constraint = match &self.peek().token_type {
                TokenType::Key => {
                    self.advance();
                    Constraint::Key
                }
                TokenType::Identifier(ident) => {
                    let ident_str = ident.clone();
                    self.advance();
                    match ident_str.as_str() {
                        "unique" => Constraint::Unique,
                        "non_null" => Constraint::NonNull,
                        _ => {
                            self.add_error(
                                ErrorCode::E2012,
                                format!("Unknown constraint: {}", ident_str)
                            );
                            return Err(());
                        }
                    }
                }
                _ => {
                    self.add_error(
                        ErrorCode::E2012,
                        format!("Expected constraint, got {:?}", self.peek().token_type)
                    );
                    return Err(());
                }
            };
            
            constraints.push(constraint);
            
            if !self.check(&TokenType::Comma) {
                break;
            }
            self.advance();
        }
        
        Ok(constraints)
    }

    fn parse_page(&mut self) -> Result<Page, ()> {
        self.expect(TokenType::Page)?;
        let name = self.expect_identifier()?;
        self.expect(TokenType::LeftBrace)?;
        
        let mut statements = Vec::new();
        while !self.check(&TokenType::RightBrace) {
            statements.push(self.parse_statement()?);
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(Page { name, statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, ()> {
        match &self.peek().token_type {
            TokenType::Title => {
                self.advance();
                let text = self.expect_string()?;
                Ok(Statement::Title(text))
            },
            TokenType::Subtitle => {
                self.advance();
                let text = self.expect_string()?;
                Ok(Statement::Subtitle(text))
            },
            TokenType::Text => {
                self.advance();
                let text = self.expect_string()?;
                Ok(Statement::Text(text))
            },
            TokenType::Button => {
                self.advance();
                let label = self.expect_string()?;
                self.expect(TokenType::LeftBrace)?;
                let mut body = Vec::new();
                while !self.check(&TokenType::RightBrace) {
                    body.push(self.parse_statement()?);
                }
                self.expect(TokenType::RightBrace)?;
                Ok(Statement::Button { label, body })
            },
            TokenType::Section => {
                self.advance();
                let title = self.expect_string()?;
                self.expect(TokenType::LeftBrace)?;
                let mut body = Vec::new();
                while !self.check(&TokenType::RightBrace) {
                    body.push(self.parse_statement()?);
                }
                self.expect(TokenType::RightBrace)?;
                Ok(Statement::Section { title, body })
            },
            TokenType::Let => {
                self.advance();
                let name = self.expect_identifier()?;
                
                // Check for optional type annotation
                let type_annotation = if self.check(&TokenType::Colon) {
                    self.advance();
                    Some(self.parse_type()?)
                } else {
                    None
                };
                
                // Check for optional initialization
                let value = if self.check(&TokenType::Assign) {
                    self.advance();
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                
                // Must have either type annotation or value (or both)
                if type_annotation.is_none() && value.is_none() {
                    self.add_error(
                        ErrorCode::E2004,
                        format!("Variable '{}' must have either a type annotation or an initializer", name)
                    );
                    return Err(());
                }
                
                Ok(Statement::Let { name, type_annotation, value })
            },
            TokenType::If => {
                self.advance();
                let condition = self.parse_expression()?;
                self.expect(TokenType::LeftBrace)?;
                let mut then_branch = Vec::new();
                while !self.check(&TokenType::RightBrace) {
                    then_branch.push(self.parse_statement()?);
                }
                self.expect(TokenType::RightBrace)?;
                
                let else_branch = if self.check(&TokenType::Else) {
                    self.advance();
                    self.expect(TokenType::LeftBrace)?;
                    let mut else_stmts = Vec::new();
                    while !self.check(&TokenType::RightBrace) {
                        else_stmts.push(self.parse_statement()?);
                    }
                    self.expect(TokenType::RightBrace)?;
                    Some(else_stmts)
                } else {
                    None
                };
                
                Ok(Statement::If { condition, then_branch, else_branch })
            },
            TokenType::Return => {
                self.advance();
                let value = self.parse_expression()?;
                Ok(Statement::Return(value))
            },
            TokenType::Identifier(_) => {
                // Could be assignment or function call
                let name_or_expr = self.parse_expression()?;
                
                // Check if it's an assignment (after identifier comes =)
                // For now, simple check: if expression is just an identifier and next token is Assign
                if let Expr::Identifier(name) = &name_or_expr {
                    if self.check(&TokenType::Assign) {
                        self.advance(); // consume =
                        let value = self.parse_expression()?;
                        return Ok(Statement::Assign { name: name.clone(), value });
                    }
                }
                
                // Otherwise it should be a function call
                if let Expr::FunctionCall(call) = name_or_expr {
                    Ok(Statement::FunctionCall(call))
                } else {
                    self.add_error(
                        ErrorCode::E2001,
                        "Expected function call or assignment".to_string()
                    );
                    Err(())
                }
            },
            _ => {
                self.add_error(
                    ErrorCode::E2001,
                    format!("Unexpected token in statement: {:?}", self.peek().token_type)
                );
                Err(())
            }
        }
    }

    fn parse_function_def(&mut self) -> Result<FunctionDef, ()> {
        self.expect(TokenType::Function)?;
        let name = self.expect_identifier()?;
        self.expect(TokenType::LeftParen)?;
        let params = self.parse_parameters()?;
        self.expect(TokenType::RightParen)?;
        self.expect(TokenType::Arrow)?;
        let return_type = self.parse_type()?;
        self.expect(TokenType::LeftBrace)?;
        
        let mut body = Vec::new();
        while !self.check(&TokenType::RightBrace) {
            body.push(self.parse_statement()?);
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(FunctionDef { name, params, return_type, body })
    }

    fn parse_external_function(&mut self) -> Result<ExternalFunction, ()> {
        self.expect(TokenType::External)?;
        self.expect(TokenType::Function)?;
        let name = self.expect_identifier()?;
        self.expect(TokenType::LeftParen)?;
        let params = self.parse_parameters()?;
        self.expect(TokenType::RightParen)?;
        self.expect(TokenType::Arrow)?;
        let return_type = self.parse_type()?;
        self.expect(TokenType::From)?;
        let module = self.expect_string()?;
        
        Ok(ExternalFunction { name, params, return_type, module })
    }

    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, ()> {
        let mut params = Vec::new();
        
        if self.check(&TokenType::RightParen) {
            return Ok(params);
        }
        
        loop {
            let name = self.expect_identifier()?;
            self.expect(TokenType::Colon)?;
            let param_type = self.parse_type()?;
            params.push(Parameter { name, param_type });
            
            if !self.check(&TokenType::Comma) {
                break;
            }
            self.advance();
        }
        
        Ok(params)
    }

    fn parse_test(&mut self) -> Result<Test, ()> {
        self.expect(TokenType::Test)?;
        let name = self.expect_string()?;
        self.expect(TokenType::LeftBrace)?;
        
        let mut body = Vec::new();
        while !self.check(&TokenType::RightBrace) {
            body.push(self.parse_statement()?);
        }
        
        self.expect(TokenType::RightBrace)?;
        Ok(Test { name, body })
    }

    fn parse_expression(&mut self) -> Result<Expr, ()> {
        self.parse_chain()
    }

    fn parse_chain(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_where_sort()?;
        
        while self.check(&TokenType::Arrow) {
            self.advance();
            let right = self.parse_where_sort()?;
            left = Expr::Chain {
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_where_sort(&mut self) -> Result<Expr, ()> {
        let mut expr = self.parse_or()?;
        
        loop {
            if self.check(&TokenType::Where) {
                // Parse: table where condition
                self.advance();
                let condition = self.parse_or()?;
                expr = Expr::Where {
                    table: Box::new(expr),
                    condition: Box::new(condition),
                };
            } else if self.check_identifier_value("sort") {
                // Parse: table sort by col1 [asc|desc], col2 [asc|desc], ...
                self.advance();
                self.expect(TokenType::By)?;
                
                let mut columns = Vec::new();
                loop {
                    let col_name = self.expect_identifier()?;
                    let ascending = if self.check(&TokenType::Asc) {
                        self.advance();
                        true
                    } else if self.check(&TokenType::Desc) {
                        self.advance();
                        false
                    } else {
                        true  // Default to ascending
                    };
                    columns.push(SortColumn { name: col_name, ascending });
                    
                    if !self.check(&TokenType::Comma) {
                        break;
                    }
                    self.advance();
                }
                
                expr = Expr::SortBy {
                    table: Box::new(expr),
                    columns,
                };
            } else {
                break;
            }
        }
        
        Ok(expr)
    }

    fn parse_or(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_and()?;
        
        while self.check(&TokenType::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinaryOp {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_equality()?;
        
        while self.check(&TokenType::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::BinaryOp {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_comparison()?;
        
        while self.check(&TokenType::Equals) || self.check(&TokenType::NotEquals) {
            let op = if self.check(&TokenType::Equals) {
                BinaryOp::Equal
            } else {
                BinaryOp::NotEqual
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_addition()?;
        
        while matches!(self.peek().token_type, 
            TokenType::LessThan | TokenType::LessThanEquals | 
            TokenType::GreaterThan | TokenType::GreaterThanEquals) {
            
            let op = match self.peek().token_type {
                TokenType::LessThan => BinaryOp::LessThan,
                TokenType::LessThanEquals => BinaryOp::LessThanEqual,
                TokenType::GreaterThan => BinaryOp::GreaterThan,
                TokenType::GreaterThanEquals => BinaryOp::GreaterThanEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_addition()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_multiplication()?;
        
        while self.check(&TokenType::Plus) || self.check(&TokenType::Minus) {
            let op = if self.check(&TokenType::Plus) {
                BinaryOp::Add
            } else {
                BinaryOp::Subtract
            };
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_unary()?;
        
        while self.check(&TokenType::Star) || self.check(&TokenType::Slash) || self.check(&TokenType::Percent) {
            let op = match self.peek().token_type {
                TokenType::Star => BinaryOp::Multiply,
                TokenType::Slash => BinaryOp::Divide,
                TokenType::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ()> {
        if self.check(&TokenType::Not) || self.check(&TokenType::Minus) {
            let op = if self.check(&TokenType::Not) {
                UnaryOp::Not
            } else {
                UnaryOp::Negate
            };
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expr::UnaryOp {
                op,
                operand: Box::new(operand),
            });
        }
        
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, ()> {
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.check(&TokenType::Dot) {
                self.advance();
                let field = self.expect_identifier()?;
                expr = Expr::FieldAccess {
                    object: Box::new(expr),
                    field,
                };
            } else if self.check(&TokenType::LeftBracket) {
                self.advance();
                
                // Check if it's column selection [col1, col2] or index [expr]
                // Column selection starts with identifier and may have commas
                if self.check_identifier() {
                    // Try parsing as column selection
                    let first_col = self.expect_identifier()?;
                    
                    if self.check(&TokenType::Comma) {
                        // Multiple columns: definitely column selection
                        let mut columns = vec![first_col];
                        while self.check(&TokenType::Comma) {
                            self.advance();
                            columns.push(self.expect_identifier()?);
                        }
                        self.expect(TokenType::RightBracket)?;
                        expr = Expr::ColumnSelect {
                            table: Box::new(expr),
                            columns,
                        };
                    } else if self.check(&TokenType::RightBracket) {
                        // Single column: table[col]
                        self.advance();
                        expr = Expr::ColumnSelect {
                            table: Box::new(expr),
                            columns: vec![first_col],
                        };
                    } else {
                        // Something else after identifier - not column selection
                        // This is actually an error, but we'll just fail
                        self.add_error(
                            ErrorCode::E2001,
                            "Expected ',' or ']' in column selection".to_string()
                        );
                        return Err(());
                    }
                } else {
                    // Not an identifier, parse as index expression
                    let index = self.parse_expression()?;
                    self.expect(TokenType::RightBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
            } else {
                break;
            }
        }
        
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ()> {
        let token = self.peek().clone();
        
        match &token.token_type {
            TokenType::IntLiteral(n) => {
                self.advance();
                Ok(Expr::IntLiteral(*n))
            },
            TokenType::FloatLiteral(f) => {
                self.advance();
                Ok(Expr::FloatLiteral(*f))
            },
            TokenType::StringLiteral(s) => {
                self.advance();
                Ok(Expr::StringLiteral(s.clone()))
            },
            TokenType::BoolLiteral(b) => {
                self.advance();
                Ok(Expr::BoolLiteral(*b))
            },
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();
                
                // Check for function call
                if self.check(&TokenType::LeftParen) {
                    self.advance();
                    let args = self.parse_arguments()?;
                    self.expect(TokenType::RightParen)?;
                    Ok(Expr::FunctionCall(FunctionCall { name, args }))
                } else {
                    Ok(Expr::Identifier(name))
                }
            },
            TokenType::Underscore => {
                self.advance();
                Ok(Expr::Identifier("_".to_string()))
            },
            TokenType::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenType::RightParen)?;
                Ok(expr)
            },
            TokenType::LeftBracket => {
                // Parse array literal: [expr1, expr2, ...]
                self.advance();
                let mut elements = Vec::new();
                
                // Handle empty array
                if self.check(&TokenType::RightBracket) {
                    self.advance();
                    return Ok(Expr::ArrayLiteral(elements));
                }
                
                // Parse first element
                elements.push(self.parse_expression()?);
                
                // Parse remaining elements
                while self.check(&TokenType::Comma) {
                    self.advance(); // consume comma
                    elements.push(self.parse_expression()?);
                }
                
                self.expect(TokenType::RightBracket)?;
                Ok(Expr::ArrayLiteral(elements))
            },
            TokenType::Filter => {
                // Parse filter literal: filter(column, single/multi)
                self.advance();
                self.expect(TokenType::LeftParen)?;
                let column = self.expect_string()?;
                self.expect(TokenType::Comma)?;
                
                let mode_token = self.advance().clone();
                let mode = match &mode_token.token_type {
                    TokenType::Single => FilterMode::Single,
                    TokenType::Multi => FilterMode::Multi,
                    _ => {
                        self.add_error(
                            ErrorCode::E2011,
                            format!("Expected 'single' or 'multi', got {:?}", mode_token.token_type)
                        );
                        return Err(());
                    }
                };
                
                self.expect(TokenType::RightParen)?;
                Ok(Expr::FilterLiteral(FilterDef { column, mode }))
            },
            _ => {
                self.add_error(
                    ErrorCode::E2001,
                    format!("Unexpected token in expression: {:?}", token.token_type)
                );
                Err(())
            }
        }
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expr>, ()> {
        let mut args = Vec::new();
        
        if self.check(&TokenType::RightParen) {
            return Ok(args);
        }
        
        loop {
            args.push(self.parse_expression()?);
            
            if !self.check(&TokenType::Comma) {
                break;
            }
            self.advance();
        }
        
        Ok(args)
    }

    // Helper methods
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
    }
    
    fn check_identifier(&self) -> bool {
        if self.is_at_end() {
            return false;
        }
        matches!(self.peek().token_type, TokenType::Identifier(_))
    }
    
    fn check_identifier_value(&self, value: &str) -> bool {
        if self.is_at_end() {
            return false;
        }
        matches!(&self.peek().token_type, TokenType::Identifier(id) if id == value)
    }

    fn expect(&mut self, token_type: TokenType) -> Result<(), ()> {
        if self.check(&token_type) {
            self.advance();
            Ok(())
        } else {
            self.add_error(
                ErrorCode::E2011,
                format!("Expected {:?}, got {:?}", token_type, self.peek().token_type)
            );
            Err(())
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ()> {
        match &self.peek().token_type {
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            },
            _ => {
                self.add_error(
                    ErrorCode::E2002,
                    format!("Expected identifier, got {:?}", self.peek().token_type)
                );
                Err(())
            }
        }
    }

    fn expect_string(&mut self) -> Result<String, ()> {
        match &self.peek().token_type {
            TokenType::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            },
            _ => {
                self.add_error(
                    ErrorCode::E2011,
                    format!("Expected string literal, got {:?}", self.peek().token_type)
                );
                Err(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_source(source: &str) -> Result<Program, String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|diag| diag.format_all())?;
        let mut parser = Parser::new(tokens);
        parser.parse().map_err(|diag| diag.format_all())
    }

    #[test]
    fn test_parse_simple_page() {
        let source = r#"
            page TestPage {
            }
        "#;
        let program = parse_source(source).unwrap();
        assert_eq!(program.items.len(), 1);
        
        match &program.items[0] {
            ProgramItem::Page(page) => {
                assert_eq!(page.name, "TestPage");
                assert_eq!(page.statements.len(), 0);
            },
            _ => panic!("Expected Page item"),
        }
    }

    #[test]
    fn test_parse_table_definition() {
        let source = r#"
            table User {
                id: number
                name: string
                active: boolean
            }
        "#;
        let program = parse_source(source).unwrap();
        assert_eq!(program.items.len(), 1);
        
        match &program.items[0] {
            ProgramItem::TableDef(table) => {
                assert_eq!(table.name, "User");
                assert_eq!(table.fields.len(), 3);
                assert_eq!(table.fields[0].name, "id");
                assert_eq!(table.fields[1].name, "name");
                assert_eq!(table.fields[2].name, "active");
            },
            _ => panic!("Expected TableDef item"),
        }
    }

    #[test]
    fn test_parse_variable_declaration_with_type() {
        let source = r#"
            page Test {
                let x: number = 42
            }
        "#;
        let program = parse_source(source).unwrap();
        
        match &program.items[0] {
            ProgramItem::Page(page) => {
                assert_eq!(page.statements.len(), 1);
                match &page.statements[0] {
                    Statement::Let { name, type_annotation, value } => {
                        assert_eq!(name, "x");
                        assert!(type_annotation.is_some());
                        assert!(value.is_some());
                    },
                    _ => panic!("Expected Let statement"),
                }
            },
            _ => panic!("Expected Page item"),
        }
    }

    #[test]
    fn test_parse_variable_declaration_without_value() {
        let source = r#"
            page Test {
                let result: number
            }
        "#;
        let program = parse_source(source).unwrap();
        
        match &program.items[0] {
            ProgramItem::Page(page) => {
                match &page.statements[0] {
                    Statement::Let { name, type_annotation, value } => {
                        assert_eq!(name, "result");
                        assert!(type_annotation.is_some());
                        assert!(value.is_none());
                    },
                    _ => panic!("Expected Let statement"),
                }
            },
            _ => panic!("Expected Page item"),
        }
    }

    #[test]
    fn test_parse_assignment_statement() {
        let source = r#"
            page Test {
                let x: number
                x = 42
            }
        "#;
        let program = parse_source(source).unwrap();
        
        match &program.items[0] {
            ProgramItem::Page(page) => {
                assert_eq!(page.statements.len(), 2);
                match &page.statements[1] {
                    Statement::Assign { name, value } => {
                        assert_eq!(name, "x");
                        assert!(matches!(value, Expr::IntLiteral(_)));
                    },
                    _ => panic!("Expected Assign statement"),
                }
            },
            _ => panic!("Expected Page item"),
        }
    }

    #[test]
    fn test_parse_function_definition() {
        let source = r#"
            function add(x: number, y: number) -> number {
                return x + y
            }
        "#;
        let program = parse_source(source).unwrap();
        
        match &program.items[0] {
            ProgramItem::FunctionDef(func) => {
                assert_eq!(func.name, "add");
                assert_eq!(func.params.len(), 2);
                assert_eq!(func.params[0].name, "x");
                assert_eq!(func.params[1].name, "y");
                // return_type is Type, not Option<Type>
                assert_eq!(func.body.len(), 1);
            },
            _ => panic!("Expected FunctionDef item"),
        }
    }

    #[test]
    fn test_parse_return_statement() {
        let source = r#"
            function test() -> number {
                return 42
            }
        "#;
        let program = parse_source(source).unwrap();
        
        match &program.items[0] {
            ProgramItem::FunctionDef(func) => {
                match &func.body[0] {
                    Statement::Return(expr) => {
                        assert!(matches!(expr, Expr::IntLiteral(42)));
                    },
                    _ => panic!("Expected Return statement"),
                }
            },
            _ => panic!("Expected FunctionDef item"),
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let source = r#"
            page Test {
                if true {
                    text "yes"
                } else {
                    text "no"
                }
            }
        "#;
        let program = parse_source(source).unwrap();
        
        match &program.items[0] {
            ProgramItem::Page(page) => {
                match &page.statements[0] {
                    Statement::If { condition: _, then_branch, else_branch } => {
                        assert_eq!(then_branch.len(), 1);
                        assert!(else_branch.is_some());
                        assert_eq!(else_branch.as_ref().unwrap().len(), 1);
                    },
                    _ => panic!("Expected If statement"),
                }
            },
            _ => panic!("Expected Page item"),
        }
    }

    #[test]
    fn test_parse_forall_loop() {
        let source = r#"
            page Test {
                table User { id: number }
                let users: table(User) = load_csv(User, "users.csv")
                forall user in users {
                    display user.id
                }
            }
        "#;
        let program = parse_source(source).unwrap();
        
        match &program.items[0] {
            ProgramItem::Page(page) => {
                // Find the forall statement
                let forall_stmt = page.statements.iter().find(|stmt| matches!(stmt, Statement::Forall { .. }));
                assert!(forall_stmt.is_some());
            },
            _ => panic!("Expected Page item"),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let source = r#"
            page Test {
                display(42)
            }
        "#;
        let program = parse_source(source).unwrap();
        
        match &program.items[0] {
            ProgramItem::Page(page) => {
                match &page.statements[0] {
                    Statement::FunctionCall(FunctionCall { name, args }) => {
                        assert_eq!(name, "display");
                        assert_eq!(args.len(), 1);
                    },
                    _ => panic!("Expected function call"),
                }
            },
            _ => panic!("Expected Page item"),
        }
    }

    #[test]
    fn test_parse_binary_expression() {
        let source = r#"
            page Test {
                let result: number = 10 + 5 * 2
            }
        "#;
        let program = parse_source(source).unwrap();
        
        match &program.items[0] {
            ProgramItem::Page(page) => {
                match &page.statements[0] {
                    Statement::Let { value: Some(expr), .. } => {
                        assert!(matches!(expr, Expr::BinaryOp { .. }));
                    },
                    _ => panic!("Expected Let with expression"),
                }
            },
            _ => panic!("Expected Page item"),
        }
    }

    #[test]
    fn test_parse_field_access() {
        let source = r#"
            page Test {
                display user.name
            }
        "#;
        let program = parse_source(source).unwrap();
        
        match &program.items[0] {
            ProgramItem::Page(page) => {
                match &page.statements[0] {
                    Statement::FunctionCall(FunctionCall { args, .. }) => {
                        match &args[0] {
                            Expr::FieldAccess { object: _, field } => {
                                assert_eq!(field, "name");
                            },
                            _ => panic!("Expected field access"),
                        }
                    },
                    _ => panic!("Expected function call"),
                }
            },
            _ => panic!("Expected Page item"),
        }
    }

    #[test]
    fn test_parse_multiple_pages() {
        let source = r#"
            page Page1 { }
            page Page2 { }
        "#;
        let program = parse_source(source).unwrap();
        assert_eq!(program.items.len(), 2);
    }

    #[test]
    fn test_parse_external_function() {
        let source = r#"
            external function process(data: string) -> string from "module.py"
        "#;
        let program = parse_source(source).unwrap();
        
        match &program.items[0] {
            ProgramItem::ExternalFunction(ext) => {
                assert_eq!(ext.name, "process");
                assert_eq!(ext.params.len(), 1);
                // return_type is Type, not Option<Type>
            },
            _ => panic!("Expected ExternalFunction item"),
        }
    }

    #[test]
    fn test_parse_error_missing_brace() {
        let source = "page Test {";
        let result = parse_source(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_unexpected_token() {
        let source = "invalid syntax here";
        let result = parse_source(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_complex_program() {
        let source = r#"
            table Employee {
                id: number
                name: string
                salary: currency
            }
            
            function calculate_bonus(salary: currency) -> currency {
                return salary * 0.1
            }
            
            page EmployeeReport {
                let employees: table(Employee) = load_csv(Employee, "data.csv")
                forall emp in employees {
                    let bonus: currency = calculate_bonus(emp.salary)
                    display emp.name
                    display bonus
                }
            }
        "#;
        let program = parse_source(source).unwrap();
        assert_eq!(program.items.len(), 3); // table, function, page
    }
}
