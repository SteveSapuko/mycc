use crate::stmt::TypeDeclr;
use crate::expr::NumLiteral;
use crate::semantics::semantic_err::*;
use crate::token::Lexeme;


impl NumLiteral {
    pub fn get_type(&self) -> ValueType {
        match self {
            NumLiteral::U8(_) => ValueType::U8,
            NumLiteral::I8(_) => ValueType::I8,

            NumLiteral::U16(_) => ValueType::U16,
            NumLiteral::I16(_) => ValueType::I16,

            NumLiteral::U32(_) => ValueType::U32,
            NumLiteral::I32(_) => ValueType::I32,

            NumLiteral::U64(_) => ValueType::U64,
            NumLiteral::I64(_) => ValueType::I64,
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum ValueType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    Void,
    Pointer(Box<ValueType>),
    Array(Box<ValueType>, u16),
    CustomType(CustomType),
}

#[derive(Clone, Debug, PartialEq)]
pub enum CustomType {
    CustomStruct(CustomStruct),
    CustomEnum(CustomEnum),
}


impl CustomType {
    pub fn name(&self) -> String {
        match self {
            CustomType::CustomStruct(s) => {
                s.name.clone()
            }

            CustomType::CustomEnum(e) => {
                e.name.clone()
            }
        }
    }
}

impl CustomStruct {
    pub fn get_field_type(&self, field: &Lexeme) -> Option<ValueType> {
        let access_name = field.data();
        
        for f in &self.fields {
            if f.0 == access_name {
                return Some(f.1.clone())
            }
        }

        None
    }
}

impl CustomEnum {
    pub fn eval_type(&self, variant: &Lexeme) -> Result<ValueType, SemanticErr> {
        let variant_name = variant.data();

        if self.variants.contains(&variant_name) {
            return Ok(ValueType::CustomType(CustomType::CustomEnum(self.clone())))
        }

        Err(SemanticErr::NoEnumVariant(self.clone(), variant.clone()))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CustomStruct {
    pub name: String,
    pub fields: Vec<(String, ValueType)>
}

#[derive(Clone, Debug, PartialEq)]
pub struct CustomEnum {
    pub name: String,
    pub variants: Vec<String>
}

impl ValueType {
    pub fn from_declr(declr: &TypeDeclr, defined_types: &Vec<CustomType>) -> Result<ValueType, Lexeme> {
        match declr {
            TypeDeclr::Basic(lex) => {
                let type_text = lex.data();

                match type_text.as_str() {
                    "u8" => return Ok(ValueType::U8),
                    "i8" => return Ok(ValueType::I8),
                    
                    "u16" => return Ok(ValueType::U16),
                    "i16" => return Ok(ValueType::I16),

                    "u32" => return Ok(ValueType::U32),
                    "i32" => return Ok(ValueType::I32),

                    "u64" => return Ok(ValueType::U64),
                    "i64" => return Ok(ValueType::I64),

                    _ => {}
                }

                for custom_type in defined_types {
                    if type_text == custom_type.name() {
                        return Ok(ValueType::CustomType(custom_type.clone()))
                    }
                }

                return Err(lex.clone())
            }

            TypeDeclr::Pointer(p) => {
                let points_to = ValueType::from_declr(p, defined_types)?;
                return Ok(ValueType::Pointer(Box::new(points_to)))
            }

            TypeDeclr::Array(item_t, size) => {
                let item_type = ValueType::from_declr(item_t, defined_types)?;
                return Ok(ValueType::Array(Box::new(item_type), *size))
            }
        }
    }
}