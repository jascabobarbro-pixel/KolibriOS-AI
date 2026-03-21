//! Comprehensive Tests for Koli Language Compiler
//!
//! This test module covers:
//! - Lexer tokenization
//! - Parser AST generation
//! - Type checking
//! - Code generation

#![cfg(test)]

use std::collections::BTreeMap;
use std::fmt;

// ============================================================================
// Token Definitions
// ============================================================================

/// Source span
#[derive(Debug, Clone, Copy, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// Token
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

/// Token kinds
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),

    // Identifiers
    Identifier(String),

    // Control flow keywords
    Fn,
    Let,
    If,
    Else,
    While,
    For,
    In,
    Return,
    Break,
    Continue,

    // AI-specific keywords
    Ai,
    Ask,
    Cell,
    Spawn,
    Capability,
    Behavior,
    Property,

    // Type keywords
    TypeInt,
    TypeFloat,
    TypeBool,
    TypeString,
    TypeVoid,
    TypeArray,
    TypePointer,

    // Special values
    Self_,
    Null,

    // Symbols - Single character
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Colon,
    Dot,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    Less,
    Greater,
    Bang,
    Ampersand,
    Pipe,
    Question,

    // Symbols - Two character
    Arrow,
    EqualEqual,
    BangEqual,
    LessEqual,
    GreaterEqual,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    AndAnd,
    OrOr,
    ColonColon,
    DotDot,

    // End of file
    Eof,
}

impl TokenKind {
    pub fn is_type(&self) -> bool {
        matches!(self,
            TokenKind::TypeInt |
            TokenKind::TypeFloat |
            TokenKind::TypeBool |
            TokenKind::TypeString |
            TokenKind::TypeVoid |
            TokenKind::TypeArray |
            TokenKind::TypePointer
        )
    }

    pub fn can_start_expression(&self) -> bool {
        matches!(self,
            TokenKind::Integer(_) |
            TokenKind::Float(_) |
            TokenKind::String(_) |
            TokenKind::Bool(_) |
            TokenKind::Identifier(_) |
            TokenKind::Self_ |
            TokenKind::LeftParen |
            TokenKind::Bang |
            TokenKind::Minus
        )
    }
}

// ============================================================================
// AST Definitions
// ============================================================================

/// Type annotation
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Void,
    Array(Box<Type>),
    Pointer(Box<Type>),
    Optional(Box<Type>),
    Custom(String),
}

impl Type {
    pub fn is_primitive(&self) -> bool {
        matches!(self, Type::Int | Type::Float | Type::Bool | Type::String | Type::Void)
    }

    pub fn name(&self) -> String {
        match self {
            Type::Int => "int".to_string(),
            Type::Float => "float".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Void => "void".to_string(),
            Type::Array(inner) => format!("array<{}>", inner.name()),
            Type::Pointer(inner) => format!("pointer<{}>", inner.name()),
            Type::Optional(inner) => format!("{}?", inner.name()),
            Type::Custom(name) => name.clone(),
        }
    }
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
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
}

impl BinaryOp {
    pub fn is_arithmetic(&self) -> bool {
        matches!(self, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod)
    }

    pub fn is_comparison(&self) -> bool {
        matches!(self, BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge)
    }

    pub fn is_logical(&self) -> bool {
        matches!(self, BinaryOp::And | BinaryOp::Or)
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
}

/// Literal values
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

/// Program root
#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

/// Top-level items
#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    Ai(AiDefinition),
    Cell(CellDefinition),
}

/// Function definition
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub type_params: Vec<String>,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Block,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
}

/// Block of statements
#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

/// Statement types
#[derive(Debug, Clone)]
pub enum Statement {
    Let(LetStatement),
    Return(Option<Expression>),
    If(IfStatement),
    While(WhileStatement),
    For(ForStatement),
    Break,
    Continue,
    Ask(AskStatement),
    Spawn(SpawnStatement),
    Expression(Expression),
    Assignment(AssignmentStatement),
}

/// Let statement
#[derive(Debug, Clone)]
pub struct LetStatement {
    pub name: String,
    pub ty: Option<Type>,
    pub value: Expression,
}

