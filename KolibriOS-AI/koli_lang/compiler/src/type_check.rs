//! Type Checking for Koli Language
//!
//! This module provides type inference and checking for Koli programs,
//! ensuring type safety before code generation.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::ast::*;
use super::CompileError;

/// Type-checked program with symbol tables
#[derive(Debug, Clone)]
pub struct TypedProgram {
    pub items: Vec<TypedItem>,
    pub symbols: SymbolTable,
}

/// Type-checked top-level item
#[derive(Debug, Clone)]
pub enum TypedItem {
    Function(TypedFunction),
    Ai(TypedAiDefinition),
    Cell(TypedCellDefinition),
}

/// Type-checked function
#[derive(Debug, Clone)]
pub struct TypedFunction {
    pub name: String,
    pub params: Vec<TypedParameter>,
    pub return_type: Type,
    pub body: TypedBlock,
}

/// Type-checked parameter
#[derive(Debug, Clone)]
pub struct TypedParameter {
    pub name: String,
    pub ty: Type,
}

/// Type-checked block
#[derive(Debug, Clone)]
pub struct TypedBlock {
    pub statements: Vec<TypedStatement>,
    pub return_type: Type,
}

/// Type-checked statement
#[derive(Debug, Clone)]
pub enum TypedStatement {
    Let {
        name: String,
        ty: Type,
        value: TypedExpression,
    },
    Return(Option<TypedExpression>),
    If {
        condition: TypedExpression,
        then_block: TypedBlock,
        else_block: Option<TypedBlock>,
    },
    For {
        var: String,
        iterable: TypedExpression,
        body: TypedBlock,
    },
    Ask(TypedExpression),
    Expression(TypedExpression),
    Assignment {
        target: TypedExpression,
        value: TypedExpression,
    },
}

/// Type-checked expression
#[derive(Debug, Clone)]
pub struct TypedExpression {
    pub kind: TypedExprKind,
    pub ty: Type,
}

/// Type-checked expression kind
#[derive(Debug, Clone)]
pub enum TypedExprKind {
    Literal(Literal),
    Identifier(String),
    Binary {
        left: Box<TypedExpression>,
        op: BinaryOp,
        right: Box<TypedExpression>,
    },
    Call {
        name: String,
        args: Vec<TypedExpression>,
    },
    MethodCall {
        receiver: Box<TypedExpression>,
        method: String,
        args: Vec<TypedExpression>,
    },
    FieldAccess {
        receiver: Box<TypedExpression>,
        field: String,
    },
    AiCall {
        model: String,
        prompt: Box<TypedExpression>,
    },
}

/// Type-checked AI definition
#[derive(Debug, Clone)]
pub struct TypedAiDefinition {
    pub name: String,
    pub capabilities: Vec<TypedAiCapability>,
}

/// Type-checked AI capability
#[derive(Debug, Clone)]
pub struct TypedAiCapability {
    pub name: String,
    pub capability_type: Type,
}

/// Type-checked Cell definition
#[derive(Debug, Clone)]
pub struct TypedCellDefinition {
    pub name: String,
    pub properties: Vec<TypedProperty>,
    pub behaviors: Vec<TypedBehavior>,
}

/// Type-checked property
#[derive(Debug, Clone)]
pub struct TypedProperty {
    pub name: String,
    pub ty: Type,
    pub default_value: Option<TypedExpression>,
}

/// Type-checked behavior
#[derive(Debug, Clone)]
pub struct TypedBehavior {
    pub name: String,
    pub params: Vec<TypedParameter>,
    pub return_type: Type,
    pub body: TypedBlock,
}

/// Symbol table for type checking
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    /// Variable bindings with their types
    bindings: BTreeMap<String, Type>,
    /// Function signatures
    functions: BTreeMap<String, FunctionSignature>,
    /// AI definitions
    ai_definitions: BTreeMap<String, AiSignature>,
    /// Cell definitions
    cell_definitions: BTreeMap<String, CellSignature>,
    /// Parent scope (for nested scopes)
    parent: Option<Box<SymbolTable>>,
}

/// Function signature
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
}

/// AI signature
#[derive(Debug, Clone)]
pub struct AiSignature {
    pub capabilities: Vec<(String, Type)>,
}

/// Cell signature
#[derive(Debug, Clone)]
pub struct CellSignature {
    pub properties: Vec<(String, Type)>,
    pub behaviors: BTreeMap<String, FunctionSignature>,
}

