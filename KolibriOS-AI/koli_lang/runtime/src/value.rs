//! Values for Koli Language Runtime

use alloc::string::String;
use alloc::vec::Vec;

/// Koli value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Integer value
    Int(i64),
    /// Floating point value
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// String value
    String(String),
    /// Array value
    Array(Vec<Value>),
    /// Object/dictionary value
    Object(alloc::collections::BTreeMap<String, Value>),
    /// Nil value
    Nil,
    /// Function reference
    Function(FunctionRef),
    /// AI model reference
    AiModel(String),
}

/// Function reference
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionRef {
    pub name: String,
    pub arity: usize,
}

impl Value {
    /// Check if value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Nil => false,
            _ => true,
        }
    }

    /// Get type name
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Nil => "nil",
            Value::Function(_) => "function",
            Value::AiModel(_) => "ai_model",
        }
    }

    /// Convert to string representation
    pub fn to_string_repr(&self) -> String {
        match self {
            Value::Int(n) => alloc::format!("{}", n),
            Value::Float(f) => alloc::format!("{}", f),
            Value::Bool(b) => alloc::format!("{}", b),
            Value::String(s) => s.clone(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string_repr()).collect();
                alloc::format!("[{}]", items.join(", "))
            }
            Value::Object(obj) => {
                let items: Vec<String> = obj.iter()
                    .map(|(k, v)| alloc::format!("{}: {}", k, v.to_string_repr()))
                    .collect();
                alloc::format!("{{{}}}", items.join(", "))
            }
            Value::Nil => String::from("nil"),
            Value::Function(f) => alloc::format!("<function {}>", f.name),
            Value::AiModel(m) => alloc::format!("<ai_model {}>", m),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Nil
    }
}
