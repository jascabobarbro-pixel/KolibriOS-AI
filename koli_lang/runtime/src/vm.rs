//! Virtual Machine for Koli Language

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::value::Value;
use super::gc::GarbageCollector;
use super::RuntimeError;

/// Virtual Machine
pub struct VirtualMachine {
    stack: Vec<Value>,
    call_stack: Vec<CallFrame>,
    globals: BTreeMap<String, Value>,
    functions: BTreeMap<String, Function>,
}

/// Call frame
pub struct CallFrame {
    pub return_ip: usize,
    pub base_ptr: usize,
    pub locals: BTreeMap<String, Value>,
}

/// Function representation
pub struct Function {
    pub name: String,
    pub bytecode: Vec<u8>,
    pub arity: usize,
}

impl VirtualMachine {
    /// Create a new VM
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(1024),
            call_stack: Vec::new(),
            globals: BTreeMap::new(),
            functions: BTreeMap::new(),
        }
    }

    /// Run bytecode
    pub fn run(&mut self, bytecode: &[u8], gc: &mut GarbageCollector) -> Result<Value, RuntimeError> {
        let mut ip = 0;

        while ip < bytecode.len() {
            let opcode = bytecode[ip];
            ip += 1;

            match opcode {
                OP_CONST => {
                    let idx = bytecode[ip] as usize;
                    ip += 1;
                    // Load constant at index
                }
                OP_ADD => {
                    let b = self.stack.pop().ok_or(RuntimeError::StackOverflow)?;
                    let a = self.stack.pop().ok_or(RuntimeError::StackOverflow)?;
                    // Push a + b
                }
                OP_SUB => {
                    let b = self.stack.pop().ok_or(RuntimeError::StackOverflow)?;
                    let a = self.stack.pop().ok_or(RuntimeError::StackOverflow)?;
                    // Push a - b
                }
                OP_MUL => {
                    let b = self.stack.pop().ok_or(RuntimeError::StackOverflow)?;
                    let a = self.stack.pop().ok_or(RuntimeError::StackOverflow)?;
                    // Push a * b
                }
                OP_DIV => {
                    let b = self.stack.pop().ok_or(RuntimeError::StackOverflow)?;
                    let a = self.stack.pop().ok_or(RuntimeError::StackOverflow)?;
                    // Push a / b
                }
                OP_CALL => {
                    let argc = bytecode[ip] as usize;
                    ip += 1;
                    // Call function with argc arguments
                }
                OP_RETURN => {
                    if let Some(frame) = self.call_stack.pop() {
                        ip = frame.return_ip;
                    } else {
                        break;
                    }
                }
                OP_HALT => break,
                _ => return Err(RuntimeError::ExecutionError(
                    String::from("Unknown opcode")
                )),
            }
        }

        self.stack.pop().ok_or(RuntimeError::ExecutionError(
            String::from("No value on stack")
        ))
    }

    /// Call a function by name
    pub fn call(&mut self, name: &str, args: &[Value], gc: &mut GarbageCollector) -> Result<Value, RuntimeError> {
        if let Some(func) = self.functions.get(name) {
            if args.len() != func.arity {
                return Err(RuntimeError::TypeError(
                    alloc::format!("Expected {} arguments, got {}", func.arity, args.len())
                ));
            }

            // Push arguments to stack
            for arg in args {
                self.stack.push(arg.clone());
            }

            // Create call frame
            self.call_stack.push(CallFrame {
                return_ip: 0,
                base_ptr: self.stack.len() - args.len(),
                locals: BTreeMap::new(),
            });

            // Run function bytecode
            self.run(&func.bytecode, gc)
        } else {
            Err(RuntimeError::FunctionNotFound(String::from(name)))
        }
    }

    /// Register a function
    pub fn register_function(&mut self, func: Function) {
        self.functions.insert(func.name.clone(), func);
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}

// Opcodes
pub const OP_CONST: u8 = 0x01;
pub const OP_ADD: u8 = 0x02;
pub const OP_SUB: u8 = 0x03;
pub const OP_MUL: u8 = 0x04;
pub const OP_DIV: u8 = 0x05;
pub const OP_CALL: u8 = 0x06;
pub const OP_RETURN: u8 = 0x07;
pub const OP_HALT: u8 = 0x08;
