use super::*;

impl Expr {
    pub fn generate_typed_expr(&self, ss: &ScopeStack) -> Result<TypedExpr, SemanticErr> {
        match self {
            Expr::Primary(p) => {
                let typed_primary = p.generate_typed_expr(ss)?;
                
                return Ok(TypedExpr::Primary(typed_primary.final_type(), Box::new(typed_primary)))
            }

            Expr::Cast(casted_expr, op_location, declr_to_type) => {
                let to_type = ValueType::from_declr(declr_to_type, &ss.defined_types)?;

                let typed_casted_expr = casted_expr.generate_typed_expr(ss)?;

                //TODO make sure only certain types can be cast

                return Ok(TypedExpr::Cast(to_type, Box::new(typed_casted_expr)))
            }

            Expr::Comparison(binary_expr) => {
                let typed_binary_expr = binary_expr.generate_typed_binary_expr(ss)?;

                //u8 acts as bool
                return Ok(TypedExpr::Comparison(ValueType::U8, Box::new(typed_binary_expr)))
            }

            Expr::Equality(binary_expr) => {
                let typed_binary_expr = binary_expr.generate_typed_binary_expr(ss)?;

                //u8 acts as bool
                return Ok(TypedExpr::Equality(ValueType::U8, Box::new(typed_binary_expr)))
            }

            Expr::Term(binary_expr) => {
                let typed_binary_expr = binary_expr.generate_typed_binary_expr(ss)?;

                return Ok(TypedExpr::Term(typed_binary_expr.left.final_type(), Box::new(typed_binary_expr)))
            }

            Expr::Unary(op, expr) => {
                let typed_expr = expr.generate_typed_expr(ss)?;

                if !typed_expr.final_type().is_primitive_type() {
                    return Err(SemanticErr::CantOp(op.clone()))
                }

                return Ok(TypedExpr::Unary(typed_expr.final_type(), op.data(), Box::new(typed_expr)))
            }

            Expr::Assign(assign) => {
                let typed_left = assign.left.generate_typed_expr(ss)?;
                let typed_right = assign.right.generate_typed_expr(ss)?;

                if !typed_left.is_assignable() {
                    return Err(SemanticErr::CantOp(assign.operator.clone()))
                }

                let left_type = typed_left.final_type();
                let right_type = typed_right.final_type();

                if left_type != right_type {
                    return Err(SemanticErr::WrongType(left_type, right_type, assign.operator.clone()))
                }

                return Ok(TypedExpr::Assign(left_type, Box::new(typed_left), Box::new(typed_right)))
            }
        
            Expr::FnCall(fn_name, args) => {
                let fn_template = match ss.get_fn_from_name(fn_name.data()) {
                    Some(t) => t,
                    None => return Err(SemanticErr::UndeclaredFn(fn_name.clone()))
                };

                let typed_args = args.generate_typed_args(ss)?;

                if fn_template.parameters.len() != typed_args.items.len() {
                    return Err(SemanticErr::FnArityErr(fn_name.clone()))
                }

                for template_param in fn_template.parameters.iter().enumerate() {
                    let (param_n, param_type) = template_param;

                    if typed_args.items[param_n].final_type() != *param_type {
                        return Err(SemanticErr::WrongType(param_type.clone(), typed_args.items[param_n].final_type(), fn_name.clone()))
                    }
                }

                return Ok(TypedExpr::FnCall(fn_template.ret_type, fn_name.data(), typed_args))
            }

            Expr::Shift(value, op, num) => {
                let typed_value = value.generate_typed_expr(ss)?;
                let value_type = typed_value.final_type();
                
                if !value_type.is_primitive_type() {
                    return Err(SemanticErr::CantOp(op.clone()))
                }

                if let NumLiteral::U8(shift_amount) = num {
                    if *shift_amount > (value_type.size(&ss.defined_types) * 8) as u8 { //size in bits
                        return Err(SemanticErr::ShiftAmountErr(op.clone()))
                    }

                    return Ok(TypedExpr::Shift(value_type, Box::new(typed_value), op.data(), num.clone()))
                } else {
                    return Err(SemanticErr::WrongType(ValueType::U8, num.get_type(), op.clone()))
                }
            }
        }
    }
}

