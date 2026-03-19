//! Abstract Syntax Tree for Koli Language
//!
//! Defines the AST structures for representing Koli programs.
//! Supports AI-native constructs, cells, behaviors, and all language features.

use alloc::string::String;
use alloc::vec::Vec;

/// Program root - the top-level AST node
#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

/// Top-level items in a Koli program
#[derive(Debug, Clone)]
pub enum Item {
    /// Function definition
    Function(Function),
    /// AI agent definition
    Ai(AiDefinition),
    /// Cell definition
    Cell(CellDefinition),
}

/// Function definition
#[derive(Debug, Clone)]
pub struct Function {
    /// Function name
    pub name: String,
    /// Type parameters (generics)
    pub type_params: Vec<String>,
    /// Function parameters
    pub params: Vec<Parameter>,
    /// Return type (None for void)
    pub return_type: Option<Type>,
    /// Function body
    pub body: Block,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub ty: Type,
}

/// Type annotation
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Integer type (i64)
    Int,
    /// Floating point type (f64)
    Float,
    /// Boolean type
    Bool,
    /// String type
    String,
    /// Void type (no value)
    Void,
    /// Array type with element type
    Array(Box<Type>),
    /// Pointer type
    Pointer(Box<Type>),
    /// Optional type (nullable)
    Optional(Box<Type>),
    /// Custom/named type
    Custom(String),
}

impl Type {
    /// Check if this is a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(self, Type::Int | Type::Float | Type::Bool | Type::String | Type::Void)
    }

    /// Get the name of the type as a string
    pub fn name(&self) -> String {
        match self {
            Type::Int => String::from("int"),
            Type::Float => String::from("float"),
            Type::Bool => String::from("bool"),
            Type::String => String::from("string"),
            Type::Void => String::from("void"),
            Type::Array(inner) => alloc::format!("array<{}>", inner.name()),
            Type::Pointer(inner) => alloc::format!("pointer<{}>", inner.name()),
            Type::Optional(inner) => alloc::format!("{}?", inner.name()),
            Type::Custom(name) => name.clone(),
        }
    }
}

/// Block of statements
#[derive(Debug, Clone)]
pub struct Block {
    /// Statements in the block
    pub statements: Vec<Statement>,
}

/// Statement types
#[derive(Debug, Clone)]
pub enum Statement {
    /// Variable declaration: let name: Type = value;
    Let(LetStatement),
    /// Return statement: return expr?;
    Return(Option<Expression>),
    /// If statement: if cond { } else { }
    If(IfStatement),
    /// While statement: while cond { }
    While(WhileStatement),
    /// For-in statement: for x in iter { }
    For(ForStatement),
    /// Break statement
    Break,
    /// Continue statement
    Continue,
    /// AI ask statement: ask model prompt;
    Ask(AskStatement),
    /// Spawn statement: spawn CellType(args);
    Spawn(SpawnStatement),
    /// Expression statement
    Expression(Expression),
    /// Assignment statement: target = value;
    Assignment(AssignmentStatement),
}

/// Let (variable declaration) statement
#[derive(Debug, Clone)]
pub struct LetStatement {
    /// Variable name
    pub name: String,
    /// Optional type annotation (for type inference, can be None)
    pub ty: Option<Type>,
    /// Initial value
    pub value: Expression,
}

/// If statement
#[derive(Debug, Clone)]
pub struct IfStatement {
    /// Condition expression
    pub condition: Expression,
    /// Then block
    pub then_block: Block,
    /// Optional else block
    pub else_block: Option<Block>,
}

/// While statement
#[derive(Debug, Clone)]
pub struct WhileStatement {
    /// Condition expression
    pub condition: Expression,
    /// Loop body
    pub body: Block,
}

/// For-in statement
#[derive(Debug, Clone)]
pub struct ForStatement {
    /// Loop variable name
    pub var: String,
    /// Iterable expression
    pub iterable: Expression,
    /// Loop body
    pub body: Block,
}

/// Ask statement for AI interaction
#[derive(Debug, Clone)]
pub struct AskStatement {
    /// Optional AI model name
    pub model: Option<String>,
    /// Prompt expression
    pub prompt: Expression,
}

/// Spawn statement for creating cells
#[derive(Debug, Clone)]
pub struct SpawnStatement {
    /// Cell type to spawn
    pub cell_type: String,
    /// Constructor arguments
    pub args: Vec<Expression>,
}

/// Assignment statement
#[derive(Debug, Clone)]
pub struct AssignmentStatement {
    /// Target expression (variable, field access, or index)
    pub target: Expression,
    /// Value to assign
    pub value: Expression,
}

/// Expression types
#[derive(Debug, Clone)]
pub enum Expression {
    /// Literal value
    Literal(Literal),
    /// Identifier (variable or function name)
    Identifier(String),
    /// Binary operation: left op right
    Binary(BinaryExpr),
    /// Unary operation: op expr
    Unary(UnaryExpr),
    /// Function call: name(args)
    Call(CallExpr),
    /// Method call: receiver.method(args)
    MethodCall(MethodCallExpr),
    /// Field access: receiver.field
    FieldAccess(FieldAccessExpr),
    /// Index access: receiver[index]
    Index(IndexExpr),
    /// AI call: model(prompt)
    AiCall(AiCallExpr),
    /// Array literal: [elements]
    Array(ArrayExpr),
    /// Struct literal: { field: value, ... }
    Struct(StructExpr),
}

/// Literal values
#[derive(Debug, Clone)]
pub enum Literal {
    /// Integer literal
    Int(i64),
    /// Floating point literal
    Float(f64),
    /// String literal
    String(String),
    /// Boolean literal
    Bool(bool),
    /// Null literal
    Null,
}

