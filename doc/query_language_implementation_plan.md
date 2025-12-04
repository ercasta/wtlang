# Query Language Implementation Plan

## Overview

This document provides a comprehensive implementation plan for adding query language features and reference support to WTLang, as described in:
- `builtin_query_language.md` - Simple query operations (where, sort, set operations, column selection)
- `keys_and_refs.md` - Reference types and key constraints for relational operations

## Current State Analysis

### What's Already Implemented

1. **Basic Query Operations (Partial)**:
   - ✅ `where` - Implemented as builtin function, but limited (uses pandas query, doesn't support infix syntax)
   - ✅ `sort` - Implemented as builtin function
   - ✅ `aggregate` - Implemented as builtin function
   - ❌ Infix operators for queries (`users where age > 18`) - Not implemented
   - ❌ Set operations (`+`, `-`, `&` for union/minus/intersect) - Operators exist but not for tables
   - ❌ Column subset selection (`users[name, surname]`) - Bracket syntax exists but not for column selection

2. **Type System**:
   - ✅ Basic types (int, float, string, date, currency, bool)
   - ✅ Table types with schemas
   - ✅ Filter types
   - ❌ Reference types (`ref TableName`) - Not implemented
   - ❌ Key constraints - Not implemented

3. **AST/IR Infrastructure**:
   - ✅ Expression types (BinaryOp, FieldAccess, Index)
   - ✅ IR module with Type system and nodes
   - ❌ IR nodes for query operations
   - ❌ AST/IR for reference types

### What Needs to Be Implemented

Based on the documentation:

#### From `builtin_query_language.md`:
1. Infix `where` syntax: `users where age > 18`
2. Set operations on tables:
   - Union: `elders + children`
   - Minus: `users - male_adults`
   - Intersect: `adults & male`
3. Column subset selection: `users[name, surname]`
4. Multi-column sort: `users sort by name asc, age desc`

#### From `keys_and_refs.md`:
1. `key` constraint for table fields
2. `ref` type for references to other tables
3. Reference navigation: `employee.department`
4. Automatic join/lookup when accessing references

## Implementation Plan

### Phase 1: Lexer and AST Extensions (Week 1)

#### 1.1 Lexer Changes (`crates/wtlang-core/src/lexer.rs`)

**Add new keywords:**
```rust
// In TokenType enum
pub enum TokenType {
    // ... existing keywords ...
    Where,      // Make "where" a keyword instead of just a builtin
    By,         // For "sort by"
    Asc,        // For "sort by x asc"
    Desc,       // For "sort by x desc"
    Key,        // For table field keys
    Ref,        // For reference types
    // ... rest ...
}
```

**Update identifier matching:**
```rust
// In read_identifier() method
fn read_identifier(&mut self) -> Result<Token, ()> {
    // ... existing code ...
    let token_type = match value.as_str() {
        // ... existing keywords ...
        "where" => TokenType::Where,
        "by" => TokenType::By,
        "asc" => TokenType::Asc,
        "desc" => TokenType::Desc,
        "key" => TokenType::Key,
        "ref" => TokenType::Ref,
        // ... rest ...
    };
    // ...
}
```

**Special handling for set operators:**
- `+`, `-`, `&` already exist in lexer
- Need to handle them differently based on context (table operations vs arithmetic)
- This will be handled in parser/semantic analysis

**Estimated effort:** 2-3 hours
**Files to modify:** 
- `crates/wtlang-core/src/lexer.rs`

#### 1.2 AST Extensions (`crates/wtlang-core/src/ast.rs`)

**Extend Type enum for references:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // ... existing types ...
    Ref(String),  // Ref(TableName) - reference to another table
}
```

**Extend Constraint enum for key:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    // ... existing constraints ...
    Key,  // Mark field as primary key
}
```

**Add new expression types for query operations:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // ... existing expressions ...
    
    // Infix where: users where age > 18
    Where { table: Box<Expr>, condition: Box<Expr> },
    
    // Sort with multiple columns: users sort by name asc, age desc
    SortBy { table: Box<Expr>, columns: Vec<SortColumn> },
    
    // Column subset: users[name, surname]
    ColumnSelect { table: Box<Expr>, columns: Vec<String> },
    
    // Set operations (union, minus, intersect) - reuse BinaryOp with new operators
}

#[derive(Debug, Clone, PartialEq)]
pub struct SortColumn {
    pub name: String,
    pub ascending: bool,  // true for asc, false for desc
}

