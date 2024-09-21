use std::{fs, io, process};

use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::stmt::Stmt;
use crate::token::Token;
use crate::value::Value;

mod environment;
mod expr;
mod function;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
mod token;
mod utils;
mod value;

static mut HAD_RUNTIME_ERROR: bool = false;

pub struct RuntimeError {
    token: Token,
    message: String,
}
pub enum Exception {
    RuntimeError(RuntimeError),
    Return(Value),
}

impl Exception {
    fn runtime_error<T>(token: Token, message: String) -> Result<T, Exception> {
        Err(Exception::RuntimeError(RuntimeError { token, message }))
    }
}

impl RuntimeError {
    fn error(&self) {
        eprintln!("{}", self.message);
        eprintln!("[line {}]", self.token.line);

        unsafe {
            HAD_RUNTIME_ERROR = true;
        }
    }
}

fn runtime_error() -> bool {
    unsafe { HAD_RUNTIME_ERROR }
}

fn check_runtime_error() {
    unsafe {
        if HAD_RUNTIME_ERROR {
            process::exit(70)
        }
    }
}

pub fn print_error(line: usize, location: &str, message: &str) {
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

    if runtime_error() {
        process::exit(64);
    }
    let mut parser = Parser::new(&mut tokens);
    let stmts: Vec<Stmt> = parser.parse();
    let mut interpreter = Interpreter::new();
    interpreter.interpret(&stmts);
}