impl PrimaryExpr {
    pub fn generate_typed_expr(&self, ss: &ScopeStack) -> Result<TypedPrimaryExpr, SemanticErr> {
        match self {
            PrimaryExpr::NumLiteral(num, _) => {
                return Ok(TypedPrimaryExpr::NumLiteral(num.clone()))
            }

            PrimaryExpr::Grouping(body) => {
                let typed_body = body.generate_typed_expr(ss)?;
                return Ok(TypedPrimaryExpr::Grouping(typed_body))
            }

            PrimaryExpr::EnumVariant(enum_name, enum_variant) => {
                let enum_template = ss.get_custom_type_from_name(enum_name.clone())?;

                if let CustomType::CustomEnum(template) = &enum_template {
                    let variant_number = match template.get_variant(enum_variant.data()) {
                        Some(t) => t,
                        None => return Err(SemanticErr::NoEnumVariant(template.clone(), enum_variant.clone()))
                    };

                    return Ok(TypedPrimaryExpr::EnumVariant(template.clone(), (enum_variant.data(), variant_number)))
                    
                }

                return Err(SemanticErr::WrongAccess(enum_template, enum_name.clone()))
            }
        
            PrimaryExpr::Variable(var) => {
                let typed_var = var.generate_typed_variable(ss, None)?;
                return Ok(TypedPrimaryExpr::Variable(typed_var))
            }

            PrimaryExpr::Ref(ref_op, var) => {
                let typed_var = var.generate_typed_variable(ss, None)?;
                let var_type = typed_var.final_type();

                match ref_op.data().as_str() {
                    "&" => {
                        return Ok(TypedPrimaryExpr::Ref(
                            ValueType::Pointer(Box::new(var_type)),
                            ref_op.data(),
                            typed_var))
                    }

                    "*" => {
                        match var_type.dereference() {
                            None => {
                                return Err(SemanticErr::CantDeref(ref_op.clone()))
                            }

                            Some(derefed_type) => {
                                return Ok(TypedPrimaryExpr::Ref(
                                    derefed_type,
                                    ref_op.data(),
                                    typed_var))
                            }
                        }
                    }

                    _ => unreachable!()
                }
            }
        }
    }
}

impl Variable {
    pub fn generate_typed_variable(&self, ss: &ScopeStack, parent: Option<StructTemplate>) -> Result<TypedVariable, SemanticErr> {
        match self {
            Variable::Id(id) => {
                match parent {
                    None => {
                        let id_type = match ss.get_var_type_from_name(id.data()) {
                            Some(t) => t,
                            None => {
                                return Err(SemanticErr::UndeclaredVar(id.clone()))
                            }
                        };

                        return Ok(TypedVariable::Id(id_type, id.data()))
                    }

                    Some(parent_template) => {
                        let id_type = match parent_template.get_field(id.data()) {
                            Some((t, _)) => t,
                            None => {
                                return Err(SemanticErr::NoStructField(parent_template, id.clone()))
                            }
                        };

                        return Ok(TypedVariable::Id(id_type, id.data()))
                    }
                }
            }

            Variable::StructField(s) => {
                let (head, tail) = &**s;

                let typed_head = head.generate_typed_variable(ss, parent)?;
                let head_type = typed_head.final_type();

                if let ValueType::CustomStruct(struct_name) = head_type {
                    let head_template = get_custom_struct(struct_name, &ss.defined_types).unwrap();

                    let typed_tail = tail.generate_typed_variable(ss, Some(head_template))?;

                    return Ok(TypedVariable::StructField(
                        typed_tail.final_type(),
                        Box::new((typed_head, typed_tail))
                    ))
                }

                return Err(SemanticErr::NotAStruct(tail.get_first_lexeme()))
            }
        
            Variable::Array(arr_name, index) => {
                let typed_array_head = arr_name.generate_typed_variable(ss, parent)?;
                let array_type = typed_array_head.final_type();

                let head_item_type = match array_type {
                    ValueType::Array(item_type, _) => *item_type,

                    ValueType::Pointer(to_type) => *to_type,

                    _ => return Err(SemanticErr::NotAnArray(arr_name.get_first_lexeme()))
                };


                let typed_index = index.generate_typed_expr(ss)?;

                return Ok(TypedVariable::Array(head_item_type, Box::new(typed_array_head), typed_index))
            }
            
        }
    }
}

impl BinaryExpr {
    pub fn generate_typed_binary_expr(&self, ss: &ScopeStack) -> Result<TypedBinaryExpr, SemanticErr> {
        let typed_left = self.left.generate_typed_expr(ss)?;
        let typed_right = self.right.generate_typed_expr(ss)?;

        if typed_left.final_type() != typed_right.final_type() {
            return Err(SemanticErr::WrongType(typed_left.final_type(), typed_right.final_type(), self.operator.clone()))
        }

        if !typed_left.final_type().is_primitive_type() {
            return Err(SemanticErr::CantOp(self.operator.clone()))
        }

        Ok(TypedBinaryExpr {
            left: typed_left,
            operator: self.operator.data(),
            right: typed_right })
    }
}

impl Args {
    pub fn generate_typed_args(&self, ss: &ScopeStack) -> Result<TypedArgs, SemanticErr> {
        let mut typed_items: Vec<TypedExpr> = vec![];
        
        for arg in &self.items {
            typed_items.push(arg.generate_typed_expr(ss)?);
        }
        
        Ok(TypedArgs { items: typed_items })
    }
}