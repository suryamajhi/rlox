use crate::expr;
use crate::expr::{Expr, Visitor};
use crate::token::Literal;

pub struct AstPrinter {}

impl AstPrinter {
    pub fn print(&mut self, expr: &Expr) -> String {
        self.visit_expr(expr)
    }

    fn parenthesize(&mut self, name: &str, exprs: Vec<&Expr>) -> String {
        let mut string = String::from("(");
        string.push_str(name);

        for expr in exprs {
            string.push(' ');
            string.push_str(&self.visit_expr(expr));
        }
        string.push(')');
        string
    }
}

impl expr::Visitor<String> for AstPrinter {
    fn visit_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Literal { value } => match value {
                Literal::String(value) => value.to_string(),
                Literal::Number(value) => format!("{:?}", value),
                Literal::Bool(value) => value.to_string(),
                Literal::None => String::from("nil"),
            },
            Expr::Unary { operator, right } => self.parenthesize(&operator.lexeme, vec![right]),
            Expr::Grouping { expr } => self.parenthesize("group", vec![expr]),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.parenthesize(&operator.lexeme, vec![left, right]),
            Expr::Var { name } => name.lexeme.to_string(),
            Expr::Assign { name, expr } => self.parenthesize(&name.lexeme, vec![expr]),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.parenthesize(&operator.lexeme, vec![left, right]),
        }
    }
}
