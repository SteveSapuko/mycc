use crate::expr::NumLiteral;

use super::*;

pub struct Value {
    pub expr: TypedExpr,
    pub location: ValueLocation,
    pub value_size: u16,
}

pub fn create_two_values(cg: &mut CodeGenerator, x: &TypedExpr, y: &TypedExpr) -> (Value, Value, u16) {
    let mut stack_increased_by: u16 = 0;
    
    let mut x_location;
    let x_size = x.final_type().size(&cg.defined_types);

    let y_location;
    let y_size = y.final_type().size(&cg.defined_types);

    if x.value_known_at_compile() {
        x_location = ValueLocation::Immediate;
    } else if x.location_known_at_compile() {
        x_location = ValueLocation::BpPlus(x.get_var_offset_from_bp(cg));
    } else {
        x.generate_onto_stack(cg);
        x_location = ValueLocation::SpMinus(x_size);
        stack_increased_by += x_size;
    }

    if y.value_known_at_compile() {
        y_location = ValueLocation::Immediate;
    } else if y.location_known_at_compile() {
        y_location = ValueLocation::BpPlus(y.get_var_offset_from_bp(cg));
    } else {
        y.generate_onto_stack(cg);
        y_location = ValueLocation::SpMinus(y_size);
        stack_increased_by += y_size;
    }

    //if Y was also put on the stack, the X's location relative to the stack needs to be updated

    if let ValueLocation::SpMinus(offset) = &mut x_location {
        *offset = stack_increased_by;
    }

    let value_x = Value {
        expr: x.clone(),
        location: x_location,
        value_size: x_size
    };

    let value_y = Value {
        expr: y.clone(),
        location: y_location,
        value_size: y_size
    };

    (value_x, value_y, stack_increased_by)
}

pub fn create_value(cg: &mut CodeGenerator, x: &TypedExpr) -> Value {
    let x_location: ValueLocation;
    let x_size = x.final_type().size(&cg.defined_types);

    if x.value_known_at_compile() {
        x_location = ValueLocation::Immediate;
    } else if x.location_known_at_compile() {
        x_location = ValueLocation::BpPlus(x.get_var_offset_from_bp(cg));
    } else {
        x.generate_onto_stack(cg);
        x_location = ValueLocation::SpMinus(x_size);
    }
    
    Value {
        expr: x.clone(),
        location: x_location,
        value_size: x_size}
}

pub fn clean_after_two_values(cg: &mut CodeGenerator, x: Value, y: Value) {
    let mut amount_to_decrease_sp: u16 = 0;

    if let ValueLocation::SpMinus(n) = x.location {
        amount_to_decrease_sp += n;
    }

    if let ValueLocation::SpMinus(n) = y.location {
        amount_to_decrease_sp += n;
    }

    if amount_to_decrease_sp != 0 {
        cg.decrease_sp_by(amount_to_decrease_sp, R0);
    }
}

impl Value {
    pub fn ld_nth_byte_to_reg(&self, reg: REG, nth: u16, cg: &mut CodeGenerator) {
        match self.location {
            ValueLocation::Immediate => {
                let value = self.expr.get_nth_byte(nth, cg);
                cg.write_instruction(Instruction::Imr(reg, value));
            }

            ValueLocation::BpPlus(offset_from_base) => {
                if offset_from_base + nth as i16 >= 0 {
                    let total_offset = offset_from_base as u16 + nth;

                    cg.ld_bp_plus_n_to_reg(reg, total_offset);

                } else {
                    todo!()
                }
            }

            ValueLocation::SpMinus(offset_from_sp) => {
                cg.ld_sp_minus_to_reg(reg, offset_from_sp + nth);
            }
        }
    }
}

pub enum ValueLocation {
    SpMinus(u16),
    BpPlus(i16),
    Immediate,
}

impl TypedExpr {
    pub fn get_var_offset_from_bp(&self, cg: &CodeGenerator) -> i16 {
        if let TypedExpr::Primary(_, primary) = self {
            let var_offset =
            match &**primary {
                TypedPrimaryExpr::Grouping(body) => body.get_var_offset_from_bp(cg),

                TypedPrimaryExpr::Variable(var) => var.get_total_offset(),

                TypedPrimaryExpr::Ref(_, _, var) => var.get_total_offset(),
            
                _ => unreachable!()
            };

            let (_, var_offset_from_bp) = cg.get_var(self.get_first_id());

            return var_offset + var_offset_from_bp as i16
        }

        //should only be called on expressions that contain only vars
        unreachable!()
    }

    pub fn get_nth_byte(&self, nth: u16, cg: &CodeGenerator) -> u8 {
        match self {
            TypedExpr::Primary(_, primary) => primary.get_nth_byte(nth, cg),

            TypedExpr::Cast(to_type, original_expr) => {
                let original_type = original_expr.final_type();

                if original_type.size(&cg.defined_types) < to_type.size(&cg.defined_types) && to_type.is_signed_type(){
                    //TODO casting to signed values
                    todo!()
                } else {
                    original_expr.get_nth_byte(nth, cg)
                }
            }

            _ => panic!("get_nth_byte() should not be called on non-immediate values")
        }
    }

