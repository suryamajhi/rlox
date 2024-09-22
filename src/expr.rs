use std::hash::{Hash, Hasher};

use crate::token::{Literal, Token};

pub trait Visitor<T> {
    fn visit_expr(&mut self, expr: &Expr) -> T;
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal {
        uid: u8,
        value: Literal,
    },
    Unary {
        uid: u8,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        uid: u8,
        expr: Box<Expr>,
    },
    Binary {
        uid: u8,
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Var {
        uid: u8,
        name: Token,
    },
    Assign {
        uid: u8,
        name: Token,
        value: Box<Expr>,
    },
    Logical {
        uid: u8,
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        uid: u8,
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Get {
        uid: u8,
        object: Box<Expr>,
        name: Token,
    },
    Set {
        uid: u8,
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    This {
        uid: u8,
        keyword: Token,
    },
    Super {
        uid: u8,
        keyword: Token,
        method: Token,
    },
}

impl Expr {
    fn get_uid(&self) -> u8 {
        match self {
            Expr::Literal { uid, .. } => *uid,
            Expr::Unary { uid, .. } => *uid,
            Expr::Grouping { uid, .. } => *uid,
            Expr::Binary { uid, .. } => *uid,
            Expr::Var { uid, .. } => *uid,
            Expr::Assign { uid, .. } => *uid,
            Expr::Logical { uid, .. } => *uid,
            Expr::Call { uid, .. } => *uid,
            Expr::Set { uid, .. } => *uid,
            Expr::Get { uid, .. } => *uid,
            Expr::This { uid, .. } => *uid,
            Expr::Super { uid, .. } => *uid,
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.get_uid() == other.get_uid()
    }
}

impl Eq for Expr {}

impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_uid().hash(state)
    }
}