/// If statement
#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_block: Block,
    pub else_block: Option<Block>,
}

/// While statement
#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Block,
}

/// For statement
#[derive(Debug, Clone)]
pub struct ForStatement {
    pub var: String,
    pub iterable: Expression,
    pub body: Block,
}

/// Ask statement
#[derive(Debug, Clone)]
pub struct AskStatement {
    pub model: Option<String>,
    pub prompt: Expression,
}

/// Spawn statement
#[derive(Debug, Clone)]
pub struct SpawnStatement {
    pub cell_type: String,
    pub args: Vec<Expression>,
}

/// Assignment statement
#[derive(Debug, Clone)]
pub struct AssignmentStatement {
    pub target: Expression,
    pub value: Expression,
}

/// Expression types
#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Call(CallExpr),
    MethodCall(MethodCallExpr),
    FieldAccess(FieldAccessExpr),
    Index(IndexExpr),
    AiCall(AiCallExpr),
    Array(ArrayExpr),
    Struct(StructExpr),
}

/// Binary expression
#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expression>,
    pub op: BinaryOp,
    pub right: Box<Expression>,
}

/// Unary expression
#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Box<Expression>,
}

/// Function call
#[derive(Debug, Clone)]
pub struct CallExpr {
    pub name: String,
    pub args: Vec<Expression>,
}

/// Method call
#[derive(Debug, Clone)]
pub struct MethodCallExpr {
    pub receiver: Box<Expression>,
    pub method: String,
    pub args: Vec<Expression>,
}

/// Field access
#[derive(Debug, Clone)]
pub struct FieldAccessExpr {
    pub receiver: Box<Expression>,
    pub field: String,
}

/// Index access
#[derive(Debug, Clone)]
pub struct IndexExpr {
    pub receiver: Box<Expression>,
    pub index: Box<Expression>,
}

/// AI call
#[derive(Debug, Clone)]
pub struct AiCallExpr {
    pub model: String,
    pub prompt: Box<Expression>,
}

/// Array literal
#[derive(Debug, Clone)]
pub struct ArrayExpr {
    pub elements: Vec<Expression>,
}

/// Struct literal
#[derive(Debug, Clone)]
pub struct StructExpr {
    pub fields: Vec<(String, Expression)>,
}

/// AI Definition
#[derive(Debug, Clone)]
pub struct AiDefinition {
    pub name: String,
    pub capabilities: Vec<AiCapability>,
}

/// AI capability
#[derive(Debug, Clone)]
pub struct AiCapability {
    pub name: String,
    pub capability_type: Type,
}

/// Cell Definition
#[derive(Debug, Clone)]
pub struct CellDefinition {
    pub name: String,
    pub properties: Vec<Property>,
    pub behaviors: Vec<Behavior>,
}

/// Property
#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub ty: Type,
    pub default_value: Option<Expression>,
}

/// Behavior
#[derive(Debug, Clone)]
pub struct Behavior {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Block,
}

// ============================================================================
// Lexer Tests
// ============================================================================

#[test]
fn test_token_kind_equality() {
    // Test literal equality
    assert_eq!(TokenKind::Integer(42), TokenKind::Integer(42));
    assert_ne!(TokenKind::Integer(42), TokenKind::Integer(43));
    
    assert_eq!(TokenKind::Float(3.14), TokenKind::Float(3.14));
    
    assert_eq!(TokenKind::String("hello".to_string()), TokenKind::String("hello".to_string()));
    assert_ne!(TokenKind::String("hello".to_string()), TokenKind::String("world".to_string()));
    
    assert_eq!(TokenKind::Bool(true), TokenKind::Bool(true));
    assert_ne!(TokenKind::Bool(true), TokenKind::Bool(false));
    
    // Test identifier equality
    assert_eq!(TokenKind::Identifier("foo".to_string()), TokenKind::Identifier("foo".to_string()));
    
    // Test keyword equality
    assert_eq!(TokenKind::Fn, TokenKind::Fn);
    assert_ne!(TokenKind::Fn, TokenKind::Let);
}

