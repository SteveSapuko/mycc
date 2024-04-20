use super::*;

impl Expr {
    pub fn eval_type(&self, ss: &mut ScopeStack) -> Result<ValueType, SemanticErr> {
        match self {
            Expr::Assign(a) => {
                let (left, right) = &**a;

                //can only assign to variables
                if let Expr::Primary(p) = &*left {
                    if let PrimaryExpr::Variable(_) = &**p {
                        let left_type = left.eval_type(ss)?;

                        let right_type = right.eval_type(ss)?;

                        if left_type != right_type {
                            return Err(SemanticErr::WrongType(left_type, right_type, right.get_first_lexeme()))
                        }

                        return Ok(left_type)
                    }
                }

                return Err(SemanticErr::NotAVar(left.get_first_lexeme()))
            }

            //TODO make sure only works on certain types
            Expr::Cast(value, op, cast_to_type_declr) => {
                let value_type = value.eval_type(ss)?;
                
                if match value_type {
                    ValueType::Array(_, _) => true,
                    ValueType::CustomStruct(_) => true,
                    ValueType::Void => true,
                    _ => false,
                } {
                    return Err(SemanticErr::CannotCast(op.clone()))
                }


                let to_type = match ValueType::from_declr(cast_to_type_declr, ss) {
                    Ok(t) => t,
                    Err(l) => return Err(SemanticErr::UnknownType(l))
                };

                if match to_type {
                    ValueType::Array(_, _) => true,
                    ValueType::CustomStruct(_) => true,
                    ValueType::Void => true,
                    _ => false,
                } {
                    return Err(SemanticErr::CannotCast(op.clone()))
                }

                return Ok(to_type)
            }

            //TODO make sure only works on certain types
            Expr::Comparison(b) => b.eval_type(ss),

            //TODO make sure only works on certain types
            Expr::Equality(b) => b.eval_type(ss),

            //TODO make sure only works on certain types
            Expr::Shift(v, op, shift_by) => {
                let v_type = v.eval_type(ss)?;

                if !matches!(shift_by, NumLiteral::U8(_)) {
                    return Err(SemanticErr::ShiftAmountErr(op.clone()))
                }

                return Ok(v_type)
            }

            //TODO make sure only works on certain types
            Expr::Term(b) => b.eval_type(ss),

            //TODO make sure only works on certain types
            Expr::Unary(op, right) => {
                let right_type = right.eval_type(ss)?;
                
                if match right_type {
                    ValueType::Array(_, _) => true,
                    ValueType::CustomStruct(_) => true,
                    ValueType::CustomEnum(_) => true,
                    ValueType::Void => true,
                    _ => false,
                } {
                    return Err(SemanticErr::CannotOp(op.clone()))
                }

                return Ok(right_type)
            }
            
            Expr::FnCall(fn_name, args) => {
                let fn_template = match ss.get_fn_from_name(fn_name.data()) {
                    Some(t) => t,
                    None => return Err(SemanticErr::UndeclaredFn(fn_name.clone()))
                };

                if args.items.len() != fn_template.parameters.len() {
                    return Err(SemanticErr::FnArityErr(fn_name.clone()))
                }

                for a in args.items.iter().enumerate() {
                    let (i, arg) = a;

                    let arg_type = arg.eval_type(ss)?;

                    if arg_type != fn_template.parameters[i] {
                        return Err(SemanticErr::WrongType(fn_template.parameters[i].clone(), arg_type, arg.get_first_lexeme()))
                    }
                }

                return Ok(fn_template.ret_type)
            }

            Expr::Primary(p) => {
                match &**p {
                    PrimaryExpr::NumLiteral(n, _) => {
                        return Ok(n.get_type())
                    }

                    PrimaryExpr::EnumVariant(name, variant) => {
                        let type_by_name = ss.get_type_from_string(name.clone())?;

                        if let CustomType::CustomEnum(en) = &type_by_name {
                            if en.variants.contains(&variant.data()) {
                                return Ok(ValueType::CustomEnum(en.clone()))
                            }
                            return Err(SemanticErr::NoEnumVariant(en.clone(), variant.clone()))
                        }

                        return Err(SemanticErr::WrongAccess(type_by_name, variant.clone()))
                    }

                    PrimaryExpr::Variable(var) => {
                        return Ok(var.eval_type(ss, None))?
                    }

                    PrimaryExpr::Grouping(g) => {
                        return Ok(g.eval_type(ss))?
                    }

                    PrimaryExpr::Ref(op, var) => {
                        let var_type = var.eval_type(ss, None)?;
                        
                        match op.data().as_str() {
                            "&" => {
                                return Ok(ValueType::Pointer(Box::new(var_type)))
                            }

                            "*" => {
                                if let ValueType::Pointer(points_to) = var_type {
                                    return Ok(*points_to)
                                }

                                return Err(SemanticErr::CantDeref(op.clone()))
                            }

                            _ => unreachable!()
                        }
                    }
                }
            }
        }
    }
}

