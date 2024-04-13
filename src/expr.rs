use std::fmt::{Debug, Display};
use crate::token::*;

pub trait Expr: Debug + Display {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr>;
    fn get_type(&self) -> &'static str;
}

#[derive(Debug)]
pub struct BaseExpr {}

impl Expr for BaseExpr {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        vec![]
    }

    fn get_type(&self) -> &'static str {
        "BaseExpr"
    }
}

impl Display for BaseExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}


#[derive(Debug)]
pub enum PrimaryExpr {
    Grouping(Box<dyn Expr>),
    NumLiteral(NumLiteral),
    Variable(Variable),
    EnumVariant(Lexeme, Lexeme),
    Ref(Lexeme, Variable),
}

impl Expr for PrimaryExpr {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        let mut children: Vec<&mut dyn Expr> = vec![];
        
        match self {
            Self::Grouping(g) => {
                for e in g.get_mut_expressions() {
                    children.push(e);
                }
            }
            Self::NumLiteral(e) => children.push(e),

            Self::Variable(v) => {
                for e in v.get_mut_expressions() {
                    children.push(e);
                }
            }

            Self::Ref(op, v) => {
                for e in v.get_mut_expressions() {
                    children.push(e);
                }
            }

            _ => {}
        }

        children
    }

    fn get_type(&self) -> &'static str {
        "PrimaryExpr"
    }
}

impl Display for PrimaryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Grouping(e) => write!(f, "({})", e),
            Self::NumLiteral(n) => write!(f, "{}", n),
            Self::Variable(v) => write!(f, "{}", v),
            Self::EnumVariant(n, v) => write!(f, "Enum {} Variant {}", n, v),
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

impl Expr for BinaryExpr {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        let mut children: Vec<&mut dyn Expr> = vec![];    
        
        for e in self.left.get_mut_expressions() {
            children.push(e);
        }

        for e in self.right.get_mut_expressions() {
            children.push(e);
        }

        children
    }

    fn get_type(&self) -> &'static str {
        "BinaryExpr"
    }
}

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

impl Expr for UnaryExpr {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        let mut children: Vec<&mut dyn Expr> = vec![];    
        
        for e in self.right.get_mut_expressions() {
            children.push(e);
        }

        children
    }

    fn get_type(&self) -> &'static str {
        "UnaryExpr"
    }
}

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

impl Expr for CastExpr {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        let mut children: Vec<&mut dyn Expr> = vec![];    
        
        for e in self.value.get_mut_expressions() {
            children.push(e);
        }

        children
    }

    fn get_type(&self) -> &'static str {
        "CastExpr"
    }
}

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

impl Expr for NumLiteral {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        vec![self]
    }

    fn get_type(&self) -> &'static str {
        "BaseExpr"
    }
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

impl Expr for Variable {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        let mut children: Vec<&mut dyn Expr> = vec![];

        if let Variable::Array(arr, index) = self {
            for e in arr.get_mut_expressions() {
                children.push(e);
            }

            for e in index.get_mut_expressions() {
                children.push(e);
            }

        }

        children
    }

    fn get_type(&self) -> &'static str {
        "BaseExpr"
    }
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

#[derive(Debug)]
pub struct FnCall {
    pub name: Lexeme,
    pub args: Args
}

impl Expr for FnCall {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        let mut children: Vec<&mut dyn Expr> = vec![];

        for e in self.args.items.iter_mut() {
            for i in e.get_mut_expressions() {
                children.push(i);
            }
        }

        children
    }

    fn get_type(&self) -> &'static str {
        "BaseExpr"
    }
}

impl Display for FnCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FnCall {}\nArgs: {}", self.name, self.args)
    }
}

#[derive(Debug)]
pub struct Args {
    pub items: Vec<Box<dyn Expr>>
}

impl Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for a in &self.items {
            write!(f, "\n{}", a)?;
        }

        Ok(())
    }
}