/// Binary expression: left op right
#[derive(Debug, Clone)]
pub struct BinaryExpr {
    /// Left operand
    pub left: Box<Expression>,
    /// Operator
    pub op: BinaryOp,
    /// Right operand
    pub right: Box<Expression>,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    /// Addition: +
    Add,
    /// Subtraction: -
    Sub,
    /// Multiplication: *
    Mul,
    /// Division: /
    Div,
    /// Modulo: %
    Mod,
    /// Equality: ==
    Eq,
    /// Inequality: !=
    Ne,
    /// Less than: <
    Lt,
    /// Less than or equal: <=
    Le,
    /// Greater than: >
    Gt,
    /// Greater than or equal: >=
    Ge,
    /// Logical and: &&
    And,
    /// Logical or: ||
    Or,
}

impl BinaryOp {
    /// Check if this is an arithmetic operator
    pub fn is_arithmetic(&self) -> bool {
        matches!(self, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod)
    }

    /// Check if this is a comparison operator
    pub fn is_comparison(&self) -> bool {
        matches!(self, BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge)
    }

    /// Check if this is a logical operator
    pub fn is_logical(&self) -> bool {
        matches!(self, BinaryOp::And | BinaryOp::Or)
    }
}

/// Unary expression: op expr
#[derive(Debug, Clone)]
pub struct UnaryExpr {
    /// Operator
    pub op: UnaryOp,
    /// Operand
    pub expr: Box<Expression>,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Logical negation: !
    Not,
    /// Numeric negation: -
    Neg,
}

/// Function call expression
#[derive(Debug, Clone)]
pub struct CallExpr {
    /// Function name
    pub name: String,
    /// Arguments
    pub args: Vec<Expression>,
}

/// Method call expression
#[derive(Debug, Clone)]
pub struct MethodCallExpr {
    /// Receiver object
    pub receiver: Box<Expression>,
    /// Method name (empty string for anonymous method call)
    pub method: String,
    /// Arguments
    pub args: Vec<Expression>,
}

/// Field access expression
#[derive(Debug, Clone)]
pub struct FieldAccessExpr {
    /// Receiver object
    pub receiver: Box<Expression>,
    /// Field name
    pub field: String,
}

/// Index access expression
#[derive(Debug, Clone)]
pub struct IndexExpr {
    /// Receiver (array or object)
    pub receiver: Box<Expression>,
    /// Index expression
    pub index: Box<Expression>,
}

/// AI call expression
#[derive(Debug, Clone)]
pub struct AiCallExpr {
    /// AI model name
    pub model: String,
    /// Prompt expression
    pub prompt: Box<Expression>,
}

/// Array literal expression
#[derive(Debug, Clone)]
pub struct ArrayExpr {
    /// Array elements
    pub elements: Vec<Expression>,
}

/// Struct literal expression
#[derive(Debug, Clone)]
pub struct StructExpr {
    /// Field name-value pairs
    pub fields: Vec<(String, Expression)>,
}

/// AI Definition - defines an AI agent
#[derive(Debug, Clone)]
pub struct AiDefinition {
    /// AI agent name
    pub name: String,
    /// AI capabilities
    pub capabilities: Vec<AiCapability>,
}

/// AI capability
#[derive(Debug, Clone)]
pub struct AiCapability {
    /// Capability name
    pub name: String,
    /// Capability type
    pub capability_type: Type,
}

/// Cell Definition - defines a living cell
#[derive(Debug, Clone)]
pub struct CellDefinition {
    /// Cell name
    pub name: String,
    /// Cell properties (state)
    pub properties: Vec<Property>,
    /// Cell behaviors (methods)
    pub behaviors: Vec<Behavior>,
}

/// Property - defines a cell's state variable
#[derive(Debug, Clone)]
pub struct Property {
    /// Property name
    pub name: String,
    /// Property type
    pub ty: Type,
    /// Default value
    pub default_value: Option<Expression>,
}

/// Behavior - defines a cell's method
#[derive(Debug, Clone)]
pub struct Behavior {
    /// Behavior name
    pub name: String,
    /// Behavior parameters
    pub params: Vec<Parameter>,
    /// Return type (None for void)
    pub return_type: Option<Type>,
    /// Behavior body
    pub body: Block,
}

/// Source code location for error reporting
#[derive(Debug, Clone, Copy, Default)]
pub struct SourceLocation {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Byte offset
    pub offset: usize,
}

/// AST node with location information
#[derive(Debug, Clone)]
pub struct Located<T> {
    /// The AST node
    pub node: T,
    /// Source location
    pub location: SourceLocation,
}

impl<T> Located<T> {
    /// Create a new located node
    pub fn new(node: T, location: SourceLocation) -> Self {
        Self { node, location }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_name() {
        assert_eq!(Type::Int.name(), "int");
        assert_eq!(Type::String.name(), "string");
        assert_eq!(Type::Array(Box::new(Type::Int)).name(), "array<int>");
        assert_eq!(Type::Optional(Box::new(Type::Int)).name(), "int?");
    }

    #[test]
    fn test_binary_op_classification() {
        assert!(BinaryOp::Add.is_arithmetic());
        assert!(BinaryOp::Eq.is_comparison());
        assert!(BinaryOp::And.is_logical());
        assert!(!BinaryOp::Add.is_comparison());
    }

    #[test]
    fn test_type_primitive() {
        assert!(Type::Int.is_primitive());
        assert!(Type::Void.is_primitive());
        assert!(!Type::Array(Box::new(Type::Int)).is_primitive());
    }
}