impl BinaryExpr {
    pub fn eval_type(&self, ss: &mut ScopeStack) -> Result<ValueType, SemanticErr> {
        let left_type = self.left.eval_type(ss)?;
        let right_type = self.right.eval_type(ss)?;

        if left_type != right_type {
            return Err(SemanticErr::WrongType(left_type, right_type, self.operator.clone()))
        }

        if match right_type {
            ValueType::Array(_, _) => true,
            ValueType::CustomStruct(_) => true,
            ValueType::CustomEnum(_) => true,
            ValueType::Void => true,
            _ => false,
        } {
            return Err(SemanticErr::CannotOp(self.operator.clone()))
        }

        if match left_type {
            ValueType::Array(_, _) => true,
            ValueType::CustomStruct(_) => true,
            ValueType::CustomEnum(_) => true,
            ValueType::Void => true,
            _ => false,
        } {
            return Err(SemanticErr::CannotOp(self.operator.clone()))
        }

        return Ok(left_type)
    }
}

impl Variable {
    pub fn eval_type(&self, ss: &mut ScopeStack, parent_struct_type: Option<StructTemplate>) -> Result<ValueType, SemanticErr> {
        match self {
            Variable::Id(lexeme) => {
                match parent_struct_type {
                    Some(parent) => {
                        let field_type = match parent.get_field_type(lexeme) {
                            Some(t) => t,
                            None => return Err(SemanticErr::NoStructField(parent, lexeme.clone()))
                        };

                        return Ok(field_type)
                    }

                    None => {
                        let var_type = match ss.get_var_type_from_name(lexeme.data()) {
                            Some(t) => t,
                            None => return Err(SemanticErr::UndeclaredVar(lexeme.clone()))
                        };
        
                        return Ok(var_type)
                    }
                }
            }

            Variable::Array(array_head, index) => {
                let index_type = index.eval_type(ss)?;
                let array_type = array_head.eval_type(ss, parent_struct_type)?;

                if let ValueType::Pointer(points_to) = &array_type {
                    match index_type {
                        ValueType::I16 => {},
                        ValueType::U16 => {},
                        _ => return Err(SemanticErr::WrongType(ValueType::U16, index_type, index.get_first_lexeme()))
                    }

                    return Ok(*points_to.clone())
                }

                if let ValueType::Array(item_type, _) = array_type {
                    if !matches!(index_type, ValueType::U16) {
                        return Err(SemanticErr::WrongType(ValueType::U16, index_type, index.get_first_lexeme()))
                    }

                    return Ok(*item_type)
                }
                
                return Err(SemanticErr::NotAnArray(array_head.get_first_lexeme()))
            }

            Variable::StructField(s) => {
                let instance_head = &s.0;
                let instance_field = &s.1;

                let instance_type = instance_head.eval_type(ss, parent_struct_type)?;
                if let ValueType::CustomStruct(struct_name) = instance_type {
                    let head_type = ss.get_custom_struct(struct_name).unwrap();
                    return instance_field.eval_type(ss, Some(head_type))
                }

                return Err(SemanticErr::NotAStruct(instance_head.get_first_lexeme()))
            }
        }
    }
}