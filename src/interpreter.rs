use crate::expr::Expr;
use crate::token::{Literal, Token, TokenType};
use crate::value::Value;
use crate::{expr, Exception};

type Result<T> = std::result::Result<T, Exception>;

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }

    pub fn evaluate(&self, expr: &Expr) -> Result<Value> {
        expr::Visitor::visit_expr(self, expr)
    }

    fn visit_literal_expr(&self, literal: &Literal) -> Value {
        match literal {
            Literal::String(value) => Value::String(value.to_string()),
            Literal::Number(value) => Value::Number(*value),
            Literal::Bool(value) => Value::Boolean(*value),
            Literal::None => Value::Nil,
        }
    }

    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> Result<Value> {
        let right = self.evaluate(right)?;
        match operator.token_type {
            TokenType::BANG => Ok(Value::Boolean(!Interpreter::is_truthy(&right))),
            TokenType::MINUS => match right {
                Value::Number(value) => Ok(Value::Number(-value)),
                _ => Interpreter::number_operand_error(operator),
            },
            _ => Interpreter::number_operand_error(operator),
        }
    }

    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> Result<Value> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match operator.token_type {
            // Arithmetic Binary Operations
            TokenType::MINUS => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left - right)),
                _ => Interpreter::number_operand_error(operator),
            },
            TokenType::PLUS => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
                (Value::String(left), Value::String(right)) => {
                    Ok(Value::String(format!("{}{}", left, right)))
                }
                (Value::String(left), Value::Number(right)) => {
                    Ok(Value::String(format!("{}{}", left, right)))
                }
                (Value::Number(left), Value::String(right)) => {
                    Ok(Value::String(format!("{}{}", left, right)))
                }
                _ => Interpreter::number_operand_error(operator),
            },
            TokenType::SLASH => match (left, right) {
                (Value::Number(left), Value::Number(right)) => match right {
                    0f64 => Exception::runtime_error(
                        operator.clone(),
                        String::from("Cannot divide by zero"),
                    ),
                    _ => Ok(Value::Number(left / right)),
                },
                _ => Interpreter::number_operand_error(operator),
            },
            TokenType::STAR => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left * right)),
                _ => Interpreter::number_operand_error(operator),
            },

            // Comparisons
            TokenType::GREATER => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left > right)),
                _ => Interpreter::number_operand_error(operator),
            },
            TokenType::GREATER_EQUAL => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left >= right)),
                _ => Interpreter::number_operand_error(operator),
            },
            TokenType::LESS => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left < right)),
                _ => Interpreter::number_operand_error(operator),
            },
            TokenType::LESS_EQUAL => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left <= right)),
                _ => Interpreter::number_operand_error(operator),
            },
            TokenType::BANG_EQUAL => Ok(Value::Boolean(!Interpreter::is_equal(&left, &right))),
            TokenType::EQUAL_EQUAL => Ok(Value::Boolean(Interpreter::is_equal(&left, &right))),

            _ => panic!("unexpected operator for binary expression"),
        }
    }

    fn is_equal(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Nil, Value::Nil) => true,
            (Value::Boolean(left), Value::Boolean(right)) => left == right,
            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            _ => false,
        }
    }

    fn is_truthy(value: &Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Boolean(value) => *value,
            _ => true,
        }
    }

    fn number_operand_error<T>(operator: &Token) -> Result<T> {
        Exception::runtime_error(operator.clone(), String::from("Operands must be a number"))
    }
}

impl expr::Visitor<Result<Value>> for Interpreter {
    fn visit_expr(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal { value } => Ok(self.visit_literal_expr(value)),
            Expr::Unary { operator, right } => self.visit_unary_expr(operator, right),
            Expr::Grouping { expr } => self.evaluate(expr),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary_expr(left, operator, right),
            Expr::Var { .. } => Ok(Value::Nil),
            Expr::Assign { .. } => Ok(Value::Nil),
        }
    }
}
