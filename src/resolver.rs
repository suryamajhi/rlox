use crate::expr::Expr;
use crate::interpreter::Interpreter;
use crate::stmt::Stmt;
use crate::token::Token;
use crate::RuntimeError;
use crate::{expr, print_error, stmt, Exception};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::Arguments;

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionType {
    NONE,
    FUNCTION,
}
pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::NONE,
        }
    }

    fn visit_block_stmt(&mut self, stmts: &Vec<Stmt>) {
        self.begin_scope();
        self.resolve_block(stmts);
        self.end_scope();
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop().expect("stack underflow");
    }

    pub fn resolve_block(&mut self, stmts: &Vec<Stmt>) {
        for stmt in stmts {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        stmt::Visitor::visit_stmt(self, stmt);
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        expr::Visitor::visit_expr(self, expr);
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) {
        self.declare(name);
        if let Some(initializer) = initializer {
            self.resolve_expr(initializer);
        }
        self.define(name);
    }

    fn peek_scopes_mut(&mut self) -> &mut HashMap<String, bool> {
        self.scopes.last_mut().expect("stack is empty")
    }

    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.peek_scopes_mut();
        if scope.contains_key(&name.lexeme) {
            RuntimeError {
                token: name.clone(),
                message: "Already a variable with this name in this scope.".to_string(),
            }
            .error();
        }

        scope.insert(name.lexeme.to_string(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        self.peek_scopes_mut().insert(name.lexeme.to_string(), true);
    }

    fn visit_var_expr(&mut self, name: &Token, expr: &Expr) {
        if let Some(scope) = self.scopes.last() {
            if let Some(false) = scope.get(&name.lexeme) {
                print_error(
                    name.line,
                    &name.lexeme,
                    "Can't read local variable in it's own initializer",
                );
            }
        }

        self.resolve_local(expr, name)
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(&name.lexeme) {
                self.interpreter.resolve(expr, self.scopes.len() - 1 - i);
            }
        }
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr, expr: &Expr) {
        self.resolve_expr(value);
        self.resolve_local(expr, name);
    }

    fn visit_function_stmt(&mut self, name: &Token, params: &Vec<Token>, body: &Vec<Stmt>) {
        self.declare(name);
        self.define(name);

        self.resolve_function(params, body, FunctionType::FUNCTION);
    }

    fn resolve_function(
        &mut self,
        params: &Vec<Token>,
        body: &Vec<Stmt>,
        function_type: FunctionType,
    ) {
        let enclosing_function = self.current_function.clone();
        self.current_function = function_type;

        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_block(body);
        self.end_scope();
        self.current_function = enclosing_function;
    }

    fn visit_expr_stmt(&mut self, expr: &Expr) {
        self.resolve_expr(expr);
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Box<Stmt>>,
    ) {
        self.resolve_expr(condition);
        self.resolve_stmt(then_branch);
        if let Some(else_branch) = else_branch {
            self.resolve_stmt(else_branch);
        }
    }

    fn visit_print_stmt(&mut self, expr: &Expr) {
        self.resolve_expr(expr);
    }

    fn visit_return_stmt(&mut self, name: &Token, value: &Option<Expr>) {
        if self.current_function == FunctionType::NONE {
            RuntimeError {
                token: name.clone(),
                message: "Can't return from top-level code".to_string(),
            }
            .error();
        }
        if let Some(value) = value {
            self.resolve_expr(value);
        }
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) {
        self.resolve_expr(condition);
        self.resolve_stmt(body);
    }

    fn visit_binary_expr(&mut self, left: &Expr, right: &Expr) {
        self.resolve_expr(left);
        self.resolve_expr(right);
    }

    fn visit_call_expr(&mut self, callee: &Expr, arguments: &Vec<Expr>) {
        self.resolve_expr(callee);
        for arg in arguments {
            self.resolve_expr(arg);
        }
    }

    fn visit_grouping_expr(&mut self, expr: &Expr) {
        self.resolve_expr(expr);
    }

    fn visit_unary_expr(&mut self, right: &Expr) {
        self.resolve_expr(right);
    }
}

impl expr::Visitor<()> for Resolver<'_> {
    fn visit_expr(&mut self, expr: &Expr) -> () {
        match expr {
            Expr::Literal { .. } => {}
            Expr::Unary {
                operator, right, ..
            } => self.visit_unary_expr(right),
            Expr::Grouping { expr, .. } => self.visit_grouping_expr(expr),
            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => self.visit_binary_expr(left, right),
            Expr::Var { name, .. } => self.visit_var_expr(name, expr),
            Expr::Assign { name, value, .. } => self.visit_assign_expr(name, value, expr),
            Expr::Logical {
                left,
                operator,
                right,
                ..
            } => self.visit_binary_expr(left, right),
            Expr::Call {
                callee, arguments, ..
            } => self.visit_call_expr(callee, arguments),
        }
    }
}

impl stmt::Visitor<()> for Resolver<'_> {
    fn visit_stmt(&mut self, stmt: &Stmt) {
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
            Stmt::Function { name, params, body } => self.visit_function_stmt(name, params, body),
            Stmt::Return { keyword, value } => self.visit_return_stmt(keyword, value),
        }
    }
}
