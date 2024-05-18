use crate::semantics::semantic_err::SemanticErr;
use crate::stmt::TypeDeclr;
use crate::token::*;
use crate::types::ValueType;

#[macro_export]
macro_rules! try_into_all_types {
    ($self:ident, $t:ident) => {
        match $t {
            ValueType::U8 => $self.try_into().ok().map(|x| NumLiteral::U8(x)),
            ValueType::I8 => $self.try_into().ok().map(|x| NumLiteral::U8(x)),
            ValueType::U16 => $self.try_into().ok().map(|x| NumLiteral::U16(x)),
            ValueType::I16 => $self.try_into().ok().map(|x| NumLiteral::I16(x)),
            ValueType::U32 => $self.try_into().ok().map(|x| NumLiteral::U32(x)),
            ValueType::I32 => $self.try_into().ok().map(|x| NumLiteral::I32(x)),
            ValueType::U64 => $self.try_into().ok().map(|x| NumLiteral::I64(x)),
            ValueType::I64 => $self.try_into().ok().map(|x| NumLiteral::U64(x)),
            _ => None,
        }
    };
}

#[derive(Debug, Clone)]
pub enum Expr {
    Assign(Box<BinaryExpr>),
    Equality(Box<BinaryExpr>),
    Comparison(Box<BinaryExpr>),
    Term(Box<BinaryExpr>),
    Shift(Box<Expr>, Lexeme, NumLiteral),
    Unary(Lexeme, Box<Expr>),
    Cast(Box<Expr>, Lexeme, TypeDeclr),
    FnCall(Lexeme, Args),
    Primary(Box<PrimaryExpr>),
}

#[derive(Debug, Clone)]
pub enum PrimaryExpr {
    Grouping(Expr),
    NumLiteral(NumLiteral, Lexeme),
    Variable(Variable),
    EnumVariant(Lexeme, Lexeme),
    Ref(Lexeme, Variable),
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Expr,
    pub operator: Lexeme,
    pub right: Expr,
}

#[derive(Debug, Clone, PartialEq)]
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

impl NumLiteral {
    pub fn try_implicit_cast(&self, ty: ValueType) -> Option<NumLiteral> {
        match *self {
            Self::U8(x) => try_into_all_types!(x, ty),
            Self::I8(x) => try_into_all_types!(x, ty),
            Self::U16(x) => try_into_all_types!(x, ty),
            Self::I16(x) => try_into_all_types!(x, ty),
            Self::U32(x) => try_into_all_types!(x, ty),
            Self::I32(x) => try_into_all_types!(x, ty),
            Self::U64(x) => try_into_all_types!(x, ty),
            Self::I64(x) => try_into_all_types!(x, ty),
        }
    }

