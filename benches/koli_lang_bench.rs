//! Koli Language Benchmarks
//!
//! Performance benchmarks for lexer, parser, and codegen.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};

/// Token types
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Fn,
    Let,
    If,
    Else,
    While,
    For,
    In,
    Return,
    Ai,
    Ask,
    Cell,
    Spawn,
    Identifier(String),
    Integer(i64),
    Float(f64),
    StringLiteral(String),
    Bool(bool),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    Assign,
    PlusAssign,
    MinusAssign,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semicolon,
    Arrow,
    Eof,
}

/// Lexer
struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }
    
    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        
        if self.pos >= self.input.len() {
            return Token::Eof;
        }
        
        let c = self.input[self.pos];
        
        // Single character tokens
        match c {
            '+' => {
                self.pos += 1;
                if self.peek() == '=' {
                    self.pos += 1;
                    return Token::PlusAssign;
                }
                return Token::Plus;
            }
            '-' => {
                self.pos += 1;
                if self.peek() == '>' {
                    self.pos += 1;
                    return Token::Arrow;
                }
                if self.peek() == '=' {
                    self.pos += 1;
                    return Token::MinusAssign;
                }
                return Token::Minus;
            }
            '*' => { self.pos += 1; return Token::Star; }
            '/' => { self.pos += 1; return Token::Slash; }
            '%' => { self.pos += 1; return Token::Percent; }
            '(' => { self.pos += 1; return Token::LParen; }
            ')' => { self.pos += 1; return Token::RParen; }
            '{' => { self.pos += 1; return Token::LBrace; }
            '}' => { self.pos += 1; return Token::RBrace; }
            '[' => { self.pos += 1; return Token::LBracket; }
            ']' => { self.pos += 1; return Token::RBracket; }
            ',' => { self.pos += 1; return Token::Comma; }
            ':' => { self.pos += 1; return Token::Colon; }
            ';' => { self.pos += 1; return Token::Semicolon; }
            '=' => {
                self.pos += 1;
                if self.peek() == '=' {
                    self.pos += 1;
                    return Token::Eq;
                }
                return Token::Assign;
            }
            '!' => {
                self.pos += 1;
                if self.peek() == '=' {
                    self.pos += 1;
                    return Token::Ne;
                }
                return Token::Not;
            }
            '<' => {
                self.pos += 1;
                if self.peek() == '=' {
                    self.pos += 1;
                    return Token::Le;
                }
                return Token::Lt;
            }
            '>' => {
                self.pos += 1;
                if self.peek() == '=' {
                    self.pos += 1;
                    return Token::Ge;
                }
                return Token::Gt;
            }
            '&' => {
                self.pos += 1;
                if self.peek() == '&' {
                    self.pos += 1;
                    return Token::And;
                }
            }
            '|' => {
                self.pos += 1;
                if self.peek() == '|' {
                    self.pos += 1;
                    return Token::Or;
                }
            }
            _ => {}
        }
        
        // Numbers
        if c.is_ascii_digit() {
            let start = self.pos;
            while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
            
            let num_str: String = self.input[start..self.pos].iter().collect();
            return Token::Integer(num_str.parse().unwrap_or(0));
        }
        
        // Identifiers and keywords
        if c.is_alphabetic() || c == '_' {
            let start = self.pos;
            while self.pos < self.input.len() && 
                  (self.input[self.pos].is_alphanumeric() || self.input[self.pos] == '_') {
                self.pos += 1;
            }
            
            let ident: String = self.input[start..self.pos].iter().collect();
            
            return match ident.as_str() {
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
                "true" => Token::Bool(true),
                "false" => Token::Bool(false),
                _ => Token::Identifier(ident),
            };
        }
        
        // Strings
        if c == '"' {
            self.pos += 1;
            let start = self.pos;
            while self.pos < self.input.len() && self.input[self.pos] != '"' {
                self.pos += 1;
            }
            let s: String = self.input[start..self.pos].iter().collect();
            self.pos += 1; // Skip closing quote
            return Token::StringLiteral(s);
        }
        
        self.pos += 1;
        Token::Eof
    }
    
    fn peek(&self) -> char {
        if self.pos + 1 < self.input.len() {
            self.input[self.pos + 1]
        } else {
            '\0'
        }
    }
    
    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }
    
    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token == Token::Eof {
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}

/// AST Node (simplified)
#[derive(Debug, Clone)]
enum Expr {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Identifier(String),
    BinaryOp(Box<Expr>, BinOp, Box<Expr>),
    UnaryOp(UnOp, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug, Clone, Copy)]
enum BinOp {
    Add, Sub, Mul, Div,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}

#[derive(Debug, Clone, Copy)]
enum UnOp {
    Neg, Not,
}

