use super::{CustomEnum, StructTemplate, CustomType, Lexeme, ValueType};

#[derive(Debug)]
pub enum SemanticErr {
    NoStructField(StructTemplate, Lexeme),
    NoEnumVariant(CustomEnum, Lexeme),
    StructDuplicateFields(Lexeme, Lexeme), //Struct Name, Field Name
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
    FnDuplicateParams(Lexeme),
    CantReturn(Lexeme),
    CantBreak(Lexeme),
    EnumDuplicateVariants(Lexeme),
    RecursiveStruct(String),
    CannotCast(Lexeme),
    CannotOp(Lexeme),
}