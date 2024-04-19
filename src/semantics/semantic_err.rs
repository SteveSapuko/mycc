use super::{CustomEnum, CustomStruct, CustomType, Lexeme, TypeDeclr, ValueType};

#[derive(Debug)]
pub enum SemanticErr {
    NoStructField(CustomStruct, Lexeme),
    NoEnumVariant(CustomEnum, Lexeme),
    StructDuplicateFields(Lexeme, Lexeme), //Struct Name, Field Name
    WrongAccess(CustomType, Lexeme),
    UnknownType(Lexeme),
    NotAStruct(Lexeme),
    NotAnArray(Lexeme),
    UndeclaredVar(Lexeme),
    WrongType(ValueType, ValueType), //(SHOULD, IS, LOCATION)
    UsedId(Lexeme),
    AccessError(Lexeme),
}