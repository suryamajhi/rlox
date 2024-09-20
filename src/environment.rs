use crate::token::Token;
use crate::value::Value;
use crate::Exception;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn new_local(enclosing: Environment) -> Self {
        Environment {
            enclosing: Some(Box::new(enclosing)),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Value, Exception> {
        if let Some(value) = self.values.get(&name.lexeme) {
            return Ok(value.clone());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.clone().get(name);
        }

        Exception::runtime_error(name.clone(), format!("Undefined variable {}.", name.lexeme))
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<(), Exception> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            return Ok(());
        }

        if let Some(ref mut enclosing) = self.enclosing {
            enclosing.assign(name, value)?;
            return Ok(());
        }

        Exception::runtime_error(name.clone(), format!("Undefined variable {}.", name.lexeme))
    }
}
