use std::env;
use std::io::{stdin, stdout, BufRead, Write};
use std::process::exit;

use rlox::{Parser, Result, Vm};

fn main() -> Result<()> {
    let mut vm = Vm::init();
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => repl(&mut vm)?,
        2 => {
            let source = std::fs::read_to_string(&args[1])?;
            let mut parser = Parser::new(source);
            parser.parse(&mut vm);
        }
        _ => {
            eprintln!("Usage: rlox [path]");
            exit(1);
        }
    }
    Ok(())
}

fn repl(vm: &mut Vm) -> Result<()> {
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
            let mut parser = Parser::new(source.join("\n"));
            source.clear();
            parser.parse(vm);
            parser.clear_error();
        }
    }
    Ok(())
}
