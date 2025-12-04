// IR Node Definitions
//
// Platform-independent representation of program structure

use serde::{Deserialize, Serialize};
use crate::ir::types::*;
use std::path::PathBuf;

/// Source location information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRange {
    pub file: PathBuf,
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Position { line, column }
    }
}

impl Default for SourceRange {
    fn default() -> Self {
        SourceRange {
            file: PathBuf::from("<unknown>"),
            start: Position::new(0, 0),
            end: Position::new(0, 0),
        }
    }
}

/// Target code location (for source maps)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetLocation {
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
}

/// Top-level IR items
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IRItem {
    TableDef {
        name: String,
        schema: TableSchema,
        source_loc: SourceRange,
    },
    
    FunctionDef {
        name: String,
        params: Vec<Param>,
        return_type: Type,
        body: Vec<IRNode>,
        is_external: bool,
        external_info: Option<ExternalInfo>,
        source_loc: SourceRange,
    },
    
    PageDef {
        name: String,
        body: Vec<IRNode>,
        source_loc: SourceRange,
    },
    
    TestDef {
        name: String,
        body: Vec<IRNode>,
        source_loc: SourceRange,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalInfo {
    pub language: String,
    pub module: String,
}

/// IR statements/nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IRNode {
    // UI Display
    ShowTable {
        table: Box<IRExpr>,
        filters: Vec<FilterSpec>,
        editable: bool,
        key: String,
        source_loc: SourceRange,
        target_loc: Option<TargetLocation>,
    },
    
    ShowText {
        text: String,
        style: TextStyle,
        source_loc: SourceRange,
    },
    
    Button {
        label: String,
        body: Vec<IRNode>,
        source_loc: SourceRange,
    },
    
    Section {
        title: String,
        body: Vec<IRNode>,
        source_loc: SourceRange,
    },
    
    // Control Flow
    Conditional {
        condition: Box<IRExpr>,
        then_branch: Vec<IRNode>,
        else_branch: Option<Vec<IRNode>>,
        source_loc: SourceRange,
    },
    
    Loop {
        variable: String,
        iterable: Box<IRExpr>,
        body: Vec<IRNode>,
        source_loc: SourceRange,
    },
    
    // Variables
    Binding {
        name: String,
        ty: Type,
        value: Option<Box<IRExpr>>,
        source_loc: SourceRange,
    },
    
    Assignment {
        target: String,
        value: Box<IRExpr>,
        source_loc: SourceRange,
    },
    
    // Expression statement
    ExprStmt {
        expr: Box<IRExpr>,
        source_loc: SourceRange,
    },
    
    // Return
    Return {
        value: Option<Box<IRExpr>>,
        source_loc: SourceRange,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterSpec {
    pub column: String,
    pub mode: FilterMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortSpec {
    pub column: String,
    pub ascending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextStyle {
    Title,
    Subtitle,
    Normal,
}

/// IR expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IRExpr {
    Literal {
        value: Literal,
        ty: Type,
    },
    
    Variable {
        name: String,
        ty: Type,
    },
    
    BinaryOp {
        op: BinOp,
        left: Box<IRExpr>,
        right: Box<IRExpr>,
        ty: Type,
    },
    
    UnaryOp {
        op: UnOp,
        operand: Box<IRExpr>,
        ty: Type,
    },
    
    FunctionCall {
        function: String,
        args: Vec<IRExpr>,
        ty: Type,
    },
    
    FieldAccess {
        object: Box<IRExpr>,
        field: String,
        ty: Type,
    },
    
    Index {
        object: Box<IRExpr>,
        index: Box<IRExpr>,
        ty: Type,
    },
    
    Chain {
        left: Box<IRExpr>,
        right: Box<IRExpr>,
        ty: Type,
    },
    
    TableConstructor {
        fields: Vec<(String, IRExpr)>,
        ty: Type,
    },
    
    ArrayConstructor {
        elements: Vec<IRExpr>,
        ty: Type,
    },
    
    Lambda {
        params: Vec<String>,
        body: Box<IRExpr>,
        ty: Type,
    },
    
    // Query operations
    Where {
        table: Box<IRExpr>,
        condition: Box<IRExpr>,
        ty: Type,
    },
    
    SortBy {
        table: Box<IRExpr>,
        columns: Vec<SortSpec>,
        ty: Type,
    },
    
    ColumnSelect {
        table: Box<IRExpr>,
        columns: Vec<String>,
        ty: Type,
    },
    
    // Set operations
    Union {
        left: Box<IRExpr>,
        right: Box<IRExpr>,
        ty: Type,
    },
    
    Minus {
        left: Box<IRExpr>,
        right: Box<IRExpr>,
        ty: Type,
    },
    
    Intersect {
        left: Box<IRExpr>,
        right: Box<IRExpr>,
        ty: Type,
    },
    
    // Reference navigation (automatic join/lookup)
    RefNavigation {
        object: Box<IRExpr>,
        field: String,
        target_table: String,
        ty: Type,
    },
}

impl IRExpr {
    pub fn get_type(&self) -> &Type {
        match self {
            IRExpr::Literal { ty, .. } |
            IRExpr::Variable { ty, .. } |
            IRExpr::BinaryOp { ty, .. } |
            IRExpr::UnaryOp { ty, .. } |
            IRExpr::FunctionCall { ty, .. } |
            IRExpr::FieldAccess { ty, .. } |
            IRExpr::Index { ty, .. } |
            IRExpr::Chain { ty, .. } |
            IRExpr::TableConstructor { ty, .. } |
            IRExpr::ArrayConstructor { ty, .. } |
            IRExpr::Lambda { ty, .. } |
            IRExpr::Where { ty, .. } |
            IRExpr::SortBy { ty, .. } |
            IRExpr::ColumnSelect { ty, .. } |
            IRExpr::Union { ty, .. } |
            IRExpr::Minus { ty, .. } |
            IRExpr::Intersect { ty, .. } |
            IRExpr::RefNavigation { ty, .. } => ty,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    
    // Set operations (used separately from Union/Minus/Intersect IRExpr variants)
    // These are for when we need to represent set ops as binary operations
    Union,
    SetMinus,
    Intersect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnOp {
    Neg,
    Not,
}

// Conversion from AST to IR for operators
impl From<&crate::ast::BinaryOp> for BinOp {
    fn from(op: &crate::ast::BinaryOp) -> Self {
        match op {
            crate::ast::BinaryOp::Add => BinOp::Add,
            crate::ast::BinaryOp::Subtract => BinOp::Sub,
            crate::ast::BinaryOp::Multiply => BinOp::Mul,
            crate::ast::BinaryOp::Divide => BinOp::Div,
            crate::ast::BinaryOp::Modulo => BinOp::Mod,
            crate::ast::BinaryOp::Equal => BinOp::Eq,
            crate::ast::BinaryOp::NotEqual => BinOp::Ne,
            crate::ast::BinaryOp::LessThan => BinOp::Lt,
            crate::ast::BinaryOp::LessThanEqual => BinOp::Le,
            crate::ast::BinaryOp::GreaterThan => BinOp::Gt,
            crate::ast::BinaryOp::GreaterThanEqual => BinOp::Ge,
            crate::ast::BinaryOp::And => BinOp::And,
            crate::ast::BinaryOp::Or => BinOp::Or,
            crate::ast::BinaryOp::Union => BinOp::Union,
            crate::ast::BinaryOp::Minus => BinOp::SetMinus,
            crate::ast::BinaryOp::Intersect => BinOp::Intersect,
        }
    }
}

impl From<&crate::ast::UnaryOp> for UnOp {
    fn from(op: &crate::ast::UnaryOp) -> Self {
        match op {
            crate::ast::UnaryOp::Not => UnOp::Not,
            crate::ast::UnaryOp::Negate => UnOp::Neg,
        }
    }
}
