use super::*;

impl TypedExpr {
    pub fn value_known_at_compile(&self) -> bool {
        match self {
            TypedExpr::Cast(_, value) => value.value_known_at_compile(),

            TypedExpr::Primary(_, primary) => {
                match &**primary {
                    TypedPrimaryExpr::NumLiteral(_) => true,
                    TypedPrimaryExpr::EnumVariant(_, _) => true,
                    TypedPrimaryExpr::Grouping(body) => body.value_known_at_compile(),

                    _ => false,
                }
            }

            _ => false,
        }
    }

    pub fn location_known_at_compile(&self) -> bool {
        match self {
            TypedExpr::Cast(_, value) => value.location_known_at_compile(),

            TypedExpr::Primary(_, primary) => {
                match &**primary {
                    TypedPrimaryExpr::Grouping(body) => body.location_known_at_compile(),

                    TypedPrimaryExpr::Ref(_, op, var) => {
                        
                    }
                }
            }
        }
    }
}