// Extend BinaryOp for set operations
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // ... existing operators ...
    
    // Set operations on tables
    Union,      // +
    Minus,      // -
    Intersect,  // &
}
```

**Estimated effort:** 3-4 hours
**Files to modify:**
- `crates/wtlang-core/src/ast.rs`

#### 1.3 Parser Extensions (`crates/wtlang-core/src/parser.rs`)

**Parse type annotations with ref:**
```rust
fn parse_type(&mut self) -> Result<Type, String> {
    match &self.current.token_type {
        // ... existing type parsing ...
        TokenType::Ref => {
            self.advance();
            let table_name = self.expect_identifier()?;
            Ok(Type::Ref(table_name))
        }
        // ...
    }
}
```

**Parse constraints with key:**
```rust
fn parse_constraint(&mut self) -> Result<Constraint, String> {
    match &self.current.token_type {
        // ... existing constraints ...
        TokenType::Key => {
            self.advance();
            Ok(Constraint::Key)
        }
        // ...
    }
}
```

**Parse infix where expressions:**
```rust
// In expression parsing, add precedence for where (lower than comparison)
fn parse_where_expr(&mut self, left: Expr) -> Result<Expr, String> {
    if self.match_token(&TokenType::Where) {
        self.advance();
        let condition = self.parse_comparison_expr()?;
        Ok(Expr::Where {
            table: Box::new(left),
            condition: Box::new(condition),
        })
    } else {
        Ok(left)
    }
}
```

**Parse sort by expressions:**
```rust
fn parse_sort_expr(&mut self, left: Expr) -> Result<Expr, String> {
    // Expect: sort by column1 [asc|desc], column2 [asc|desc], ...
    self.expect_token(&TokenType::Identifier("sort".to_string()))?;
    self.expect_token(&TokenType::By)?;
    
    let mut columns = Vec::new();
    loop {
        let col_name = self.expect_identifier()?;
        let ascending = match &self.current.token_type {
            TokenType::Asc => { self.advance(); true }
            TokenType::Desc => { self.advance(); false }
            _ => true  // Default to ascending
        };
        columns.push(SortColumn { name: col_name, ascending });
        
        if !self.match_token(&TokenType::Comma) {
            break;
        }
        self.advance();
    }
    
    Ok(Expr::SortBy {
        table: Box::new(left),
        columns,
    })
}
```

**Parse column selection (bracket notation):**
```rust
// Extend parse_postfix_expr to handle column selection
fn parse_postfix_expr(&mut self) -> Result<Expr, String> {
    let mut expr = self.parse_primary_expr()?;
    
    loop {
        match &self.current.token_type {
            TokenType::LeftBracket => {
                self.advance();
                
                // Check if it's column selection or index
                // Column selection: [col1, col2, ...]
                // Index: [expr]
                if self.is_identifier() {
                    let mut columns = vec![self.expect_identifier()?];
                    while self.match_token(&TokenType::Comma) {
                        self.advance();
                        columns.push(self.expect_identifier()?);
                    }
                    self.expect_token(&TokenType::RightBracket)?;
                    expr = Expr::ColumnSelect {
                        table: Box::new(expr),
                        columns,
                    };
                } else {
                    // Regular index
                    let index = self.parse_expr()?;
                    self.expect_token(&TokenType::RightBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
            }
            // ... other postfix operations ...
        }
    }
    
    Ok(expr)
}
```

**Handle set operators with context:**
```rust
// In binary operator parsing, detect table types for set operations
fn parse_additive_expr(&mut self) -> Result<Expr, String> {
    let mut left = self.parse_multiplicative_expr()?;
    
    while matches!(&self.current.token_type, TokenType::Plus | TokenType::Minus) {
        let op = match &self.current.token_type {
            TokenType::Plus => BinaryOp::Add,  // Could be Union for tables
            TokenType::Minus => BinaryOp::Subtract,  // Could be Minus for tables
            _ => unreachable!(),
        };
        self.advance();
        let right = self.parse_multiplicative_expr()?;
        
        // Type resolution will determine if this is set operation or arithmetic
        left = Expr::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        };
    }
    
    Ok(left)
}

// Similar for & operator (intersect vs bitwise and)
```

**Update expression precedence:**
```
1. Primary (literals, identifiers, function calls)
2. Postfix (field access, index, column selection)
3. Unary (not, negate)
4. Multiplicative (*, /, %)
5. Additive (+, -)
6. Comparison (<, >, <=, >=)
7. Equality (==, !=)
8. And (&& or table intersect &)
9. Or (|| or table union +)
10. Where (where clause)
11. Sort (sort by)
12. Chain (->)
```

**Estimated effort:** 8-12 hours
**Files to modify:**
- `crates/wtlang-core/src/parser.rs`

**Testing:**
- Create test files in `tests/fixtures/valid/`:
  - `query_where.wt` - Test infix where
  - `query_sort.wt` - Test sort by
  - `query_columns.wt` - Test column selection
  - `query_sets.wt` - Test set operations
  - `refs_basic.wt` - Test key and ref syntax
- Add parser unit tests

### Phase 2: IR Extensions (Week 2)

#### 2.1 IR Type System (`crates/wtlang-core/src/ir/types.rs`)

**Extend Type enum:**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    // ... existing types ...
    
    /// Reference to another table by key
    Ref {
        table_name: String,
        key_field: String,  // Which field in target table is the key
    },
}
```

**Extend Constraint enum:**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Constraint {
    // ... existing constraints ...
    PrimaryKey(String),  // Field name that is the primary key
}
```

**Add schema helpers for keys and refs:**
```rust
impl TableSchema {
    // ... existing methods ...
    
    pub fn get_key_field(&self) -> Option<&Field> {
        // Find field marked with PrimaryKey constraint
        for field in &self.fields {
            for constraint in &self.constraints {
                if let Constraint::PrimaryKey(name) = constraint {
                    if name == &field.name {
                        return Some(field);
                    }
                }
            }
        }
        None
    }
    
    pub fn has_ref_to(&self, table_name: &str) -> bool {
        self.fields.iter().any(|f| {
            matches!(&f.ty, FieldType::Ref { table_name: tn, .. } if tn == table_name)
        })
    }
}
```

**Extend FieldType:**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FieldType {
    // ... existing types ...
    Ref { 
        table_name: String,
        key_field: String,
    },
}
```

**Estimated effort:** 3-4 hours
**Files to modify:**
- `crates/wtlang-core/src/ir/types.rs`

#### 2.2 IR Nodes (`crates/wtlang-core/src/ir/nodes.rs`)

**Add IR expressions for query operations:**
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IRExpr {
    // ... existing expressions ...
    
    /// Where clause: filter table by condition
    Where {
        table: Box<IRExpr>,
        condition: Box<IRExpr>,
        result_type: Type,
        source_loc: SourceRange,
    },
    
    /// Sort by one or more columns
    SortBy {
        table: Box<IRExpr>,
        columns: Vec<SortSpec>,
        result_type: Type,  // Same table type as input
        source_loc: SourceRange,
    },
    
    /// Column subset selection
    ColumnSelect {
        table: Box<IRExpr>,
        columns: Vec<String>,
        result_type: Type,  // Table with subset of columns
        source_loc: SourceRange,
    },
    
    /// Set union
    Union {
        left: Box<IRExpr>,
        right: Box<IRExpr>,
        result_type: Type,
        source_loc: SourceRange,
    },
    
    /// Set difference
    Minus {
        left: Box<IRExpr>,
        right: Box<IRExpr>,
        result_type: Type,
        source_loc: SourceRange,
    },
    
    /// Set intersection
    Intersect {
        left: Box<IRExpr>,
        right: Box<IRExpr>,
        result_type: Type,
        source_loc: SourceRange,
    },
    
    /// Reference navigation: employee.department
    /// When field is a ref type, automatically perform lookup
    RefNavigation {
        object: Box<IRExpr>,
        field: String,
        target_table: String,  // Table being referenced
        result_type: Type,     // Type of the referenced table
        source_loc: SourceRange,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortSpec {
    pub column: String,
    pub ascending: bool,
}
```

**Estimated effort:** 2-3 hours
**Files to modify:**
- `crates/wtlang-core/src/ir/nodes.rs`

#### 2.3 IR Builder (`crates/wtlang-core/src/ir/builder.rs`)

**Add AST→IR lowering for new expressions:**

```rust
impl IRBuilder {
    // ... existing methods ...
    
    fn lower_expr(&mut self, expr: &ast::Expr) -> Result<IRExpr, String> {
        match expr {
            // ... existing expression lowering ...
            
            ast::Expr::Where { table, condition } => {
                let table_ir = self.lower_expr(table)?;
                let table_type = table_ir.get_type();
                
                // Verify table is actually a table type
                if !table_type.is_table() {
                    return Err(format!("where clause requires a table, got {}", table_type));
                }
                
                let condition_ir = self.lower_expr(condition)?;
                
                // Verify condition is boolean
                if condition_ir.get_type() != &Type::Bool {
                    return Err(format!("where condition must be boolean, got {}", 
                        condition_ir.get_type()));
                }
                
                Ok(IRExpr::Where {
                    table: Box::new(table_ir),
                    condition: Box::new(condition_ir),
                    result_type: table_type.clone(),
                    source_loc: SourceRange::default(),
                })
            }
            
            ast::Expr::SortBy { table, columns } => {
                let table_ir = self.lower_expr(table)?;
                let table_type = table_ir.get_type();
                
                if !table_type.is_table() {
                    return Err(format!("sort requires a table, got {}", table_type));
                }
                
                let schema = table_type.as_table().unwrap();
                
                // Verify all columns exist in table
                for col in columns {
                    if !schema.has_field(&col.name) {
                        return Err(format!("Table {} has no column '{}'", 
                            schema.name, col.name));
                    }
                }
                
                let sort_specs = columns.iter().map(|c| SortSpec {
                    column: c.name.clone(),
                    ascending: c.ascending,
                }).collect();
                
                Ok(IRExpr::SortBy {
                    table: Box::new(table_ir),
                    columns: sort_specs,
                    result_type: table_type.clone(),
                    source_loc: SourceRange::default(),
                })
            }
            
            ast::Expr::ColumnSelect { table, columns } => {
                let table_ir = self.lower_expr(table)?;
                let table_type = table_ir.get_type();
                
                if !table_type.is_table() {
                    return Err(format!("column selection requires a table, got {}", table_type));
                }
                
                let schema = table_type.as_table().unwrap();
                
                // Verify all columns exist
                for col in columns {
                    if !schema.has_field(col) {
                        return Err(format!("Table {} has no column '{}'", schema.name, col));
                    }
                }
                
                // Create new schema with subset of fields
                let new_fields: Vec<Field> = columns.iter()
                    .filter_map(|col| schema.get_field(col).cloned())
                    .collect();
                
                let new_schema = TableSchema {
                    name: format!("{}[{}]", schema.name, columns.join(",")),
                    fields: new_fields,
                    constraints: Vec::new(),  // Column subset doesn't preserve constraints
                };
                
                Ok(IRExpr::ColumnSelect {
                    table: Box::new(table_ir),
                    columns: columns.clone(),
                    result_type: Type::Table(new_schema),
                    source_loc: SourceRange::default(),
                })
            }
            
            ast::Expr::BinaryOp { op, left, right } => {
                let left_ir = self.lower_expr(left)?;
                let right_ir = self.lower_expr(right)?;
                let left_type = left_ir.get_type();
                let right_type = right_ir.get_type();
                
                // Check if this is a set operation on tables
                match op {
                    ast::BinaryOp::Add if left_type.is_table() && right_type.is_table() => {
                        // Union operation
                        self.check_table_compatibility(left_type, right_type)?;
                        Ok(IRExpr::Union {
                            left: Box::new(left_ir),
                            right: Box::new(right_ir),
                            result_type: left_type.clone(),
                            source_loc: SourceRange::default(),
                        })
                    }
                    ast::BinaryOp::Subtract if left_type.is_table() && right_type.is_table() => {
                        // Minus operation
                        self.check_table_compatibility(left_type, right_type)?;
                        Ok(IRExpr::Minus {
                            left: Box::new(left_ir),
                            right: Box::new(right_ir),
                            result_type: left_type.clone(),
                            source_loc: SourceRange::default(),
                        })
                    }
                    // ... handle other cases ...
                }
            }
            
            ast::Expr::FieldAccess { object, field } => {
                let object_ir = self.lower_expr(object)?;
                let object_type = object_ir.get_type();
                
                // Check if this is a reference navigation
                if let Type::Table(schema) = object_type {
                    if let Some(field_info) = schema.get_field(field) {
                        if let FieldType::Ref { table_name, .. } = &field_info.ty {
                            // This is a reference navigation - need to perform lookup
                            let target_schema = self.get_table_schema(table_name)?;
                            return Ok(IRExpr::RefNavigation {
                                object: Box::new(object_ir),
                                field: field.clone(),
                                target_table: table_name.clone(),
                                result_type: Type::Table(target_schema),
                                source_loc: SourceRange::default(),
                            });
                        }
                    }
                }
                
                // Regular field access
                // ... existing field access logic ...
            }
        }
    }
    
    fn check_table_compatibility(&self, left: &Type, right: &Type) -> Result<(), String> {
        // Verify both are tables with same schema (or compatible schemas)
        let left_schema = left.as_table()
            .ok_or("Expected table type")?;
        let right_schema = right.as_table()
            .ok_or("Expected table type")?;
        
        // Check if schemas are compatible (same fields with same types)
        if left_schema.fields.len() != right_schema.fields.len() {
            return Err(format!("Table schemas are incompatible: different number of fields"));
        }
        
        for (lf, rf) in left_schema.fields.iter().zip(right_schema.fields.iter()) {
            if lf.name != rf.name || lf.ty != rf.ty {
                return Err(format!("Table schemas are incompatible: field '{}' doesn't match", lf.name));
            }
        }
        
        Ok(())
    }
    
    fn get_table_schema(&self, table_name: &str) -> Result<TableSchema, String> {
        // Look up table schema from symbol table
        self.module.get_table_schema(table_name)
            .cloned()
            .ok_or_else(|| format!("Unknown table: {}", table_name))
    }
}
```

**Estimated effort:** 10-12 hours
**Files to modify:**
- `crates/wtlang-core/src/ir/builder.rs`
- `crates/wtlang-core/src/ir/module.rs` (add helper methods)

**Testing:**
- Update IR builder tests to cover new expression types
- Test error cases (invalid where conditions, incompatible schemas, etc.)

### Phase 3: Semantic Analysis (Week 3)

#### 3.1 Symbol Table Extensions (`crates/wtlang-core/src/symbols.rs`)

**Track key fields in table definitions:**
```rust
pub struct SymbolTable {
    // ... existing fields ...
    
    /// Map of table name to key field name
    table_keys: HashMap<String, String>,
    
    /// Map of table name to fields that reference other tables
    table_refs: HashMap<String, Vec<(String, String)>>,  // (field_name, target_table)
}

impl SymbolTable {
    pub fn register_key(&mut self, table_name: String, key_field: String) {
        self.table_keys.insert(table_name, key_field);
    }
    
    pub fn register_ref(&mut self, table_name: String, field_name: String, target_table: String) {
        self.table_refs
            .entry(table_name)
            .or_insert_with(Vec::new)
            .push((field_name, target_table));
    }
    
    pub fn get_key_field(&self, table_name: &str) -> Option<&String> {
        self.table_keys.get(table_name)
    }
    
    pub fn get_ref_target(&self, table_name: &str, field_name: &str) -> Option<&String> {
        self.table_refs.get(table_name)?
            .iter()
            .find(|(f, _)| f == field_name)
            .map(|(_, t)| t)
    }
}
```

**Estimated effort:** 3-4 hours
**Files to modify:**
- `crates/wtlang-core/src/symbols.rs`

#### 3.2 Semantic Analysis (`crates/wtlang-core/src/semantics.rs`)

**Add validation for key constraints:**
```rust
impl SemanticAnalyzer {
    // ... existing methods ...
    
    fn analyze_table_def(&mut self, table: &ast::TableDef) -> Result<(), String> {
        // ... existing table analysis ...
        
        // Find key fields
        let mut key_fields = Vec::new();
        for field in &table.fields {
            for constraint in &field.constraints {
                if matches!(constraint, ast::Constraint::Key) {
                    key_fields.push(field.name.clone());
                }
            }
        }
        
        // Validate: at most one key field per table
        if key_fields.len() > 1 {
            self.diagnostics.add_error(
                ErrorCode::E3015,
                format!("Table '{}' has multiple key fields: {}. Only one key is allowed.",
                    table.name, key_fields.join(", ")),
                0, 0
            );
            return Err("Multiple key fields".to_string());
        }
        
        // Register key in symbol table
        if let Some(key) = key_fields.first() {
            self.symbols.register_key(table.name.clone(), key.clone());
        }
        
        // Find and validate reference fields
        for field in &table.fields {
            if let ast::Type::Ref(target_table) = &field.field_type {
                // Check target table exists
                if !self.symbols.has_table(target_table) {
                    self.diagnostics.add_error(
                        ErrorCode::E3016,
                        format!("Reference to unknown table '{}'", target_table),
                        0, 0
                    );
                    return Err("Unknown reference target".to_string());
                }
                
                // Check target table has a key
                if self.symbols.get_key_field(target_table).is_none() {
                    self.diagnostics.add_error(
                        ErrorCode::E3017,
                        format!("Table '{}' cannot be referenced because it has no key field",
                            target_table),
                        0, 0
                    );
                    return Err("Reference to table without key".to_string());
                }
                
                // Register the reference
                self.symbols.register_ref(
                    table.name.clone(),
                    field.name.clone(),
                    target_table.clone()
                );
            }
        }
        
        Ok(())
    }
}
```

**Estimated effort:** 4-5 hours
**Files to modify:**
- `crates/wtlang-core/src/semantics.rs`

**Testing:**
- Test key constraint validation
- Test reference validation (missing target, no key, etc.)

### Phase 4: Code Generation for Streamlit (Week 4)

#### 4.1 Generate Query Operations (`crates/wtlang-compiler/src/codegen.rs`)

**Generate code for where expressions:**
```rust
impl CodeGenerator {
    // ... existing methods ...
    
    fn generate_expr(&mut self, expr: &ir::IRExpr) -> Result<String, String> {
        match expr {
            // ... existing expression generation ...
            
            IRExpr::Where { table, condition, .. } => {
                let table_code = self.generate_expr(table)?;
                let condition_code = self.generate_where_condition(condition)?;
                
                // Generate pandas query
                Ok(format!("{}.query({})", table_code, condition_code))
            }
            
            IRExpr::SortBy { table, columns, .. } => {
                let table_code = self.generate_expr(table)?;
                
                // Build column list and ascending flags
                let col_names: Vec<String> = columns.iter()
                    .map(|c| format!("'{}'", c.column))
                    .collect();
                let ascending: Vec<String> = columns.iter()
                    .map(|c| c.ascending.to_string())
                    .collect();
                
                if columns.len() == 1 {
                    Ok(format!("{}.sort_values(by={}, ascending={})",
                        table_code, col_names[0], ascending[0]))
                } else {
                    Ok(format!("{}.sort_values(by=[{}], ascending=[{}])",
                        table_code,
                        col_names.join(", "),
                        ascending.join(", ")))
                }
            }
            
            IRExpr::ColumnSelect { table, columns, .. } => {
                let table_code = self.generate_expr(table)?;
                let cols = columns.iter()
                    .map(|c| format!("'{}'", c))
                    .collect::<Vec<_>>()
                    .join(", ");
                
                Ok(format!("{}[[{}]]", table_code, cols))
            }
            
            IRExpr::Union { left, right, .. } => {
                let left_code = self.generate_expr(left)?;
                let right_code = self.generate_expr(right)?;
                
                // Use pd.concat for union
                Ok(format!("pd.concat([{}, {}], ignore_index=True).drop_duplicates()",
                    left_code, right_code))
            }
            
            IRExpr::Minus { left, right, .. } => {
                let left_code = self.generate_expr(left)?;
                let right_code = self.generate_expr(right)?;
                
                // Set difference using merge with indicator
                let temp_var = format!("_diff_{}", self.get_unique_key());
                self.add_line(format!("{} = {}.merge({}, how='left', indicator=True)",
                    temp_var, left_code, right_code));
                
                Ok(format!("{}[{}['_merge'] == 'left_only'].drop('_merge', axis=1)",
                    temp_var, temp_var))
            }
            
            IRExpr::Intersect { left, right, .. } => {
                let left_code = self.generate_expr(left)?;
                let right_code = self.generate_expr(right)?;
                
                // Use merge for intersection
                Ok(format!("{}.merge({}, how='inner')", left_code, right_code))
            }
            
            IRExpr::RefNavigation { object, field, target_table, .. } => {
                let object_code = self.generate_expr(object)?;
                
                // Get key field of target table
                let target_key = self.get_table_key(target_table)?;
                
                // Generate lookup code
                // This will create a merged dataframe
                let result_var = format!("_ref_{}_{}", field, self.get_unique_key());
                
                self.add_line(format!(
                    "{} = {}.merge({}, left_on='{}', right_on='{}', how='left')",
                    result_var,
                    object_code,
                    target_table.to_lowercase(),  // Assume table is loaded as variable
                    field,
                    target_key
                ));
                
                Ok(result_var)
            }
        }
    }
    
    fn generate_where_condition(&mut self, condition: &ir::IRExpr) -> Result<String, String> {
        // Convert IR condition to pandas query string
        match condition {
            IRExpr::BinaryOp { op, left, right, .. } => {
                let left_str = self.generate_where_condition(left)?;
                let right_str = self.generate_where_condition(right)?;
                
                let op_str = match op {
                    ir::BinaryOp::Equal => "==",
                    ir::BinaryOp::NotEqual => "!=",
                    ir::BinaryOp::LessThan => "<",
                    ir::BinaryOp::LessThanOrEqual => "<=",
                    ir::BinaryOp::GreaterThan => ">",
                    ir::BinaryOp::GreaterThanOrEqual => ">=",
                    ir::BinaryOp::And => "and",
                    ir::BinaryOp::Or => "or",
                    _ => return Err("Invalid operator in where clause".to_string()),
                };
                
                Ok(format!("({} {} {})", left_str, op_str, right_str))
            }
            
            IRExpr::FieldAccess { field, .. } => {
                // In query string, just use column name
                Ok(field.clone())
            }
            
            IRExpr::IntLiteral(n) => Ok(n.to_string()),
            IRExpr::FloatLiteral(f) => Ok(f.to_string()),
            IRExpr::StringLiteral(s) => Ok(format!("'{}'", s)),
            IRExpr::BoolLiteral(b) => Ok(b.to_string()),
            
            _ => Err("Unsupported expression in where clause".to_string()),
        }
    }
    
    fn get_table_key(&self, table_name: &str) -> Result<String, String> {
        // Look up key field from IR module
        self.ir_module
            .get_table_schema(table_name)
            .and_then(|schema| schema.get_key_field())
            .map(|field| field.name.clone())
            .ok_or_else(|| format!("Table {} has no key field", table_name))
    }
}
```

**Estimated effort:** 12-15 hours
**Files to modify:**
- `crates/wtlang-compiler/src/codegen.rs`

**Testing:**
- Test each query operation generates correct pandas code
- Integration tests with example .wt files
- Verify generated code runs correctly

### Phase 5: Update LSP (Week 5)

#### 5.1 LSP Diagnostics (`crates/wtlang-lsp/src/main.rs`)

**Add error codes for query operations:**
```rust
// In error_codes.md, add:
// E3015: Multiple key fields in table
// E3016: Reference to unknown table
// E3017: Reference to table without key
// E3018: Where clause on non-table
// E3019: Sort on non-table
// E3020: Column selection on non-table
// E3021: Set operation on non-table
// E3022: Incompatible table schemas in set operation
```

#### 5.2 LSP Hover (`crates/wtlang-lsp/src/main.rs`)

**Add hover for ref types:**
```rust
fn generate_hover_for_ref_field(field_name: &str, target_table: &str, key_field: &str) -> String {
    format!(
        "**{}**: ref {}\n\nReferences table `{}` via key field `{}`\n\n\
         Accessing this field will perform a lookup in the {} table.",
        field_name, target_table, target_table, key_field, target_table
    )
}
```

#### 5.3 LSP Completion (`crates/wtlang-lsp/src/main.rs`)

**Add completions for query keywords:**
```rust
// When completing after table expression:
completions.push(CompletionItem {
    label: "where".to_string(),
    kind: Some(CompletionItemKind::KEYWORD),
    detail: Some("Filter table by condition".to_string()),
    documentation: Some(Documentation::String(
        "Example: users where age > 18".to_string()
    )),
    ..Default::default()
});

completions.push(CompletionItem {
    label: "sort by".to_string(),
    kind: Some(CompletionItemKind::KEYWORD),
    detail: Some("Sort table by columns".to_string()),
    documentation: Some(Documentation::String(
        "Example: users sort by name asc, age desc".to_string()
    )),
    ..Default::default()
});
```

**Add column completions in where/sort context:**
```rust
// When inside where clause or sort by, suggest table columns
if in_where_or_sort_context() {
    let table_type = get_table_being_queried();
    if let Some(schema) = table_type.as_table() {
        for field in &schema.fields {
            completions.push(CompletionItem {
                label: field.name.clone(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some(format!("{}", field.ty)),
                ..Default::default()
            });
        }
    }
}
```

**Estimated effort:** 8-10 hours
**Files to modify:**
- `crates/wtlang-lsp/src/main.rs`
- `doc/error_codes.md`

### Phase 6: Documentation and Examples (Week 6)

#### 6.1 Update Language Tutorial

**Add section on query language:**
```markdown
## Query Language

WTLang provides a concise query language for working with tables:

### Filtering with WHERE

Use the `where` keyword to filter tables:

```wtlang
let adults = users where age >= 18
let premium_users = users where (age >= 18 and subscription == "premium")
```

### Sorting

Sort tables using `sort by`:

```wtlang
let sorted = users sort by name asc
let sorted_multi = users sort by name asc, age desc
```

### Column Selection

Select specific columns using bracket notation:

```wtlang
let names_only = users[name, email]
```

### Set Operations

Perform set operations on tables with the same schema:

```wtlang
// Union (combine tables, removing duplicates)
let all_special = seniors + minors

// Difference (rows in first but not second)
let regular_users = all_users - premium_users

// Intersection (rows in both tables)
let active_premium = active_users & premium_users
```
```

**Add section on references:**
```markdown
## Table References

Define relationships between tables using keys and references:

### Primary Keys

Mark a field as the primary key using the `key` constraint:

```wtlang
table Department {
    code: string key
    name: string
}
```

### Reference Types

Use `ref` to create references to other tables:

```wtlang
table Employee {
    id: string key
    name: string
    department: ref Department  // References Department table
}
```

### Navigating References

Access referenced tables directly:

```wtlang
let employees = load_csv("employees.csv", Employee)

// Get departments (automatically joins Employee and Department)
let departments = employees.department
```
```

**Estimated effort:** 4-5 hours
**Files to modify:**
- `doc/tutorial.md`
- `doc/syntax_reference.md`

#### 6.2 Create Examples

**Create `examples/08_query_language.wt`:**
```wtlang
table User {
    id: int key
    name: string
    age: int
    city: string
    subscription: string
}

page QueryExamples {
    title "Query Language Examples"
    
    let users = load_csv("data/users.csv", User)
    
    // WHERE clause
    section "Filtering" {
        let adults = users where age >= 18
        show(adults)
        
        let premium = users where subscription == "premium"
        show(premium)
        
        let young_premium = users where (age < 30 and subscription == "premium")
        show(young_premium)
    }
    
    // SORT BY
    section "Sorting" {
        let by_name = users sort by name asc
        show(by_name)
        
        let by_age_name = users sort by age desc, name asc
        show(by_age_name)
    }
    
    // Column selection
    section "Column Selection" {
        let names = users[name, city]
        show(names)
    }
    
    // Set operations
    section "Set Operations" {
        let seniors = users where age >= 65
        let minors = users where age < 18
        
        let special_rates = seniors + minors
        show(special_rates)
        
        let regular = users - special_rates
        show(regular)
    }
}
```

**Create `examples/09_references.wt`:**
```wtlang
table Department {
    code: string key
    name: string
    budget: currency
}

table Employee {
    id: string key
    name: string
    department: ref Department
    salary: currency
}

page ReferenceExample {
    title "Working with References"
    
    let departments = load_csv("data/departments.csv", Department)
    let employees = load_csv("data/employees.csv", Employee)
    
    section "Employees" {
        show(employees)
    }
    
    section "Departments via Reference" {
        // Automatically looks up department for each employee
        let employee_depts = employees.department
        show(employee_depts)
    }
    
    section "Filtered by Department" {
        let it_employees = employees where department.name == "IT"
        show(it_employees)
    }
}
```

**Create test data files:**
- `examples/data/departments.csv`
- Update `examples/data/users.csv` if needed

**Estimated effort:** 3-4 hours
**Files to create:**
- `examples/08_query_language.wt`
- `examples/09_references.wt`
- `examples/data/departments.csv`

#### 6.3 Update Tests

**Create parser tests:**
```rust
#[test]
fn test_where_clause() {
    let input = "let adults = users where age > 18";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    // ... assertions ...
}

#[test]
fn test_sort_by() {
    let input = "let sorted = users sort by name asc, age desc";
    // ... test ...
}

#[test]
fn test_column_select() {
    let input = "let names = users[name, email]";
    // ... test ...
}

#[test]
fn test_ref_type() {
    let input = r#"
        table Dept { code: string key }
        table Emp { dept: ref Dept }
    "#;
    // ... test ...
}
```

**Create integration tests:**
- Compile all example files
- Verify generated Python code is correct
- Run generated code (if possible)

**Estimated effort:** 6-8 hours
**Files to modify:**
- `crates/wtlang-core/src/parser.rs` (add tests)
- `tests/` directory (add integration tests)

### Phase 7: Error Handling and Edge Cases (Week 7)

#### 7.1 Error Scenarios to Handle

1. **Query Operations on Non-Tables**:
   - `let x = 5 where y > 3` → Error: where requires table
   - `let x = "hello" sort by name` → Error: sort requires table

2. **Invalid Column Names**:
   - `users where invalid_col > 5` → Error: column doesn't exist
   - `users[invalid, missing]` → Error: columns don't exist

3. **Type Mismatches in Where**:
   - `users where age == "hello"` → Error: comparing int with string
   - `users where name > 5` → Error: can't compare string with int

4. **Incompatible Schemas in Set Ops**:
   - `users + products` → Error: different schemas
   - `users - partial_users[name]` → Error: different columns

5. **Reference Errors**:
   - `ref UnknownTable` → Error: table doesn't exist
   - `ref TableWithoutKey` → Error: target has no key
   - Multiple keys in one table → Error: only one key allowed

6. **Circular References**:
   - Table A refs B, B refs A → Should be allowed
   - Table A refs itself → Should be allowed (self-reference)

**Implement comprehensive error messages:**
```rust
// Example error messages
match error_code {
    ErrorCode::E3018 => format!(
        "Cannot use 'where' on non-table type '{}'. \
         The 'where' clause can only filter tables.",
        actual_type
    ),
    ErrorCode::E3022 => format!(
        "Cannot perform set operation on incompatible tables. \
         Table '{}' has schema [{}] but table '{}' has schema [{}].",
        left_table, left_schema, right_table, right_schema
    ),
    // ... etc ...
}
```

**Estimated effort:** 8-10 hours
**Files to modify:**
- All semantic analysis and IR builder files
- `doc/error_codes.md`

### Phase 8: Testing and Validation (Week 8)

#### 8.1 Unit Tests

- Lexer: Test new tokens (where, by, key, ref, asc, desc)
- Parser: Test all new syntax forms
- AST: Verify structures are correct
- IR Builder: Test lowering of all new expressions
- Semantic Analysis: Test all error cases
- Code Generator: Test pandas code generation

#### 8.2 Integration Tests

- Compile all examples
- Verify generated code
- Run with actual data (if possible)

#### 8.3 LSP Tests

- Test hover on ref fields
- Test completion in where/sort contexts
- Test diagnostics for all error cases

#### 8.4 Example Validation

Run compiler on all examples:
```powershell
scripts\test_examples.ps1
```

Verify:
- All examples compile successfully
- Generated Python code is syntactically correct
- Code runs without errors (manual testing)

**Estimated effort:** 12-15 hours

## Summary

### Total Estimated Time: 8 weeks (320-360 hours)

### Phases:
1. **Week 1**: Lexer and AST Extensions
2. **Week 2**: IR Extensions
3. **Week 3**: Semantic Analysis
4. **Week 4**: Code Generation
5. **Week 5**: LSP Updates
6. **Week 6**: Documentation and Examples
7. **Week 7**: Error Handling
8. **Week 8**: Testing and Validation

### Files to Modify:

**Core Library** (`crates/wtlang-core/src/`):
- `lexer.rs` - Add keywords and tokens
- `ast.rs` - Extend types and expressions
- `parser.rs` - Parse new syntax
- `symbols.rs` - Track keys and references
- `semantics.rs` - Validate constraints
- `ir/types.rs` - Extend IR type system
- `ir/nodes.rs` - Add IR expressions
- `ir/builder.rs` - AST→IR lowering
- `ir/module.rs` - Helper methods

**Compiler** (`crates/wtlang-compiler/src/`):
- `codegen.rs` - Generate pandas code

**LSP** (`crates/wtlang-lsp/src/`):
- `main.rs` - Diagnostics, hover, completion

**Documentation** (`doc/`):
- `tutorial.md` - Add query language section
- `syntax_reference.md` - Document new syntax
- `error_codes.md` - Add new error codes

**Examples** (`examples/`):
- `08_query_language.wt` - Query examples
- `09_references.wt` - Reference examples
- `data/departments.csv` - Test data

**Tests** (`tests/`):
- Add unit tests for all components
- Add integration tests

### Risk Factors:

1. **Set Operation Complexity**: Union/minus/intersect on pandas DataFrames can be tricky. May need to adjust approach based on pandas capabilities.

2. **Reference Navigation Performance**: Automatic joins for references could be slow on large datasets. May need to add optimization hints.

3. **Parser Ambiguity**: The `where` keyword vs function might create parsing ambiguities. Careful precedence handling needed.

4. **Backward Compatibility**: Need to ensure existing code still works. The `where` builtin function should still work alongside new syntax.

### Success Criteria:

- ✅ All example files compile successfully
- ✅ Generated pandas code is correct and runs
- ✅ LSP provides useful completions and diagnostics
- ✅ All tests pass
- ✅ Documentation is complete and accurate
- ✅ Error messages are clear and helpful

### Future Enhancements (Not in this plan):

- Aggregation with group by: `users group by city aggregate sum(age)`
- Joins beyond references: `employees join departments on ...`
- Subqueries: `users where id in (select id from active)`
- Window functions: `row_number() over (partition by dept)`
- Query optimization in IR

## Next Steps

After reviewing this plan:
1. Start with Phase 1 (Lexer and AST)
2. Create branch: `feature/query-language`
3. Implement incrementally, testing after each phase
4. Review and adjust plan based on findings
5. Document any deviations or improvements
