use std::env;
use std::io::{stdin, stdout, BufRead, Write};
use std::process::exit;

use rlox::{Parser, Result, Vm};

fn main() -> Result<()> {
    let mut vm = Vm::init();
    let args: Vec<String> = env::args().collect();
    if args.len() != 1 {
        eprintln!("Usage: scandemo");
        exit(1);
    }

    let mut lines = stdin().lock().lines();
    let mut line_no = 1;
    let mut source: Vec<String> = Vec::new();
    loop {
        print!("{:4}> ", line_no);
        stdout().flush()?;
        let mut line = match lines.next() {
            None => break,
            Some(line) => line?,
        };
        line_no += 1;
        if line.ends_with('\\') {
            line.pop();
            source.push(line);
            continue;
        } else {
            source.push(line);
            let mut parser = Parser::new(source.join("\n"), &mut vm);
            source.clear();
            parser.show_tokens();
            parser.clear_error();
        }
    }

    Ok(())
}
