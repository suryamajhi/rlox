use crate::environment::{EnvRef, Environment};
use crate::expr::Expr;
use crate::function::{Callable, Function, NativeFunction};
use crate::stmt::Stmt;
use crate::token::{Literal, Token, TokenType};
use crate::value::Value;
use crate::{expr, stmt, Exception};
use std::collections::HashMap;
use std::fmt::Arguments;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

type Result<T> = std::result::Result<T, Exception>;

pub struct Interpreter {
    environment: EnvRef,
    pub globals: EnvRef,
    locals: HashMap<Expr, usize>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new();
        globals.borrow_mut().define(
            "clock".to_string(),
            Value::NativeFunction(NativeFunction {
                arity: 0,
                callable: |_, _| {
                    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    Value::Number(timestamp.as_millis() as f64)
                },
            }),
        );

        Interpreter {
            environment: globals.clone(),
            globals,
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, stmts: &Vec<Stmt>) {
        for stmt in stmts {
            match self.execute(stmt) {
                Ok(_) => {}
                Err(e) => match e {
                    Exception::RuntimeError(e) => {
                        e.error();
                        process::exit(70);
                    }
                    _ => {}
                },
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        stmt::Visitor::visit_stmt(self, stmt)
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        self.locals.insert(expr.clone(), depth);
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
        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), value);
        Ok(())
    }

    fn lookup_variable(&self, name: &Token, expr: &Expr) -> Result<Value> {
        let distance = self.locals.get(expr);

        match distance {
            None => self.globals.borrow().get(name),
            Some(distance) => self.environment.borrow().get_at(*distance, &name.lexeme),
        }
    }
    fn visit_var_expr(&self, name: &Token, expr: &Expr) -> Result<Value> {
        self.lookup_variable(name, expr)
    }

    fn visit_assign_expr(&mut self, name: &Token, expr: &Expr) -> Result<Value> {
        let value = self.evaluate(expr)?;

        let distance = self.locals.get(expr);
        match distance {
            Some(distance) => self
                .environment
                .borrow_mut()
                .assign_at(*distance, name, &value),
            None => self.environment.borrow_mut().assign(name, value.clone())?,
        }

        Ok(value)
    }

    fn visit_block_stmt(&mut self, stms: &Vec<Stmt>) -> Result<()> {
        let local_env = Environment::new_local(&self.environment);
        self.execute_block(stms, local_env)
    }

    pub fn execute_block(&mut self, stmts: &Vec<Stmt>, environment: EnvRef) -> Result<()> {
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

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Box<Stmt>>,
    ) -> Result<()> {
        let value = self.evaluate(condition)?;
        if (Interpreter::is_truthy(&value)) {
            self.execute(then_branch)?;
        } else {
            match else_branch {
                None => {}
                Some(stmt) => {
                    self.execute(stmt)?;
                }
            }
        }
        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> Result<()> {
        while Interpreter::is_truthy(&self.evaluate(condition)?) {
            self.execute(body)?;
        }
        Ok(())
    }

    fn visit_logical_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<Value> {
        let left = self.evaluate(left)?;
        if operator.token_type == TokenType::OR {
            if Interpreter::is_truthy(&left) {
                return Ok(left);
            }
        } else {
            if !Interpreter::is_truthy(&left) {
                return Ok(left);
            }
        }
        self.evaluate(right)
    }

    fn visit_call_expr(
        &mut self,
        callee: &Expr,
        paren: &Token,
        arguments: &Vec<Expr>,
    ) -> Result<Value> {
        let callee = self.evaluate(callee)?;

        let mut args = vec![];
        for argument in arguments {
            args.push(self.evaluate(argument)?);
        }
        match callee {
            Value::Function(func) => {
                if arguments.len() != func.arity() {
                    return Exception::runtime_error(
                        paren.clone(),
                        format!(
                            "Expected {} arguments but got {}.",
                            func.arity(),
                            arguments.len()
                        ),
                    );
                }
                return func.call(self, args);
            }
            Value::NativeFunction(func) => {
                if arguments.len() != func.arity() {
                    return Exception::runtime_error(
                        paren.clone(),
                        format!(
                            "Expected {} arguments but got {}.",
                            func.arity(),
                            arguments.len()
                        ),
                    );
                }
                return func.call(self, args);
            }
            _ => Exception::runtime_error(
                paren.clone(),
                "Can only call functions and classes.".to_string(),
            ),
        }
    }

    fn visit_function_stmt(&mut self, name: &Token, function_stmt: &Stmt) -> Result<()> {
        let function = Function::new(function_stmt.clone(), self.environment.clone());
        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), Value::Function(function));
        Ok(())
    }

    fn visit_return_stmt(&mut self, value: &Option<Expr>) -> Result<()> {
        match value {
            None => Err(Exception::Return(Value::Nil)),
            Some(expr) => Err(Exception::Return(self.evaluate(expr)?)),
        }
    }
}

impl expr::Visitor<Result<Value>> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal { value, .. } => Ok(self.visit_literal_expr(value)),
            Expr::Unary {
                operator, right, ..
            } => self.visit_unary_expr(operator, right),
            Expr::Grouping { expr, .. } => self.evaluate(expr),
            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => self.visit_binary_expr(left, operator, right),
            Expr::Var { name, .. } => self.visit_var_expr(name, expr),
            Expr::Assign { name, value, .. } => self.visit_assign_expr(name, value),
            Expr::Logical {
                left,
                operator,
                right,
                ..
            } => self.visit_logical_expr(left, operator, right),
            Expr::Call {
                callee,
                paren,
                arguments,
                ..
            } => self.visit_call_expr(callee, paren, arguments),
        }
    }
}

impl stmt::Visitor<Result<()>> for Interpreter {
    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Expression(expr) => self.visit_expr_stmt(expr),
            Stmt::Print(expr) => self.visit_print_stmt(expr),
            Stmt::Var { name, initializer } => self.visit_var_stmt(name, initializer),
            Stmt::Block(stmts) => self.visit_block_stmt(stmts),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::While { condition, body } => self.visit_while_stmt(condition, body),
            Stmt::Function { name, .. } => self.visit_function_stmt(name, stmt),
            Stmt::Return { keyword, value } => self.visit_return_stmt(value),
        }
    }
}
