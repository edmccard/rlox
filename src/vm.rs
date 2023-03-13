use std::cell::RefCell;
use std::fmt;
use std::rc::{Rc, Weak};

use crate::code::{Chunk, Op};
use crate::parser::Parser;
use crate::Value;

pub(crate) type Obj = Rc<RefCell<Object>>;
type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(PartialEq, PartialOrd)]
pub(crate) struct Object {
    payload: Payload,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.payload.fmt(f)
    }
}

#[derive(PartialEq, PartialOrd)]
enum Payload {
    String(Box<str>),
}

impl fmt::Display for Payload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Payload::String(v) => write!(f, "\"{}\"", v),
        }
    }
}

pub struct Vm {
    stack: Vec<Value>,
    heap: Vec<Weak<RefCell<Object>>>,
}

impl Vm {
    const MAX_STACK: usize = 1024;

    pub fn init() -> Self {
        Vm {
            stack: Vec::new(),
            heap: Vec::new(),
        }
    }

    fn error(msg: &str) -> Result<()> {
        Err(RuntimeError::new(msg.to_string()))
    }

    pub fn interpret(&mut self, source: String) -> Result<()> {
        let mut parser = Parser::new(source);
        match parser.parse(self) {
            Some(chunk) => self.run(&chunk),
            None => Ok(()),
        }
    }

    fn run(&mut self, chunk: &Chunk) -> Result<()> {
        let mut ip = chunk.instructions();
        while let Some(inst) = ip.next() {
            #[cfg(feature = "trace_execution")]
            {
                self.trace_stack();
                chunk.disassemble_instruction(inst, ip.offset - inst.len());
            }

            let result = match inst.opcode() {
                Op::Nil => self.push(Value::Nil),
                Op::True => self.push(Value::TRUE),
                Op::False => self.push(Value::FALSE),
                Op::Return => {
                    println!("{}", self.pop());
                    break;
                }
                Op::Not => {
                    let arg = bool::from(self.pop());
                    self.push(Value::Boolean(!arg))
                }
                Op::Negate => {
                    let arg = self.pop();
                    match arg {
                        Value::Number(v) => self.push(Value::Number(-v)),
                        _ => Vm::error("operand must be a number"),
                    }
                }
                Op::Equal => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(Value::Boolean(a == b))
                }
                Op::Greater => {
                    let (a, b) = self.arithmetic_args()?;
                    self.push(Value::Boolean(a > b))
                }
                Op::Less => {
                    let (a, b) = self.arithmetic_args()?;
                    self.push(Value::Boolean(a < b))
                }
                Op::Add => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.push(Value::Number(a + b))
                        }
                        (Value::Object(a), Value::Object(b)) => {
                            self.add_objects(a, b)
                        }
                        _ => Err(RuntimeError::new(
                            "operands must be numbers or strings".to_string(),
                        )),
                    }
                }
                Op::Subtract => {
                    let (a, b) = self.arithmetic_args()?;
                    self.push(Value::Number(a - b))
                }
                Op::Multiply => {
                    let (a, b) = self.arithmetic_args()?;
                    self.push(Value::Number(a * b))
                }
                Op::Divide => {
                    let (a, b) = self.arithmetic_args()?;
                    self.push(Value::Number(a / b))
                }
                Op::Constant => {
                    let constant = chunk.get_constant(inst.operand());
                    self.push(constant)
                }
                _ => Vm::error("unknown opcode"),
            };
            result.map_err(|e| {
                let offset = ip.offset - inst.len();
                let line = chunk.get_line(offset);
                self.stack.clear();
                e.with_line(line)
            })?;
        }

        Ok(())
    }

    pub(crate) fn new_string(&mut self, text: &str) -> Value {
        let object = Object {
            payload: Payload::String(Box::from(text)),
        };
        let obj = Rc::new(RefCell::new(object));
        self.heap.push(Rc::downgrade(&obj));
        Value::Object(obj)
    }

    fn add_objects(&mut self, a: Obj, b: Obj) -> Result<()> {
        match (&a.borrow().payload, &b.borrow().payload) {
            (Payload::String(a), Payload::String(b)) => {
                let value = self.new_string(&[a.as_ref(), b.as_ref()].concat());
                self.push(value)
            }
            _ => Err(RuntimeError::new(
                "operands must be numbers or strings".to_string(),
            )),
        }
    }

    fn push(&mut self, val: Value) -> Result<()> {
        if self.stack.len() < Vm::MAX_STACK {
            self.stack.push(val);
            Ok(())
        } else {
            Vm::error("stack overflow")
        }
    }

    fn pop(&mut self) -> Value {
        assert!(!self.stack.is_empty());
        self.stack.pop().unwrap()
    }

    fn arithmetic_args(&mut self) -> Result<(f64, f64)> {
        let b = self.pop();
        let a = self.pop();
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => Ok((a, b)),
            _ => Err(RuntimeError::new("operands must be numbers".to_string())),
        }
    }

    #[cfg(feature = "trace_execution")]
    fn trace_stack(&self) {
        print!("          ");
        for elem in &self.stack {
            print!("[ {} ]", elem);
        }
        println!();
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{}", .msg)]
pub struct RuntimeError {
    msg: String,
}

impl RuntimeError {
    fn new(msg: String) -> Self {
        RuntimeError { msg }
    }

    fn with_line(&self, line: u32) -> Self {
        RuntimeError {
            msg: format!("[line {}] {}", line, self.msg),
        }
    }
}
