//! Lexer for Koli Language
//!
//! Tokenizes Koli source code into a sequence of tokens for the parser.
//! Supports all Koli language features including AI-native constructs.

use alloc::string::String;
use alloc::vec::Vec;

/// Tokenize source code
pub fn tokenize(source: &str) -> Result<Vec<Token>, super::CompileError> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
}

/// Lexer state
struct Lexer {
    source: String,
    position: usize,
    tokens: Vec<Token>,
}

impl Lexer {
    fn new(source: &str) -> Self {
        Self {
            source: String::from(source),
            position: 0,
            tokens: Vec::new(),
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, super::CompileError> {
        while self.position < self.source.len() {
            self.skip_whitespace_and_comments()?;

            if self.position >= self.source.len() {
                break;
            }

            let ch = self.current_char();

            if ch.is_ascii_digit() {
                self.read_number()?;
            } else if ch.is_alphabetic() || ch == '_' {
                self.read_identifier();
            } else if ch == '"' {
                self.read_string()?;
            } else {
                self.read_symbol()?;
            }
        }

        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: super::Span {
                start: self.position,
                end: self.position,
            },
        });

        Ok(alloc::vec::Vec::from_iter(self.tokens.drain(..)))
    }

    fn current_char(&self) -> char {
        self.source[self.position..].chars().next().unwrap_or('\0')
    }

