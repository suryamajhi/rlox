use crate::function::{Function, NativeFunction};
use std::fmt;
use std::fmt::Formatter;
use crate::class::{Class, ClassInstance, ClassInstanceRef};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(String),
    Function(Function),
    NativeFunction(NativeFunction),
    Class(Class),
    ClassInstance(ClassInstanceRef),
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Value::Boolean(value) => value.to_string(),
            Value::Number(value) => value.to_string(),
            Value::String(value) => value.to_string(),
            Value::Nil => String::from("nil"),
            Value::Function(func) => format!("{}", func),
            Value::NativeFunction(_) => "<native fn>".to_string(),
            Value::Class(class) => format!("{}", class),
            Value::ClassInstance(instance) => format!("{}", instance.borrow().to_string()),
        };
        write!(f, "{}", s)
    }
}