    pub fn negate(&self) -> Result<NumLiteral, ()> {
        match *self {
            Self::U8(n) => {
                if n > (i8::MIN as i16 * -1) as u8 {
                    return Ok(NumLiteral::I16((n as i16) * -1));
                } else {
                    return Ok(NumLiteral::I8(((n as i16) * -1) as i8));
                }
            }

            Self::U16(n) => {
                if n > (i16::MIN as i32 * -1) as u16 {
                    return Ok(NumLiteral::I32((n as i32) * -1));
                } else {
                    return Ok(NumLiteral::I16(((n as i32) * -1) as i16));
                }
            }

            Self::U32(n) => {
                if n > (i32::MIN as i64 * -1) as u32 {
                    return Ok(NumLiteral::I64((n as i64) * -1));
                } else {
                    return Ok(NumLiteral::I32(((n as i64) * -1) as i32));
                }
            }

            Self::U64(n) => {
                if n > 9_223_372_036_854_775_807 {
                    //abs(i64::MIN)
                    return Err(());
                } else {
                    return Ok(NumLiteral::I64((n as i64) * -1));
                }
            }

            _ => panic!("Negative values should not be in a UnaryExpr"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Variable {
    Id(Lexeme),
    StructField(Box<(Variable, Variable)>),
    Array(Box<Variable>, Expr),
}

#[derive(Debug, Clone)]
pub struct Args {
    pub items: Vec<Expr>,
}

impl BinaryExpr {
    pub fn neg_unary_literals(&mut self) -> Result<(), Lexeme> {
        self.left.neg_unary_literals()?;
        self.right.neg_unary_literals()?;
        Ok(())
    }
}

impl Variable {
    pub fn neg_unary_literals(&mut self) -> Result<(), Lexeme> {
        match self {
            Variable::Array(n, i) => {
                n.neg_unary_literals()?;
                i.neg_unary_literals()?;
            }

            Variable::StructField(s) => {
                s.0.neg_unary_literals()?;
                s.1.neg_unary_literals()?;
            }

            Variable::Id(_) => {}
        }

        Ok(())
    }

    pub fn get_first_lexeme(&self) -> Lexeme {
        match self {
            Variable::Array(n, _) => n.get_first_lexeme(),
            Variable::Id(id) => id.clone(),
            Variable::StructField(s) => s.0.get_first_lexeme(),
        }
    }
}

impl Expr {
    pub fn neg_unary_literals(&mut self) -> Result<(), Lexeme> {
        match self {
            Expr::Unary(operator, e) => {
                if operator.data() == "-" {
                    if let Expr::Primary(p) = &mut **e {
                        if let PrimaryExpr::NumLiteral(n, l) = &**p {
                            match n.negate() {
                                Ok(new) => {
                                    *self = Expr::Primary(Box::new(PrimaryExpr::NumLiteral(
                                        new,
                                        l.clone(),
                                    )))
                                }
                                Err(_) => return Err(operator.clone()),
                            }
                        }
                    }
                }
            }

            Expr::Assign(a) => {
                a.left.neg_unary_literals()?;
                a.right.neg_unary_literals()?;
            }

            Expr::Cast(e, _, _) => {
                e.neg_unary_literals()?;
            }

            Expr::Comparison(e) => e.neg_unary_literals()?,

            Expr::Equality(e) => e.neg_unary_literals()?,

            Expr::FnCall(_, a) => {
                for arg in a.items.iter_mut() {
                    arg.neg_unary_literals()?;
                }
            }

            Expr::Primary(p) => match &mut **p {
                PrimaryExpr::Grouping(g) => g.neg_unary_literals()?,

                PrimaryExpr::Variable(v) => v.neg_unary_literals()?,

                PrimaryExpr::Ref(_, v) => {
                    v.neg_unary_literals()?;
                }

                _ => {}
            },

            Expr::Shift(v, _, _) => v.neg_unary_literals()?,

            Expr::Term(b) => b.neg_unary_literals()?,
        }

        Ok(())
    }

    pub fn get_first_lexeme(&self) -> Lexeme {
        match self {
            Expr::Assign(a) => a.left.get_first_lexeme(),

            Expr::Cast(v, _, _) => v.get_first_lexeme(),

            Expr::Comparison(binary) => binary.left.get_first_lexeme(),

            Expr::Equality(binary) => binary.left.get_first_lexeme(),

            Expr::Shift(v, _, _) => v.get_first_lexeme(),

            Expr::Term(b) => b.left.get_first_lexeme(),

            Expr::Unary(left, _) => left.clone(),

            Expr::FnCall(name, _) => name.clone(),

            Expr::Primary(p) => match &**p {
                PrimaryExpr::EnumVariant(n, _) => n.clone(),

                PrimaryExpr::Grouping(g) => g.get_first_lexeme(),

                PrimaryExpr::NumLiteral(_, l) => l.clone(),

                PrimaryExpr::Ref(op, _) => op.clone(),

                PrimaryExpr::Variable(v) => v.get_first_lexeme(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NumLiteral;
    use super::ValueType;

    #[test]
    fn test_implicit_cast() {
        let x = NumLiteral::I8(2);
        assert_eq!(x.try_implicit_cast(ValueType::U8), Some(NumLiteral::U8(2)));

        let x = NumLiteral::U16(10_000);
        assert_eq!(x.try_implicit_cast(ValueType::U8), None);

        let x = NumLiteral::I16(-10_000);
        assert_eq!(x.try_implicit_cast(ValueType::U32), None);

        let x = NumLiteral::U16(10_000);
        assert_eq!(
            x.try_implicit_cast(ValueType::U32),
            Some(NumLiteral::U32(10_000))
        );
    }
}
