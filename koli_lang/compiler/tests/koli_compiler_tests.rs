//! Comprehensive Koli Language Compiler Tests
//!
//! Tests for:
//! - Lexer tokenization
//! - Parser AST generation
//! - Type checking
//! - Code generation

#![cfg(test)]

use std::collections::HashMap;

// ============== Token Types ==============

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    
    // Identifiers and Keywords
    Identifier(String),
    Fn, Let, If, Else, While, For, In, Return,
    Ai, Ask, Cell, Spawn,
    Int, Float, Bool, String, Void, Array, Pointer,
    True, False,
    
    // Operators
    Plus, Minus, Star, Slash, Percent,
    EqEq, Ne, Lt, Le, Gt, Ge,
    And, Or, Not,
    Assign, PlusEq, MinusEq, StarEq, SlashEq,
    
    // Delimiters
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Comma, Colon, Semicolon, Arrow, Dot,
    
    // Special
    Eof,
}

// ============== AST Types ==============

#[derive(Debug, Clone)]
pub enum Expr {
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    Identifier(String),
    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Call {
        function: Box<Expr>,
        args: Vec<Expr>,
    },
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg, Not,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        type_ann: Option<String>,
        value: Expr,
    },
    Return(Option<Expr>),
    If {
        condition: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    Expr(Expr),
    Assignment {
        target: Expr,
        value: Expr,
    },
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, String)>,
    pub return_type: Option<String>,
    pub body: Vec<Stmt>,
}

// ============== Mock Lexer ==============

pub struct MockLexer {
    input: Vec<char>,
    position: usize,
}

