// Parser for WTLang
use crate::ast::*;
use crate::lexer::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut items = Vec::new();
        
        while !self.is_at_end() {
            items.push(self.parse_program_item()?);
        }
        
        Ok(Program { items })
    }

    fn parse_program_item(&mut self) -> Result<ProgramItem, String> {
        match &self.peek().token_type {
            TokenType::Table => Ok(ProgramItem::TableDef(self.parse_table_def()?)),
            TokenType::Page => Ok(ProgramItem::Page(self.parse_page()?)),
            TokenType::Function => Ok(ProgramItem::FunctionDef(self.parse_function_def()?)),
            TokenType::External => Ok(ProgramItem::ExternalFunction(self.parse_external_function()?)),
            TokenType::Test => Ok(ProgramItem::Test(self.parse_test()?)),
            _ => Err(format!("Expected table, page, function, external, or test, got {:?}", self.peek().token_type)),
        }
    }

    fn parse_table_def(&mut self) -> Result<TableDef, String> {
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

    fn parse_field(&mut self) -> Result<Field, String> {
        let name = self.expect_identifier()?;
        self.expect(TokenType::Colon)?;
        let field_type = self.parse_type()?;
        
        let mut constraints = Vec::new();
        if self.check(&TokenType::LeftBracket) {
            self.advance();
            constraints = self.parse_constraints()?;
            self.expect(TokenType::RightBracket)?;
        }
        
        Ok(Field { name, field_type, constraints })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let token = self.advance();
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
            _ => Err(format!("Expected type, got {:?}", token.token_type)),
        }
    }

    fn parse_constraints(&mut self) -> Result<Vec<Constraint>, String> {
        let mut constraints = Vec::new();
        
        loop {
            let ident = self.expect_identifier()?;
            let constraint = match ident.as_str() {
                "unique" => Constraint::Unique,
                "non_null" => Constraint::NonNull,
                _ => return Err(format!("Unknown constraint: {}", ident)),
            };
            constraints.push(constraint);
            
            if !self.check(&TokenType::Comma) {
                break;
            }
            self.advance();
        }
        
        Ok(constraints)
    }

    fn parse_page(&mut self) -> Result<Page, String> {
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

    fn parse_statement(&mut self) -> Result<Statement, String> {
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
                    return Err(format!(
                        "Variable '{}' must have either a type annotation or an initializer",
                        name
                    ));
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
                    Err(format!("Expected function call or assignment"))
                }
            },
            _ => Err(format!("Unexpected token in statement: {:?}", self.peek().token_type)),
        }
    }

    fn parse_function_def(&mut self) -> Result<FunctionDef, String> {
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

    fn parse_external_function(&mut self) -> Result<ExternalFunction, String> {
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

    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, String> {
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

    fn parse_test(&mut self) -> Result<Test, String> {
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

    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_chain()
    }

    fn parse_chain(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_or()?;
        
        while self.check(&TokenType::Arrow) {
            self.advance();
            let right = self.parse_or()?;
            left = Expr::Chain {
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
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

    fn parse_and(&mut self) -> Result<Expr, String> {
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

    fn parse_equality(&mut self) -> Result<Expr, String> {
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

    fn parse_comparison(&mut self) -> Result<Expr, String> {
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

    fn parse_addition(&mut self) -> Result<Expr, String> {
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

    fn parse_multiplication(&mut self) -> Result<Expr, String> {
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

    fn parse_unary(&mut self) -> Result<Expr, String> {
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

    fn parse_postfix(&mut self) -> Result<Expr, String> {
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
                let index = self.parse_expression()?;
                self.expect(TokenType::RightBracket)?;
                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else {
                break;
            }
        }
        
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
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
                
                let mode_token = self.advance();
                let mode = match &mode_token.token_type {
                    TokenType::Single => FilterMode::Single,
                    TokenType::Multi => FilterMode::Multi,
                    _ => return Err(format!("Expected 'single' or 'multi', got {:?}", mode_token.token_type)),
                };
                
                self.expect(TokenType::RightParen)?;
                Ok(Expr::FilterLiteral(FilterDef { column, mode }))
            },
            _ => Err(format!("Unexpected token in expression: {:?}", token.token_type)),
        }
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expr>, String> {
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

    fn expect(&mut self, token_type: TokenType) -> Result<(), String> {
        if self.check(&token_type) {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", token_type, self.peek().token_type))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, String> {
        match &self.peek().token_type {
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            },
            _ => Err(format!("Expected identifier, got {:?}", self.peek().token_type)),
        }
    }

    fn expect_string(&mut self) -> Result<String, String> {
        match &self.peek().token_type {
            TokenType::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            },
            _ => Err(format!("Expected string literal, got {:?}", self.peek().token_type)),
        }
    }
}
