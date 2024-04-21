use super::*;
use crate::types::*;
use crate::expr::*;


impl Expr {
    pub fn cgen(&self, cg: &mut CodeGenerator) {
        match self {
            Expr::Primary(p) => {
                match &**p {
                    PrimaryExpr::Grouping(e) => e.cgen(cg),

                    PrimaryExpr::NumLiteral(n, _) => {
                        for i in 0..n.size() {
                            n.ld_nth_byte_to_reg(R0, i, cg);
                            cg.write_instruction(Instruction::Push(R0));
                            cg.increase_sp_by(1);
                        }
                    }

                    PrimaryExpr::Variable(v) => {
                        if v.has_no_arrays() {
                            match v {
                                Variable::Id(var_name) => {
                                    let var = cg.get_var(var_name.data());
                                    

                                }

                                _ => todo!()
                            }
                        } else {

                        }
                    }
                }
            }
        }
    }
}

impl NumLiteral {
    pub fn ld_nth_byte_to_reg(&self, reg: REG, n: u16, cg: &mut CodeGenerator) {
        let value_to_load: u8 = 
        match self {
            NumLiteral::U8(v) => {
                if n == 0 {
                    *v
                } else {
                    0
                }
            }

            NumLiteral::I8(v) => {
                if n == 0 {
                    *v as u8
                } else {
                    if *v < 0 { //extend sign bit
                        255
                    } else {
                        0
                    }
                }
            }

            NumLiteral::U16(v) => {
                if n < 2 {
                    (*v << n * 8) as u8
                } else {
                    0
                }
            }

            NumLiteral::I16(v) => {
                if n < 2 {
                    (*v << n * 8) as u8
                } else {
                    if *v < 0 { //extend sign bit
                        255
                    } else {
                        0
                    }
                }
            }

            NumLiteral::U32(v) => {
                if n < 4 {
                    (*v << n * 8) as u8
                } else {
                    0
                }
            }

            NumLiteral::I32(v) => {
                if n < 4 {
                    (*v << n * 8) as u8
                } else {
                    if *v < 0 { //extend sign bit
                        255
                    } else {
                        0
                    }
                }
            }

            NumLiteral::U64(v) => {
                if n < 8 {
                    (*v << n * 8) as u8
                } else {
                    0
                }
            }

            NumLiteral::I64(v) => {
                if n < 8 {
                    (*v << n * 8) as u8
                } else {
                    if *v < 0 { //extend sign bit
                        255
                    } else {
                        0
                    }
                }
            }
        };

        /*
        imr REG VALUE
        */

        cg.write_instruction(Instruction::Imr(reg, value_to_load));
    }
}