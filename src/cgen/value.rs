
use super::*;
use crate::expr::*;

pub struct Value {
    expr: Expr,
    location: ValueLocation,
    value_size: u16,

    //in case of a cast, where, for example, the actual value is
    //2 bytes large, but it's cast to a 4 byte size
    pub uncasted_size: u16 
}

impl Value {
    pub fn new(expr: Expr, cg: &CodeGenerator) -> Self {
        let location: ValueLocation;

        if expr.value_known_at_compile() {
            location = ValueLocation::Immediate;
        } else if expr.location_known_at_compile() {
            
        }

        

        Value {
            expr: expr,
            location,
            value_size: expr.eval_type(&mut cg.ss).unwrap().size(&cg.ss),
            uncasted_size: () }
    }
}

pub enum ValueLocation {
    SpMinus(u16),
    BpPlus(u16),
    Immediate,
}