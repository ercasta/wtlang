// Symbol table implementation for WTLang

use crate::ast::Type;
use std::collections::HashMap;
use std::sync::Arc;

/// Symbol table for a single scope
#[derive(Debug, Clone)]
pub struct Scope {
    /// Parent scope (None for global scope)
    parent: Option<Arc<Scope>>,
    
    /// Symbols defined in this scope
    symbols: HashMap<String, Symbol>,
    
    /// Scope kind for error messages
    kind: ScopeKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScopeKind {
    Global,
    Page,
    Section,
    Button,
    IfBranch,
    ForallLoop,
    FunctionBody,
    TestBody,
}

/// Information about a symbol
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    
    /// Symbol type
    pub symbol_type: Type,
    
    /// Kind of symbol
    pub kind: SymbolKind,
    
    /// Whether the symbol has been assigned a value
    pub is_initialized: bool,
    
    /// Whether the symbol can be reassigned (for future use)
    pub is_mutable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable,
    Parameter,
    LoopVariable,
    Table,
    Function,
    ExternalFunction,
}

#[derive(Debug, Clone)]
pub enum SymbolError {
    Redefinition {
        name: String,
    },
    UndefinedVariable {
        name: String,
    },
    TypeMismatch {
        name: String,
        expected: Type,
        found: Type,
    },
    UninitializedVariable {
        name: String,
    },
}

impl std::fmt::Display for SymbolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolError::Redefinition { name } => {
                write!(f, "Variable '{}' is already defined", name)
            }
            SymbolError::UndefinedVariable { name } => {
                write!(f, "Undefined variable '{}'", name)
            }
            SymbolError::TypeMismatch { name, expected, found } => {
                write!(f, "Type mismatch for '{}': expected {:?}, found {:?}", name, expected, found)
            }
            SymbolError::UninitializedVariable { name } => {
                write!(f, "Variable '{}' used before being initialized", name)
            }
        }
    }
}

impl std::error::Error for SymbolError {}

impl Scope {
    /// Create a new scope with optional parent
    pub fn new(parent: Option<Arc<Scope>>, kind: ScopeKind) -> Self {
        Scope {
            parent,
            symbols: HashMap::new(),
            kind,
        }
    }
    
    /// Define a new symbol in this scope
    pub fn define(&mut self, name: String, symbol: Symbol) -> Result<(), SymbolError> {
        if self.symbols.contains_key(&name) {
            return Err(SymbolError::Redefinition { name });
        }
        self.symbols.insert(name, symbol);
        Ok(())
    }
    
    /// Look up a symbol in this scope or parent scopes
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
            .or_else(|| self.parent.as_ref()?.lookup(name))
    }
    
    /// Look up a symbol only in this scope (not parent)
    pub fn lookup_local(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }
    
    /// Get scope kind
    pub fn kind(&self) -> ScopeKind {
        self.kind
    }
    
    /// Get all symbols in this scope
    pub fn symbols(&self) -> &HashMap<String, Symbol> {
        &self.symbols
    }
}

