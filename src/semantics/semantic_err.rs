use super::{EnumTemplate, StructTemplate, CustomType, Lexeme, ValueType};

#[derive(Debug)]
pub enum SemanticErr {
    NoStructField(StructTemplate, Lexeme),
    NoEnumVariant(EnumTemplate, Lexeme),
    WrongAccess(CustomType, Lexeme),
    UnknownType(Lexeme),
    NotAStruct(Lexeme),
    NotAnArray(Lexeme),
    UndeclaredVar(Lexeme),
    WrongType(ValueType, ValueType, Lexeme), //(SHOULD, IS, LOCATION)
    UsedId(Lexeme),
    CantDeref(Lexeme),
    NotAVar(Lexeme),
    ShiftAmountErr(Lexeme),
    UndeclaredFn(Lexeme),
    FnArityErr(Lexeme),
    DuplicateParams(Lexeme),
    CantReturn(Lexeme),
    CantBreak(Lexeme),
    EnumDuplicateVariants(Lexeme),
    RecursiveStruct(String),
    CantCast(Lexeme),
    CantOp(Lexeme),
}