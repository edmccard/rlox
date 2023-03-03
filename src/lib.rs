use std::fmt;

mod code;
mod parser;
mod scanner;
mod vm;

pub use anyhow::Result;
pub use parser::Parser;
pub use vm::Vm;

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
}

impl Value {
    const TRUE: Value = Value::Boolean(true);
    const FALSE: Value = Value::Boolean(false);
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(v) => write!(f, "{}", v),
            Value::Number(v) => write!(f, "{}", v),
        }
    }
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        !matches!(value, Value::Nil | Value::Boolean(false))
    }
}
