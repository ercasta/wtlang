// IR Type System
//
// Fully resolved and type-checked type information for the IR

use serde::{Deserialize, Serialize};
use std::fmt;

/// Fully resolved types in the IR
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    /// Basic types
    Int,
    Float,
    String,
    Bool,
    Date,
    Currency,
    
    /// Table with fully resolved schema
    Table(TableSchema),
    
    /// Filter specification
    Filter {
        table_name: String,
        mode: FilterMode,
    },
    
    /// Function type
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    
    /// Unit type (no value)
    Unit,
    
    /// Error type (for error recovery)
    Error,
}

impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float | Type::Currency)
    }
    
    pub fn is_comparable(&self) -> bool {
        matches!(self, Type::Int | Type::Float | Type::Currency | Type::String | Type::Date | Type::Bool)
    }
    
    pub fn is_table(&self) -> bool {
        matches!(self, Type::Table(_))
    }
    
    pub fn as_table(&self) -> Option<&TableSchema> {
        match self {
            Type::Table(schema) => Some(schema),
            _ => None,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::String => write!(f, "string"),
            Type::Bool => write!(f, "bool"),
            Type::Date => write!(f, "date"),
            Type::Currency => write!(f, "currency"),
            Type::Table(schema) => write!(f, "table<{}>", schema.name),
            Type::Filter { table_name, mode } => write!(f, "filter<{}, {:?}>", table_name, mode),
            Type::Function { params, return_type } => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", return_type)
            }
            Type::Unit => write!(f, "()"),
            Type::Error => write!(f, "<error>"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TableSchema {
    pub name: String,
    pub fields: Vec<Field>,
    pub constraints: Vec<Constraint>,
}

impl TableSchema {
    pub fn new(name: String) -> Self {
        TableSchema {
            name,
            fields: Vec::new(),
            constraints: Vec::new(),
        }
    }
    
    pub fn has_field(&self, field_name: &str) -> bool {
        self.fields.iter().any(|f| f.name == field_name)
    }
    
    pub fn get_field(&self, field_name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == field_name)
    }
    
    pub fn get_field_type(&self, field_name: &str) -> Option<&FieldType> {
        self.get_field(field_name).map(|f| &f.ty)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub ty: FieldType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FieldType {
    Int,
    Float,
    String,
    Bool,
    Date,
    Currency,
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldType::Int => write!(f, "int"),
            FieldType::Float => write!(f, "float"),
            FieldType::String => write!(f, "string"),
            FieldType::Bool => write!(f, "bool"),
            FieldType::Date => write!(f, "date"),
            FieldType::Currency => write!(f, "currency"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Constraint {
    Unique(String),        // Field name
    NonNull(String),       // Field name
    PrimaryKey(String),    // Field name
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FilterMode {
    Single,
    Multi,
}

/// Convert AST type to IR type (without table resolution yet)
impl From<&crate::ast::Type> for Type {
    fn from(ast_type: &crate::ast::Type) -> Self {
        match ast_type {
            crate::ast::Type::Int => Type::Int,
            crate::ast::Type::Float => Type::Float,
            crate::ast::Type::String => Type::String,
            crate::ast::Type::Bool => Type::Bool,
            crate::ast::Type::Date => Type::Date,
            crate::ast::Type::Currency => Type::Currency,
            crate::ast::Type::Filter => Type::Error, // Will be resolved during semantic analysis
            crate::ast::Type::Table(name) => {
                // Placeholder - will be resolved with actual schema during semantic analysis
                Type::Table(TableSchema::new(name.clone()))
            }
        }
    }
}

impl From<&crate::ast::FilterMode> for FilterMode {
    fn from(mode: &crate::ast::FilterMode) -> Self {
        match mode {
            crate::ast::FilterMode::Single => FilterMode::Single,
            crate::ast::FilterMode::Multi => FilterMode::Multi,
        }
    }
}

impl From<&crate::ast::Type> for FieldType {
    fn from(ast_type: &crate::ast::Type) -> Self {
        match ast_type {
            crate::ast::Type::Int => FieldType::Int,
            crate::ast::Type::Float => FieldType::Float,
            crate::ast::Type::String => FieldType::String,
            crate::ast::Type::Bool => FieldType::Bool,
            crate::ast::Type::Date => FieldType::Date,
            crate::ast::Type::Currency => FieldType::Currency,
            _ => panic!("Cannot convert {:?} to FieldType", ast_type),
        }
    }
}
