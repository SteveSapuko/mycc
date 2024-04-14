use crate::token::*;
use crate::expr::*;
use std::fmt::{Display, Debug};



pub enum Stmt {

}


#[derive(Debug)]
pub struct VarDeclr {
    pub name: Lexeme,
    pub var_type: TypeDeclr,
    pub value: Option<Expr>
}



#[derive(Debug)]
pub enum TypeDeclr {
    Basic(Lexeme),
    Pointer(Box<TypeDeclr>),
    Array(Box<TypeDeclr>, u16),
}



#[derive(Debug)]
pub struct Parameters {
    pub params: Vec<(Lexeme, TypeDeclr)>
}

