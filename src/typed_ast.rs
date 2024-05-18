use crate::expr::*;
use crate::semantics::FnTemplate;
use crate::types::*;


#[derive(Debug, Clone)]
pub enum TypedExpr {
    Assign(ValueType, Box<TypedExpr>, Box<TypedExpr>),
    Equality(ValueType, Box<TypedBinaryExpr>),
    Comparison(ValueType, Box<TypedBinaryExpr>),
    Term(ValueType, Box<TypedBinaryExpr>),
    Shift(ValueType, Box<TypedExpr>, String, NumLiteral),
    Unary(ValueType, String, Box<TypedExpr>),
    Cast(ValueType, Box<TypedExpr>),
    FnCall(ValueType, String, TypedArgs),
    Primary(ValueType, Box<TypedPrimaryExpr>),
}

impl TypedExpr {
    pub fn final_type(&self) -> ValueType {
        match self {
            TypedExpr::Assign(t, _, _) => t.clone(),
            TypedExpr::Cast(t, _) => t.clone(),
            TypedExpr::Comparison(t, _) => t.clone(),
            TypedExpr::Equality(t, _) => t.clone(),
            TypedExpr::FnCall(t, _, _) => t.clone(),
            TypedExpr::Shift(t, _, _, _) => t.clone(),
            TypedExpr::Term(t, _) => t.clone(),
            TypedExpr::Unary(t, _, _) => t.clone(),

            TypedExpr::Primary(t, _) => t.clone(),
        }
    }

    ///ensures that expressions such as
    ///1 = 2 or &x = 5 are illegal
    ///but x[1].y are
    pub fn is_assignable(&self) -> bool {
        match self {
            TypedExpr::Primary(_, primary_expr) => {
                match &**primary_expr {
                    TypedPrimaryExpr::Variable(_) => true,
                    TypedPrimaryExpr::Ref(_, ref_op, _) => {
                        if ref_op.as_str() == "*" {
                            true
                        } else {
                            false
                        }
                    }

                    _ => false
                }
            }

            TypedExpr::Assign(_, assign_left, _) => {
                assign_left.is_assignable()
            }

            _ => false
        }
    }
}


#[derive(Debug, Clone)]
pub enum TypedPrimaryExpr {
    Grouping(TypedExpr),
    NumLiteral(NumLiteral),
    Variable(TypedVariable),
    EnumVariant(EnumTemplate, (String, u8,)), //Enum Template, (Enum Variant, Variant Number)
    Ref(ValueType, String, TypedVariable),
}

impl TypedPrimaryExpr {
    pub fn final_type(&self) -> ValueType {
        match self {
            TypedPrimaryExpr::Grouping(e) => e.final_type(),
            
            TypedPrimaryExpr::EnumVariant(enum_template, _) => {
                ValueType::CustomEnum(enum_template.clone())
            }

            TypedPrimaryExpr::NumLiteral(n) => n.get_type(),

            TypedPrimaryExpr::Variable(v) => v.final_type(),

            TypedPrimaryExpr::Ref(t, _, _) => {
                t.clone()
            }
        }
    }
}


#[derive(Debug, Clone)]
pub enum TypedVariable {
    Id(ValueType, String),
    StructField(ValueType, Box<(TypedVariable, TypedVariable)>),
    Array(ValueType, Box<TypedVariable>, TypedExpr),
}

impl TypedVariable {
    pub fn final_type(&self) -> ValueType {
        match self {
            TypedVariable::Id(t, _) => t.clone(),
            TypedVariable::Array(t, _, _) => t.clone(),
            TypedVariable::StructField(t, _) => t.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypedBinaryExpr {
    pub left: TypedExpr,
    pub operator: String,
    pub right: TypedExpr,
}

#[derive(Debug, Clone)]
pub struct TypedArgs {
    pub items: Vec<TypedExpr>
}


#[derive(Debug, Clone)]
pub enum TypedStmt {
    VarDeclr(String, ValueType, Option<TypedExpr>),
    FnDeclr(FnTemplate, Box<TypedStmt>),
    CustomTypeDeclr,

    ExprStmt(TypedExpr),
    LoopStmt(Box<TypedStmt>),
    WhileStmt(TypedExpr, Box<TypedStmt>),
    IfStmt(TypedExpr, Box<TypedStmt>, Option<Box<TypedStmt>>),
    BreakStmt,
    ReturnStmt(TypedExpr),
    Block(Vec<TypedStmt>),
}

#[derive(Debug, Clone)]
pub struct TypedParameters {
    pub items: Vec<(String, ValueType)>
}