use crate::stmt::TypeDeclr;
use crate::expr::*;
use crate::semantics::*;

use self::semantic_err::SemanticErr;


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
    CustomEnum(EnumTemplate),
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructTemplate {
    pub name: String,
    pub fields: Vec<(String, ValueType, u16)> //u16 is in bytes from first byte of struct
}

impl StructTemplate {
    ///Returns offset from head, as well as type
    pub fn get_field(&self, field_name: String) -> Option<(ValueType, u16)> {
        for f in &self.fields {
            if f.0 == field_name {
                return Some((f.1.clone(), f.2))
            }
        }

        //this function is used only if we know the input code to be valid
        None
    }
    
}

#[derive(Clone, Debug, PartialEq)]
pub enum CustomType {
    CustomStruct(StructTemplate),
    CustomEnum(EnumTemplate),
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
pub struct EnumTemplate {
    pub name: String,
    pub variants: Vec<(String, u8)>
}

impl EnumTemplate {
    pub fn get_variant(&self, target: String) -> Option<u8> {
        for v in self.variants.iter() {
            if v.0 == target {
                return Some(v.1)
            }
        }
        None
    }
}

pub fn get_custom_struct(target_name: String, defined_types: &Vec<CustomType>) -> Option<StructTemplate> {
    for t in defined_types {
        if let CustomType::CustomStruct(s) = &t {
            if s.name == target_name {
                return Some(s.clone())
            }
        }
    }

    None
}

impl ValueType {
    pub fn from_declr(declr: &TypeDeclr, defined_types: &Vec<CustomType>) -> Result<ValueType, SemanticErr> {
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

                for t in defined_types {
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

                return Err(SemanticErr::UnknownType(lex.clone()))
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

    ///being_defined is for structs that may not be known/legal yet,
    ///but we want to assume they're valid
    pub fn from_declr_new_struct(declr: &TypeDeclr, defined_types: &Vec<CustomType>, being_defined: &Vec<String>) -> Result<ValueType, SemanticErr> {
        match declr {
            TypeDeclr::Basic(id) => {
                if let Ok(t) = ValueType::from_declr(declr, defined_types) {
                    return Ok(t)
                }
                
                if !being_defined.contains(&id.data()) {
                    return Err(SemanticErr::UnknownType(id.clone()))
                }

                Ok(
                    ValueType::CustomStruct(id.data())
                )
            },

            TypeDeclr::Pointer(p) => Ok(
                ValueType::Pointer(
                    Box::new(ValueType::from_declr_new_struct(p, defined_types, being_defined)?)
                )
            ),

            TypeDeclr::Array(item_type, size) => Ok(
                ValueType::Array(
                    Box::new(ValueType::from_declr_new_struct(item_type, defined_types, being_defined)?),
                    *size)
            )
        }
    }

    pub fn size(&self, defined_types: &Vec<CustomType>) -> u16 {
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
                item_type.size(defined_types) * *size
            }

            Self::Void => 0,

            Self::CustomEnum(_) => 1,

            Self::CustomStruct(struct_name) => {
                let custom_struct = get_custom_struct(struct_name.clone(), defined_types).expect("should have been caught");
                let mut sum: u16 = 0;

                for f in custom_struct.fields {
                    sum += f.1.size(defined_types);
                }

                sum
            }
        }
    }

    pub fn dereference(&self) -> Option<ValueType> {
        if let ValueType::Pointer(points_to_type) = self {
            return Some(*points_to_type.clone())
        }

        None
    }

    pub fn is_primitive_type(&self) -> bool {
        match self {
            ValueType::Array(_, _) => false,
            ValueType::CustomEnum(_) => false,
            ValueType::CustomStruct(_) => false,
            ValueType::Void => false,

            _ => true,
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