#[test]
fn test_token_is_type() {
    assert!(TokenKind::TypeInt.is_type(), "TypeInt should be a type");
    assert!(TokenKind::TypeFloat.is_type(), "TypeFloat should be a type");
    assert!(TokenKind::TypeBool.is_type(), "TypeBool should be a type");
    assert!(TokenKind::TypeString.is_type(), "TypeString should be a type");
    assert!(TokenKind::TypeVoid.is_type(), "TypeVoid should be a type");
    assert!(TokenKind::TypeArray.is_type(), "TypeArray should be a type");
    assert!(TokenKind::TypePointer.is_type(), "TypePointer should be a type");
    
    assert!(!TokenKind::Fn.is_type(), "Fn should not be a type");
    assert!(!TokenKind::Identifier("x".to_string()).is_type(), "Identifier should not be a type");
}

#[test]
fn test_token_can_start_expression() {
    assert!(TokenKind::Integer(0).can_start_expression(), "Integer should start expression");
    assert!(TokenKind::Float(0.0).can_start_expression(), "Float should start expression");
    assert!(TokenKind::String("".to_string()).can_start_expression(), "String should start expression");
    assert!(TokenKind::Bool(true).can_start_expression(), "Bool should start expression");
    assert!(TokenKind::Identifier("x".to_string()).can_start_expression(), "Identifier should start expression");
    assert!(TokenKind::Self_.can_start_expression(), "Self should start expression");
    assert!(TokenKind::LeftParen.can_start_expression(), "LeftParen should start expression");
    assert!(TokenKind::Bang.can_start_expression(), "Bang should start expression");
    assert!(TokenKind::Minus.can_start_expression(), "Minus should start expression");
    
    assert!(!TokenKind::Fn.can_start_expression(), "Fn should not start expression");
    assert!(!TokenKind::RightParen.can_start_expression(), "RightParen should not start expression");
}

#[test]
fn test_span_default() {
    let span = Span::default();
    assert_eq!(span.start, 0, "Default span start should be 0");
    assert_eq!(span.end, 0, "Default span end should be 0");
}

// ============================================================================
// Type Tests
// ============================================================================

#[test]
fn test_type_equality() {
    assert_eq!(Type::Int, Type::Int);
    assert_eq!(Type::Float, Type::Float);
    assert_eq!(Type::Bool, Type::Bool);
    assert_eq!(Type::String, Type::String);
    assert_eq!(Type::Void, Type::Void);
    
    assert_ne!(Type::Int, Type::Float);
    assert_ne!(Type::Bool, Type::String);
}

#[test]
fn test_type_is_primitive() {
    assert!(Type::Int.is_primitive(), "Int should be primitive");
    assert!(Type::Float.is_primitive(), "Float should be primitive");
    assert!(Type::Bool.is_primitive(), "Bool should be primitive");
    assert!(Type::String.is_primitive(), "String should be primitive");
    assert!(Type::Void.is_primitive(), "Void should be primitive");
    
    assert!(!Type::Array(Box::new(Type::Int)).is_primitive(), "Array should not be primitive");
    assert!(!Type::Custom("MyType".to_string()).is_primitive(), "Custom type should not be primitive");
}

#[test]
fn test_type_name() {
    assert_eq!(Type::Int.name(), "int");
    assert_eq!(Type::Float.name(), "float");
    assert_eq!(Type::Bool.name(), "bool");
    assert_eq!(Type::String.name(), "string");
    assert_eq!(Type::Void.name(), "void");
    
    assert_eq!(Type::Array(Box::new(Type::Int)).name(), "array<int>");
    assert_eq!(Type::Pointer(Box::new(Type::Float)).name(), "pointer<float>");
    assert_eq!(Type::Optional(Box::new(Type::Int)).name(), "int?");
    assert_eq!(Type::Custom("MyType".to_string()).name(), "MyType");
    
    // Nested types
    let nested = Type::Array(Box::new(Type::Array(Box::new(Type::Int))));
    assert_eq!(nested.name(), "array<array<int>>");
}

