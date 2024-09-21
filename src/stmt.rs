use crate::expr::Expr;
use crate::token::Token;

pub trait Visitor<T> {
    fn visit_stmt(&mut self, stmt: &Stmt) -> T;
}

pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    }
}
