use anyhow::Error;
use num_enum::UnsafeFromPrimitive;

use crate::code::{Chunk, Op};
use crate::scanner::{Scanner, Token, TokenType};
use crate::{Result, Value, Vm};

#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    UnsafeFromPrimitive
)]
#[repr(u32)]
enum Prec {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Prec {
    fn next(self) -> Self {
        unsafe { Prec::from_unchecked(self as u32 + 1) }
    }

    fn for_op_type(ty: TokenType) -> Self {
        match ty {
            TokenType::Minus | TokenType::Plus => Prec::Term,
            TokenType::Slash | TokenType::Star => Prec::Factor,
            TokenType::BangEqual | TokenType::EqualEqual => Prec::Equality,
            TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => Prec::Comparison,
            _ => Prec::None,
        }
    }
}

pub struct Parser {
    scanner: Scanner,
    code: Vec<Chunk>,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
}

impl Parser {
    pub fn new(source: String) -> Parser {
        Parser {
            scanner: Scanner::new(source),
            code: Vec::new(),
            current: Token::default(),
            previous: Token::default(),
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn parse(&mut self, vm: &mut Vm) -> bool {
        self.code.push(Chunk::new());

        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "expect end of expression");

        self.emit_op(Op::Return);

        let had_error = self.had_error;
        let chunk = self.chunk();

        if !had_error {
            #[cfg(feature = "print_code")]
            chunk.disassemble("<script>");
            vm.run(chunk).unwrap();
        }

        !self.had_error
    }

    #[cfg(debug_assertions)]
    pub fn show_tokens(&mut self) {
        let mut line: u32 = 0;
        loop {
            self.advance();
            if self.had_error {
                break;
            }
            let token = self.current;
            if token.line() != line {
                line = token.line();
                print!("{:4} ", line);
            } else {
                print!("   | ");
            }
            println!("{:12} {}", token.ty(), self.scanner.token_text(token));
            if token.ty() == TokenType::Eof {
                break;
            }
        }
    }

    #[cfg(feature = "bench_mode")]
    pub fn bench(&mut self) -> Result<()> {
        let mut b1 = 0usize;
        let mut b2 = 0usize;
        let mut b3 = 0usize;
        let mut b4 = 0usize;
        loop {
            let token = self.scanner.scan_token()?;
            b1 += token.ty() as u8 as usize;
            b2 += token.start();
            b3 += token.end();
            b4 += token.line() as usize;

            if token.ty() == TokenType::Eof {
                break;
            }
        }
        println!("{} {} {} {}", b1, b2, b3, b4);

        Ok(())
    }

    fn chunk(&mut self) -> &mut Chunk {
        &mut self.code[0]
    }

    fn advance(&mut self) {
        self.previous = self.current;
        loop {
            match self.scanner.scan_token() {
                Ok(token) => {
                    self.current = token;
                    let line = self.current.line();
                    if line != self.previous.line() {
                        self.chunk().new_line(line);
                    }
                    break;
                }
                Err(e) => self.scan_error(e),
            }
        }
    }

    fn consume(&mut self, ty: TokenType, msg: &str) {
        if self.current.ty() == ty {
            self.advance();
        } else {
            self.error_at(self.current, msg)
        }
    }

    fn parse_precedence(&mut self, precedence: Prec) {
        self.advance();

        match self.previous.ty() {
            TokenType::LeftParen => self.grouping(),
            TokenType::Minus | TokenType::Bang => self.unary(),
            TokenType::Number => self.number(),
            TokenType::Nil | TokenType::True | TokenType::False => {
                self.literal()
            }
            _ => {
                self.error("expect expression");
                return;
            }
        }

        while precedence <= Prec::for_op_type(self.current.ty()) {
            self.advance();
            match self.previous.ty() {
                TokenType::Minus
                | TokenType::Plus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::EqualEqual
                | TokenType::Greater
                | TokenType::GreaterEqual
                | TokenType::Less
                | TokenType::LessEqual => self.binary(),
                _ => unreachable!(),
            }
        }
    }

    fn number(&mut self) {
        let value = self
            .scanner
            .token_text(self.previous)
            .parse::<f64>()
            .unwrap();
        self.emit_constant(value);
    }

    fn literal(&mut self) {
        let op = match self.previous.ty() {
            TokenType::Nil => Op::Nil,
            TokenType::True => Op::True,
            TokenType::False => Op::False,
            _ => unreachable!(),
        };
        self.emit_op(op);
    }

    fn expression(&mut self) {
        self.parse_precedence(Prec::Assignment);
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "expect ')' after expression");
    }

    fn unary(&mut self) {
        let operator_type = self.previous.ty();

        self.parse_precedence(Prec::Unary);

        match operator_type {
            TokenType::Minus => self.emit_op(Op::Negate),
            TokenType::Bang => self.emit_op(Op::Not),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self) {
        let operator_type = self.previous.ty();
        self.parse_precedence(Prec::for_op_type(operator_type).next());

        match operator_type {
            TokenType::Plus => self.emit_op(Op::Add),
            TokenType::Minus => self.emit_op(Op::Subtract),
            TokenType::Star => self.emit_op(Op::Multiply),
            TokenType::Slash => self.emit_op(Op::Divide),
            TokenType::EqualEqual => self.emit_op(Op::Equal),
            TokenType::Less => self.emit_op(Op::Less),
            TokenType::Greater => self.emit_op(Op::Greater),
            TokenType::BangEqual => {
                self.emit_op(Op::Equal);
                self.emit_op(Op::Not);
            }
            TokenType::GreaterEqual => {
                self.emit_op(Op::Less);
                self.emit_op(Op::Not);
            }
            TokenType::LessEqual => {
                self.emit_op(Op::Greater);
                self.emit_op(Op::Not);
            }
            _ => unreachable!(),
        }
    }

    fn emit_op(&mut self, op: Op) {
        self.chunk().write_op(op);
    }

    fn emit_constant(&mut self, value: f64) {
        let chunk = self.chunk();
        let arg = match chunk.add_constant(Value::Number(value)) {
            Ok(idx) => idx,
            Err(e) => {
                self.error(&e.to_string());
                return;
            }
        };
        chunk.write_op_arg(Op::Constant, arg);
    }

    pub fn clear_error(&mut self) {
        self.had_error = false;
        self.panic_mode = false;
    }

    fn scan_error(&mut self, err: Error) {
        self.report_error(self.previous.line(), format!(": {}", err));
    }

    fn error(&mut self, msg: &str) {
        self.error_at(self.previous, msg);
    }

    fn error_at(&mut self, token: Token, msg: &str) {
        let msg = match token.ty() {
            TokenType::Eof => format!(" at end: {}", msg),
            _ => format!(" at '{}': {}", self.scanner.token_text(token), msg),
        };
        self.report_error(token.line(), msg);
    }

    fn report_error(&mut self, line: u32, msg: String) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        self.had_error = true;
        eprintln!("[line {}] Error{}", line, msg);
    }
}