/// Global symbol table managing all scopes
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// Global scope containing tables, functions, etc.
    global: Arc<Scope>,
    
    /// Current scope stack during analysis
    current_scopes: Vec<Arc<Scope>>,
    
    /// Map of table name to key field name
    table_keys: HashMap<String, String>,
    
    /// Map of table name to fields that reference other tables
    /// Each entry is (field_name, target_table)
    table_refs: HashMap<String, Vec<(String, String)>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            global: Arc::new(Scope::new(None, ScopeKind::Global)),
            current_scopes: vec![],
            table_keys: HashMap::new(),
            table_refs: HashMap::new(),
        }
    }
    
    /// Enter a new scope
    pub fn push_scope(&mut self, kind: ScopeKind) {
        let parent = self.current_scope();
        let new_scope = Arc::new(Scope::new(Some(parent.clone()), kind));
        self.current_scopes.push(new_scope);
    }
    
    /// Exit current scope
    pub fn pop_scope(&mut self) -> Option<Arc<Scope>> {
        self.current_scopes.pop()
    }
    
    /// Get current scope (or global if no scopes pushed)
    pub fn current_scope(&self) -> Arc<Scope> {
        self.current_scopes.last()
            .cloned()
            .unwrap_or_else(|| self.global.clone())
    }
    
    /// Get global scope
    pub fn global_scope(&self) -> Arc<Scope> {
        self.global.clone()
    }
    
    /// Define symbol in current scope
    pub fn define(&mut self, name: String, symbol: Symbol) -> Result<(), SymbolError> {
        if self.current_scopes.is_empty() {
            // Define in global scope
            Arc::make_mut(&mut self.global).define(name, symbol)
        } else {
            // Define in current scope
            let idx = self.current_scopes.len() - 1;
            Arc::make_mut(&mut self.current_scopes[idx]).define(name, symbol)
        }
    }
    
    /// Look up symbol from current scope
    pub fn lookup(&self, name: &str) -> Option<Symbol> {
        self.current_scope().lookup(name).cloned()
    }
    
    /// Mark a variable as initialized
    pub fn mark_initialized(&mut self, name: &str) -> Result<(), SymbolError> {
        // Try to find and update the symbol in current or parent scopes
        for i in (0..self.current_scopes.len()).rev() {
            let scope = &self.current_scopes[i];
            if scope.lookup_local(name).is_some() {
                let scope_mut = Arc::make_mut(&mut self.current_scopes[i]);
                if let Some(symbol) = scope_mut.symbols.get_mut(name) {
                    symbol.is_initialized = true;
                    return Ok(());
                }
            }
        }
        
        // Check global scope
        if self.global.lookup_local(name).is_some() {
            let global_mut = Arc::make_mut(&mut self.global);
            if let Some(symbol) = global_mut.symbols.get_mut(name) {
                symbol.is_initialized = true;
                return Ok(());
            }
        }
        
        Err(SymbolError::UndefinedVariable { name: name.to_string() })
    }
    
    /// Register a key field for a table
    pub fn register_key(&mut self, table_name: String, key_field: String) {
        self.table_keys.insert(table_name, key_field);
    }
    
    /// Register a reference field for a table
    pub fn register_ref(&mut self, table_name: String, field_name: String, target_table: String) {
        self.table_refs
            .entry(table_name)
            .or_insert_with(Vec::new)
            .push((field_name, target_table));
    }
    
    /// Get the key field for a table
    pub fn get_key_field(&self, table_name: &str) -> Option<&String> {
        self.table_keys.get(table_name)
    }
    
    /// Get the target table for a reference field
    pub fn get_ref_target(&self, table_name: &str, field_name: &str) -> Option<&String> {
        self.table_refs.get(table_name)?
            .iter()
            .find(|(f, _)| f == field_name)
            .map(|(_, t)| t)
    }
    
    /// Check if a table exists in the symbol table
    pub fn has_table(&self, table_name: &str) -> bool {
        self.lookup(table_name)
            .map(|s| s.kind == SymbolKind::Table)
            .unwrap_or(false)
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_and_lookup_in_global() {
        let mut table = SymbolTable::new();
        
        let symbol = Symbol {
            name: "global_var".to_string(),
            symbol_type: Type::Float,
            kind: SymbolKind::Variable,
            is_initialized: true,
            is_mutable: true,
        };
        
        table.define("global_var".to_string(), symbol).unwrap();
        let found = table.lookup("global_var").unwrap();
        
        assert_eq!(found.name, "global_var");
        assert_eq!(found.symbol_type, Type::Float);
        assert!(found.is_initialized);
    }

    #[test]
    fn test_nested_scopes() {
        let mut table = SymbolTable::new();
        
        // Define in global
        let global_sym = Symbol {
            name: "outer".to_string(),
            symbol_type: Type::String,
            kind: SymbolKind::Variable,
            is_initialized: true,
            is_mutable: true,
        };
        table.define("outer".to_string(), global_sym).unwrap();
        
        // Enter page scope
        table.push_scope(ScopeKind::Page);
        let page_sym = Symbol {
            name: "inner".to_string(),
            symbol_type: Type::Int,
            kind: SymbolKind::Variable,
            is_initialized: true,
            is_mutable: true,
        };
        table.define("inner".to_string(), page_sym).unwrap();
        
        // Inner scope can see both
        assert!(table.lookup("outer").is_some());
        assert!(table.lookup("inner").is_some());
        
        // Exit scope
        table.pop_scope();
        
        // Outer scope can't see inner
        assert!(table.lookup("outer").is_some());
        assert!(table.lookup("inner").is_none());
    }

    #[test]
    fn test_shadowing() {
        let mut table = SymbolTable::new();
        
        // Define in global
        let outer_sym = Symbol {
            name: "x".to_string(),
            symbol_type: Type::Int,
            kind: SymbolKind::Variable,
            is_initialized: true,
            is_mutable: true,
        };
        table.define("x".to_string(), outer_sym).unwrap();
        
        // Enter function scope
        table.push_scope(ScopeKind::FunctionBody);
        let inner_sym = Symbol {
            name: "x".to_string(),
            symbol_type: Type::String,
            kind: SymbolKind::Variable,
            is_initialized: true,
            is_mutable: true,
        };
        table.define("x".to_string(), inner_sym).unwrap();
        
        // Inner shadows outer
        let found = table.lookup("x").unwrap();
        assert_eq!(found.symbol_type, Type::String);
        
        // Exit scope
        table.pop_scope();
        
        // Back to outer
        let found = table.lookup("x").unwrap();
        assert_eq!(found.symbol_type, Type::Int);
    }

    #[test]
    fn test_redefinition_error() {
        let mut table = SymbolTable::new();
        
        let sym1 = Symbol {
            name: "var".to_string(),
            symbol_type: Type::Float,
            kind: SymbolKind::Variable,
            is_initialized: true,
            is_mutable: true,
        };
        table.define("var".to_string(), sym1).unwrap();
        
        let sym2 = Symbol {
            name: "var".to_string(),
            symbol_type: Type::String,
            kind: SymbolKind::Variable,
            is_initialized: true,
            is_mutable: true,
        };
        let result = table.define("var".to_string(), sym2);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            SymbolError::Redefinition { name } => assert_eq!(name, "var"),
            _ => panic!("Expected Redefinition error"),
        }
    }

    #[test]
    fn test_undefined_variable() {
        let table = SymbolTable::new();
        assert!(table.lookup("undefined").is_none());
    }

    #[test]
    fn test_mark_initialized() {
        let mut table = SymbolTable::new();
        
        let symbol = Symbol {
            name: "var".to_string(),
            symbol_type: Type::Int,
            kind: SymbolKind::Variable,
            is_initialized: false,
            is_mutable: true,
        };
        table.define("var".to_string(), symbol).unwrap();
        
        // Initially not initialized
        assert!(!table.lookup("var").unwrap().is_initialized);
        
        // Mark as initialized
        table.mark_initialized("var").unwrap();
        
        // Now initialized
        assert!(table.lookup("var").unwrap().is_initialized);
    }

    #[test]
    fn test_mark_initialized_in_nested_scope() {
        let mut table = SymbolTable::new();
        
        table.push_scope(ScopeKind::Page);
        let symbol = Symbol {
            name: "var".to_string(),
            symbol_type: Type::Float,
            kind: SymbolKind::Variable,
            is_initialized: false,
            is_mutable: true,
        };
        table.define("var".to_string(), symbol).unwrap();
        
        table.push_scope(ScopeKind::IfBranch);
        table.mark_initialized("var").unwrap();
        
        // Should be marked as initialized in parent scope
        table.pop_scope();
        assert!(table.lookup("var").unwrap().is_initialized);
    }

    #[test]
    fn test_function_parameters() {
        let mut table = SymbolTable::new();
        
        table.push_scope(ScopeKind::FunctionBody);
        
        let param = Symbol {
            name: "x".to_string(),
            symbol_type: Type::Int,
            kind: SymbolKind::Parameter,
            is_initialized: true,  // Parameters are initialized by definition
            is_mutable: true,
        };
        table.define("x".to_string(), param).unwrap();
        
        let found = table.lookup("x").unwrap();
        assert_eq!(found.kind, SymbolKind::Parameter);
        assert!(found.is_initialized);
    }

    #[test]
    fn test_loop_variables() {
        let mut table = SymbolTable::new();
        
        table.push_scope(ScopeKind::ForallLoop);
        
        let loop_var = Symbol {
            name: "item".to_string(),
            symbol_type: Type::String,
            kind: SymbolKind::LoopVariable,
            is_initialized: true,
            is_mutable: false,  // Loop variables typically shouldn't be reassigned
        };
        table.define("item".to_string(), loop_var).unwrap();
        
        let found = table.lookup("item").unwrap();
        assert_eq!(found.kind, SymbolKind::LoopVariable);
    }

    #[test]
    fn test_multiple_scope_levels() {
        let mut table = SymbolTable::new();
        
        // Global
        let global_sym = Symbol {
            name: "global".to_string(),
            symbol_type: Type::Float,
            kind: SymbolKind::Variable,
            is_initialized: true,
            is_mutable: true,
        };
        table.define("global".to_string(), global_sym).unwrap();
        
        // Page scope
        table.push_scope(ScopeKind::Page);
        let page_sym = Symbol {
            name: "page_var".to_string(),
            symbol_type: Type::String,
            kind: SymbolKind::Variable,
            is_initialized: true,
            is_mutable: true,
        };
        table.define("page_var".to_string(), page_sym).unwrap();
        
        // Section scope
        table.push_scope(ScopeKind::Section);
        let section_sym = Symbol {
            name: "section_var".to_string(),
            symbol_type: Type::Bool,
            kind: SymbolKind::Variable,
            is_initialized: true,
            is_mutable: true,
        };
        table.define("section_var".to_string(), section_sym).unwrap();
        
        // All three should be visible
        assert!(table.lookup("global").is_some());
        assert!(table.lookup("page_var").is_some());
        assert!(table.lookup("section_var").is_some());
        
        // Exit section
        table.pop_scope();
        assert!(table.lookup("global").is_some());
        assert!(table.lookup("page_var").is_some());
        assert!(table.lookup("section_var").is_none());
        
        // Exit page
        table.pop_scope();
        assert!(table.lookup("global").is_some());
        assert!(table.lookup("page_var").is_none());
    }

    #[test]
    fn test_symbol_kinds() {
        let mut table = SymbolTable::new();
        
        // Table definition
        let table_sym = Symbol {
            name: "User".to_string(),
            symbol_type: Type::Table("User".to_string()),
            kind: SymbolKind::Table,
            is_initialized: true,
            is_mutable: false,
        };
        table.define("User".to_string(), table_sym).unwrap();
        
        // Function definition (functions don't have a specific Type variant in this AST)
        let func_sym = Symbol {
            name: "add".to_string(),
            symbol_type: Type::String,  // Placeholder since there's no Type::Function
            kind: SymbolKind::Function,
            is_initialized: true,
            is_mutable: false,
        };
        table.define("add".to_string(), func_sym).unwrap();
        
        assert_eq!(table.lookup("User").unwrap().kind, SymbolKind::Table);
        assert_eq!(table.lookup("add").unwrap().kind, SymbolKind::Function);
    }

    #[test]
    fn test_scope_isolation_between_pages() {
        let mut table = SymbolTable::new();
        
        // Page 1
        table.push_scope(ScopeKind::Page);
        let sym1 = Symbol {
            name: "page1_var".to_string(),
            symbol_type: Type::Int,
            kind: SymbolKind::Variable,
            is_initialized: true,
            is_mutable: true,
        };
        table.define("page1_var".to_string(), sym1).unwrap();
        table.pop_scope();
        
        // Page 2
        table.push_scope(ScopeKind::Page);
        // Should not see page1_var
        assert!(table.lookup("page1_var").is_none());
        
        let sym2 = Symbol {
            name: "page2_var".to_string(),
            symbol_type: Type::String,
            kind: SymbolKind::Variable,
            is_initialized: true,
            is_mutable: true,
        };
        table.define("page2_var".to_string(), sym2).unwrap();
        assert!(table.lookup("page2_var").is_some());
    }
}
