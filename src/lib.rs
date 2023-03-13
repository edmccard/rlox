use std::{fmt, rc::Rc};

pub use anyhow::Result;
pub use parser::Parser;
pub use vm::Vm;

mod code;
mod parser;
mod scanner;
mod vm;

#[derive(Clone, Default, PartialEq, PartialOrd)]
pub(crate) enum Value {
    #[default]
    Nil,
    Boolean(bool),
    Number(f64),
    String(Rc<str>),
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
            Value::String(v) => write!(f, "{}", v),
        }
    }
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        !matches!(value, Value::Nil | Value::Boolean(false))
    }
}
