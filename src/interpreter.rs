use std::process;
use crate::expr::Expr;
use crate::token::{Literal, Token, TokenType};
use crate::value::Value;
use crate::{expr, Exception, stmt};
use crate::environment::Environment;
use crate::stmt::Stmt;

type Result<T> = std::result::Result<T, Exception>;

pub struct Interpreter {
    environment: Environment
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new()
        }
    }

    pub fn interpret(&mut self, stmts: &Vec<Stmt>) {
        for stmt in stmts {
            match self.execute(stmt) {
                Ok(_) => {}
                Err(e) => match e { Exception::RuntimeError(e) => {
                    e.error();
                    process::exit(70);
                } }
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        stmt::Visitor::visit_stmt(self, stmt)
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value> {
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

    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> Result<Value> {
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

    fn visit_binary_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<Value> {
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

    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<()> {
        self.evaluate(expr).map(|_| ())
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<()> {
        let res = self.evaluate(expr)?;
        println!("{}", res);
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> Result<()> {
        let mut value = Value::Nil;
        if let Some(expr) = initializer {
            value = self.evaluate(expr)?;
        }
        self.environment.define(name.lexeme.clone(), value);
        Ok(())
    }

    fn lookup_variable(&self, name: &Token) -> Result<Value> {
        self.environment.get(name)
    }
    fn visit_var_expr(&self, name: &Token) -> Result<Value> {
        self.lookup_variable(name)
    }

    fn visit_assign_expr(&mut self, name: &Token, expr: &Expr) -> Result<Value> {
        let value = self.evaluate(expr)?;
        self.environment.assign(name, value.clone())?;
        Ok(value)
    }

    fn visit_block_stmt(&mut self, stms: &Vec<Stmt>) -> Result<()> {
        self.execute_block(stms, Environment::new_local(self.environment.clone()))
    }

    fn execute_block(&mut self, stmts: &Vec<Stmt>, environment: Environment) -> Result<()> {
        let previous = self.environment.clone();
        self.environment = environment;
        for stmt in stmts {
            if let Err(e) = self.execute(stmt) {
                self.environment = previous;
                return Err(e);
            }
        }
        self.environment = previous;
        Ok(())
    }
}

impl expr::Visitor<Result<Value>> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal { value } => Ok(self.visit_literal_expr(value)),
            Expr::Unary { operator, right } => self.visit_unary_expr(operator, right),
            Expr::Grouping { expr } => self.evaluate(expr),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary_expr(left, operator, right),
            Expr::Var { name } => self.visit_var_expr(name),
            Expr::Assign { name, expr } => self.visit_assign_expr(name, expr),
        }
    }
}

impl stmt::Visitor<Result<()>> for Interpreter {
    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Expression(expr) => self.visit_expr_stmt(expr),
            Stmt::Print(expr) => self.visit_print_stmt(expr),
            Stmt::Var {name, initializer} => self.visit_var_stmt(name, initializer),
            Stmt::Block(stmts) => self.visit_block_stmt(stmts)
        }
    }
}
