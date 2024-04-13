use crate::token::*;
use crate::expr::*;
use crate::stmt::*;

mod expr_parsing;
mod stmt_parsing;

use std::any::Any;
use std::mem::discriminant;

pub struct Parser {
    data: Vec<Lexeme>,
    ptr: usize,
}

impl Parser {
    pub fn new(data: Vec<Lexeme>) -> Self {
        Self {
            data,
            ptr: 0
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Box<dyn Stmt>>, &'static str> {
        let mut ast: Vec<Box<dyn Stmt>> = vec![];

        while self.current().tok != Token::EOF {
            ast.push(self.parse_declr()?);
        }

        for stmt in ast.iter_mut() {
            for e in stmt.get_mut_expressions() {
                if e.get_type() == "UnaryExpr" {
                    if e.
                }
            }
        }

        Ok(ast)
    }
    
    fn match_tok(&mut self, t: Token) -> bool {
        if self.current().tok == t {
            self.ptr += 1;
            return true
        }

        false
    }

    fn match_tok_type(&mut self, t: Token) -> bool {
        let temp = self.current();

        if discriminant(&temp.tok) == discriminant(&t) {
            self.ptr += 1;
            return true
        }

        false
    }

    pub fn current(&self) -> Lexeme {
        self.data[self.ptr].clone()
    }

    fn previous(&self) -> Lexeme {
        self.data[self.ptr - 1].clone()
    }

    fn look_ahead(&self) -> Lexeme {
        self.data[self.ptr + 1].clone()
    }

    fn go_back(&mut self) {
        self.ptr -= 1;
    }
}