impl SymbolTable {
    /// Create a new empty symbol table
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a child scope
    pub fn child(&self) -> Self {
        Self {
            bindings: BTreeMap::new(),
            functions: BTreeMap::new(),
            ai_definitions: BTreeMap::new(),
            cell_definitions: BTreeMap::new(),
            parent: Some(Box::new(self.clone())),
        }
    }

    /// Insert a binding
    pub fn insert(&mut self, name: String, ty: Type) {
        self.bindings.insert(name, ty);
    }

    /// Look up a binding
    pub fn lookup(&self, name: &str) -> Option<Type> {
        self.bindings
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref()?.lookup(name))
    }

    /// Insert a function
    pub fn insert_function(&mut self, name: String, sig: FunctionSignature) {
        self.functions.insert(name, sig);
    }

    /// Look up a function
    pub fn lookup_function(&self, name: &str) -> Option<FunctionSignature> {
        self.functions
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref()?.lookup_function(name))
    }

    /// Insert an AI definition
    pub fn insert_ai(&mut self, name: String, sig: AiSignature) {
        self.ai_definitions.insert(name, sig);
    }

    /// Look up an AI definition
    pub fn lookup_ai(&self, name: &str) -> Option<AiSignature> {
        self.ai_definitions
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref()?.lookup_ai(name))
    }

    /// Insert a cell definition
    pub fn insert_cell(&mut self, name: String, sig: CellSignature) {
        self.cell_definitions.insert(name, sig);
    }

    /// Look up a cell definition
    pub fn lookup_cell(&self, name: &str) -> Option<CellSignature> {
        self.cell_definitions
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref()?.lookup_cell(name))
    }
}

/// Type check a program
pub fn check(program: &Program) -> Result<TypedProgram, CompileError> {
    let mut checker = TypeChecker::new();
    checker.check_program(program)
}

/// Type checker state
struct TypeChecker {
    symbols: SymbolTable,
}

