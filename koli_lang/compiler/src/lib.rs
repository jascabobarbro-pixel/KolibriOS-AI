//! Koli Language Compiler
//!
//! The Kolibri Language (Koli) is an AI-first programming language designed
//! for seamless integration with the KolibriOS AI operating system.
//!
//! Key features:
//! - Natural language-like syntax
//! - Native AI constructs
//! - Memory-safe by design
//! - Concurrent by default

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod codegen;
pub mod type_check;

use alloc::string::String;
use alloc::vec::Vec;

/// Compile Koli source code to target
pub fn compile(source: &str, options: CompileOptions) -> Result<CompileOutput, CompileError> {
    // Lexical analysis
    let tokens = lexer::tokenize(source)?;
    
    // Parsing
    let ast = parser::parse(&tokens)?;
    
    // Type checking
    let typed_ast = type_check::check(&ast)?;
    
    // Code generation
    let output = codegen::generate(&typed_ast, &options)?;
    
    Ok(output)
}

/// Compilation options
#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub target: Target,
    pub optimization: OptimizationLevel,
    pub debug_info: bool,
    pub output_path: Option<String>,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            target: Target::Native,
            optimization: OptimizationLevel::Default,
            debug_info: false,
            output_path: None,
        }
    }
}

/// Compilation target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Native,
    Wasm,
    Bytecode,
    LLVM,
}

/// Optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    None,
    Default,
    Aggressive,
    Size,
}

/// Compilation output
#[derive(Debug)]
pub struct CompileOutput {
    pub code: Vec<u8>,
    pub symbols: Vec<Symbol>,
    pub warnings: Vec<Warning>,
    pub debug_info: Option<DebugInfo>,
}

/// Symbol information
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub offset: usize,
    pub size: usize,
}

/// Symbol kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Variable,
    Constant,
    Type,
    Module,
}

/// Warning message
#[derive(Debug, Clone)]
pub struct Warning {
    pub message: String,
    pub span: Span,
    pub severity: Severity,
}

/// Severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Hint,
    Warning,
    Error,
}

/// Source code span
#[derive(Debug, Clone, Copy, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// Debug information
#[derive(Debug)]
pub struct DebugInfo {
    pub line_table: Vec<LineEntry>,
    pub source_map: Vec<SourceMapping>,
}

/// Line table entry
#[derive(Debug, Clone)]
pub struct LineEntry {
    pub address: usize,
    pub line: usize,
    pub column: usize,
}

/// Source mapping
#[derive(Debug, Clone)]
pub struct SourceMapping {
    pub source_line: usize,
    pub generated_offset: usize,
}

/// Compilation error
#[derive(Debug, Clone, thiserror::Error)]
pub enum CompileError {
    #[error("Lexer error: {0}")]
    LexerError(String),
    
    #[error("Parser error: {0}")]
    ParserError(String),
    
    #[error("Type error: {0}")]
    TypeError(String),
    
    #[error("Code generation error: {0}")]
    CodeGenError(String),
    
    #[error("IO error: {0}")]
    IoError(String),
}
