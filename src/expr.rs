use crate::token::{Literal, Token};

pub trait Visitor<T> {
    fn visit_expr(&mut self, expr: &Expr) -> T;
}

#[derive(Debug)]
pub enum Expr {
    Literal {
        value: Literal,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Var {
        name: Token,
    },
    Assign {
        name: Token,
        expr: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
}
