use crate::expr::*;

impl Variable {
    pub fn location_known_at_compile(&self) -> bool {
        match self {
            Self::Id(_) => true,
            Self::StructField(s) => {
                s.0.location_known_at_compile() && s.1.location_known_at_compile()
            }
            Self::Array(_, index) => {
                index.value_known_at_compile()
            }
        }
    }
}

impl Expr {
    pub fn location_known_at_compile(&self) -> bool {
        match self {
            Expr::Assign(b) => {
                let (left, right) = &**b;

                left.location_known_at_compile() && right.location_known_at_compile()
            }

            Expr::Cast(v, _, _) => v.location_known_at_compile(),

            Expr::Primary(p) => {
                match &**p {
                    PrimaryExpr::EnumVariant(_, _) => true,

                    PrimaryExpr::NumLiteral(_, _) => true,

                    PrimaryExpr::Grouping(e) => e.location_known_at_compile(),

                    PrimaryExpr::Variable(v) => v.location_known_at_compile(),

                    PrimaryExpr::Ref(_, _) => false,
                }
            }

            _ => false
        }

    }
    
    pub fn value_known_at_compile(&self) -> bool {
        match self {
            Expr::Cast(e, _, _) => {
                return e.value_known_at_compile()
            }

            Expr::Primary(p) => {
                return match **p {
                    PrimaryExpr::EnumVariant(_, _) => true,
                    PrimaryExpr::NumLiteral(_, _) => true,
                    PrimaryExpr::Grouping(e) => e.value_known_at_compile(),

                    _ => false
                }
            }

            _ => {} 
        }

        false
    }
}