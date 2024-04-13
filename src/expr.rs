use std::fmt::{Debug, Display};
use crate::token::*;

pub trait Expr: Debug + Display {}

#[derive(Debug)]
pub enum PrimaryExpr {
    Grouping(Box<dyn Expr>),
    NumLiteral(NumLiteral),
    Variable(Variable),
    Ref(Lexeme, Variable),
}

impl Expr for PrimaryExpr {}

impl Display for PrimaryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Grouping(e) => write!(f, "({})", e),
            Self::NumLiteral(n) => write!(f, "{}", n),
            Self::Variable(v) => write!(f, "{}", v),
            Self::Ref(op, v) => write!(f, "RefOp {} on {}", op, v),
        }
    }
}

#[derive(Debug)]
pub struct BinaryExpr {
    pub left: Box<dyn Expr>,
    pub operator: Lexeme,
    pub right: Box<dyn Expr>,
}

impl Expr for BinaryExpr {}

impl Display for BinaryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.left, self.operator, self.right)
    }
}

#[derive(Debug)]
pub struct UnaryExpr {
    pub operator: Lexeme,
    pub right: Box<dyn Expr>,
}

impl Expr for UnaryExpr {}

impl Display for UnaryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.operator, self.right)
    }
}

#[derive(Debug)]
pub struct CastExpr {
    pub value: Box<dyn Expr>,
    pub to_type: Lexeme,
}

impl Expr for CastExpr {}

impl Display for CastExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} cast to {})", self.value, self.to_type)
    }
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

impl Display for NumLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::U8(v) => write!(f, "{}", v),
            Self::U16(v) => write!(f, "{}", v),
            Self::U32(v) => write!(f, "{}", v),
            Self::U64(v) => write!(f, "{}", v),
            
            Self::I8(v) => write!(f, "{}", v),
            Self::I16(v) => write!(f, "{}", v),
            Self::I32(v) => write!(f, "{}", v),
            Self::I64(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug)]
pub enum Variable {
    Id(Lexeme),
    StructField(Box<(Variable, Variable)>),
    Array(Box<Variable>, Box<dyn Expr>),
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(l) => write!(f, "{}", l),
            Self::StructField(b) => write!(f, "({}.{})", b.0 , b.1),
            Self::Array(name, index) => write!(f, "({}[{}])", name, index),
        }
    }
}