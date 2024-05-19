use self::value::{clean_after_two_values, create_two_values, create_value, ValueLocation};

use super::*;

impl TypedExpr {
    ///Value which the Expr eventaully evaluates to
    ///is placed on the top of the stack
    pub fn generate_onto_stack(&self, cg: &mut CodeGenerator) {
        match self {
            TypedExpr::Term(final_type, binary_expr) => {
                let (left, op, right) = (&binary_expr.left, &binary_expr.operator, &binary_expr.right);

                let (left_value, right_value, stack_increased_by) = create_two_values(cg, left, right);

                let save_to_location: ValueLocation;
                if stack_increased_by == 0 {
                    save_to_location = ValueLocation::Immediate;
                } else {
                    save_to_location = ValueLocation::SpMinus(stack_increased_by);
                }


                match op.as_str() {
                    "+" => {
                        /*
                        first byte:
                            left.ld_nth_byte_to_reg(0, R0)
                            right.ld_nth_byte_to_reg(0, R1)
                            rmov R0
                            add R1
                            amov R0
                            
                            save_to_location R0

                            if more than 1 byte:
                                zac
                                amov R0
                                adc R0
                                amov R0

                            if location is immediate:
                                increase_sp_by(1)
                        
                            if more than 1 byte:
                                push R0


                        for each byte after first:
                            left.ld_nth_byte_to_reg(n, R0)
                            right.ld_nth_byte_to_reg(n, R1)
                            rmov R0
                            add R1

                            amov R1 ;stores result
                            zac
                            amov R0
                            adc R0
                            amov R0 ;stores carry result

                            rmov R1
                            pop R1 ;stores previous carry
                            add R1
                            amov R1 ;stores final result
                            push R1

                            zac
                            adc R0
                            amov R0
                            
                            increase_sp_by(1)
                            push R0
                        */

                        left_value.ld_nth_byte_to_reg(R0, 0, cg);
                        right_value.ld_nth_byte_to_reg(R1, 0, cg);
                        cg.write_instruction(Instruction::Rmov(R0));
                        cg.write_instruction(Instruction::Add(R1));
                        cg.write_instruction(Instruction::Amov(R0));
                        cg.write_instruction(Instruction::Push(R0));

                        if final_type.size(&cg.defined_types) > 1 {
                            cg.write_instruction(Instruction::Zac);
                            cg.write_instruction(Instruction::Amov(R0));
                            cg.write_instruction(Instruction::Adc(R0));
                            cg.write_instruction(Instruction::Amov(R0));

                            cg.increase_sp_by(1);
                            cg.write_instruction(Instruction::Push(R0));

                            for nth_byte in 1..final_type.size(&cg.defined_types) {
                                left_value.ld_nth_byte_to_reg(R0, nth_byte, cg);
                                right_value.ld_nth_byte_to_reg(R1, nth_byte, cg);

                                cg.write_instruction(Instruction::Rmov(R0));
                                cg.write_instruction(Instruction::Add(R1));

                                cg.write_instruction(Instruction::Amov(R1));
                                cg.write_instruction(Instruction::Zac);
                                cg.write_instruction(Instruction::Amov(R0));
                                cg.write_instruction(Instruction::Adc(R0));
                                cg.write_instruction(Instruction::Amov(R0));

                                cg.write_instruction(Instruction::Rmov(R1));
                                cg.write_instruction(Instruction::Pop(R1));
                                cg.write_instruction(Instruction::Add(R1));
                                cg.write_instruction(Instruction::Amov(R1));
                                cg.write_instruction(Instruction::Push(R1));

                                cg.write_instruction(Instruction::Zac);
                                cg.write_instruction(Instruction::Adc(R0));
                                cg.write_instruction(Instruction::Amov(R0));

                                cg.increase_sp_by(1);
                                cg.write_instruction(Instruction::Push(R0));
                            }
                            
                        } else {
                            cg.increase_sp_by(1);
                        }
                    }

                    "-" => {
                        todo!()
                    }

                    _ => unreachable!()
                }

                clean_after_two_values(cg, left_value, right_value);
            }

            TypedExpr::Primary(final_type, primary) => {
                match &**primary {
                    TypedPrimaryExpr::Grouping(body) => body.generate_onto_stack(cg),

                    TypedPrimaryExpr::EnumVariant(_, variant) => {
                        cg.write_instruction(Instruction::Imr(R0, variant.1));
                        cg.write_instruction(Instruction::Push(R0));
                        cg.increase_sp_by(1);
                    }

                    TypedPrimaryExpr::NumLiteral(num) => {
                        for nth in 0..num.get_type().size(&cg.defined_types) {
                            let value = num.get_nth_byte(nth);
                            cg.write_instruction(Instruction::Imr(R0, value));
                            cg.write_instruction(Instruction::Push(R0));
                            cg.increase_sp_by(1);
                        }
                    }

                    TypedPrimaryExpr::Variable(var) => {
                        if var.location_known_at_compile() {
                            let (_, head_offset_from_bp) = cg.get_var(var.get_first_id());
                            let offset_from_bp = head_offset_from_bp as i16 + var.get_total_offset();

                            if offset_from_bp >= 0 {

                            } else {
                                //TODO: add negative pointer arithmetic
                                todo!()
                            }

                        } else {

                        }
                    }
                }
            }
        }
    }
}