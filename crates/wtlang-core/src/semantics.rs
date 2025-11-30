// Semantic analysis for WTLang
// Type checking, symbol table building, and validation

use crate::ast::*;
use crate::symbols::*;

pub struct SemanticAnalyzer {
    symbols: SymbolTable,
    errors: Vec<SemanticError>,
}

#[derive(Debug, Clone)]
pub enum SemanticError {
    UndefinedVariable {
        name: String,
    },
    Redefinition {
        name: String,
    },
    TypeMismatch {
        expected: String,
        found: String,
    },
    UninitializedVariable {
        name: String,
    },
    MissingTypeOrInitializer {
        name: String,
    },
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemanticError::UndefinedVariable { name } => {
                write!(f, "Undefined variable: '{}'", name)
            }
            SemanticError::Redefinition { name } => {
                write!(f, "Variable '{}' is already defined", name)
            }
            SemanticError::TypeMismatch { expected, found } => {
                write!(f, "Type mismatch: expected {}, found {}", expected, found)
            }
            SemanticError::UninitializedVariable { name } => {
                write!(f, "Variable '{}' used before initialization", name)
            }
            SemanticError::MissingTypeOrInitializer { name } => {
                write!(f, "Variable '{}' must have either a type annotation or initializer", name)
            }
        }
    }
}

