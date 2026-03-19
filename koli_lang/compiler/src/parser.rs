//! Parser for Koli Language AST
//!
//! Parses tokens into an Abstract Syntax Tree (AST).
//! Supports all Koli language features including AI-native constructs,
//! cells, behaviors, and advanced expressions.

use alloc::string::String;
use alloc::vec::Vec;

use super::lexer::Token;
use super::lexer::TokenKind;
use super::ast::*;
use super::Span;

/// Parse tokens into AST
pub fn parse(tokens: &[Token]) -> Result<Program, super::CompileError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

/// Parser state
struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn new(tokens: &[Token]) -> Self {
        Self {
            tokens: Vec::from(tokens),
            position: 0,
        }
    }

    fn parse_program(&mut self) -> Result<Program, super::CompileError> {
        let mut items = Vec::new();

        while !self.is_at_end() {
            items.push(self.parse_item()?);
        }

        Ok(Program { items })
    }

    fn parse_item(&mut self) -> Result<Item, super::CompileError> {
        if self.check_token(TokenKind::Fn) {
            self.parse_function()
        } else if self.check_token(TokenKind::Ai) {
            self.parse_ai_definition()
        } else if self.check_token(TokenKind::Cell) {
            self.parse_cell_definition()
        } else {
            Err(super::CompileError::ParserError(
                String::from("Expected function (fn), AI definition (ai), or cell definition (cell)")
            ))
        }
    }

    fn parse_function(&mut self) -> Result<Item, super::CompileError> {
        self.advance(); // consume 'fn'

        let name = self.expect_identifier()?;

        // Type parameters (generics)
        let type_params = if self.check_token(TokenKind::Less) {
            self.parse_type_params()?
        } else {
            Vec::new()
        };

        self.expect_token(TokenKind::LeftParen)?;
        let params = self.parse_params()?;
        self.expect_token(TokenKind::RightParen)?;

        // Return type
        let return_type = if self.check_token(TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_block()?;

        Ok(Item::Function(Function {
            name,
            type_params,
            params,
            return_type,
            body,
        }))
    }

    fn parse_type_params(&mut self) -> Result<Vec<String>, super::CompileError> {
        self.expect_token(TokenKind::Less)?;
        let mut params = Vec::new();

        loop {
            params.push(self.expect_identifier()?);
            if !self.check_token(TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        self.expect_token(TokenKind::Greater)?;
        Ok(params)
    }

    fn parse_ai_definition(&mut self) -> Result<Item, super::CompileError> {
        self.advance(); // consume 'ai'

        let name = self.expect_identifier()?;
        self.expect_token(TokenKind::LeftBrace)?;

        let mut capabilities = Vec::new();
        while !self.check_token(TokenKind::RightBrace) && !self.is_at_end() {
            capabilities.push(self.parse_ai_capability()?);
        }

        self.expect_token(TokenKind::RightBrace)?;

        Ok(Item::Ai(AiDefinition { name, capabilities }))
    }

    fn parse_ai_capability(&mut self) -> Result<AiCapability, super::CompileError> {
        // Check for 'capability' keyword (optional)
        if self.check_token(TokenKind::Capability) {
            self.advance();
        }

        let name = self.expect_identifier()?;
        self.expect_token(TokenKind::Colon)?;
        let capability_type = self.parse_type()?;

        Ok(AiCapability { name, capability_type })
    }

    fn parse_cell_definition(&mut self) -> Result<Item, super::CompileError> {
        self.advance(); // consume 'cell'

        let name = self.expect_identifier()?;
        self.expect_token(TokenKind::LeftBrace)?;

        let mut properties = Vec::new();
        let mut behaviors = Vec::new();

        while !self.check_token(TokenKind::RightBrace) && !self.is_at_end() {
            // Check for property or behavior
            if self.check_token(TokenKind::Let) {
                properties.push(self.parse_property()?);
            } else if self.check_token(TokenKind::Behavior) {
                behaviors.push(self.parse_behavior_keyword()?);
            } else if self.check_token(TokenKind::Identifier(_)) {
                // Could be property (let) or behavior
                // Peek ahead to determine
                let is_behavior = self.is_behavior_start();
                if is_behavior {
                    behaviors.push(self.parse_behavior()?);
                } else {
                    properties.push(self.parse_property_simple()?);
                }
            } else {
                break;
            }
        }

        self.expect_token(TokenKind::RightBrace)?;

        Ok(Item::Cell(CellDefinition {
            name,
            properties,
            behaviors,
        }))
    }

    fn is_behavior_start(&self) -> bool {
        // Look ahead to see if this is a behavior (identifier followed by paren)
        let saved_pos = self.position;
        let result = if let Some(token) = self.tokens.get(saved_pos) {
            if let TokenKind::Identifier(_) = &token.kind {
                if let Some(next) = self.tokens.get(saved_pos + 1) {
                    matches!(next.kind, TokenKind::LeftParen)
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };
        result
    }

    fn parse_property(&mut self) -> Result<Property, super::CompileError> {
        self.advance(); // consume 'let'

        let name = self.expect_identifier()?;
        self.expect_token(TokenKind::Colon)?;
        let ty = self.parse_type()?;

        let default_value = if self.check_token(TokenKind::Equal) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Optional semicolon
        if self.check_token(TokenKind::Semicolon) {
            self.advance();
        }

        Ok(Property {
            name,
            ty,
            default_value,
        })
    }

    fn parse_property_simple(&mut self) -> Result<Property, super::CompileError> {
        // Parse property without 'let' keyword
        let name = self.expect_identifier()?;
        self.expect_token(TokenKind::Colon)?;
        let ty = self.parse_type()?;

        let default_value = if self.check_token(TokenKind::Equal) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Optional semicolon
        if self.check_token(TokenKind::Semicolon) {
            self.advance();
        }

        Ok(Property {
            name,
            ty,
            default_value,
        })
    }

    fn parse_behavior(&mut self) -> Result<Behavior, super::CompileError> {
        let name = self.expect_identifier()?;

        self.expect_token(TokenKind::LeftParen)?;
        let params = self.parse_params()?;
        self.expect_token(TokenKind::RightParen)?;

        // Return type
        let return_type = if self.check_token(TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_block()?;

        Ok(Behavior {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_behavior_keyword(&mut self) -> Result<Behavior, super::CompileError> {
        self.advance(); // consume 'behavior'
        self.parse_behavior()
    }

    fn parse_params(&mut self) -> Result<Vec<Parameter>, super::CompileError> {
        let mut params = Vec::new();

        if !self.check_token(TokenKind::RightParen) {
            loop {
                let name = self.expect_identifier()?;
                self.expect_token(TokenKind::Colon)?;
                let ty = self.parse_type()?;

                params.push(Parameter { name, ty });

                if !self.check_token(TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        Ok(params)
    }

    fn parse_type(&mut self) -> Result<Type, super::CompileError> {
        // Handle primitive types
        let base_type = if self.check_token(TokenKind::TypeInt) {
            self.advance();
            Type::Int
        } else if self.check_token(TokenKind::TypeFloat) {
            self.advance();
            Type::Float
        } else if self.check_token(TokenKind::TypeBool) {
            self.advance();
            Type::Bool
        } else if self.check_token(TokenKind::TypeString) {
            self.advance();
            Type::String
        } else if self.check_token(TokenKind::TypeVoid) {
            self.advance();
            Type::Void
        } else if self.check_token(TokenKind::TypeArray) {
            self.advance();
            self.expect_token(TokenKind::Less)?;
            let inner = self.parse_type()?;
            self.expect_token(TokenKind::Greater)?;
            Type::Array(Box::new(inner))
        } else if self.check_token(TokenKind::TypePointer) {
            self.advance();
            self.expect_token(TokenKind::Less)?;
            let inner = self.parse_type()?;
            self.expect_token(TokenKind::Greater)?;
            Type::Pointer(Box::new(inner))
        } else if self.check_token(TokenKind::Identifier(_)) {
            let name = self.expect_identifier()?;
            Type::Custom(name)
        } else {
            return Err(super::CompileError::ParserError(
                String::from("Expected type")
            ));
        };

        // Handle optional types (e.g., int?)
        let result = if self.check_token(TokenKind::Question) {
            self.advance();
            Type::Optional(Box::new(base_type))
        } else {
            base_type
        };

        Ok(result)
    }

    fn parse_block(&mut self) -> Result<Block, super::CompileError> {
        self.expect_token(TokenKind::LeftBrace)?;

        let mut statements = Vec::new();
        while !self.check_token(TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        self.expect_token(TokenKind::RightBrace)?;

        Ok(Block { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, super::CompileError> {
        if self.check_token(TokenKind::Let) {
            self.parse_let_statement()
        } else if self.check_token(TokenKind::Return) {
            self.parse_return_statement()
        } else if self.check_token(TokenKind::If) {
            self.parse_if_statement()
        } else if self.check_token(TokenKind::While) {
            self.parse_while_statement()
        } else if self.check_token(TokenKind::For) {
            self.parse_for_statement()
        } else if self.check_token(TokenKind::Break) {
            self.advance();
            if self.check_token(TokenKind::Semicolon) {
                self.advance();
            }
            Ok(Statement::Break)
        } else if self.check_token(TokenKind::Continue) {
            self.advance();
            if self.check_token(TokenKind::Semicolon) {
                self.advance();
            }
            Ok(Statement::Continue)
        } else if self.check_token(TokenKind::Ask) {
            self.parse_ask_statement()
        } else if self.check_token(TokenKind::Spawn) {
            self.parse_spawn_statement()
        } else {
            self.parse_expression_or_assignment()
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, super::CompileError> {
        self.advance(); // consume 'let'
        let name = self.expect_identifier()?;

        let ty = if self.check_token(TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect_token(TokenKind::Equal)?;
        let value = self.parse_expression()?;

        if self.check_token(TokenKind::Semicolon) {
            self.advance();
        }

        Ok(Statement::Let(LetStatement { name, ty, value }))
    }

    fn parse_return_statement(&mut self) -> Result<Statement, super::CompileError> {
        self.advance(); // consume 'return'
        let value = if !self.check_token(TokenKind::Semicolon) && !self.check_token(TokenKind::RightBrace) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        if self.check_token(TokenKind::Semicolon) {
            self.advance();
        }
        Ok(Statement::Return(value))
    }

    fn parse_if_statement(&mut self) -> Result<Statement, super::CompileError> {
        self.advance(); // consume 'if'
        let condition = self.parse_expression()?;
        let then_block = self.parse_block()?;

        let else_block = if self.check_token(TokenKind::Else) {
            self.advance();
            if self.check_token(TokenKind::If) {
                // else if
                Some(Block {
                    statements: vec![self.parse_if_statement()?],
                })
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        Ok(Statement::If(IfStatement {
            condition,
            then_block,
            else_block,
        }))
    }

    fn parse_while_statement(&mut self) -> Result<Statement, super::CompileError> {
        self.advance(); // consume 'while'
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement::While(WhileStatement { condition, body }))
    }

    fn parse_for_statement(&mut self) -> Result<Statement, super::CompileError> {
        self.advance(); // consume 'for'

        let var = self.expect_identifier()?;

        self.expect_token(TokenKind::In)?;
        let iterable = self.parse_expression()?;

        let body = self.parse_block()?;

        Ok(Statement::For(ForStatement {
            var,
            iterable,
            body,
        }))
    }

    fn parse_ask_statement(&mut self) -> Result<Statement, super::CompileError> {
        self.advance(); // consume 'ask'

        // Check for AI model name
        let model = if self.check_token(TokenKind::Identifier(_)) {
            let name = self.expect_identifier()?;
            Some(name)
        } else {
            None
        };

        let prompt = self.parse_expression()?;

        if self.check_token(TokenKind::Semicolon) {
            self.advance();
        }

        Ok(Statement::Ask(AskStatement { model, prompt }))
    }

    fn parse_spawn_statement(&mut self) -> Result<Statement, super::CompileError> {
        self.advance(); // consume 'spawn'
        let cell_type = self.expect_identifier()?;

        let args = if self.check_token(TokenKind::LeftParen) {
            self.advance();
            let args = self.parse_expression_list()?;
            self.expect_token(TokenKind::RightParen)?;
            args
        } else {
            Vec::new()
        };

        if self.check_token(TokenKind::Semicolon) {
            self.advance();
        }

        Ok(Statement::Spawn(SpawnStatement { cell_type, args }))
    }

    fn parse_expression_or_assignment(&mut self) -> Result<Statement, super::CompileError> {
        let expr = self.parse_expression()?;

        // Check for assignment
        if self.check_token(TokenKind::Equal) {
            self.advance();
            let value = self.parse_expression()?;
            if self.check_token(TokenKind::Semicolon) {
                self.advance();
            }
            Ok(Statement::Assignment(AssignmentStatement {
                target: expr,
                value,
            }))
        } else if self.check_token(TokenKind::PlusEqual) {
            self.advance();
            let value = self.parse_expression()?;
            if self.check_token(TokenKind::Semicolon) {
                self.advance();
            }
            Ok(Statement::Assignment(AssignmentStatement {
                target: expr.clone(),
                value: Expression::Binary(BinaryExpr {
                    left: Box::new(expr),
                    op: BinaryOp::Add,
                    right: Box::new(value),
                }),
            }))
        } else if self.check_token(TokenKind::MinusEqual) {
            self.advance();
            let value = self.parse_expression()?;
            if self.check_token(TokenKind::Semicolon) {
                self.advance();
            }
            Ok(Statement::Assignment(AssignmentStatement {
                target: expr.clone(),
                value: Expression::Binary(BinaryExpr {
                    left: Box::new(expr),
                    op: BinaryOp::Sub,
                    right: Box::new(value),
                }),
            }))
        } else {
            if self.check_token(TokenKind::Semicolon) {
                self.advance();
            }
            Ok(Statement::Expression(expr))
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, super::CompileError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expression, super::CompileError> {
        let mut left = self.parse_and()?;

        while self.check_token(TokenKind::OrOr) {
            self.advance();
            let right = self.parse_and()?;
            left = Expression::Binary(BinaryExpr {
                left: Box::new(left),
                op: BinaryOp::Or,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expression, super::CompileError> {
        let mut left = self.parse_equality()?;

        while self.check_token(TokenKind::AndAnd) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expression::Binary(BinaryExpr {
                left: Box::new(left),
                op: BinaryOp::And,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, super::CompileError> {
        let mut left = self.parse_comparison()?;

        loop {
            let op = if self.check_token(TokenKind::EqualEqual) {
                BinaryOp::Eq
            } else if self.check_token(TokenKind::BangEqual) {
                BinaryOp::Ne
            } else {
                break;
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::Binary(BinaryExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, super::CompileError> {
        let mut left = self.parse_term()?;

        loop {
            let op = if self.check_token(TokenKind::Less) {
                BinaryOp::Lt
            } else if self.check_token(TokenKind::LessEqual) {
                BinaryOp::Le
            } else if self.check_token(TokenKind::Greater) {
                BinaryOp::Gt
            } else if self.check_token(TokenKind::GreaterEqual) {
                BinaryOp::Ge
            } else {
                break;
            };
            self.advance();
            let right = self.parse_term()?;
            left = Expression::Binary(BinaryExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expression, super::CompileError> {
        let mut left = self.parse_factor()?;

        loop {
            let op = if self.check_token(TokenKind::Plus) {
                BinaryOp::Add
            } else if self.check_token(TokenKind::Minus) {
                BinaryOp::Sub
            } else {
                break;
            };
            self.advance();
            let right = self.parse_factor()?;
            left = Expression::Binary(BinaryExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expression, super::CompileError> {
        let mut left = self.parse_unary()?;

        loop {
            let op = if self.check_token(TokenKind::Star) {
                BinaryOp::Mul
            } else if self.check_token(TokenKind::Slash) {
                BinaryOp::Div
            } else if self.check_token(TokenKind::Percent) {
                BinaryOp::Mod
            } else {
                break;
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expression::Binary(BinaryExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, super::CompileError> {
        if self.check_token(TokenKind::Bang) {
            self.advance();
            let expr = self.parse_unary()?;
            Ok(Expression::Unary(UnaryExpr {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            }))
        } else if self.check_token(TokenKind::Minus) {
            self.advance();
            let expr = self.parse_unary()?;
            Ok(Expression::Unary(UnaryExpr {
                op: UnaryOp::Neg,
                expr: Box::new(expr),
            }))
        } else {
            self.parse_postfix()
        }
    }

    fn parse_postfix(&mut self) -> Result<Expression, super::CompileError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check_token(TokenKind::LeftParen) {
                // Function/method call
                self.advance();
                let args = self.parse_expression_list()?;
                self.expect_token(TokenKind::RightParen)?;

                expr = if let Expression::Identifier(name) = expr {
                    Expression::Call(CallExpr { name, args })
                } else {
                    Expression::MethodCall(MethodCallExpr {
                        receiver: Box::new(expr),
                        args,
                    })
                };
            } else if self.check_token(TokenKind::Dot) {
                // Field access
                self.advance();
                let field = self.expect_identifier()?;

                // Check if it's a method call
                if self.check_token(TokenKind::LeftParen) {
                    self.advance();
                    let args = self.parse_expression_list()?;
                    self.expect_token(TokenKind::RightParen)?;

                    expr = Expression::MethodCall(MethodCallExpr {
                        receiver: Box::new(expr),
                        method: field,
                        args,
                    });
                } else {
                    expr = Expression::FieldAccess(FieldAccessExpr {
                        receiver: Box::new(expr),
                        field,
                    });
                }
            } else if self.check_token(TokenKind::LeftBracket) {
                // Index access
                self.advance();
                let index = self.parse_expression()?;
                self.expect_token(TokenKind::RightBracket)?;

                expr = Expression::Index(IndexExpr {
                    receiver: Box::new(expr),
                    index: Box::new(index),
                });
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, super::CompileError> {
        let token = self.current_token();

        match &token.kind {
            TokenKind::Integer(n) => {
                self.advance();
                Ok(Expression::Literal(Literal::Int(*n)))
            }
            TokenKind::Float(f) => {
                self.advance();
                Ok(Expression::Literal(Literal::Float(*f)))
            }
            TokenKind::String(s) => {
                self.advance();
                Ok(Expression::Literal(Literal::String(s.clone())))
            }
            TokenKind::Bool(b) => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(*b)))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Expression::Literal(Literal::Null))
            }
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(Expression::Identifier(name.clone()))
            }
            TokenKind::Self_ => {
                self.advance();
                Ok(Expression::Identifier(String::from("self")))
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_token(TokenKind::RightParen)?;
                Ok(expr)
            }
            TokenKind::LeftBracket => {
                // Array literal
                self.advance();
                let elements = self.parse_expression_list()?;
                self.expect_token(TokenKind::RightBracket)?;
                Ok(Expression::Array(ArrayExpr { elements }))
            }
            TokenKind::LeftBrace => {
                // Struct literal
                self.advance();
                let fields = self.parse_struct_fields()?;
                self.expect_token(TokenKind::RightBrace)?;
                Ok(Expression::Struct(StructExpr { fields }))
            }
            _ => Err(super::CompileError::ParserError(
                String::from("Expected expression")
            )),
        }
    }

    fn parse_expression_list(&mut self) -> Result<Vec<Expression>, super::CompileError> {
        let mut exprs = Vec::new();

        if !self.check_token(TokenKind::RightParen) && !self.check_token(TokenKind::RightBracket) {
            loop {
                exprs.push(self.parse_expression()?);
                if !self.check_token(TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        Ok(exprs)
    }

    fn parse_struct_fields(&mut self) -> Result<Vec<(String, Expression)>, super::CompileError> {
        let mut fields = Vec::new();

        while !self.check_token(TokenKind::RightBrace) && !self.is_at_end() {
            let name = self.expect_identifier()?;
            self.expect_token(TokenKind::Colon)?;
            let value = self.parse_expression()?;
            fields.push((name, value));

            if !self.check_token(TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        Ok(fields)
    }

    // Helper methods
    fn current_token(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token {
            kind: TokenKind::Eof,
            span: Span::default(),
        })
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.position += 1;
        }
        self.tokens.get(self.position - 1).unwrap()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_token().kind, TokenKind::Eof)
    }

    fn check_token(&self, kind: TokenKind) -> bool {
        core::mem::discriminant(&self.current_token().kind) == core::mem::discriminant(&kind)
    }

    fn expect_identifier(&mut self) -> Result<String, super::CompileError> {
        match &self.current_token().kind {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(super::CompileError::ParserError(
                String::from("Expected identifier")
            )),
        }
    }

    fn expect_token(&mut self, kind: TokenKind) -> Result<(), super::CompileError> {
        if self.check_token(kind) {
            self.advance();
            Ok(())
        } else {
            Err(super::CompileError::ParserError(
                alloc::format!("Unexpected token: expected {:?}, got {:?}", kind, self.current_token().kind)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_function() {
        let source = "fn main() { return 42; }";
        let tokens = super::super::lexer::tokenize(source).unwrap();
        let program = parse(&tokens).unwrap();

        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::Function(f) => {
                assert_eq!(f.name, "main");
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_ai_definition() {
        let source = "ai Assistant { capability: code_completion }";
        let tokens = super::super::lexer::tokenize(source).unwrap();
        let program = parse(&tokens).unwrap();

        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::Ai(ai) => {
                assert_eq!(ai.name, "Assistant");
                assert_eq!(ai.capabilities.len(), 1);
            }
            _ => panic!("Expected AI definition"),
        }
    }

    #[test]
    fn test_parse_cell() {
        let source = r#"
            cell Memory {
                let size: int = 1024
                behavior alloc(n: int) -> pointer { }
            }
        "#;
        let tokens = super::super::lexer::tokenize(source).unwrap();
        let program = parse(&tokens).unwrap();

        match &program.items[0] {
            Item::Cell(cell) => {
                assert_eq!(cell.name, "Memory");
            }
            _ => panic!("Expected cell definition"),
        }
    }

    #[test]
    fn test_parse_for_loop() {
        let source = "fn test() { for x in items { } }";
        let tokens = super::super::lexer::tokenize(source).unwrap();
        let program = parse(&tokens).unwrap();

        match &program.items[0] {
            Item::Function(f) => {
                match &f.body.statements[0] {
                    Statement::For(for_stmt) => {
                        assert_eq!(for_stmt.var, "x");
                    }
                    _ => panic!("Expected for statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_binary_expression() {
        let source = "fn test() { let x = 1 + 2 * 3; }";
        let tokens = super::super::lexer::tokenize(source).unwrap();
        let program = parse(&tokens).unwrap();

        match &program.items[0] {
            Item::Function(f) => {
                match &f.body.statements[0] {
                    Statement::Let(let_stmt) => {
                        // Should be 1 + (2 * 3) due to precedence
                        match &let_stmt.value {
                            Expression::Binary(bin) => {
                                assert_eq!(bin.op, BinaryOp::Add);
                            }
                            _ => panic!("Expected binary expression"),
                        }
                    }
                    _ => panic!("Expected let statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_method_call() {
        let source = "fn test() { self.process(); }";
        let tokens = super::super::lexer::tokenize(source).unwrap();
        let program = parse(&tokens).unwrap();

        match &program.items[0] {
            Item::Function(f) => {
                match &f.body.statements[0] {
                    Statement::Expression(expr) => {
                        match expr {
                            Expression::MethodCall(call) => {
                                // Check receiver is "self"
                            }
                            _ => panic!("Expected method call"),
                        }
                    }
                    _ => panic!("Expected expression statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }
}