/// Parser (simplified)
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }
    
    fn parse_expr(&mut self) -> Expr {
        self.parse_comparison()
    }
    
    fn parse_comparison(&mut self) -> Expr {
        let mut left = self.parse_additive();
        
        while let Some(op) = self.peek_comparison_op() {
            self.pos += 1;
            let right = self.parse_additive();
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        
        left
    }
    
    fn parse_additive(&mut self) -> Expr {
        let mut left = self.parse_multiplicative();
        
        while let Some(op) = self.peek_additive_op() {
            self.pos += 1;
            let right = self.parse_multiplicative();
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        
        left
    }
    
    fn parse_multiplicative(&mut self) -> Expr {
        let mut left = self.parse_unary();
        
        while let Some(op) = self.peek_multiplicative_op() {
            self.pos += 1;
            let right = self.parse_unary();
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        
        left
    }
    
    fn parse_unary(&mut self) -> Expr {
        if self.check(&Token::Minus) {
            self.pos += 1;
            let expr = self.parse_primary();
            return Expr::UnaryOp(UnOp::Neg, Box::new(expr));
        }
        if self.check(&Token::Not) {
            self.pos += 1;
            let expr = self.parse_primary();
            return Expr::UnaryOp(UnOp::Not, Box::new(expr));
        }
        self.parse_primary()
    }
    
    fn parse_primary(&mut self) -> Expr {
        if self.pos >= self.tokens.len() {
            return Expr::Integer(0);
        }
        
        match &self.tokens[self.pos] {
            Token::Integer(n) => {
                let n = *n;
                self.pos += 1;
                Expr::Integer(n)
            }
            Token::Float(n) => {
                let n = *n;
                self.pos += 1;
                Expr::Float(n)
            }
            Token::StringLiteral(s) => {
                let s = s.clone();
                self.pos += 1;
                Expr::String(s)
            }
            Token::Bool(b) => {
                let b = *b;
                self.pos += 1;
                Expr::Bool(b)
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.pos += 1;
                
                if self.check(&Token::LParen) {
                    self.pos += 1;
                    let mut args = Vec::new();
                    if !self.check(&Token::RParen) {
                        args.push(self.parse_expr());
                        while self.check(&Token::Comma) {
                            self.pos += 1;
                            args.push(self.parse_expr());
                        }
                    }
                    self.pos += 1; // Skip RParen
                    Expr::Call(name, args)
                } else {
                    Expr::Identifier(name)
                }
            }
            Token::LParen => {
                self.pos += 1;
                let expr = self.parse_expr();
                self.pos += 1; // Skip RParen
                expr
            }
            _ => Expr::Integer(0),
        }
    }
    
    fn peek_comparison_op(&self) -> Option<BinOp> {
        if self.pos >= self.tokens.len() {
            return None;
        }
        match &self.tokens[self.pos] {
            Token::Eq => Some(BinOp::Eq),
            Token::Ne => Some(BinOp::Ne),
            Token::Lt => Some(BinOp::Lt),
            Token::Le => Some(BinOp::Le),
            Token::Gt => Some(BinOp::Gt),
            Token::Ge => Some(BinOp::Ge),
            _ => None,
        }
    }
    
    fn peek_additive_op(&self) -> Option<BinOp> {
        if self.pos >= self.tokens.len() {
            return None;
        }
        match &self.tokens[self.pos] {
            Token::Plus => Some(BinOp::Add),
            Token::Minus => Some(BinOp::Sub),
            _ => None,
        }
    }
    
    fn peek_multiplicative_op(&self) -> Option<BinOp> {
        if self.pos >= self.tokens.len() {
            return None;
        }
        match &self.tokens[self.pos] {
            Token::Star => Some(BinOp::Mul),
            Token::Slash => Some(BinOp::Div),
            _ => None,
        }
    }
    
    fn check(&self, token: &Token) -> bool {
        self.pos < self.tokens.len() && &self.tokens[self.pos] == token
    }
}

// Sample code for benchmarking
fn generate_code(lines: usize) -> String {
    let mut code = String::new();
    for i in 0..lines {
        code.push_str(&format!(
            "let x{} = {} + {} * {};\n",
            i, i, i + 1, i + 2
        ));
    }
    code
}

fn generate_complex_code(count: usize) -> String {
    let mut code = String::new();
    for i in 0..count {
        code.push_str(&format!(
            r#"
fn calculate_{}(a: int, b: int) -> int {{
    let result = a + b * 2;
    if result > 100 {{
        return result - 100;
    }} else {{
        return result + 100;
    }}
}}
"#,
            i
        ));
    }
    code
}

// Benchmarks

fn bench_lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");
    
    for lines in [10, 100, 1000].iter() {
        let code = generate_code(*lines);
        group.throughput(Throughput::Bytes(code.len() as u64));
        
        group.bench_with_input(BenchmarkId::from_parameter(lines), lines, |b, _| {
            b.iter(|| {
                let mut lexer = Lexer::new(&code);
                black_box(lexer.tokenize())
            });
        });
    }
    
    group.finish();
}

fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");
    
    for lines in [10, 100, 1000].iter() {
        let code = generate_code(*lines);
        let mut lexer = Lexer::new(&code);
        let tokens = lexer.tokenize();
        
        group.bench_with_input(BenchmarkId::from_parameter(lines), lines, |b, _| {
            b.iter(|| {
                let mut parser = Parser::new(tokens.clone());
                while parser.pos < parser.tokens.len() {
                    black_box(parser.parse_expr());
                }
            });
        });
    }
    
    group.finish();
}

fn bench_token_creation(c: &mut Criterion) {
    c.bench_function("token_identifier", |b| {
        b.iter(|| Token::Identifier("variable_name".to_string()))
    });
    
    c.bench_function("token_integer", |b| {
        b.iter(|| Token::Integer(12345))
    });
}

fn bench_complex_lexing(c: &mut Criterion) {
    let code = generate_complex_code(100);
    
    c.bench_function("complex_lexer_100_functions", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(&code);
            black_box(lexer.tokenize())
        });
    });
}

fn bench_expression_parsing(c: &mut Criterion) {
    let code = "1 + 2 * 3 - 4 / 5 + 6 * 7 - 8 / 9";
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize();
    
    c.bench_function("expression_parsing", |b| {
        b.iter(|| {
            let mut parser = Parser::new(tokens.clone());
            black_box(parser.parse_expr())
        });
    });
}

criterion_group!(
    benches,
    bench_lexer,
    bench_parser,
    bench_token_creation,
    bench_complex_lexing,
    bench_expression_parsing,
);

criterion_main!(benches);
