use crate::token::*;
use crate::expr::*;
use std::fmt::{Display, Debug};

pub trait Stmt: Display + Debug {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr>;
}

#[derive(Debug)]
pub struct ExprStmt {
    pub e: Box<dyn Expr>
}

impl Stmt for ExprStmt {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        return self.e.get_mut_expressions()
    }
}

impl Display for ExprStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExprStmt {};", self.e)
    }
}

#[derive(Debug)]
pub struct LoopStmt {
    pub body: BlockStmt
}

impl Stmt for LoopStmt {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        return self.body.get_mut_expressions()
    }
}

impl Display for LoopStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Loop:{}", self.body)
    }
}

#[derive(Debug)]
pub struct WhileStmt {
    pub condition: Box<dyn Expr>,
    pub body: BlockStmt,
}

impl Stmt for WhileStmt {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        let mut children: Vec<&mut dyn Expr> = vec![];

        children.extend(self.condition.get_mut_expressions());
        children.extend(self.body.get_mut_expressions());

        children
    }

}

impl Display for WhileStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "While {} do {}", self.condition, self.body)
    }
}

#[derive(Debug)]
pub struct IfStmt {
    pub condition: Box<dyn Expr>,
    pub true_branch: BlockStmt,
    pub false_branch: Option<BlockStmt>,
}

impl Stmt for IfStmt {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        let mut children: Vec<&mut dyn Expr> = vec![];

        children.extend(self.condition.get_mut_expressions());
        children.extend(self.true_branch.get_mut_expressions());
        if let Some(t) = &mut self.false_branch {
            children.extend(t.get_mut_expressions());
        }

        children
    }
}

impl Display for IfStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "If {} do {}", self.condition, self.true_branch)?;
        if let Some(b) = &self.false_branch {
            write!(f, "\nelse do {}", b)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct BreakStmt {
    pub key: Lexeme
}

impl Stmt for BreakStmt {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        return vec![]
    }
}

impl Display for BreakStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Break")
    }
}

#[derive(Debug)]
pub struct ReturnStmt {
    pub key: Lexeme,
    pub value: Box<dyn Expr>,
}

impl Stmt for ReturnStmt {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        let mut children: Vec<&mut dyn Expr> = vec![];

        children.extend(self.value.get_mut_expressions());

        children
    }
}

impl Display for ReturnStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Return {}", self.value)
    }
}


#[derive(Debug)]
pub struct BlockStmt {
    pub body: Vec<Box<dyn Stmt>>
}

impl Stmt for BlockStmt {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        let mut children: Vec<&mut dyn Expr> = vec![];

        for s in self.body.iter_mut() {
            children.extend(s.get_mut_expressions());
        }

        children
    }
}

impl Display for BlockStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for s in &self.body {
            write!(f, "\n{}", s)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct VarDeclr {
    pub name: Lexeme,
    pub var_type: TypeDeclr,
    pub value: Option<Box<dyn Expr>>
}

impl Stmt for VarDeclr {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        let mut children: Vec<&mut dyn Expr> = vec![];

        if let Some(v) = &mut self.value {
            children.extend(v.get_mut_expressions());
        }

        children
    }
}

impl Display for VarDeclr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Declare: {}  Type: {}", self.name, self.var_type)?;
        if let Some(t) = &self.value {
            write!(f, "  Value: {}", t)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum TypeDeclr {
    Basic(Lexeme),
    Pointer(Box<TypeDeclr>),
    Array(Box<TypeDeclr>, u16),
}

impl Display for TypeDeclr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic(t) => write!(f, "Type {}", t),
            Self::Array(item_type, size) => write!(f, "Array of: {} Size: {}", item_type, size),
            Self::Pointer(t) => write!(f, "Pointer to {}", t),
        }
    }
}

#[derive(Debug)]
pub struct StructDeclr {
    pub name: Lexeme,
    pub fields: Parameters,
}

impl Stmt for StructDeclr {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        vec![]
    }
}

impl Display for StructDeclr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Declare Struct: {}\nFields: {}", self.name, self.fields)
    }
}

#[derive(Debug)]
pub struct EnumDeclr {
    pub name: Lexeme,
    pub variants: Vec<Lexeme>,
}

impl Stmt for EnumDeclr {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        vec![]
    }
}

impl Display for EnumDeclr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Define Enum {}\nVariants:", self.name)?;
        for v in &self.variants {
            write!(f, "\n{}", v)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct FnDeclr {
    pub name: Lexeme,
    pub parameters: Parameters,
    pub ret_type: TypeDeclr,
    pub body: BlockStmt,
}

impl Stmt for FnDeclr {
    fn get_mut_expressions(&mut self) -> Vec<&mut dyn Expr> {
        let mut children: Vec<&mut dyn Expr> = vec![];

        children.extend(self.body.get_mut_expressions());

        children
    }
}

impl Display for FnDeclr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Declare Function {} Parameters: {} Body: {}", self.name, self.parameters, self.body)
    }
}

#[derive(Debug)]
pub struct Parameters {
    pub params: Vec<(Lexeme, TypeDeclr)>
}

impl Display for Parameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for p in &self.params {
            write!(f, "\n{}: {}", p.0, p.1)?;
        }

        Ok(())
    }
}