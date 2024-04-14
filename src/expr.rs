
use crate::stmt::TypeDeclr;
use crate::token::*;


#[derive(Debug)]
pub enum Expr {
    Assign(Box<(Expr, Expr)>),
    Equality(Box<BinaryExpr>),
    Comparison(Box<BinaryExpr>),
    Term(Box<BinaryExpr>),
    Shift(Box<Expr>, Lexeme, NumLiteral),
    Unary(Lexeme, Box<Expr>),
    Cast(Box<Expr>, Lexeme, TypeDeclr),
    FnCall(Lexeme, Args),
    Primary(Box<PrimaryExpr>),
}

#[derive(Debug)]
pub enum PrimaryExpr {
    Grouping(Expr),
    NumLiteral(NumLiteral),
    Variable(Variable),
    EnumVariant(Lexeme, Lexeme),
    Ref(Lexeme, Variable),
}

#[derive(Debug)]
pub struct BinaryExpr {
    pub left: Expr,
    pub operator: Lexeme,
    pub right: Expr,
}

#[derive(Debug)]
pub enum NumLiteral {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
}


#[derive(Debug)]
pub enum Variable {
    Id(Lexeme),
    StructField(Box<(Variable, Variable)>),
    Array(Box<Variable>, Expr),
}

#[derive(Debug)]
pub struct Args {
    pub items: Vec<Expr>
}
