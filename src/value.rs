use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(String),
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Value::Boolean(value) => value.to_string(),
            Value::Number(value) => value.to_string(),
            Value::String(value) => value.to_string(),
            Value::Nil => String::from("nil"),
        };
        write!(f, "{}", s)
    }
}