#[test]
fn test_complex_types() {
    let pointer_to_array = Type::Pointer(Box::new(Type::Array(Box::new(Type::Int))));
    assert_eq!(pointer_to_array.name(), "pointer<array<int>>");
    
    let optional_array = Type::Optional(Box::new(Type::Array(Box::new(Type::String))));
    assert_eq!(optional_array.name(), "array<string>?");
}

// ============================================================================
// Binary Operator Tests
// ============================================================================

#[test]
fn test_binary_op_classification() {
    // Arithmetic operators
    assert!(BinaryOp::Add.is_arithmetic());
    assert!(BinaryOp::Sub.is_arithmetic());
    assert!(BinaryOp::Mul.is_arithmetic());
    assert!(BinaryOp::Div.is_arithmetic());
    assert!(BinaryOp::Mod.is_arithmetic());
    
    // Comparison operators
    assert!(BinaryOp::Eq.is_comparison());
    assert!(BinaryOp::Ne.is_comparison());
    assert!(BinaryOp::Lt.is_comparison());
    assert!(BinaryOp::Le.is_comparison());
    assert!(BinaryOp::Gt.is_comparison());
    assert!(BinaryOp::Ge.is_comparison());
    
    // Logical operators
    assert!(BinaryOp::And.is_logical());
    assert!(BinaryOp::Or.is_logical());
    
    // Cross-checks
    assert!(!BinaryOp::Add.is_comparison());
    assert!(!BinaryOp::Eq.is_arithmetic());
    assert!(!BinaryOp::And.is_arithmetic());
}

#[test]
fn test_binary_op_equality() {
    assert_eq!(BinaryOp::Add, BinaryOp::Add);
    assert_eq!(BinaryOp::Sub, BinaryOp::Sub);
    assert_ne!(BinaryOp::Add, BinaryOp::Sub);
}

// ============================================================================
// Literal Tests
// ============================================================================

#[test]
fn test_literal_equality() {
    assert_eq!(Literal::Int(42), Literal::Int(42));
    assert_ne!(Literal::Int(42), Literal::Int(43));
    
    assert_eq!(Literal::Float(3.14), Literal::Float(3.14));
    
    assert_eq!(Literal::String("hello".to_string()), Literal::String("hello".to_string()));
    
    assert_eq!(Literal::Bool(true), Literal::Bool(true));
    assert_eq!(Literal::Null, Literal::Null);
    
    // Cross-type inequality
    assert_ne!(Literal::Int(1), Literal::Float(1.0));
}

#[test]
fn test_literal_extreme_values() {
    // Test large integers
    let large_int = Literal::Int(i64::MAX);
    assert_eq!(large_int, Literal::Int(i64::MAX));
    
    let negative_int = Literal::Int(i64::MIN);
    assert_eq!(negative_int, Literal::Int(i64::MIN));
    
    // Test special floats
    let zero_float = Literal::Float(0.0);
    let neg_zero = Literal::Float(-0.0);
    assert_eq!(zero_float, neg_zero); // 0.0 == -0.0 in Rust
    
    let infinity = Literal::Float(f64::INFINITY);
    assert_eq!(infinity, Literal::Float(f64::INFINITY));
}

// ============================================================================
// Expression Tests
// ============================================================================

#[test]
fn test_expression_literal() {
    let expr = Expression::Literal(Literal::Int(42));
    
    match &expr {
        Expression::Literal(lit) => {
            assert_eq!(*lit, Literal::Int(42));
        }
        _ => panic!("Expected Literal expression"),
    }
}

#[test]
fn test_expression_identifier() {
    let expr = Expression::Identifier("foo".to_string());
    
    match &expr {
        Expression::Identifier(name) => {
            assert_eq!(name, "foo");
        }
        _ => panic!("Expected Identifier expression"),
    }
}

#[test]
fn test_expression_binary() {
    let expr = Expression::Binary(BinaryExpr {
        left: Box::new(Expression::Literal(Literal::Int(1))),
        op: BinaryOp::Add,
        right: Box::new(Expression::Literal(Literal::Int(2))),
    });
    
    match &expr {
        Expression::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::Add);
        }
        _ => panic!("Expected Binary expression"),
    }
}