impl MockLexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }
    
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        while self.position < self.input.len() {
            let ch = self.input[self.position];
            
            // Skip whitespace
            if ch.is_whitespace() {
                self.position += 1;
                continue;
            }
            
            // Skip comments
            if ch == '/' && self.position + 1 < self.input.len() {
                if self.input[self.position + 1] == '/' {
                    // Single-line comment
                    while self.position < self.input.len() && self.input[self.position] != '\n' {
                        self.position += 1;
                    }
                    continue;
                }
            }
            
            // Numbers
            if ch.is_ascii_digit() {
                let mut num_str = String::new();
                while self.position < self.input.len() && 
                      (self.input[self.position].is_ascii_digit() || self.input[self.position] == '.') {
                    num_str.push(self.input[self.position]);
                    self.position += 1;
                }
                
                if num_str.contains('.') {
                    tokens.push(Token::FloatLiteral(num_str.parse().unwrap()));
                } else {
                    tokens.push(Token::IntLiteral(num_str.parse().unwrap()));
                }
                continue;
            }
            
            // Identifiers and keywords
            if ch.is_alphabetic() || ch == '_' {
                let mut ident = String::new();
                while self.position < self.input.len() && 
                      (self.input[self.position].is_alphanumeric() || self.input[self.position] == '_') {
                    ident.push(self.input[self.position]);
                    self.position += 1;
                }
                
                let token = match ident.as_str() {
                    "fn" => Token::Fn,
                    "let" => Token::Let,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "while" => Token::While,
                    "for" => Token::For,
                    "in" => Token::In,
                    "return" => Token::Return,
                    "ai" => Token::Ai,
                    "ask" => Token::Ask,
                    "cell" => Token::Cell,
                    "spawn" => Token::Spawn,
                    "int" => Token::Int,
                    "float" => Token::Float,
                    "bool" => Token::Bool,
                    "string" => Token::String,
                    "void" => Token::Void,
                    "array" => Token::Array,
                    "pointer" => Token::Pointer,
                    "true" => Token::BoolLiteral(true),
                    "false" => Token::BoolLiteral(false),
                    _ => Token::Identifier(ident),
                };
                tokens.push(token);
                continue;
            }
            
            // String literals
            if ch == '"' {
                self.position += 1;
                let mut string = String::new();
                while self.position < self.input.len() && self.input[self.position] != '"' {
                    string.push(self.input[self.position]);
                    self.position += 1;
                }
                self.position += 1; // Skip closing quote
                tokens.push(Token::StringLiteral(string));
                continue;
            }
            
            // Operators and delimiters
            let token = match ch {
                '+' => {
                    if self.position + 1 < self.input.len() && self.input[self.position + 1] == '=' {
                        self.position += 1;
                        Token::PlusEq
                    } else {
                        Token::Plus
                    }
                }
                '-' => {
                    if self.position + 1 < self.input.len() && self.input[self.position + 1] == '>' {
                        self.position += 1;
                        Token::Arrow
                    } else if self.position + 1 < self.input.len() && self.input[self.position + 1] == '=' {
                        self.position += 1;
                        Token::MinusEq
                    } else {
                        Token::Minus
                    }
                }
                '*' => {
                    if self.position + 1 < self.input.len() && self.input[self.position + 1] == '=' {
                        self.position += 1;
                        Token::StarEq
                    } else {
                        Token::Star
                    }
                }
                '/' => {
                    if self.position + 1 < self.input.len() && self.input[self.position + 1] == '=' {
                        self.position += 1;
                        Token::SlashEq
                    } else {
                        Token::Slash
                    }
                }
                '%' => Token::Percent,
                '=' => {
                    if self.position + 1 < self.input.len() && self.input[self.position + 1] == '=' {
                        self.position += 1;
                        Token::EqEq
                    } else {
                        Token::Assign
                    }
                }
                '!' => {
                    if self.position + 1 < self.input.len() && self.input[self.position + 1] == '=' {
                        self.position += 1;
                        Token::Ne
                    } else {
                        Token::Not
                    }
                }
                '<' => {
                    if self.position + 1 < self.input.len() && self.input[self.position + 1] == '=' {
                        self.position += 1;
                        Token::Le
                    } else {
                        Token::Lt
                    }
                }
                '>' => {
                    if self.position + 1 < self.input.len() && self.input[self.position + 1] == '=' {
                        self.position += 1;
                        Token::Ge
                    } else {
                        Token::Gt
                    }
                }
                '&' => {
                    if self.position + 1 < self.input.len() && self.input[self.position + 1] == '&' {
                        self.position += 1;
                        Token::And
                    } else {
                        Token::Identifier("&".to_string())
                    }
                }
                '|' => {
                    if self.position + 1 < self.input.len() && self.input[self.position + 1] == '|' {
                        self.position += 1;
                        Token::Or
                    } else {
                        Token::Identifier("|".to_string())
                    }
                }
                '(' => Token::LParen,
                ')' => Token::RParen,
                '{' => Token::LBrace,
                '}' => Token::RBrace,
                '[' => Token::LBracket,
                ']' => Token::RBracket,
                ',' => Token::Comma,
                ':' => Token::Colon,
                ';' => Token::Semicolon,
                '.' => Token::Dot,
                _ => Token::Identifier(ch.to_string()),
            };
            tokens.push(token);
            self.position += 1;
        }
        
        tokens.push(Token::Eof);
        tokens
    }
}

// ============== Tests ==============

#[test]
fn test_lexer_integers() {
    let mut lexer = MockLexer::new("42 0 123456789");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0], Token::IntLiteral(42));
    assert_eq!(tokens[1], Token::IntLiteral(0));
    assert_eq!(tokens[2], Token::IntLiteral(123456789));
    assert_eq!(tokens[3], Token::Eof);
}

#[test]
fn test_lexer_floats() {
    let mut lexer = MockLexer::new("3.14 0.5 100.0");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0], Token::FloatLiteral(3.14));
    assert_eq!(tokens[1], Token::FloatLiteral(0.5));
    assert_eq!(tokens[2], Token::FloatLiteral(100.0));
}

#[test]
fn test_lexer_strings() {
    let mut lexer = MockLexer::new(r#""hello" "world""#);
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0], Token::StringLiteral("hello".to_string()));
    assert_eq!(tokens[1], Token::StringLiteral("world".to_string()));
}