    pub fn get_first_id(&self) -> String {
        //ugly ass function, most of it needs to be unreachable but i realized that after i wrote it
        
        match self {
            TypedExpr::Assign(_, _, value) => value.get_first_id(),

            TypedExpr::Cast(_, value) => value.get_first_id(),

            TypedExpr::Comparison(_, binary) => binary.get_first_id(),

            TypedExpr::Equality(_, binary) => binary.get_first_id(),

            TypedExpr::Term(_, binary) => binary.get_first_id(),

            TypedExpr::FnCall(_, name, _ ) => name.clone(),

            TypedExpr::Shift(_, value, _, _) => value.get_first_id(),

            TypedExpr::Unary(_, _, value) => value.get_first_id(),

            TypedExpr::Primary(_, primary) => {
                match &**primary {
                    TypedPrimaryExpr::Grouping(body) => body.get_first_id(),

                    TypedPrimaryExpr::EnumVariant(name, _) => name.name.clone(),

                    TypedPrimaryExpr::NumLiteral(_) => unreachable!(),

                    TypedPrimaryExpr::Variable(var) => var.get_first_id(),

                    TypedPrimaryExpr::Ref(_, _, _) => unreachable!(),
                }
            }
        }
    }
}

impl TypedPrimaryExpr {
    pub fn get_nth_byte(&self, nth: u16, cg: &CodeGenerator) -> u8 {
        match self {
            TypedPrimaryExpr::EnumVariant(_, variant) => {
                if nth == 0 {
                    variant.1
                } else {
                    0
                }
            }

            TypedPrimaryExpr::Grouping(body) => body.get_nth_byte(nth, cg),

            TypedPrimaryExpr::NumLiteral(num) => num.get_nth_byte(nth),

            _ => panic!("not an immediate value, cannot get nth byte")
        }
    }
}

impl NumLiteral {
    pub fn get_nth_byte(&self, nth: u16) -> u8 {
        match self {
            NumLiteral::U8(num) => {
                if nth < self.size() {
                    ((*num) >> (nth * 8)) as u8
                } else {
                    0
                }
            }

            NumLiteral::I8(num) => {
                if nth < self.size() {
                    ((*num) >> (nth * 8)) as u8
                } else {
                    if *num < 0 {
                        255
                    } else {
                        0
                    }
                }
            }

            NumLiteral::U16(num) => {
                if nth < self.size() {
                    ((*num) >> (nth * 8)) as u8
                } else {
                    0
                }
            }

            NumLiteral::I16(num) => {
                if nth < self.size() {
                    ((*num) >> (nth * 8)) as u8
                } else {
                    if *num < 0 {
                        255
                    } else {
                        0
                    }
                }
            }

            NumLiteral::U32(num) => {
                if nth < self.size() {
                    ((*num) >> (nth * 8)) as u8
                } else {
                    0
                }
            }

            NumLiteral::I32(num) => {
                if nth < self.size() {
                    ((*num) >> (nth * 8)) as u8
                } else {
                    if *num < 0 {
                        255
                    } else {
                        0
                    }
                }
            }

            NumLiteral::U64(num) => {
                if nth < self.size() {
                    ((*num) >> (nth * 8)) as u8
                } else {
                    0
                }
            }

            NumLiteral::I64(num) => {
                if nth < self.size() {
                    ((*num) >> (nth * 8)) as u8
                } else {
                    if *num < 0 {
                        255
                    } else {
                        0
                    }
                }
            }
        }
    }
}

impl TypedBinaryExpr {
    pub fn get_first_id(&self) -> String {
        return self.left.get_first_id()
    }
}

impl TypedVariable {
    pub fn get_total_offset(&self) -> i16 {
        match self {
            TypedVariable::Id(_, _, offset) => *offset as i16,

            TypedVariable::StructField(_, body) => {
                let (head, tail) = &**body;

                (head.get_total_offset() + tail.get_total_offset()) as i16
            }

            TypedVariable::Array(_, _, index) => {
                if let TypedExpr::Primary(_, primary) = index {
                    if let TypedPrimaryExpr::NumLiteral(num) = &**primary {
                        if let NumLiteral::I16(offset) = num {
                            return *offset
                        }
                    }
                }

                //should only be called if index is a literal
                unreachable!()
            }
        }
    }

    pub fn get_first_id(&self) -> String {
        match self {
            TypedVariable::Id(_, id, _) => id.clone(),

            TypedVariable::Array(_, head, _) => head.get_first_id(),

            TypedVariable::StructField(_, body) => body.0.get_first_id()
        }
    }
}