    fn peek_char(&self, offset: usize) -> char {
        let mut iter = self.source[self.position..].chars();
        for _ in 0..offset {
            iter.next();
        }
        iter.next().unwrap_or('\0')
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), super::CompileError> {
        while self.position < self.source.len() {
            let ch = self.current_char();

            if ch.is_whitespace() {
                self.position += ch.len_utf8();
            } else if ch == '/' {
                let next = self.peek_char(1);
                if next == '/' {
                    // Single-line comment
                    while self.position < self.source.len() && self.current_char() != '\n' {
                        self.position += self.current_char().len_utf8();
                    }
                } else if next == '*' {
                    // Multi-line comment
                    self.position += 2; // Skip /*
                    while self.position < self.source.len() {
                        if self.current_char() == '*' && self.peek_char(1) == '/' {
                            self.position += 2; // Skip */
                            break;
                        }
                        self.position += self.current_char().len_utf8();
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    fn read_number(&mut self) -> Result<(), super::CompileError> {
        let start = self.position;
        let mut value = String::new();
        let mut is_float = false;

        while self.position < self.source.len() {
            let ch = self.current_char();
            if ch.is_ascii_digit() {
                value.push(ch);
                self.position += ch.len_utf8();
            } else if ch == '.' && !is_float {
                let next = self.peek_char(1);
                if next.is_ascii_digit() {
                    is_float = true;
                    value.push(ch);
                    self.position += ch.len_utf8();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Handle exponent notation (e.g., 1e10, 2.5e-3)
        if self.position < self.source.len() {
            let ch = self.current_char();
            if ch == 'e' || ch == 'E' {
                is_float = true;
                value.push(ch);
                self.position += ch.len_utf8();

                if self.position < self.source.len() {
                    let ch = self.current_char();
                    if ch == '+' || ch == '-' {
                        value.push(ch);
                        self.position += ch.len_utf8();
                    }
                }

                while self.position < self.source.len() {
                    let ch = self.current_char();
                    if ch.is_ascii_digit() {
                        value.push(ch);
                        self.position += ch.len_utf8();
                    } else {
                        break;
                    }
                }
            }
        }

        let kind = if is_float {
            TokenKind::Float(value.parse().unwrap_or(0.0))
        } else {
            TokenKind::Integer(value.parse().unwrap_or(0))
        };

        self.tokens.push(Token {
            kind,
            span: super::Span { start, end: self.position },
        });

        Ok(())
    }

    fn read_identifier(&mut self) {
        let start = self.position;
        let mut value = String::new();

        while self.position < self.source.len() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' {
                value.push(ch);
                self.position += ch.len_utf8();
            } else {
                break;
            }
        }

        let kind = match value.as_str() {
            // Control flow
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "return" => TokenKind::Return,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,

            // Boolean literals
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),

            // AI-specific keywords
            "ai" => TokenKind::Ai,
            "ask" => TokenKind::Ask,
            "cell" => TokenKind::Cell,
            "spawn" => TokenKind::Spawn,
            "capability" => TokenKind::Capability,
            "behavior" => TokenKind::Behavior,
            "property" => TokenKind::Property,

            // Type keywords
            "int" => TokenKind::TypeInt,
            "float" => TokenKind::TypeFloat,
            "bool" => TokenKind::TypeBool,
            "string" => TokenKind::TypeString,
            "void" => TokenKind::TypeVoid,
            "array" => TokenKind::TypeArray,
            "pointer" => TokenKind::TypePointer,

            // Special values
            "self" => TokenKind::Self_,
            "null" => TokenKind::Null,

            // Default to identifier
            _ => TokenKind::Identifier(value),
        };

        self.tokens.push(Token {
            kind,
            span: super::Span { start, end: self.position },
        });
    }

    fn read_string(&mut self) -> Result<(), super::CompileError> {
        let start = self.position;
        self.position += 1; // Skip opening quote

        let mut value = String::new();
        while self.position < self.source.len() {
            let ch = self.current_char();
            if ch == '"' {
                self.position += 1;
                break;
            } else if ch == '\\' {
                // Escape sequences
                self.position += 1;
                if self.position < self.source.len() {
                    let escaped = self.current_char();
                    match escaped {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '\\' => value.push('\\'),
                        '"' => value.push('"'),
                        '0' => value.push('\0'),
                        _ => value.push(escaped),
                    }
                    self.position += escaped.len_utf8();
                }
            } else {
                value.push(ch);
                self.position += ch.len_utf8();
            }
        }

        self.tokens.push(Token {
            kind: TokenKind::String(value),
            span: super::Span { start, end: self.position },
        });

        Ok(())
    }

    fn read_symbol(&mut self) -> Result<(), super::CompileError> {
        let start = self.position;
        let ch = self.current_char();

        // Two-character symbols
        let kind = if self.position + 1 < self.source.len() {
            let next = self.peek_char(1);
            match (ch, next) {
                ('-', '>') => {
                    self.position += 2;
                    TokenKind::Arrow
                }
                ('=', '=') => {
                    self.position += 2;
                    TokenKind::EqualEqual
                }
                ('!', '=') => {
                    self.position += 2;
                    TokenKind::BangEqual
                }
                ('<', '=') => {
                    self.position += 2;
                    TokenKind::LessEqual
                }
                ('>', '=') => {
                    self.position += 2;
                    TokenKind::GreaterEqual
                }
                ('+', '=') => {
                    self.position += 2;
                    TokenKind::PlusEqual
                }
                ('-', '=') => {
                    self.position += 2;
                    TokenKind::MinusEqual
                }
                ('*', '=') => {
                    self.position += 2;
                    TokenKind::StarEqual
                }
                ('/', '=') => {
                    self.position += 2;
                    TokenKind::SlashEqual
                }
                ('&', '&') => {
                    self.position += 2;
                    TokenKind::AndAnd
                }
                ('|', '|') => {
                    self.position += 2;
                    TokenKind::OrOr
                }
                (':', ':') => {
                    self.position += 2;
                    TokenKind::ColonColon
                }
                ('.', '.') => {
                    self.position += 2;
                    TokenKind::DotDot
                }
                _ => {
                    // Single character symbols
                    self.read_single_char_symbol(ch, start)?
                }
            }
        } else {
            self.read_single_char_symbol(ch, start)?
        };

        self.tokens.push(Token {
            kind,
            span: super::Span { start, end: self.position },
        });

        Ok(())
    }

    fn read_single_char_symbol(&mut self, ch: char, _start: usize) -> Result<TokenKind, super::CompileError> {
        let kind = match ch {
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            ':' => TokenKind::Colon,
            '.' => TokenKind::Dot,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '=' => TokenKind::Equal,
            '<' => TokenKind::Less,
            '>' => TokenKind::Greater,
            '!' => TokenKind::Bang,
            '&' => TokenKind::Ampersand,
            '|' => TokenKind::Pipe,
            '?' => TokenKind::Question,
            _ => return Err(super::CompileError::LexerError(
                alloc::format!("Unexpected character: {}", ch)
            )),
        };

        self.position += ch.len_utf8();
        Ok(kind)
    }
}

/// Token
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: super::Span,
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
    Arrow,          // ->
    EqualEqual,     // ==
    BangEqual,      // !=
    LessEqual,      // <=
    GreaterEqual,   // >=
    PlusEqual,      // +=
    MinusEqual,     // -=
    StarEqual,      // *=
    SlashEqual,     // /=
    AndAnd,         // &&
    OrOr,           // ||
    ColonColon,     // ::
    DotDot,         // ..

    // End of file
    Eof,
}

impl TokenKind {
    /// Check if this token represents a type keyword
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

    /// Check if this token can start an expression
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

use alloc::format;
use alloc::vec::Vec as StdVec;
trait FromIterator<T>: Sized {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self;
}
impl<T> FromIterator<T> for StdVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        StdVec::from_iter(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_function() {
        let source = "fn main() { return 42; }";
        let tokens = tokenize(source).unwrap();

        assert!(matches!(tokens[0].kind, TokenKind::Fn));
        assert!(matches!(tokens[1].kind, TokenKind::Identifier(ref s) if s == "main"));
        assert!(matches!(tokens[2].kind, TokenKind::LeftParen));
    }

    #[test]
    fn test_tokenize_arrow() {
        let source = "-> int";
        let tokens = tokenize(source).unwrap();

        assert!(matches!(tokens[0].kind, TokenKind::Arrow));
        assert!(matches!(tokens[1].kind, TokenKind::TypeInt));
    }

    #[test]
    fn test_tokenize_ai_definition() {
        let source = "ai Assistant { capability: code }";
        let tokens = tokenize(source).unwrap();

        assert!(matches!(tokens[0].kind, TokenKind::Ai));
        assert!(matches!(tokens[1].kind, TokenKind::Identifier(ref s) if s == "Assistant"));
    }

    #[test]
    fn test_tokenize_cell() {
        let source = "cell Memory { behavior alloc() { } }";
        let tokens = tokenize(source).unwrap();

        assert!(matches!(tokens[0].kind, TokenKind::Cell));
        assert!(matches!(tokens[1].kind, TokenKind::Identifier(ref s) if s == "Memory"));
    }

    #[test]
    fn test_tokenize_for_loop() {
        let source = "for x in items { }";
        let tokens = tokenize(source).unwrap();

        assert!(matches!(tokens[0].kind, TokenKind::For));
        assert!(matches!(tokens[1].kind, TokenKind::Identifier(ref s) if s == "x"));
        assert!(matches!(tokens[2].kind, TokenKind::In));
    }

    #[test]
    fn test_tokenize_escape_sequences() {
        let source = r#""hello\nworld""#;
        let tokens = tokenize(source).unwrap();

        assert!(matches!(tokens[0].kind, TokenKind::String(ref s) if s == "hello\nworld"));
    }

    #[test]
    fn test_tokenize_float_exponent() {
        let source = "1.5e-10";
        let tokens = tokenize(source).unwrap();

        assert!(matches!(tokens[0].kind, TokenKind::Float(f) if f == 1.5e-10));
    }

    #[test]
    fn test_tokenize_comments() {
        let source = "// comment\nfn test() { } /* multi\nline */";
        let tokens = tokenize(source).unwrap();

        // Should only have function tokens, not comments
        assert!(matches!(tokens[0].kind, TokenKind::Fn));
    }
}