impl std::error::Error for SemanticError {}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbols: SymbolTable::new(),
            errors: Vec::new(),
        }
    }
    
    pub fn analyze(&mut self, program: &Program) -> Result<(), Vec<SemanticError>> {
        // First pass: Collect global declarations (tables, function signatures)
        for item in &program.items {
            match item {
                ProgramItem::TableDef(table) => {
                    self.define_table(table);
                }
                ProgramItem::FunctionDef(func) => {
                    self.define_function_signature(func);
                }
                ProgramItem::ExternalFunction(ext) => {
                    self.define_external_function(ext);
                }
                _ => {}
            }
        }
        
        // Second pass: Check function bodies
        for item in &program.items {
            if let ProgramItem::FunctionDef(func) = item {
                self.check_function_body(func);
            }
        }
        
        // Third pass: Check pages and tests
        for item in &program.items {
            match item {
                ProgramItem::Page(page) => {
                    self.check_page(page);
                }
                ProgramItem::Test(test) => {
                    self.check_test(test);
                }
                _ => {}
            }
        }
        
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }
    
    fn define_table(&mut self, table: &TableDef) {
        if let Err(_e) = self.symbols.define(
            table.name.clone(),
            Symbol {
                name: table.name.clone(),
                symbol_type: Type::Table(table.name.clone()),
                kind: SymbolKind::Table,
                is_initialized: true,
                is_mutable: false,
            },
        ) {
            self.errors.push(SemanticError::Redefinition {
                name: table.name.clone(),
            });
        }
    }
    
    fn define_function_signature(&mut self, func: &FunctionDef) {
        if let Err(_e) = self.symbols.define(
            func.name.clone(),
            Symbol {
                name: func.name.clone(),
                symbol_type: func.return_type.clone(),
                kind: SymbolKind::Function,
                is_initialized: true,
                is_mutable: false,
            },
        ) {
            self.errors.push(SemanticError::Redefinition {
                name: func.name.clone(),
            });
        }
    }
    
    fn define_external_function(&mut self, ext: &ExternalFunction) {
        if let Err(_e) = self.symbols.define(
            ext.name.clone(),
            Symbol {
                name: ext.name.clone(),
                symbol_type: ext.return_type.clone(),
                kind: SymbolKind::ExternalFunction,
                is_initialized: true,
                is_mutable: false,
            },
        ) {
            self.errors.push(SemanticError::Redefinition {
                name: ext.name.clone(),
            });
        }
    }
    
    fn check_function_body(&mut self, func: &FunctionDef) {
        self.symbols.push_scope(ScopeKind::FunctionBody);
        
        // Add parameters to function scope
        for param in &func.params {
            if let Err(_e) = self.symbols.define(
                param.name.clone(),
                Symbol {
                    name: param.name.clone(),
                    symbol_type: param.param_type.clone(),
                    kind: SymbolKind::Parameter,
                    is_initialized: true,
                    is_mutable: false,
                },
            ) {
                self.errors.push(SemanticError::Redefinition {
                    name: param.name.clone(),
                });
            }
        }
        
        // Check function body
        for stmt in &func.body {
            self.check_statement(stmt);
        }
        
        self.symbols.pop_scope();
    }
    
    fn check_page(&mut self, page: &Page) {
        self.symbols.push_scope(ScopeKind::Page);
        
        for stmt in &page.statements {
            self.check_statement(stmt);
        }
        
        self.symbols.pop_scope();
    }
    
    fn check_test(&mut self, test: &Test) {
        self.symbols.push_scope(ScopeKind::TestBody);
        
        for stmt in &test.body {
            self.check_statement(stmt);
        }
        
        self.symbols.pop_scope();
    }
    
    fn check_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let { name, type_annotation, value } => {
                // Determine the type
                let symbol_type = if let Some(ref val) = value {
                    // Infer type from expression
                    self.infer_expr_type(val)
                } else if let Some(ref ty) = type_annotation {
                    // Use explicit type annotation
                    ty.clone()
                } else {
                    // This should be caught by parser, but double-check
                    self.errors.push(SemanticError::MissingTypeOrInitializer {
                        name: name.clone(),
                    });
                    Type::Int  // Dummy type to continue analysis
                };
                
                // Define the variable
                if let Err(_e) = self.symbols.define(
                    name.clone(),
                    Symbol {
                        name: name.clone(),
                        symbol_type: symbol_type.clone(),
                        kind: SymbolKind::Variable,
                        is_initialized: value.is_some(),
                        is_mutable: false,
                    },
                ) {
                    self.errors.push(SemanticError::Redefinition {
                        name: name.clone(),
                    });
                }
                
                // If both type annotation and value are present, check compatibility
                if let (Some(ref expected_type), Some(ref val)) = (type_annotation, value) {
                    let inferred_type = self.infer_expr_type(val);
                    if !self.types_compatible(expected_type, &inferred_type) {
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: format!("{:?}", expected_type),
                            found: format!("{:?}", inferred_type),
                        });
                    }
                }
            }
            
            Statement::Assign { name, value } => {
                // Check if variable exists
                if let Some(symbol) = self.symbols.lookup(name) {
                    // Check type compatibility if we have type information
                    let value_type = self.infer_expr_type(value);
                    if !self.types_compatible(&symbol.symbol_type, &value_type) {
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: format!("{:?}", symbol.symbol_type),
                            found: format!("{:?}", value_type),
                        });
                    }
                    
                    // Mark as initialized
                    if let Err(_) = self.symbols.mark_initialized(name) {
                        // Ignore error, variable might be in parent scope
                    }
                } else {
                    self.errors.push(SemanticError::UndefinedVariable {
                        name: name.clone(),
                    });
                }
                
                // Check the value expression
                self.check_expression(value);
            }
            
            Statement::Section { body, .. } => {
                self.symbols.push_scope(ScopeKind::Section);
                for s in body {
                    self.check_statement(s);
                }
                self.symbols.pop_scope();
            }
            
            Statement::Button { body, .. } => {
                self.symbols.push_scope(ScopeKind::Button);
                for s in body {
                    self.check_statement(s);
                }
                self.symbols.pop_scope();
            }
            
            Statement::If { condition, then_branch, else_branch } => {
                self.check_expression(condition);
                
                self.symbols.push_scope(ScopeKind::IfBranch);
                for s in then_branch {
                    self.check_statement(s);
                }
                self.symbols.pop_scope();
                
                if let Some(else_stmts) = else_branch {
                    self.symbols.push_scope(ScopeKind::IfBranch);
                    for s in else_stmts {
                        self.check_statement(s);
                    }
                    self.symbols.pop_scope();
                }
            }
            
            Statement::Forall { var, iterable, body } => {
                self.check_expression(iterable);
                
                // Infer element type before entering new scope
                let iter_type = self.infer_expr_type(iterable);
                let elem_type = self.get_element_type(&iter_type);
                
                self.symbols.push_scope(ScopeKind::ForallLoop);
                
                // Define loop variable (type is element type of iterable)
                if let Err(_e) = self.symbols.define(
                    var.clone(),
                    Symbol {
                        name: var.clone(),
                        symbol_type: elem_type,
                        kind: SymbolKind::LoopVariable,
                        is_initialized: true,
                        is_mutable: false,
                    },
                ) {
                    self.errors.push(SemanticError::Redefinition {
                        name: var.clone(),
                    });
                }
                
                for s in body {
                    self.check_statement(s);
                }
                
                self.symbols.pop_scope();
            }
            
            Statement::Return(expr) => {
                self.check_expression(expr);
            }
            
            Statement::FunctionCall(call) => {
                self.check_function_call(call);
            }
            
            _ => {}
        }
    }
    
    fn check_expression(&mut self, expr: &Expr) {
        match expr {
            Expr::Identifier(name) => {
                if let Some(symbol) = self.symbols.lookup(name) {
                    if !symbol.is_initialized {
                        self.errors.push(SemanticError::UninitializedVariable {
                            name: name.clone(),
                        });
                    }
                } else {
                    self.errors.push(SemanticError::UndefinedVariable {
                        name: name.clone(),
                    });
                }
            }
            
            Expr::FunctionCall(call) => {
                self.check_function_call(call);
            }
            
            Expr::BinaryOp { left, right, .. } => {
                self.check_expression(left);
                self.check_expression(right);
            }
            
            Expr::UnaryOp { operand, .. } => {
                self.check_expression(operand);
            }
            
            Expr::Lambda { body, .. } => {
                self.check_expression(body);
            }
            
            Expr::FieldAccess { object, .. } => {
                self.check_expression(object);
            }
            
            Expr::Index { object, index } => {
                self.check_expression(object);
                self.check_expression(index);
            }
            
            Expr::Chain { left, right } => {
                self.check_expression(left);
                self.check_expression(right);
            }
            
            Expr::ArrayLiteral(items) => {
                for item in items {
                    self.check_expression(item);
                }
            }
            
            _ => {}
        }
    }
    
    fn check_function_call(&mut self, call: &FunctionCall) {
        // Check if function exists
        if self.symbols.lookup(&call.name).is_none() {
            // It might be a builtin function, so don't error for now
            // In a more complete implementation, we'd have a list of builtins
        }
        
        // Check arguments
        for arg in &call.args {
            self.check_expression(arg);
        }
    }
    
    fn infer_expr_type(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::IntLiteral(_) => Type::Int,
            Expr::FloatLiteral(_) => Type::Float,
            Expr::StringLiteral(_) => Type::String,
            Expr::BoolLiteral(_) => Type::Bool,
            Expr::Identifier(name) => {
                self.symbols.lookup(name)
                    .map(|s| s.symbol_type.clone())
                    .unwrap_or(Type::Int)  // Default type if not found
            }
            Expr::FunctionCall(call) => {
                self.symbols.lookup(&call.name)
                    .map(|s| s.symbol_type.clone())
                    .unwrap_or(Type::Int)  // Default type if not found
            }
            _ => Type::Int,  // Simplified for now
        }
    }
    
    fn get_element_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Table(name) => Type::Table(name.clone()),
            _ => Type::Int,  // Simplified
        }
    }
    
    fn types_compatible(&self, t1: &Type, t2: &Type) -> bool {
        // Simplified type compatibility check
        t1 == t2
    }
    
    pub fn get_errors(&self) -> &[SemanticError] {
        &self.errors
    }
    
    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbols
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
