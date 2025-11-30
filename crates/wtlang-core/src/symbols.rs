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
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            global: Arc::new(Scope::new(None, ScopeKind::Global)),
            current_scopes: vec![],
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
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
