use crate::token::*;
use crate::expr::*;
use std::fmt::Debug;


#[derive(Debug, Clone)]
pub enum Stmt {
    VarDeclr(Lexeme, TypeDeclr, Option<Expr>),
    FnDeclr(Lexeme, Parameters, TypeDeclr, Box<Stmt>),
    StructDeclr(Lexeme, Parameters),
    EnumDeclr(Lexeme, Vec<Lexeme>),

    ExprStmt(Expr),
    LoopStmt(Box<Stmt>),
    WhileStmt(Expr, Box<Stmt>),
    IfStmt(Expr, Box<Stmt>, Option<Box<Stmt>>),
    BreakStmt(Lexeme),
    ReturnStmt(Lexeme, Expr),
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDeclr {
    Basic(Lexeme),
    Pointer(Box<TypeDeclr>),
    Array(Box<TypeDeclr>, u16),
}

#[derive(Debug, Clone)]
pub struct Parameters {
    pub params: Vec<(Lexeme, TypeDeclr)>
}

impl Stmt {
    pub fn neg_unary_literals(&mut self) -> Result<(), Lexeme>{
        match self {
            Self::ExprStmt(e) => e.neg_unary_literals()?,
            
            Self::Block(b) => {
                for s in b {
                    s.neg_unary_literals()?;
                }
            }
            
            Self::FnDeclr(_,_ ,_ , body) => body.neg_unary_literals()?,

            Self::IfStmt(cond, t_branch, f_branch) => {
                cond.neg_unary_literals()?;
                t_branch.neg_unary_literals()?;
                if let Some(f) = f_branch {
                    f.neg_unary_literals()?;
                }
            }

            Self::LoopStmt(b) => b.neg_unary_literals()?,

            Self::ReturnStmt(_, e) => e.neg_unary_literals()?,

            Self::VarDeclr(_,_ ,v ) => {
                if let Some(e) = v {
                    e.neg_unary_literals()?;
                }
            }

            Self::WhileStmt(cond, body) => {
                cond.neg_unary_literals()?;
                body.neg_unary_literals()?;
            }
            
            _ => {}
        }

        Ok(())
    }
}