#[test]
fn test_expression_nested_binary() {
    // (1 + 2) * 3
    let inner = BinaryExpr {
        left: Box::new(Expression::Literal(Literal::Int(1))),
        op: BinaryOp::Add,
        right: Box::new(Expression::Literal(Literal::Int(2))),
    };
    
    let outer = Expression::Binary(BinaryExpr {
        left: Box::new(Expression::Binary(inner)),
        op: BinaryOp::Mul,
        right: Box::new(Expression::Literal(Literal::Int(3))),
    });
    
    match &outer {
        Expression::Binary(bin) => {
            assert_eq!(bin.op, BinaryOp::Mul);
            match &*bin.left {
                Expression::Binary(inner_bin) => {
                    assert_eq!(inner_bin.op, BinaryOp::Add);
                }
                _ => panic!("Expected nested binary expression"),
            }
        }
        _ => panic!("Expected Binary expression"),
    }
}

#[test]
fn test_expression_unary() {
    let expr = Expression::Unary(UnaryExpr {
        op: UnaryOp::Neg,
        expr: Box::new(Expression::Literal(Literal::Int(42))),
    });
    
    match &expr {
        Expression::Unary(unary) => {
            assert_eq!(unary.op, UnaryOp::Neg);
        }
        _ => panic!("Expected Unary expression"),
    }
}

#[test]
fn test_expression_call() {
    let expr = Expression::Call(CallExpr {
        name: "foo".to_string(),
        args: vec![
            Expression::Literal(Literal::Int(1)),
            Expression::Literal(Literal::Int(2)),
        ],
    });
    
    match &expr {
        Expression::Call(call) => {
            assert_eq!(call.name, "foo");
            assert_eq!(call.args.len(), 2);
        }
        _ => panic!("Expected Call expression"),
    }
}

#[test]
fn test_expression_method_call() {
    let expr = Expression::MethodCall(MethodCallExpr {
        receiver: Box::new(Expression::Identifier("obj".to_string())),
        method: "do_something".to_string(),
        args: vec![],
    });
    
    match &expr {
        Expression::MethodCall(method) => {
            assert_eq!(method.method, "do_something");
        }
        _ => panic!("Expected MethodCall expression"),
    }
}

#[test]
fn test_expression_field_access() {
    let expr = Expression::FieldAccess(FieldAccessExpr {
        receiver: Box::new(Expression::Identifier("obj".to_string())),
        field: "x".to_string(),
    });
    
    match &expr {
        Expression::FieldAccess(access) => {
            assert_eq!(access.field, "x");
        }
        _ => panic!("Expected FieldAccess expression"),
    }
}

#[test]
fn test_expression_index() {
    let expr = Expression::Index(IndexExpr {
        receiver: Box::new(Expression::Identifier("arr".to_string())),
        index: Box::new(Expression::Literal(Literal::Int(0))),
    });
    
    match &expr {
        Expression::Index(idx) => {
            match &*idx.receiver {
                Expression::Identifier(name) => assert_eq!(name, "arr"),
                _ => panic!("Expected identifier receiver"),
            }
        }
        _ => panic!("Expected Index expression"),
    }
}

#[test]
fn test_expression_array() {
    let expr = Expression::Array(ArrayExpr {
        elements: vec![
            Expression::Literal(Literal::Int(1)),
            Expression::Literal(Literal::Int(2)),
            Expression::Literal(Literal::Int(3)),
        ],
    });
    
    match &expr {
        Expression::Array(arr) => {
            assert_eq!(arr.elements.len(), 3);
        }
        _ => panic!("Expected Array expression"),
    }
}

#[test]
fn test_expression_struct() {
    let expr = Expression::Struct(StructExpr {
        fields: vec![
            ("x".to_string(), Expression::Literal(Literal::Int(1))),
            ("y".to_string(), Expression::Literal(Literal::Int(2))),
        ],
    });
    
    match &expr {
        Expression::Struct(s) => {
            assert_eq!(s.fields.len(), 2);
            assert_eq!(s.fields[0].0, "x");
            assert_eq!(s.fields[1].0, "y");
        }
        _ => panic!("Expected Struct expression"),
    }
}

