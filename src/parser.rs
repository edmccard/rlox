use anyhow::Error;

use crate::scanner::{Scanner, Token, TokenType};
use crate::Result;

pub struct Parser {
    scanner: Scanner,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
}

impl Parser {
    pub fn new(source: String) -> Parser {
        Parser {
            scanner: Scanner::new(source),
            current: Token::default(),
            previous: Token::default(),
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn parse(&mut self) -> bool {
        true
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

    fn advance(&mut self) {
        self.previous = self.current;
        loop {
            match self.scanner.scan_token() {
                Ok(token) => {
                    self.current = token;
                    break;
                }
                Err(e) => self.report_error(self.previous, e),
            }
        }
    }

    pub fn clear_error(&mut self) {
        self.had_error = false;
        self.panic_mode = false;
    }

    fn report_error(&mut self, token: Token, err: Error) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        self.had_error = true;
        eprintln!("error: line {}: {}", token.line(), err);
    }
}
