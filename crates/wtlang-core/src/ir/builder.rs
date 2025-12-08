// AST to IR Builder
//
// Converts AST representation to IR representation with type information

use crate::ast;
use crate::ir::types::*;
use crate::ir::nodes::*;
use crate::ir::module::IRModule;
use crate::symbols::SymbolTable;
use crate::semantics::SemanticAnalyzer;
use std::path::PathBuf;

pub struct IRBuilder {
    current_file: PathBuf,
    symbol_table: SymbolTable,
    key_counter: usize,
    // Track local variable types during lowering
    local_vars: std::collections::HashMap<String, Type>,
}

impl IRBuilder {
    pub fn new() -> Self {
        IRBuilder {
            current_file: PathBuf::from("<unknown>"),
            symbol_table: SymbolTable::new(),
            key_counter: 0,
            local_vars: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_file(file: PathBuf) -> Self {
        IRBuilder {
            current_file: file,
            symbol_table: SymbolTable::new(),
            key_counter: 0,
            local_vars: std::collections::HashMap::new(),
        }
    }
    
    /// Build IR from AST program
    pub fn build(&mut self, program: &ast::Program) -> Result<IRModule, String> {
        // First, run semantic analysis to populate symbol table
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(program)
            .map_err(|errors| {
                errors.iter()
                    .map(|e| format!("{:?}", e))
                    .collect::<Vec<_>>()
                    .join("\n")
            })?;
        
        self.symbol_table = analyzer.get_symbol_table().clone();
        
        let mut ir_module = IRModule::new(
            self.current_file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("program")
                .to_string()
        );
        
        ir_module.symbols = self.symbol_table.clone();
        
        // Convert each program item
        for item in &program.items {
            match item {
                ast::ProgramItem::TableDef(table_def) => {
                    ir_module.items.push(self.lower_table_def(table_def)?);
                }
                ast::ProgramItem::Page(page) => {
                    ir_module.items.push(self.lower_page(page)?);
                }
                ast::ProgramItem::FunctionDef(func) => {
                    ir_module.items.push(self.lower_function_def(func)?);
                }
                ast::ProgramItem::ExternalFunction(ext_func) => {
                    ir_module.items.push(self.lower_external_function(ext_func)?);
                }
                ast::ProgramItem::Test(test) => {
                    ir_module.items.push(self.lower_test(test)?);
                }
            }
        }
        
        Ok(ir_module)
    }
    
    fn lower_table_def(&mut self, table_def: &ast::TableDef) -> Result<IRItem, String> {
        let mut schema = TableSchema::new(table_def.name.clone());
        
        for field in &table_def.fields {
            schema.fields.push(Field {
                name: field.name.clone(),
                ty: FieldType::from(&field.field_type),
            });
            
            for constraint in &field.constraints {
                match constraint {
                    ast::Constraint::Unique => {
                        schema.constraints.push(Constraint::Unique(field.name.clone()));
                    }
                    ast::Constraint::NonNull => {
                        schema.constraints.push(Constraint::NonNull(field.name.clone()));
                    }
                    ast::Constraint::Key => {
                        schema.constraints.push(Constraint::PrimaryKey(field.name.clone()));
                    }
                    _ => {
                        // Validate and References are not yet fully supported
                    }
                }
            }
        }
        
        Ok(IRItem::TableDef {
            name: table_def.name.clone(),
            schema,
            source_loc: SourceRange::default(),
        })
    }
    
    fn lower_page(&mut self, page: &ast::Page) -> Result<IRItem, String> {
        // Clear local vars for new page scope
        self.local_vars.clear();
        
        let body = self.lower_statements(&page.statements)?;
        
        Ok(IRItem::PageDef {
            name: page.name.clone(),
            body,
            source_loc: SourceRange::default(),
        })
    }
    
    fn lower_function_def(&mut self, func: &ast::FunctionDef) -> Result<IRItem, String> {
        // Clear local vars for new function scope
        self.local_vars.clear();
        
        let params: Vec<Param> = func.params.iter()
            .map(|p| Param {
                name: p.name.clone(),
                ty: Type::from(&p.param_type),
            })
            .collect();
        
        // Add parameters to local vars
        for param in &params {
            self.local_vars.insert(param.name.clone(), param.ty.clone());
        }
        
        let body = self.lower_statements(&func.body)?;
        
        Ok(IRItem::FunctionDef {
            name: func.name.clone(),
            params,
            return_type: Type::from(&func.return_type),
            body,
            is_external: false,
            external_info: None,
            source_loc: SourceRange::default(),
        })
    }
    
    fn lower_external_function(&mut self, ext_func: &ast::ExternalFunction) -> Result<IRItem, String> {
        let params = ext_func.params.iter()
            .map(|p| Param {
                name: p.name.clone(),
                ty: Type::from(&p.param_type),
            })
            .collect();
        
        Ok(IRItem::FunctionDef {
            name: ext_func.name.clone(),
            params,
            return_type: Type::from(&ext_func.return_type),
            body: Vec::new(),
            is_external: true,
            external_info: Some(ExternalInfo {
                language: "python".to_string(),
                module: ext_func.module.clone(),
            }),
            source_loc: SourceRange::default(),
        })
    }
    
    fn lower_test(&mut self, test: &ast::Test) -> Result<IRItem, String> {
        // Clear local vars for new test scope
        self.local_vars.clear();
        
        let body = self.lower_statements(&test.body)?;
        
        Ok(IRItem::TestDef {
            name: test.name.clone(),
            body,
            source_loc: SourceRange::default(),
        })
    }
    
    fn lower_statements(&mut self, statements: &[ast::Statement]) -> Result<Vec<IRNode>, String> {
        statements.iter()
            .map(|stmt| self.lower_statement(stmt))
            .collect()
    }
    
    fn lower_statement(&mut self, stmt: &ast::Statement) -> Result<IRNode, String> {
        match stmt {
            ast::Statement::Title(text) => {
                Ok(IRNode::ShowText {
                    text: text.clone(),
                    style: TextStyle::Title,
                    source_loc: SourceRange::default(),
                })
            }
            
            ast::Statement::Subtitle(text) => {
                Ok(IRNode::ShowText {
                    text: text.clone(),
                    style: TextStyle::Subtitle,
                    source_loc: SourceRange::default(),
                })
            }
            
            ast::Statement::Text(text) => {
                Ok(IRNode::ShowText {
                    text: text.clone(),
                    style: TextStyle::Normal,
                    source_loc: SourceRange::default(),
                })
            }
            
            ast::Statement::Button { label, body } => {
                Ok(IRNode::Button {
                    label: label.clone(),
                    body: self.lower_statements(body)?,
                    source_loc: SourceRange::default(),
                })
            }
            
            ast::Statement::Section { title, body } => {
                Ok(IRNode::Section {
                    title: title.clone(),
                    body: self.lower_statements(body)?,
                    source_loc: SourceRange::default(),
                })
            }
            
            ast::Statement::Let { name, type_annotation, value } => {
                let ty = if let Some(type_ann) = type_annotation {
                    Type::from(type_ann)
                } else if let Some(val_expr) = value {
                    self.infer_expr_type(val_expr)?
                } else {
                    return Err(format!("Variable '{}' requires either type annotation or initial value", name));
                };
                
                let ir_value = if let Some(val_expr) = value {
                    Some(Box::new(self.lower_expr(val_expr)?))
                } else {
                    None
                };
                
                // Register the variable in local environment
                self.local_vars.insert(name.clone(), ty.clone());
                
                Ok(IRNode::Binding {
                    name: name.clone(),
                    ty,
                    value: ir_value,
                    source_loc: SourceRange::default(),
                })
            }
            
            ast::Statement::Assign { name, value } => {
                Ok(IRNode::Assignment {
                    target: name.clone(),
                    value: Box::new(self.lower_expr(value)?),
                    source_loc: SourceRange::default(),
                })
            }
            
            ast::Statement::If { condition, then_branch, else_branch } => {
                Ok(IRNode::Conditional {
                    condition: Box::new(self.lower_expr(condition)?),
                    then_branch: self.lower_statements(then_branch)?,
                    else_branch: if let Some(else_stmts) = else_branch {
                        Some(self.lower_statements(else_stmts)?)
                    } else {
                        None
                    },
                    source_loc: SourceRange::default(),
                })
            }
            
            ast::Statement::Forall { var, iterable, body } => {
                Ok(IRNode::Loop {
                    variable: var.clone(),
                    iterable: Box::new(self.lower_expr(iterable)?),
                    body: self.lower_statements(body)?,
                    source_loc: SourceRange::default(),
                })
            }
            
            ast::Statement::Return(expr) => {
                Ok(IRNode::Return {
                    value: Some(Box::new(self.lower_expr(expr)?)),
                    source_loc: SourceRange::default(),
                })
            }
            
            ast::Statement::FunctionCall(call) => {
                let expr = self.lower_function_call(call)?;
                Ok(IRNode::ExprStmt {
                    expr: Box::new(expr),
                    source_loc: SourceRange::default(),
                })
            }
        }
    }
    
    fn lower_expr(&mut self, expr: &ast::Expr) -> Result<IRExpr, String> {
        match expr {
            ast::Expr::IntLiteral(val) => {
                Ok(IRExpr::Literal {
                    value: Literal::Int(*val),
                    ty: Type::Int,
                })
            }
            
            ast::Expr::FloatLiteral(val) => {
                Ok(IRExpr::Literal {
                    value: Literal::Float(*val),
                    ty: Type::Float,
                })
            }
            
            ast::Expr::StringLiteral(val) => {
                Ok(IRExpr::Literal {
                    value: Literal::String(val.clone()),
                    ty: Type::String,
                })
            }
            
            ast::Expr::BoolLiteral(val) => {
                Ok(IRExpr::Literal {
                    value: Literal::Bool(*val),
                    ty: Type::Bool,
                })
            }
            
            ast::Expr::Identifier(name) => {
                // Special handling for _ placeholder in chaining
                if name == "_" {
                    Ok(IRExpr::Variable {
                        name: "_".to_string(),
                        ty: Type::Error, // Type will be determined by context
                    })
                } else {
                    let ty = self.lookup_variable_type(name)?;
                    Ok(IRExpr::Variable {
                        name: name.clone(),
                        ty,
                    })
                }
            }
            
            ast::Expr::FunctionCall(call) => {
                self.lower_function_call(call)
            }
            
            ast::Expr::BinaryOp { op, left, right } => {
                let left_ir = self.lower_expr(left)?;
                let right_ir = self.lower_expr(right)?;
                
                // Check if this is a set operation on tables
                match op {
                    ast::BinaryOp::Union => {
                        let ty = left_ir.get_type().clone();
                        Ok(IRExpr::Union {
                            left: Box::new(left_ir),
                            right: Box::new(right_ir),
                            ty,
                        })
                    }
                    ast::BinaryOp::Minus if left_ir.get_type().is_table() => {
                        // Table set difference
                        let ty = left_ir.get_type().clone();
                        Ok(IRExpr::Minus {
                            left: Box::new(left_ir),
                            right: Box::new(right_ir),
                            ty,
                        })
                    }
                    ast::BinaryOp::Intersect => {
                        let ty = left_ir.get_type().clone();
                        Ok(IRExpr::Intersect {
                            left: Box::new(left_ir),
                            right: Box::new(right_ir),
                            ty,
                        })
                    }
                    _ => {
                        // Regular binary operations
                        let ty = self.infer_binary_op_type(op, left_ir.get_type(), right_ir.get_type())?;
                        Ok(IRExpr::BinaryOp {
                            op: BinOp::from(op),
                            left: Box::new(left_ir),
                            right: Box::new(right_ir),
                            ty,
                        })
                    }
                }
            }
            
            ast::Expr::UnaryOp { op, operand } => {
                let operand_ir = self.lower_expr(operand)?;
                let ty = operand_ir.get_type().clone();
                
                Ok(IRExpr::UnaryOp {
                    op: UnOp::from(op),
                    operand: Box::new(operand_ir),
                    ty,
                })
            }
            
            ast::Expr::FieldAccess { object, field } => {
                let object_ir = self.lower_expr(object)?;
                
                // Check if this is a reference navigation
                if let Some(ref_info) = self.check_ref_field(object_ir.get_type(), field) {
                    // This is a reference field - create RefNavigation node
                    Ok(IRExpr::RefNavigation {
                        object: Box::new(object_ir),
                        field: field.clone(),
                        target_table: ref_info.target_table,
                        ty: Type::Table(ref_info.target_schema),
                    })
                } else {
                    // Regular field access
                    let ty = self.infer_field_access_type(object_ir.get_type(), field)?;
                    Ok(IRExpr::FieldAccess {
                        object: Box::new(object_ir),
                        field: field.clone(),
                        ty,
                    })
                }
            }
            
            ast::Expr::Index { object, index } => {
                let object_ir = self.lower_expr(object)?;
                let index_ir = self.lower_expr(index)?;
                let ty = self.infer_index_type(object_ir.get_type())?;
                
                Ok(IRExpr::Index {
                    object: Box::new(object_ir),
                    index: Box::new(index_ir),
                    ty,
                })
            }
            
            ast::Expr::Chain { left, right } => {
                let left_ir = self.lower_expr(left)?;
                let right_ir = self.lower_expr(right)?;
                let ty = right_ir.get_type().clone();
                
                Ok(IRExpr::Chain {
                    left: Box::new(left_ir),
                    right: Box::new(right_ir),
                    ty,
                })
            }
            
            ast::Expr::TableLiteral(fields) => {
                let ir_fields: Result<Vec<_>, String> = fields.iter()
                    .map(|(name, expr)| {
                        self.lower_expr(expr).map(|ir_expr| (name.clone(), ir_expr))
                    })
                    .collect();
                
                Ok(IRExpr::TableConstructor {
                    fields: ir_fields?,
                    ty: Type::Error, // Would need schema inference
                })
            }
            
            ast::Expr::ArrayLiteral(elements) => {
                let ir_elements: Result<Vec<_>, String> = elements.iter()
                    .map(|e| self.lower_expr(e))
                    .collect();
                
                Ok(IRExpr::ArrayConstructor {
                    elements: ir_elements?,
                    ty: Type::Error, // Would need element type inference
                })
            }
            
            ast::Expr::Lambda { params, body } => {
                let body_ir = self.lower_expr(body)?;
                let return_type = body_ir.get_type().clone();
                
                Ok(IRExpr::Lambda {
                    params: params.clone(),
                    body: Box::new(body_ir),
                    ty: Type::Function {
                        params: vec![Type::Error; params.len()], // Simplified
                        return_type: Box::new(return_type),
                    },
                })
            }
            
            ast::Expr::FilterLiteral(filter_def) => {
                // For now, return an error type - filters need special handling
                Ok(IRExpr::Literal {
                    value: Literal::String(filter_def.column.clone()),
                    ty: Type::Error,
                })
            }
            
            ast::Expr::Where { table, condition } => {
                let table_ir = self.lower_expr(table)?;
                let condition_ir = self.lower_expr(condition)?;
                let ty = table_ir.get_type().clone();
                
                Ok(IRExpr::Where {
                    table: Box::new(table_ir),
                    condition: Box::new(condition_ir),
                    ty,
                })
            }
            
            ast::Expr::SortBy { table, columns } => {
                let table_ir = self.lower_expr(table)?;
                let ty = table_ir.get_type().clone();
                
                let sort_specs: Vec<SortSpec> = columns.iter()
                    .map(|col| SortSpec {
                        column: col.name.clone(),
                        ascending: col.ascending,
                    })
                    .collect();
                
                Ok(IRExpr::SortBy {
                    table: Box::new(table_ir),
                    columns: sort_specs,
                    ty,
                })
            }
            
            ast::Expr::ColumnSelect { table, columns } => {
                let table_ir = self.lower_expr(table)?;
                let ty = table_ir.get_type().clone();
                
                Ok(IRExpr::ColumnSelect {
                    table: Box::new(table_ir),
                    columns: columns.clone(),
                    ty,
                })
            }
        }
    }
    
    fn lower_function_call(&mut self, call: &ast::FunctionCall) -> Result<IRExpr, String> {
        let args: Result<Vec<_>, String> = call.args.iter()
            .map(|arg| self.lower_expr(arg))
            .collect();
        let args = args?;
        
        // Special handling for built-in functions
        let ty = match call.name.as_str() {
            "show" | "show_editable" => {
                // Extract table and filters
                if args.is_empty() {
                    return Err("show requires at least a table argument".to_string());
                }
                
                let table_expr = args[0].clone();
                let mut filters: Vec<FilterSpec> = Vec::new();
                let editable = call.name == "show_editable";
                
                // Check if there's a filters argument (array of filters)
                if args.len() > 1 {
                    // Parse filters from arguments - simplified for now
                }
                
                self.key_counter += 1;
                return Ok(IRExpr::FunctionCall {
                    function: if editable { "show_editable" } else { "show" }.to_string(),
                    args,
                    ty: Type::Unit,
                });
            }
            "load_csv" => Type::Error, // Would need table type from argument
            "save_csv" => Type::Unit,
            "where" | "sort" | "aggregate" => {
                if !args.is_empty() {
                    args[0].get_type().clone()
                } else {
                    Type::Error
                }
            }
            _ => {
                // Look up function in symbol table
                self.lookup_function_return_type(&call.name)?
            }
        };
        
        Ok(IRExpr::FunctionCall {
            function: call.name.clone(),
            args,
            ty,
        })
    }
    
    fn infer_expr_type(&self, expr: &ast::Expr) -> Result<Type, String> {
        match expr {
            ast::Expr::IntLiteral(_) => Ok(Type::Int),
            ast::Expr::FloatLiteral(_) => Ok(Type::Float),
            ast::Expr::StringLiteral(_) => Ok(Type::String),
            ast::Expr::BoolLiteral(_) => Ok(Type::Bool),
            ast::Expr::Identifier(name) => self.lookup_variable_type(name),
            _ => Ok(Type::Error), // Simplified - would need full type inference
        }
    }
    
    fn lookup_variable_type(&self, name: &str) -> Result<Type, String> {
        // Check local variables first
        if let Some(ty) = self.local_vars.get(name) {
            return Ok(ty.clone());
        }
        
        // Then check symbol table for global symbols
        if let Some(symbol) = self.symbol_table.lookup(name) {
            Ok(self.ast_type_to_ir_type(&symbol.symbol_type))
        } else {
            Err(format!("Variable '{}' not found", name))
        }
    }
    
    fn lookup_function_return_type(&self, name: &str) -> Result<Type, String> {
        if let Some(symbol) = self.symbol_table.lookup(name) {
            Ok(self.ast_type_to_ir_type(&symbol.symbol_type))
        } else {
            Ok(Type::Unit) // Default for unknown functions
        }
    }
    
    fn ast_type_to_ir_type(&self, ast_type: &ast::Type) -> Type {
        Type::from(ast_type)
    }
    
    fn infer_binary_op_type(&self, op: &ast::BinaryOp, left_ty: &Type, right_ty: &Type) -> Result<Type, String> {
        match op {
            ast::BinaryOp::Add | ast::BinaryOp::Subtract | 
            ast::BinaryOp::Multiply | ast::BinaryOp::Divide | ast::BinaryOp::Modulo => {
                if left_ty.is_numeric() && right_ty.is_numeric() {
                    Ok(left_ty.clone())
                } else {
                    Ok(Type::Error)
                }
            }
            ast::BinaryOp::Equal | ast::BinaryOp::NotEqual |
            ast::BinaryOp::LessThan | ast::BinaryOp::LessThanEqual |
            ast::BinaryOp::GreaterThan | ast::BinaryOp::GreaterThanEqual => {
                Ok(Type::Bool)
            }
            ast::BinaryOp::And | ast::BinaryOp::Or => {
                Ok(Type::Bool)
            }
            ast::BinaryOp::Union | ast::BinaryOp::Minus | ast::BinaryOp::Intersect => {
                // Set operations return the same table type as the left operand
                Ok(left_ty.clone())
            }
        }
    }
    
    fn infer_field_access_type(&self, object_ty: &Type, field: &str) -> Result<Type, String> {
        if let Some(schema) = object_ty.as_table() {
            if let Some(field_type) = schema.get_field_type(field) {
                Ok(match field_type {
                    FieldType::Int => Type::Int,
                    FieldType::Float => Type::Float,
                    FieldType::String => Type::String,
                    FieldType::Bool => Type::Bool,
                    FieldType::Date => Type::Date,
                    FieldType::Currency => Type::Currency,
                    FieldType::Ref { table_name } => {
                        // Look up the referenced table schema
                        if let Some(target_symbol) = self.symbol_table.lookup(table_name) {
                            self.ast_type_to_ir_type(&target_symbol.symbol_type)
                        } else {
                            Type::Error
                        }
                    }
                })
            } else {
                Err(format!("Field '{}' not found in table", field))
            }
        } else {
            Ok(Type::Error)
        }
    }
    
    fn infer_index_type(&self, _object_ty: &Type) -> Result<Type, String> {
        // Simplified - would need to handle array types properly
        Ok(Type::Error)
    }
    
    fn check_ref_field(&self, object_ty: &Type, field: &str) -> Option<RefInfo> {
        // Check if the field is a reference type in the table schema
        if let Some(schema) = object_ty.as_table() {
            if let Some(field_def) = schema.fields.iter().find(|f| f.name == field) {
                if let FieldType::Ref { table_name } = &field_def.ty {
                    // Look up the target table schema
                    if let Some(target_symbol) = self.symbol_table.lookup(table_name) {
                        if let Type::Table(target_schema) = self.ast_type_to_ir_type(&target_symbol.symbol_type) {
                            return Some(RefInfo {
                                target_table: table_name.clone(),
                                target_schema,
                            });
                        }
                    }
                }
            }
        }
        None
    }
}

struct RefInfo {
    target_table: String,
    target_schema: TableSchema,
}

impl Default for IRBuilder {
    fn default() -> Self {
        Self::new()
    }
}