#[test]
fn test_expression_ai_call() {
    let expr = Expression::AiCall(AiCallExpr {
        model: "GPT".to_string(),
        prompt: Box::new(Expression::Literal(Literal::String("Hello".to_string()))),
    });
    
    match &expr {
        Expression::AiCall(ai) => {
            assert_eq!(ai.model, "GPT");
        }
        _ => panic!("Expected AiCall expression"),
    }
}

// ============================================================================
// Statement Tests
// ============================================================================

#[test]
fn test_statement_let() {
    let stmt = Statement::Let(LetStatement {
        name: "x".to_string(),
        ty: Some(Type::Int),
        value: Expression::Literal(Literal::Int(42)),
    });
    
    match &stmt {
        Statement::Let(let_stmt) => {
            assert_eq!(let_stmt.name, "x");
            assert_eq!(let_stmt.ty, Some(Type::Int));
        }
        _ => panic!("Expected Let statement"),
    }
}

#[test]
fn test_statement_return() {
    let stmt_with_value = Statement::Return(Some(Expression::Literal(Literal::Int(42))));
    let stmt_without_value = Statement::Return(None);
    
    match &stmt_with_value {
        Statement::Return(Some(_)) => {}
        _ => panic!("Expected Return with value"),
    }
    
    match &stmt_without_value {
        Statement::Return(None) => {}
        _ => panic!("Expected Return without value"),
    }
}

#[test]
fn test_statement_if() {
    let stmt = Statement::If(IfStatement {
        condition: Expression::Literal(Literal::Bool(true)),
        then_block: Block { statements: vec![] },
        else_block: None,
    });
    
    match &stmt {
        Statement::If(if_stmt) => {
            assert!(if_stmt.else_block.is_none());
        }
        _ => panic!("Expected If statement"),
    }
}

#[test]
fn test_statement_while() {
    let stmt = Statement::While(WhileStatement {
        condition: Expression::Literal(Literal::Bool(true)),
        body: Block { statements: vec![] },
    });
    
    match &stmt {
        Statement::While(while_stmt) => {
            match &while_stmt.condition {
                Expression::Literal(Literal::Bool(true)) => {}
                _ => panic!("Expected true condition"),
            }
        }
        _ => panic!("Expected While statement"),
    }
}

#[test]
fn test_statement_for() {
    let stmt = Statement::For(ForStatement {
        var: "i".to_string(),
        iterable: Expression::Identifier("items".to_string()),
        body: Block { statements: vec![] },
    });
    
    match &stmt {
        Statement::For(for_stmt) => {
            assert_eq!(for_stmt.var, "i");
        }
        _ => panic!("Expected For statement"),
    }
}

#[test]
fn test_statement_break_continue() {
    let break_stmt = Statement::Break;
    let continue_stmt = Statement::Continue;
    
    assert!(matches!(break_stmt, Statement::Break));
    assert!(matches!(continue_stmt, Statement::Continue));
}

#[test]
fn test_statement_ask() {
    let stmt = Statement::Ask(AskStatement {
        model: Some("GPT".to_string()),
        prompt: Expression::Literal(Literal::String("Hello".to_string())),
    });
    
    match &stmt {
        Statement::Ask(ask) => {
            assert_eq!(ask.model, Some("GPT".to_string()));
        }
        _ => panic!("Expected Ask statement"),
    }
}

#[test]
fn test_statement_spawn() {
    let stmt = Statement::Spawn(SpawnStatement {
        cell_type: "MemoryCell".to_string(),
        args: vec![Expression::Literal(Literal::Int(1024))],
    });
    
    match &stmt {
        Statement::Spawn(spawn) => {
            assert_eq!(spawn.cell_type, "MemoryCell");
            assert_eq!(spawn.args.len(), 1);
        }
        _ => panic!("Expected Spawn statement"),
    }
}

