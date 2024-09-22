use crate::function::{Callable, Function};
use crate::interpreter::Interpreter;
use crate::token::Token;
use crate::value::Value;
use crate::{Exception, RuntimeError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{write, Formatter};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    name: String,
    super_class: Option<Box<Class>>,
    methods: HashMap<String, Function>,
}

impl Class {
    pub fn new(
        name: String,
        super_class: Option<Box<Class>>,
        methods: HashMap<String, Function>,
    ) -> Self {
        Class {
            name,
            super_class,
            methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<Value> {
        self.methods
            .get(name)
            .map(|method| Value::Function(method.clone()))
            .or(self
                .super_class
                .as_ref()
                .and_then(|super_class| super_class.find_method(name)))
    }
}

impl Callable for Class {
    fn arity(&self) -> usize {
        if let Some(initializer) = self.find_method("init") {
            match initializer {
                Value::Function(initializer) => return initializer.arity(),
                _ => panic!("initializer is not a function!"),
            }
        }
        0
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, Exception> {
        let instance = ClassInstance::new(self.clone());

        if let Some(initializer) = self.find_method("init") {
            match initializer {
                Value::Function(mut initializer) => {
                    let _ = initializer.bind(instance.clone()).call(interpreter, args);
                }
                _ => panic!("Initializer is not a function"),
            }
        }

        Ok(Value::ClassInstance(instance))
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub type ClassInstanceRef = Rc<RefCell<ClassInstance>>;

#[derive(Clone, Debug, PartialEq)]
pub struct ClassInstance {
    class: Class,
    fields: HashMap<String, Value>,
}

impl ClassInstance {
    pub fn new(class: Class) -> ClassInstanceRef {
        Rc::new(RefCell::new(ClassInstance {
            class,
            fields: HashMap::new(),
        }))
    }

    pub fn get(&self, name: &Token, instance_ref: ClassInstanceRef) -> Result<Value, Exception> {
        if let Some(val) = self.fields.get(&name.lexeme) {
            return Ok(val.clone());
        }

        if let Some(Value::Function(mut method)) = self.class.find_method(&name.lexeme) {
            let bound = method.bind(instance_ref.clone());
            return Ok(Value::Function(bound));
        }

        Err(Exception::RuntimeError(RuntimeError {
            token: name.clone(),
            message: format!("Undefined property '{}'", name.lexeme),
        }))
    }

    pub fn set(&mut self, name: &Token, value: Value) {
        self.fields.insert(name.lexeme.clone(), value);
    }
}

impl fmt::Display for ClassInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}
