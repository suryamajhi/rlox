mod parser;
mod scanner;
mod token;

use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::token::Token;
use std::{fs, io, process};

static mut HAD_RUNTIME_ERROR: bool = false;

struct RuntimeError {
    token: Token,
    message: String,
}
pub enum Exception {
    RuntimeError(RuntimeError),
}

impl Exception {
    fn runtime_error<T>(token: Token, message: String) -> Result<T, Exception> {
        Err(Exception::RuntimeError(RuntimeError { token, message }))
    }
}

fn check_runtime_error() {
    unsafe {
        if HAD_RUNTIME_ERROR {
            process::exit(70)
        }
    }
}

pub fn print_error(line: usize, location: String, message: &str) {
    eprintln!("[line {line}] Error at '{location}': {message}");
    unsafe { HAD_RUNTIME_ERROR = true }
}

pub fn run_prompt() {
    loop {
        println!("> ");
        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Valid user input");

        let user_input = user_input.trim();
        if user_input == "exit" {
            break;
        }
        run(user_input.to_string());
        unsafe {
            HAD_RUNTIME_ERROR = false;
        }
    }
}

pub fn run_file(path: &str) {
    let file_contents = fs::read_to_string(path).expect("Could not read file");
    run(file_contents);
    unsafe {
        if HAD_RUNTIME_ERROR {
            process::exit(70);
        }
    }
}

fn run(source: String) {
    let mut tokens: Vec<Token> = Vec::new();
    let mut scanner = Scanner::new(source, &mut tokens);
    scanner.scan_tokens();
    let mut parser = Parser::new(&mut tokens);

    for token in tokens {
        println!("{}", token);
    }
}