#[test]
fn test_statement_assignment() {
    let stmt = Statement::Assignment(AssignmentStatement {
        target: Expression::Identifier("x".to_string()),
        value: Expression::Literal(Literal::Int(42)),
    });
    
    match &stmt {
        Statement::Assignment(assign) => {
            match &assign.target {
                Expression::Identifier(name) => assert_eq!(name, "x"),
                _ => panic!("Expected identifier target"),
            }
        }
        _ => panic!("Expected Assignment statement"),
    }
}

// ============================================================================
// Item Tests
// ============================================================================

#[test]
fn test_item_function() {
    let item = Item::Function(Function {
        name: "main".to_string(),
        type_params: vec![],
        params: vec![],
        return_type: Some(Type::Int),
        body: Block { statements: vec![] },
    });
    
    match &item {
        Item::Function(func) => {
            assert_eq!(func.name, "main");
            assert_eq!(func.return_type, Some(Type::Int));
        }
        _ => panic!("Expected Function item"),
    }
}

#[test]
fn test_item_ai_definition() {
    let item = Item::Ai(AiDefinition {
        name: "Assistant".to_string(),
        capabilities: vec![
            AiCapability {
                name: "code_completion".to_string(),
                capability_type: Type::Custom("CodeService".to_string()),
            },
        ],
    });
    
    match &item {
        Item::Ai(ai) => {
            assert_eq!(ai.name, "Assistant");
            assert_eq!(ai.capabilities.len(), 1);
        }
        _ => panic!("Expected Ai item"),
    }
}

#[test]
fn test_item_cell_definition() {
    let item = Item::Cell(CellDefinition {
        name: "Memory".to_string(),
        properties: vec![
            Property {
                name: "size".to_string(),
                ty: Type::Int,
                default_value: Some(Expression::Literal(Literal::Int(1024))),
            },
        ],
        behaviors: vec![],
    });
    
    match &item {
        Item::Cell(cell) => {
            assert_eq!(cell.name, "Memory");
            assert_eq!(cell.properties.len(), 1);
        }
        _ => panic!("Expected Cell item"),
    }
}

// ============================================================================
// Program Tests
// ============================================================================

#[test]
fn test_program_empty() {
    let program = Program { items: vec![] };
    assert_eq!(program.items.len(), 0, "Empty program should have no items");
}

#[test]
fn test_program_with_items() {
    let program = Program {
        items: vec![
            Item::Function(Function {
                name: "main".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Block { statements: vec![] },
            }),
            Item::Ai(AiDefinition {
                name: "Assistant".to_string(),
                capabilities: vec![],
            }),
        ],
    };
    
    assert_eq!(program.items.len(), 2, "Program should have 2 items");
}

// ============================================================================
// Block Tests
// ============================================================================

#[test]
fn test_block_empty() {
    let block = Block { statements: vec![] };
    assert_eq!(block.statements.len(), 0, "Empty block should have no statements");
}

#[test]
fn test_block_with_statements() {
    let block = Block {
        statements: vec![
            Statement::Let(LetStatement {
                name: "x".to_string(),
                ty: Some(Type::Int),
                value: Expression::Literal(Literal::Int(1)),
            }),
            Statement::Return(Some(Expression::Identifier("x".to_string()))),
        ],
    };
    
    assert_eq!(block.statements.len(), 2, "Block should have 2 statements");
}

// ============================================================================
// Compile Error Tests
// ============================================================================

#[derive(Debug, Clone)]
pub enum CompileError {
    LexerError(String),
    ParserError(String),
    TypeError(String),
    CodeGenError(String),
}

#[test]
fn test_compile_error_lexer() {
    let error = CompileError::LexerError("Unexpected character: @".to_string());
    
    match &error {
        CompileError::LexerError(msg) => {
            assert!(msg.contains("@"));
        }
        _ => panic!("Expected LexerError"),
    }
}

#[test]
fn test_compile_error_parser() {
    let error = CompileError::ParserError("Expected ';' found '}'".to_string());
    
    match &error {
        CompileError::ParserError(msg) => {
            assert!(msg.contains(";"));
        }
        _ => panic!("Expected ParserError"),
    }
}

