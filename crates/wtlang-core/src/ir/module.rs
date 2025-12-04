// IR Module
//
// Top-level structure representing a complete WTLang program in IR form

use crate::ir::nodes::*;
use crate::symbols::SymbolTable;
use std::collections::HashMap;

/// Complete IR representation of a WTLang program
#[derive(Debug, Clone)]
pub struct IRModule {
    pub name: String,
    pub items: Vec<IRItem>,
    #[allow(dead_code)]
    pub symbols: SymbolTable,
    /// Type environment: maps variable names to their types
    pub type_env: HashMap<String, crate::ir::types::Type>,
}

impl IRModule {
    pub fn new(name: String) -> Self {
        IRModule {
            name,
            items: Vec::new(),
            symbols: SymbolTable::new(),
            type_env: HashMap::new(),
        }
    }
    
    /// Find a table definition by name
    pub fn find_table(&self, name: &str) -> Option<&crate::ir::types::TableSchema> {
        for item in &self.items {
            if let IRItem::TableDef { name: table_name, schema, .. } = item {
                if table_name == name {
                    return Some(schema);
                }
            }
        }
        None
    }
    
    /// Find a function definition by name
    pub fn find_function(&self, name: &str) -> Option<(&Vec<Param>, &crate::ir::types::Type)> {
        for item in &self.items {
            if let IRItem::FunctionDef { name: fn_name, params, return_type, .. } = item {
                if fn_name == name {
                    return Some((params, return_type));
                }
            }
        }
        None
    }
    
    /// Find a page definition by name
    pub fn find_page(&self, name: &str) -> Option<&Vec<IRNode>> {
        for item in &self.items {
            if let IRItem::PageDef { name: page_name, body, .. } = item {
                if page_name == name {
                    return Some(body);
                }
            }
        }
        None
    }
    
    /// Get all table names
    pub fn table_names(&self) -> Vec<&str> {
        self.items.iter()
            .filter_map(|item| {
                if let IRItem::TableDef { name, .. } = item {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get all function names
    pub fn function_names(&self) -> Vec<&str> {
        self.items.iter()
            .filter_map(|item| {
                if let IRItem::FunctionDef { name, .. } = item {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get all page names
    pub fn page_names(&self) -> Vec<&str> {
        self.items.iter()
            .filter_map(|item| {
                if let IRItem::PageDef { name, .. } = item {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect()
    }
}
