// Reverse Polish Notation (RPN)

use crate::expr;
use crate::expr::{Expr, Visitor};
use crate::token::Literal;

pub struct RpnNotation {}

impl RpnNotation {
    pub fn print(&mut self, expr: &Expr) -> String {
        self.visit_expr(expr)
    }

    fn format(&mut self, name: &str, exprs: Vec<&Expr>) -> String {
        let mut string = String::new();
        for expr in exprs {
            string.push_str(&self.visit_expr(expr));
            string.push(' ')
        }
        string.push_str(name);
        string
    }
}

impl expr::Visitor<String> for RpnNotation {
    fn visit_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Literal { value, .. } => match value {
                Literal::String(v) => v.to_string(),
                Literal::Number(v) => format!("{}", v),
                Literal::Bool(v) => v.to_string(),
                Literal::None => String::from("nil"),
            },
            Expr::Unary {
                operator, right, ..
            } => self.format(&operator.lexeme, vec![right]),
            Expr::Grouping { expr, .. } => self.format("", vec![expr]),
            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => self.format(&operator.lexeme, vec![left, right]),
            Expr::Var { .. } => String::from("nil"),
            Expr::Assign { .. } => String::from("nil"),
            Expr::Logical {
                left,
                operator,
                right,
                ..
            } => self.format(&operator.lexeme, vec![left, right]),
            Expr::Call { .. } => String::from(""),
        }
    }
}