impl TypeChecker {
    fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
        }
    }

    fn check_program(&mut self, program: &Program) -> Result<TypedProgram, CompileError> {
        // First pass: collect all declarations
        for item in &program.items {
            self.declare_item(item)?;
        }

        // Second pass: type check all items
        let mut typed_items = Vec::new();
        for item in &program.items {
            typed_items.push(self.check_item(item)?);
        }

        Ok(TypedProgram {
            items: typed_items,
            symbols: self.symbols.clone(),
        })
    }

    fn declare_item(&mut self, item: &Item) -> Result<(), CompileError> {
        match item {
            Item::Function(func) => {
                let sig = FunctionSignature {
                    params: func.params.iter().map(|p| (p.name.clone(), p.ty.clone())).collect(),
                    return_type: func.return_type.clone().unwrap_or(Type::Void),
                };
                self.symbols.insert_function(func.name.clone(), sig);
            }
            Item::Ai(ai) => {
                let sig = AiSignature {
                    capabilities: ai.capabilities.iter().map(|c| (c.name.clone(), c.capability_type.clone())).collect(),
                };
                self.symbols.insert_ai(ai.name.clone(), sig);
            }
            Item::Cell(cell) => {
                // Extract properties from behaviors that start with 'let'
                let properties: Vec<(String, Type)> = Vec::new(); // Will be filled during full parse
                let mut behaviors = BTreeMap::new();

                for behavior in &cell.behaviors {
                    // For now, assume void return if not specified
                    behaviors.insert(
                        behavior.name.clone(),
                        FunctionSignature {
                            params: Vec::new(),
                            return_type: Type::Void,
                        },
                    );
                }

                let sig = CellSignature {
                    properties,
                    behaviors,
                };
                self.symbols.insert_cell(cell.name.clone(), sig);
            }
        }
        Ok(())
    }

    fn check_item(&mut self, item: &Item) -> Result<TypedItem, CompileError> {
        match item {
            Item::Function(func) => self.check_function(func).map(TypedItem::Function),
            Item::Ai(ai) => self.check_ai_definition(ai).map(TypedItem::Ai),
            Item::Cell(cell) => self.check_cell_definition(cell).map(TypedItem::Cell),
        }
    }

    fn check_function(&mut self, func: &Function) -> Result<TypedFunction, CompileError> {
        // Create child scope for function
        let mut func_scope = self.symbols.child();

        // Add parameters to scope
        let mut typed_params = Vec::new();
        for param in &func.params {
            func_scope.insert(param.name.clone(), param.ty.clone());
            typed_params.push(TypedParameter {
                name: param.name.clone(),
                ty: param.ty.clone(),
            });
        }

        // Type check body
        let saved_symbols = self.symbols.clone();
        self.symbols = func_scope;
        let body = self.check_block(&func.body)?;
        let body_return_type = body.return_type.clone();
        self.symbols = saved_symbols;

        let return_type = func.return_type.clone().unwrap_or(Type::Void);

        // Verify return type matches
        if body_return_type != Type::Void && body_return_type != return_type {
            return Err(CompileError::TypeError(
                alloc::format!(
                    "Function {} returns {:?} but declared {:?}",
                    func.name, body_return_type, return_type
                ),
            ));
        }

        Ok(TypedFunction {
            name: func.name.clone(),
            params: typed_params,
            return_type,
            body,
        })
    }

    fn check_ai_definition(&mut self, ai: &AiDefinition) -> Result<TypedAiDefinition, CompileError> {
        let mut typed_capabilities = Vec::new();
        for cap in &ai.capabilities {
            typed_capabilities.push(TypedAiCapability {
                name: cap.name.clone(),
                capability_type: cap.capability_type.clone(),
            });
        }

        Ok(TypedAiDefinition {
            name: ai.name.clone(),
            capabilities: typed_capabilities,
        })
    }

    fn check_cell_definition(&mut self, cell: &CellDefinition) -> Result<TypedCellDefinition, CompileError> {
        let mut typed_behaviors = Vec::new();
        for behavior in &cell.behaviors {
            typed_behaviors.push(TypedBehavior {
                name: behavior.name.clone(),
                params: Vec::new(),
                return_type: Type::Void,
                body: self.check_block(&behavior.body)?,
            });
        }

        Ok(TypedCellDefinition {
            name: cell.name.clone(),
            properties: Vec::new(),
            behaviors: typed_behaviors,
        })
    }

    fn check_block(&mut self, block: &Block) -> Result<TypedBlock, CompileError> {
        let mut typed_statements = Vec::new();
        let mut return_type = Type::Void;

        for stmt in &block.statements {
            let typed = self.check_statement(stmt)?;
            if let TypedStatement::Return(Some(expr)) = &typed {
                return_type = expr.ty.clone();
            }
            typed_statements.push(typed);
        }

        Ok(TypedBlock {
            statements: typed_statements,
            return_type,
        })
    }

    fn check_statement(&mut self, stmt: &Statement) -> Result<TypedStatement, CompileError> {
        match stmt {
            Statement::Let(let_stmt) => {
                let typed_value = self.check_expression(&let_stmt.value)?;
                let ty = let_stmt.ty.clone().unwrap_or_else(|| typed_value.ty.clone());

                // Type check that value matches declared type
                if typed_value.ty != Type::Void && typed_value.ty != ty {
                    return Err(CompileError::TypeError(
                        alloc::format!(
                            "Type mismatch in let statement: expected {:?}, got {:?}",
                            ty, typed_value.ty
                        ),
                    ));
                }

                self.symbols.insert(let_stmt.name.clone(), ty.clone());

                Ok(TypedStatement::Let {
                    name: let_stmt.name.clone(),
                    ty,
                    value: typed_value,
                })
            }
            Statement::Return(expr) => {
                let typed_expr = expr.as_ref()
                    .map(|e| self.check_expression(e))
                    .transpose()?;
                Ok(TypedStatement::Return(typed_expr))
            }
            Statement::If(if_stmt) => {
                let typed_condition = self.check_expression(&if_stmt.condition)?;

                if typed_condition.ty != Type::Bool {
                    return Err(CompileError::TypeError(
                        alloc::format!("If condition must be bool, got {:?}", typed_condition.ty),
                    ));
                }

                let typed_then = self.check_block(&if_stmt.then_block)?;
                let typed_else = if_stmt.else_block.as_ref()
                    .map(|b| self.check_block(b))
                    .transpose()?;

                Ok(TypedStatement::If {
                    condition: typed_condition,
                    then_block: typed_then,
                    else_block: typed_else,
                })
            }
            Statement::Ask(expr) => {
                let typed_expr = self.check_expression(expr)?;
                Ok(TypedStatement::Ask(typed_expr))
            }
            Statement::Expression(expr) => {
                let typed_expr = self.check_expression(expr)?;
                Ok(TypedStatement::Expression(typed_expr))
            }
        }
    }

    fn check_expression(&mut self, expr: &Expression) -> Result<TypedExpression, CompileError> {
        match expr {
            Expression::Literal(lit) => {
                let ty = match lit {
                    Literal::Int(_) => Type::Int,
                    Literal::Float(_) => Type::Float,
                    Literal::String(_) => Type::String,
                    Literal::Bool(_) => Type::Bool,
                };
                Ok(TypedExpression {
                    kind: TypedExprKind::Literal(lit.clone()),
                    ty,
                })
            }
            Expression::Identifier(name) => {
                let ty = self.symbols.lookup(name).ok_or_else(|| {
                    CompileError::TypeError(alloc::format!("Undefined identifier: {}", name))
                })?;
                Ok(TypedExpression {
                    kind: TypedExprKind::Identifier(name.clone()),
                    ty,
                })
            }
            Expression::Binary(bin) => {
                let left = self.check_expression(&bin.left)?;
                let right = self.check_expression(&bin.right)?;

                let ty = self.infer_binary_type(&left.ty, bin.op, &right.ty)?;

                Ok(TypedExpression {
                    kind: TypedExprKind::Binary {
                        left: Box::new(left),
                        op: bin.op,
                        right: Box::new(right),
                    },
                    ty,
                })
            }
            Expression::Call(call) => {
                let sig = self.symbols.lookup_function(&call.name).ok_or_else(|| {
                    CompileError::TypeError(alloc::format!("Undefined function: {}", call.name))
                })?;

                let mut typed_args = Vec::new();
                for (arg, (param_name, param_ty)) in call.args.iter().zip(sig.params.iter()) {
                    let typed_arg = self.check_expression(arg)?;
                    if typed_arg.ty != *param_ty {
                        return Err(CompileError::TypeError(
                            alloc::format!(
                                "Argument type mismatch for {}: expected {:?}, got {:?}",
                                param_name, param_ty, typed_arg.ty
                            ),
                        ));
                    }
                    typed_args.push(typed_arg);
                }

                Ok(TypedExpression {
                    kind: TypedExprKind::Call {
                        name: call.name.clone(),
                        args: typed_args,
                    },
                    ty: sig.return_type,
                })
            }
            Expression::AiCall(ai_call) => {
                let typed_prompt = self.check_expression(&ai_call.prompt)?;

                // Verify AI model exists
                if !self.symbols.lookup_ai(&ai_call.model).is_some() {
                    // Allow undefined AI models for now (could be built-in)
                }

                Ok(TypedExpression {
                    kind: TypedExprKind::AiCall {
                        model: ai_call.model.clone(),
                        prompt: Box::new(typed_prompt),
                    },
                    ty: Type::String, // AI calls return strings
                })
            }
        }
    }

    fn infer_binary_type(&self, left: &Type, op: BinaryOp, right: &Type) -> Result<Type, CompileError> {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                // Arithmetic operations
                match (left, right) {
                    (Type::Int, Type::Int) => Ok(Type::Int),
                    (Type::Float, Type::Float) => Ok(Type::Float),
                    (Type::Float, Type::Int) | (Type::Int, Type::Float) => Ok(Type::Float),
                    (Type::String, Type::String) if op == BinaryOp::Add => Ok(Type::String),
                    _ => Err(CompileError::TypeError(
                        alloc::format!("Invalid operands for {:?}: {:?} and {:?}", op, left, right),
                    )),
                }
            }
            BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                // Comparison operations return bool
                Ok(Type::Bool)
            }
            BinaryOp::And | BinaryOp::Or => {
                // Logical operations
                match (left, right) {
                    (Type::Bool, Type::Bool) => Ok(Type::Bool),
                    _ => Err(CompileError::TypeError(
                        alloc::format!("Logical operations require bool operands, got {:?} and {:?}", left, right),
                    )),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_check_literal() {
        let lit = Literal::Int(42);
        let expr = Expression::Literal(lit);

        let mut checker = TypeChecker::new();
        let typed = checker.check_expression(&expr).unwrap();

        assert_eq!(typed.ty, Type::Int);
    }

    #[test]
    fn test_type_check_let_statement() {
        let stmt = Statement::Let(LetStatement {
            name: String::from("x"),
            ty: Some(Type::Int),
            value: Expression::Literal(Literal::Int(42)),
        });

        let mut checker = TypeChecker::new();
        let typed = checker.check_statement(&stmt).unwrap();

        match typed {
            TypedStatement::Let { name, ty, .. } => {
                assert_eq!(name, "x");
                assert_eq!(ty, Type::Int);
            }
            _ => panic!("Expected Let statement"),
        }
    }
}
