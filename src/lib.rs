mod code;
mod parser;
mod scanner;
mod vm;

pub use anyhow::Result;
pub use parser::Parser;

#[derive(Debug, thiserror::Error)]
#[error("{}", .msg)]
pub struct Error {
    msg: String,
}

impl Error {
    fn new(msg: String) -> Self {
        Error { msg }
    }

    fn with_line(&self, line: u32) -> Self {
        Error {
            msg: format!("line {}: {}", line, self.msg),
        }
    }
}