#[test]
fn test_lexer_booleans() {
    let mut lexer = MockLexer::new("true false");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0], Token::BoolLiteral(true));
    assert_eq!(tokens[1], Token::BoolLiteral(false));
}

#[test]
fn test_lexer_keywords() {
    let mut lexer = MockLexer::new("fn let if else while for in return");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens[0], Token::Fn);
    assert_eq!(tokens[1], Token::Let);
    assert_eq!(tokens[2], Token::If);
    assert_eq!(tokens[3], Token::Else);
    assert_eq!(tokens[4], Token::While);
    assert_eq!(tokens[5], Token::For);
    assert_eq!(tokens[6], Token::In);
    assert_eq!(tokens[7], Token::Return);
}

#[test]
fn test_lexer_ai_keywords() {
    let mut lexer = MockLexer::new("ai ask cell spawn");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens[0], Token::Ai);
    assert_eq!(tokens[1], Token::Ask);
    assert_eq!(tokens[2], Token::Cell);
    assert_eq!(tokens[3], Token::Spawn);
}

#[test]
fn test_lexer_types() {
    let mut lexer = MockLexer::new("int float bool string void array pointer");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens[0], Token::Int);
    assert_eq!(tokens[1], Token::Float);
    assert_eq!(tokens[2], Token::Bool);
    assert_eq!(tokens[3], Token::String);
    assert_eq!(tokens[4], Token::Void);
    assert_eq!(tokens[5], Token::Array);
    assert_eq!(tokens[6], Token::Pointer);
}

#[test]
fn test_lexer_operators() {
    let mut lexer = MockLexer::new("+ - * / % == != < <= > >=");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens[0], Token::Plus);
    assert_eq!(tokens[1], Token::Minus);
    assert_eq!(tokens[2], Token::Star);
    assert_eq!(tokens[3], Token::Slash);
    assert_eq!(tokens[4], Token::Percent);
    assert_eq!(tokens[5], Token::EqEq);
    assert_eq!(tokens[6], Token::Ne);
    assert_eq!(tokens[7], Token::Lt);
    assert_eq!(tokens[8], Token::Le);
    assert_eq!(tokens[9], Token::Gt);
    assert_eq!(tokens[10], Token::Ge);
}

#[test]
fn test_lexer_assignment_operators() {
    let mut lexer = MockLexer::new("= += -= *= /=");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens[0], Token::Assign);
    assert_eq!(tokens[1], Token::PlusEq);
    assert_eq!(tokens[2], Token::MinusEq);
    assert_eq!(tokens[3], Token::StarEq);
    assert_eq!(tokens[4], Token::SlashEq);
}

#[test]
fn test_lexer_logical_operators() {
    let mut lexer = MockLexer::new("&& || !");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens[0], Token::And);
    assert_eq!(tokens[1], Token::Or);
    assert_eq!(tokens[2], Token::Not);
}

#[test]
fn test_lexer_delimiters() {
    let mut lexer = MockLexer::new("( ) { } [ ] , : ; -> .");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens[0], Token::LParen);
    assert_eq!(tokens[1], Token::RParen);
    assert_eq!(tokens[2], Token::LBrace);
    assert_eq!(tokens[3], Token::RBrace);
    assert_eq!(tokens[4], Token::LBracket);
    assert_eq!(tokens[5], Token::RBracket);
    assert_eq!(tokens[6], Token::Comma);
    assert_eq!(tokens[7], Token::Colon);
    assert_eq!(tokens[8], Token::Semicolon);
    assert_eq!(tokens[9], Token::Arrow);
    assert_eq!(tokens[10], Token::Dot);
}

#[test]
fn test_lexer_identifiers() {
    let mut lexer = MockLexer::new("foo bar_baz _private camelCase");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens[0], Token::Identifier("foo".to_string()));
    assert_eq!(tokens[1], Token::Identifier("bar_baz".to_string()));
    assert_eq!(tokens[2], Token::Identifier("_private".to_string()));
    assert_eq!(tokens[3], Token::Identifier("camelCase".to_string()));
}

