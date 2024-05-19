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
                    TypedPrimaryExpr::Ref(_, op, var) => op.as_str() == "&" && var.location_known_at_compile(),

                    _ => false,
                }
            }

            TypedExpr::Assign(_, _, right) => right.value_known_at_compile(),

            _ => false,
        }
    }

    pub fn location_known_at_compile(&self) -> bool {
        match self {
            TypedExpr::Cast(_, value) => value.location_known_at_compile(),

            TypedExpr::Primary(_, primary) => {
                match &**primary {
                    TypedPrimaryExpr::Grouping(body) => body.location_known_at_compile(),

                    TypedPrimaryExpr::Ref(_, _, var) => var.location_known_at_compile(),

                    TypedPrimaryExpr::Variable(var) => var.location_known_at_compile(),

                    //should not be called if the value is known
                    _ => unreachable!()
                }
            }

            TypedExpr::Assign(_, left, right) => left.location_known_at_compile() && right.location_known_at_compile(),

            _ => false
        }
    }
}

impl TypedVariable {
    pub fn location_known_at_compile(&self) -> bool {
        match self {
            TypedVariable::Id(..) => true,

            TypedVariable::StructField(_, struct_access) => {
                let (head, tail) = &**struct_access;

                head.location_known_at_compile() && tail.location_known_at_compile()
            }

            TypedVariable::Array(_, array_head, index) => {
                array_head.location_known_at_compile() && index.value_known_at_compile()
            }
        }
    }
}
