use crate::token::Token;
use crate::value::Value;
use crate::Exception;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    values: HashMap<String, Value>,
    pub enclosing: Option<EnvRef>,
}

impl Environment {
    pub fn new() -> EnvRef {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            enclosing: None,
        }))
    }

    pub fn new_local(enclosing: &EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Environment {
            enclosing: Some(enclosing.clone()),
            values: HashMap::new(),
        }))
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get_at(&self, distance: usize, name: &str) -> Result<Value, Exception> {
        if distance == 0 {
            return Ok(self.values.get(name).unwrap().clone());
        }
        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get_at(distance - 1, name);
        }
        panic!("Could not find local scope that variable belongs to.")
    }

    pub fn get(&self, name: &Token) -> Result<Value, Exception> {
        if let Some(value) = self.values.get(&name.lexeme) {
            return Ok(value.clone());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name);
        }

        Exception::runtime_error(name.clone(), format!("Undefined variable {}.", name.lexeme))
    }

    pub fn assign_at(&mut self, distance: usize, name: &Token, value: &Value) {
        if distance == 0 {
            self.values.insert(name.lexeme.clone(), value.clone());
            return;
        }

        if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign_at(distance - 1, name, value);
            return;
        }

        panic!("Could not find local scope that variable belongs to!")
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<(), Exception> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            return Ok(());
        }

        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }

        Exception::runtime_error(name.clone(), format!("Undefined variable {}.", name.lexeme))
    }
}