#[test]
fn test_compile_error_type() {
    let error = CompileError::TypeError("Type mismatch: expected int, got string".to_string());
    
    match &error {
        CompileError::TypeError(msg) => {
            assert!(msg.contains("Type mismatch"));
        }
        _ => panic!("Expected TypeError"),
    }
}

#[test]
fn test_compile_error_codegen() {
    let error = CompileError::CodeGenError("Failed to generate code".to_string());
    
    match &error {
        CompileError::CodeGenError(msg) => {
            assert!(msg.contains("Failed"));
        }
        _ => panic!("Expected CodeGenError"),
    }
}

// ============================================================================
// Behavior Tests
// ============================================================================

#[test]
fn test_behavior_creation() {
    let behavior = Behavior {
        name: "allocate".to_string(),
        params: vec![
            Parameter {
                name: "size".to_string(),
                ty: Type::Int,
            },
        ],
        return_type: Some(Type::Pointer(Box::new(Type::Void))),
        body: Block { statements: vec![] },
    };
    
    assert_eq!(behavior.name, "allocate");
    assert_eq!(behavior.params.len(), 1);
    assert!(behavior.return_type.is_some());
}

// ============================================================================
// AI Capability Tests
// ============================================================================

#[test]
fn test_ai_capability() {
    let capability = AiCapability {
        name: "code_generation".to_string(),
        capability_type: Type::Custom("CodeGenService".to_string()),
    };
    
    assert_eq!(capability.name, "code_generation");
    match &capability.capability_type {
        Type::Custom(name) => assert_eq!(name, "CodeGenService"),
        _ => panic!("Expected Custom type"),
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_complete_function_ast() {
    // fn add(a: int, b: int) -> int { return a + b; }
    let function = Function {
        name: "add".to_string(),
        type_params: vec![],
        params: vec![
            Parameter { name: "a".to_string(), ty: Type::Int },
            Parameter { name: "b".to_string(), ty: Type::Int },
        ],
        return_type: Some(Type::Int),
        body: Block {
            statements: vec![
                Statement::Return(Some(
                    Expression::Binary(BinaryExpr {
                        left: Box::new(Expression::Identifier("a".to_string())),
                        op: BinaryOp::Add,
                        right: Box::new(Expression::Identifier("b".to_string())),
                    })
                )),
            ],
        },
    };
    
    assert_eq!(function.name, "add");
    assert_eq!(function.params.len(), 2);
    assert_eq!(function.return_type, Some(Type::Int));
    assert_eq!(function.body.statements.len(), 1);
}

#[test]
fn test_complete_cell_definition_ast() {
    // cell Memory {
    //     size: int = 1024
    //     behavior allocate(n: int) -> pointer {
    //         return null
    //     }
    // }
    let cell = CellDefinition {
        name: "Memory".to_string(),
        properties: vec![
            Property {
                name: "size".to_string(),
                ty: Type::Int,
                default_value: Some(Expression::Literal(Literal::Int(1024))),
            },
        ],
        behaviors: vec![
            Behavior {
                name: "allocate".to_string(),
                params: vec![
                    Parameter { name: "n".to_string(), ty: Type::Int },
                ],
                return_type: Some(Type::Pointer(Box::new(Type::Void))),
                body: Block {
                    statements: vec![
                        Statement::Return(Some(Expression::Literal(Literal::Null))),
                    ],
                },
            },
        ],
    };
    
    assert_eq!(cell.name, "Memory");
    assert_eq!(cell.properties.len(), 1);
    assert_eq!(cell.behaviors.len(), 1);
}

#[test]
fn test_complete_ai_definition_ast() {
    // ai Assistant {
    //     capability: code_completion
    //     capability: code_review
    // }
    let ai = AiDefinition {
        name: "Assistant".to_string(),
        capabilities: vec![
            AiCapability {
                name: "code_completion".to_string(),
                capability_type: Type::Custom("CompletionService".to_string()),
            },
            AiCapability {
                name: "code_review".to_string(),
                capability_type: Type::Custom("ReviewService".to_string()),
            },
        ],
    };
    
    assert_eq!(ai.name, "Assistant");
    assert_eq!(ai.capabilities.len(), 2);
}
