use crate::code::{Chunk, Op, Value};
use crate::Error;

type Result<T> = std::result::Result<T, Error>;

pub struct Vm {
    stack: Vec<Value>,
    ip: usize,
}

impl Vm {
    const MAX_STACK: usize = 1024;

    pub fn init() -> Self {
        Vm {
            ip: 0,
            stack: Vec::new(),
        }
    }

    pub fn run(&mut self, chunk: &Chunk) -> Result<()> {
        let mut ip = chunk.instructions(self.ip);
        while let Some(inst) = ip.next() {
            #[cfg(feature = "trace_execution")]
            {
                self.trace_stack();
                chunk.disassemble_instruction(inst, ip.offset - inst.len());
            }

            let result = match inst.opcode() {
                Op::Return => {
                    println!("{}", self.pop());
                    break;
                }
                Op::Negate => {
                    let arg = self.pop();
                    self.push(-arg)
                }
                Op::Add => {
                    let (a, b) = self.binary_args();
                    self.push(a + b)
                }
                Op::Subtract => {
                    let (a, b) = self.binary_args();
                    self.push(a - b)
                }
                Op::Multiply => {
                    let (a, b) = self.binary_args();
                    self.push(a * b)
                }
                Op::Divide => {
                    let (a, b) = self.binary_args();
                    self.push(a / b)
                }
                Op::Constant => {
                    let constant = chunk.get_constant(inst.operand());
                    self.push(constant)
                }
                _ => Err(Error::new("unknown opcode".to_string())),
            };
            result.map_err(|e| {
                let offset = ip.offset - inst.len();
                let line = chunk.get_line(offset);
                e.with_line(line)
            })?;
        }
        self.ip = ip.offset;
        Ok(())
    }

    fn push(&mut self, val: Value) -> Result<()> {
        if self.stack.len() < Vm::MAX_STACK {
            self.stack.push(val);
            Ok(())
        } else {
            Err(Error::new("stack overflow".to_string()))
        }
    }

    fn pop(&mut self) -> Value {
        assert!(!self.stack.is_empty());
        self.stack.pop().unwrap()
    }

    fn binary_args(&mut self) -> (Value, Value) {
        let b = self.pop();
        let a = self.pop();
        (a, b)
    }
}

#[cfg(feature = "trace_execution")]
impl Vm {
    fn trace_stack(&self) {
        print!("          ");
        for elem in &self.stack {
            print!("[ {} ]", elem);
        }
        println!();
    }
}
