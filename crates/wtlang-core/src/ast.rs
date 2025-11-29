// AST (Abstract Syntax Tree) definitions for WTLang

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<ProgramItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProgramItem {
    TableDef(TableDef),
    Page(Page),
    FunctionDef(FunctionDef),
    ExternalFunction(ExternalFunction),
    Test(Test),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableDef {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub field_type: Type,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    String,
    Date,
    Currency,
    Bool,
    Table(String), // Table<TypeName>
    Filter,        // Filter type for table column filters
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterMode {
    Single,
    Multi,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilterDef {
    pub column: String,
    pub mode: FilterMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    Unique,
    NonNull,
    Validate(Expr),
    References { table: String, field: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub name: String,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Title(String),
    Subtitle(String),
    Text(String),
    Button { label: String, body: Vec<Statement> },
    Section { title: String, body: Vec<Statement> },
    Let { name: String, value: Expr },
    If { condition: Expr, then_branch: Vec<Statement>, else_branch: Option<Vec<Statement>> },
    Forall { var: String, iterable: Expr, body: Vec<Statement> },
    FunctionCall(FunctionCall),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Type,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternalFunction {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Type,
    pub module: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    Identifier(String),
    FunctionCall(FunctionCall),
    BinaryOp { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    UnaryOp { op: UnaryOp, operand: Box<Expr> },
    Lambda { params: Vec<String>, body: Box<Expr> },
    FieldAccess { object: Box<Expr>, field: String },
    Index { object: Box<Expr>, index: Box<Expr> },
    Chain { left: Box<Expr>, right: Box<Expr> },
    TableLiteral(Vec<(String, Expr)>),
    ArrayLiteral(Vec<Expr>),
    FilterLiteral(FilterDef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
    Negate,
}
