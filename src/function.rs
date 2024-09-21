use crate::environment::{EnvRef, Environment};
use crate::interpreter::Interpreter;
use crate::stmt::Stmt;
use crate::value::Value;
use crate::Exception;
use std::fmt;
use std::fmt::Formatter;

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Exception>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeFunction {
    pub arity: usize,
    pub callable: fn(&mut Interpreter, Vec<Value>) -> Value,
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Exception> {
        Ok((self.callable)(interpreter, args))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    declaration: Stmt,
    closure: EnvRef,
}

impl Function {
    pub fn new(declaration: Stmt, closure: EnvRef) -> Self {
        Function {
            declaration,
            closure,
        }
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        if let Stmt::Function { params, .. } = &self.declaration {
            return params.len();
        }
        panic!("Function was not initialized with a function declaration!");
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Exception> {
        let environment = Environment::new_local(&self.closure);

        if let Stmt::Function { params, body, .. } = &self.declaration {
            for (i, param) in params.iter().enumerate() {
                environment
                    .borrow_mut()
                    .define(param.lexeme.clone(), args.get(i).unwrap().clone());
            }
            if let Err(exception) = interpreter.execute_block(body, environment) {
                return match exception {
                    Exception::RuntimeError(e) => Err(Exception::RuntimeError(e)),
                    Exception::Return(value) => Ok(value),
                };
            }
        }
        Ok(Value::Nil)
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut string = String::from("unknown");
        if let Stmt::Function { name, .. } = &self.declaration {
            string = String::from(name.lexeme.clone());
        }
        write!(f, "<fn {}>", string)
    }
}
