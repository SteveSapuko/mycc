use std::fmt::Display;

use crate::expr::*;
use crate::stmt::*;
use crate::token::*;

impl Display for TypeDeclr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic(t) => write!(f, "Type {}", t),
            Self::Array(item_type, size) => write!(f, "Array of: {} Size: {}", item_type, size),
            Self::Pointer(t) => write!(f, "Pointer to {}", t),
        }
    }
}

impl Display for Parameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for p in &self.params {
            write!(f, "\n{}: {}", p.0, p.1)?;
        }

        Ok(())
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key(s) => write!(f, "{}", s),
            Self::Op(s) => write!(f, "{}", s),
            Self::Cond(s) => write!(f, "{}", s),
            Self::Id(s) => write!(f, "{}", s),
            Self::Lit(s) => write!(f, "{}", s),
            Self::ParenOpen => write!(f, "("),
            Self::ParenClose => write!(f, ")"),
            Self::SquareOpen => write!(f, "["),
            Self::SquareClose => write!(f, "]"),
            Self::CurlyOpen => write!(f, "{{"),
            Self::CurlyClose => write!(f, "}}"),
            Self::SemiCol => write!(f, ";"),
            Self::Col => write!(f, ":"),
            Self::Comma => write!(f, ","),
            Self::Period => write!(f, "."),
            Self::Arrow => write!(f, "->"),
            Self::EOF => write!(f, "EOF"),
        }
    }
}

impl Display for Lexeme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tok)
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

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assign(t) => write!(f, "{} = {}", t.0, t.1),
            Self::Equality(t) => write!(f, "{}", t),
            Self::Comparison(t) => write!(f, "{}", t),
            Self::Term(t) => write!(f, "{}", t),
            Self::Shift(t, t2, t3) => write!(f, "({} {} {})", t, t2, t3),
            Self::Unary(t, t2) => write!(f, "({} {})", t, t2),
            Self::Cast(t, t2, t3) => write!(f, "({} {} {})", t, t2, t3),
            Self::FnCall(t, t2) => write!(f, "(Call fn {} Args: {})", t, t2),
            Self::Primary(t) => {
                match &**t {
                    PrimaryExpr::Grouping(t) => write!(f, "{}", t),
                    PrimaryExpr::NumLiteral(t, _) => write!(f, "{}", t),
                    PrimaryExpr::EnumVariant(t, t2) => write!(f, "({}::{})", t, t2),
                    PrimaryExpr::Variable(t) => write!(f, "{}", t),
                    PrimaryExpr::Ref(t, t2) => write!(f, "Ref {} on {}", t, t2),
                }
            }
        }
    }
}

impl Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for a in &self.items {
            write!(f, "\n{}", a)?;
        }
        
        Ok(())
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(t) => write!(f, "{}", t),
            Self::StructField(t) => write!(f, "{}.{}", t.0, t.1),
            Self::Array(t, t2) => write!(f, "{}[{}]", t, t2),
        }
    }
}

impl Display for BinaryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.left, self.operator, self.right)       
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VarDeclr(name, t, v) => {
                write!(f, "Declare Var: {} Type: {}", name, t)?;
                if let Some(v) = v {
                    write!(f, " Value: {}", v)?;
                }
            }

            Self::Block(b) => {
                write!(f, "Block:")?;
                for s in b {
                    write!(f, "\n{}", s)?;
                }
            } 

            Self::BreakStmt(_) => write!(f, "Break")?,

            Self::EnumDeclr(n, v) => {
                write!(f, "Declare Enum: {} Variants:", n)?;
                for v in v {
                    write!(f, "\n{}", v)?;
                }
            }

            Self::ExprStmt(e) => write!(f, "ExprStmt {}", e)?,

            Self::FnDeclr(n, arg, ret_t, b) => write!(f, "Declare Fn: {} Params: {} RetType: {} Body: {}", n, arg, ret_t, b)?,

            Self::IfStmt(c, t, fb) => {
                write!(f, "If {} Do\n{}", c, t)?;
                if let Some(fb) = fb {
                    write!(f, "\nElse Do:\n{}", fb)?;
                }
            }

            Self::LoopStmt(b) => write!(f, "Loop Body:\n{}", b)?,

            Self::ReturnStmt(_, e) => write!(f, "Return {}", e)?,

            Self::StructDeclr(n, fields) => write!(f, "Declare Struct: {} Fields: {}", n, fields)?,

            Self::WhileStmt(c, b) => write!(f, "While {} Do: \n{}", c, b)?
        }

        Ok(())
    }
}