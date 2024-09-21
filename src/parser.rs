use crate::expr::Expr;
use crate::print_error;
use crate::stmt::Stmt;
use crate::token::TokenType::*;
use crate::token::{Literal, Token, TokenType};
use std::process;

#[derive(Debug)]
pub struct ParseError;

type Result<T> = std::result::Result<T, ParseError>;

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                None => {
                    self.synchronize();
                }
                Some(stmt) => statements.push(stmt),
            }
        }
        statements
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let res;
        if self.match_token(vec![VAR]) {
            res = self.var_declaration();
        } else {
            res = self.statement();
        }
        match res {
            Ok(stmt) => Some(stmt),
            Err(_) => None,
        }
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(LEFT_PAREN, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(RIGHT_PAREN, "Expect ')' after 'if'.")?;

        let then_branch = self.statement()?;
        let mut else_branch = None;
        if self.match_token(vec![ELSE]) {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        self.consume(LEFT_PAREN, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(RIGHT_PAREN, "Expect ')' after 'if'.")?;

        let body = self.statement()?;
        Ok(Stmt::While {
            condition,
            body: Box::new(body)
        })
    }

    fn block(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while !self.check(&RIGHT_BRACE) && !self.is_at_end() {
            match self.declaration() {
                None => {
                    process::exit(65);
                }
                Some(stmt) => statements.push(stmt),
            }
        }
        match self.consume(RIGHT_BRACE, "Expect '}'.") {
            Ok(_) => {}
            Err(err) => process::exit(65),
        };
        statements
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(IDENTIFIER, "Expect variable name")?.clone();

        let mut initializer = None;
        if self.match_token(vec![EQUAL]) {
            match self.expression() {
                Ok(expr) => {
                    initializer = Some(expr);
                }
                Err(_) => {}
            }
        }

        self.consume(SEMICOLON, "Expect ';' after variable declaration")?;
        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.match_token(vec![IF]) {
            return self.if_statement();
        } else if self.match_token(vec![PRINT]) {
            return self.print_statement();
        } else if self.match_token(vec![LEFT_BRACE]) {
            return Ok(Stmt::Block(self.block()));
        } else if self.match_token(vec![WHILE]) {
            return self.while_statement();
        }
        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(SEMICOLON, "Expect ';' after value.")?;
        Ok(Stmt::Print(expr))
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(SEMICOLON, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(expr))
    }

    pub fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn logical_or(&mut self) -> Result<Expr> {
        let mut expr = self.logical_and()?;
        while self.match_token(vec![OR]) {
            let operator = self.previous().clone();
            let right = self.logical_and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;
        while self.match_token(vec![AND]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.logical_or()?;
        if self.match_token(vec![EQUAL]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            if let Expr::Var { name } = expr {
                return Ok(Expr::Assign {
                    name,
                    expr: Box::new(value),
                });
            }
            return Err(self.error(&equals, "Invalid assignment target"));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;
        while self.match_token(vec![TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;
        while self.match_token(vec![
            TokenType::GREATER,
            TokenType::GREATER_EQUAL,
            TokenType::LESS_EQUAL,
            TokenType::LESS,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;
        while self.match_token(vec![TokenType::PLUS, TokenType::MINUS]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;
        while self.match_token(vec![TokenType::SLASH, TokenType::STAR]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.match_token(vec![TokenType::BANG, TokenType::MINUS]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        if self.match_token(vec![TokenType::FALSE]) {
            return Ok(Expr::Literal {
                value: Literal::Bool(false),
            });
        }
        if self.match_token(vec![TokenType::TRUE]) {
            return Ok(Expr::Literal {
                value: Literal::Bool(true),
            });
        }
        if self.match_token(vec![TokenType::NIL]) {
            return Ok(Expr::Literal {
                value: Literal::None,
            });
        }
        if self.match_token(vec![TokenType::NUMBER, TokenType::STRING]) {
            return Ok(Expr::Literal {
                value: self.previous().literal.clone(),
            });
        }
        if self.match_token(vec![IDENTIFIER]) {
            return Ok(Expr::Var {
                name: self.previous().clone(),
            });
        }
        if self.match_token(vec![TokenType::LEFT_PAREN]) {
            let expr = self.expression()?;
            self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression")?;
            return Ok(Expr::Grouping {
                expr: Box::new(expr),
            });
        }
        Err(self.error(self.peek(), "Expression expected"))
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token> {
        if self.check(&token_type) {
            return Ok(self.advance());
        }
        Err(self.error(self.peek(), message))
    }

    fn match_token(&mut self, types: Vec<TokenType>) -> bool {
        for token_type in types.iter() {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current = self.current + 1;
        }
        self.previous()
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == *token_type
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn error(&self, token: &Token, message: &str) -> ParseError {
        print_error(token.line, &token.lexeme, message);
        ParseError {}
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == SEMICOLON {
                return;
            }
            match self.peek().token_type {
                CLASS | FUN | FOR | IF | PRINT | VAR | RETURN | WHILE => {
                    return;
                }
                _ => {}
            }
            self.advance();
        }
    }
}