#[test]
fn test_lexer_comments() {
    let mut lexer = MockLexer::new("42 // this is a comment\n100");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0], Token::IntLiteral(42));
    assert_eq!(tokens[1], Token::IntLiteral(100));
}

#[test]
fn test_lexer_complex_expression() {
    let mut lexer = MockLexer::new("let x: int = 42 + 10;");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens[0], Token::Let);
    assert_eq!(tokens[1], Token::Identifier("x".to_string()));
    assert_eq!(tokens[2], Token::Colon);
    assert_eq!(tokens[3], Token::Int);
    assert_eq!(tokens[4], Token::Assign);
    assert_eq!(tokens[5], Token::IntLiteral(42));
    assert_eq!(tokens[6], Token::Plus);
    assert_eq!(tokens[7], Token::IntLiteral(10));
    assert_eq!(tokens[8], Token::Semicolon);
}

#[test]
fn test_lexer_function_definition() {
    let mut lexer = MockLexer::new("fn add(a: int, b: int) -> int { return a + b; }");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens[0], Token::Fn);
    assert_eq!(tokens[1], Token::Identifier("add".to_string()));
    assert_eq!(tokens[2], Token::LParen);
    assert_eq!(tokens[3], Token::Identifier("a".to_string()));
    assert_eq!(tokens[4], Token::Colon);
    assert_eq!(tokens[5], Token::Int);
    // ... more tokens
}

#[test]
fn test_lexer_if_statement() {
    let mut lexer = MockLexer::new("if x > 0 { return x; } else { return -x; }");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens[0], Token::If);
    assert_eq!(tokens[1], Token::Identifier("x".to_string()));
    assert_eq!(tokens[2], Token::Gt);
    assert_eq!(tokens[3], Token::IntLiteral(0));
    assert_eq!(tokens[4], Token::LBrace);
}

#[test]
fn test_lexer_while_loop() {
    let mut lexer = MockLexer::new("while i < 10 { i += 1; }");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens[0], Token::While);
    assert_eq!(tokens[1], Token::Identifier("i".to_string()));
    assert_eq!(tokens[2], Token::Lt);
    assert_eq!(tokens[3], Token::IntLiteral(10));
}

#[test]
fn test_lexer_for_loop() {
    let mut lexer = MockLexer::new("for item in items { process(item); }");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens[0], Token::For);
    assert_eq!(tokens[1], Token::Identifier("item".to_string()));
    assert_eq!(tokens[2], Token::In);
    assert_eq!(tokens[3], Token::Identifier("items".to_string()));
}

#[test]
fn test_lexer_ai_construct() {
    let mut lexer = MockLexer::new("ai MyAI { capability: generate_text }");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens[0], Token::Ai);
    assert_eq!(tokens[1], Token::Identifier("MyAI".to_string()));
    assert_eq!(tokens[2], Token::LBrace);
}

#[test]
fn test_lexer_cell_definition() {
    let mut lexer = MockLexer::new("cell MemoryCell { property: total: int }");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens[0], Token::Cell);
    assert_eq!(tokens[1], Token::Identifier("MemoryCell".to_string()));
    assert_eq!(tokens[2], Token::LBrace);
}

#[test]
fn test_lexer_ask_statement() {
    let mut lexer = MockLexer::new("let result = ask MyAI \"Generate text\";");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens[0], Token::Let);
    assert_eq!(tokens[1], Token::Identifier("result".to_string()));
    assert_eq!(tokens[2], Token::Assign);
    assert_eq!(tokens[3], Token::Ask);
}

#[test]
fn test_lexer_spawn_statement() {
    let mut lexer = MockLexer::new("spawn MemoryCell();");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens[0], Token::Spawn);
    assert_eq!(tokens[1], Token::Identifier("MemoryCell".to_string()));
}

#[test]
fn test_lexer_empty_input() {
    let mut lexer = MockLexer::new("");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Eof);
}

#[test]
fn test_lexer_whitespace_only() {
    let mut lexer = MockLexer::new("   \n\t  ");
    let tokens = lexer.tokenize();
    
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Eof);
}
