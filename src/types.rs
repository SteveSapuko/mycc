use crate::stmt::TypeDeclr;
use crate::expr::*;
use crate::semantics::ScopeStack;
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
    CustomStruct(String),
    CustomEnum(CustomEnum),
}

impl StructTemplate {
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

#[derive(Clone, Debug, PartialEq)]
pub struct StructTemplate {
    pub name: String,
    pub fields: Vec<(String, ValueType)>
}

impl StructTemplate {
    pub fn check_recursive(&self, ss: &ScopeStack, iteration: u8) -> Result<(), String> {
        if iteration == 100 {
            return Err(self.name.clone())
        }
        
        for f in self.fields.iter() {
            //println!("field: {} type: {:#?}", f.0, f.1);
            if let ValueType::CustomStruct(s) = &f.1 {
                let child_struct = ss.get_custom_struct(s.clone()).expect("should have been caught earlier");
                child_struct.check_recursive(ss, iteration + 1)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnfinishedStruct {
    pub name: Lexeme,
    pub fields: Vec<(Lexeme, MaybeType)>
}

#[derive(Clone, Debug, PartialEq)]
pub enum CustomType {
    CustomStruct(StructTemplate),
    CustomEnum(CustomEnum),
}

impl CustomType {
    pub fn name(&self) -> String {
        match self {
            Self::CustomEnum(t) => t.name.clone(),

            Self::CustomStruct(t) => t.name.clone()
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct CustomEnum {
    pub name: String,
    pub variants: Vec<String>
}

#[derive(Clone, Debug, PartialEq)]
pub enum MaybeType {
    Resolved(ValueType),
    Unresolved(TypeDeclr),
}

impl MaybeType {
    pub fn unwrap(&self) -> ValueType {
        match self {
            Self::Resolved(t) => t.clone(),
            Self::Unresolved(_) => panic!("Unwrapped Unresolved Type"),
        }
    }
}

impl ValueType {
    pub fn from_declr(declr: &TypeDeclr, ss: &mut ScopeStack) -> Result<ValueType, Lexeme> {
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

                    "void" => return Ok(ValueType::Void),

                    _ => {}
                }

                for t in ss.defined_types() {
                    match &t {
                        CustomType::CustomEnum(e) => {
                            if lex.data() == e.name {
                                return Ok(ValueType::CustomEnum(e.clone()))
                            }
                        }

                        CustomType::CustomStruct(s) => {
                            if lex.data() == s.name {
                                return Ok(ValueType::CustomStruct(s.name.clone()))
                            }
                        }

                        
                    }
                }

                return Err(lex.clone())
            }

            TypeDeclr::Pointer(p) => {
                let points_to = ValueType::from_declr(p, ss)?;
                return Ok(ValueType::Pointer(Box::new(points_to)))
            }

            TypeDeclr::Array(item_t, size) => {
                let item_type = ValueType::from_declr(item_t, ss)?;
                return Ok(ValueType::Array(Box::new(item_type), *size))
            }
        }
    }

    pub fn from_declr_new_struct(declr: &TypeDeclr) -> ValueType {
        match declr {
            TypeDeclr::Basic(id) => ValueType::CustomStruct(id.data()),

            TypeDeclr::Pointer(p) => ValueType::Pointer(Box::new(ValueType::from_declr_new_struct(p))),

            TypeDeclr::Array(item_type, size) => {
                ValueType::Array(Box::new(ValueType::from_declr_new_struct(item_type)), *size)
            }
        }
    }

    pub fn size(&self, ss: &ScopeStack) -> u16 {
        match self {
            Self::U8 => 1,
            Self::I8 => 1,

            Self::U16 => 2,
            Self::I16 => 2,

            Self::U32 => 4,
            Self::I32 => 4,

            Self::U64 => 8,
            Self::I64 => 8,

            Self::Pointer(_) => 2,

            Self::Array(item_type, size) => {
                item_type.size(ss) * *size
            }

            Self::Void => 0,

            Self::CustomEnum(_) => 1,

            Self::CustomStruct(struct_name) => {
                let custom_struct = ss.get_custom_struct(struct_name.clone()).expect("should have been caught");
                let mut sum: u16 = 0;

                for f in custom_struct.fields {
                    sum += f.1.size(ss);
                }

                sum
            }
        }
    }
}

impl NumLiteral {
    pub fn size(&self) -> u16 {
        match self {
            Self::U8(_) => 1,
            Self::I8(_) => 1,

            Self::U16(_) => 2,
            Self::I16(_) => 2,

            Self::U32(_) => 4,
            Self::I32(_) => 4,

            Self::U64(_) => 8,
            Self::I64(_) => 8,
        }
    }
}