use std::thread::Scope;

use super::*;

impl Expr {
    pub fn eval_type(&self, ss: &mut ScopeStack) -> Result<ValueType, SemanticErr> {
        match self {
            Expr::Primary(p) => {
                match &**p {
                    PrimaryExpr::NumLiteral(n) => {
                        return Ok(n.get_type())
                    }

                    PrimaryExpr::EnumVariant(name, variant) => {
                        let type_by_name = ss.get_type_from_string(name.clone())?;

                        if let CustomType::CustomEnum(en) = &type_by_name {
                            if en.variants.contains(&variant.data()) {
                                return Ok(ValueType::CustomType(CustomType::CustomEnum(en.clone())))
                            }
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
                        todo!()
                    }
                }
            }

            _ => todo!()
        }
    }
}

impl Variable {
    /*pub fn eval_head_type(&self, ss: &ScopeStack) -> Result<ValueType, SemanticErr> {
        match self {
            Variable::Array(arr, index) => {
                let index_type = index.eval_type(ss)?;
                if !matches!(index_type, ValueType::U16) {
                    return Err(SemanticErr::WrongType(ValueType::U16, index_type))
                }

                //if array head is just an ID
                if let Variable::Id(arr_name) = &**arr {   
                    let arr_item_type = match ss.get_var_type_from_name(arr_name.data()) {
                        Some(t) => t,
                        None => return Err(SemanticErr::UndeclaredVar(arr_name.clone()))
                    };

                    return Ok(arr_item_type)
                }

                //if array head is more complicated
                let head_array_type = match ss.get_var_type_from_name(arr.get_first_id().data()) {
                    Some(t) => t,
                    None => return Err(SemanticErr::UndeclaredVar(arr.get_first_id()))
                };

                return Ok(arr.eval_child_type(ss, head_array_type)?)
            }

            Variable::StructField(s) => {
                let instance_field = &s.1;
                
                // if struct head is just an ID
                if let Variable::Id(instance_name) = &s.0 {
                    let struct_type = match ss.get_var_type_from_name(instance_name.data()) {
                        Some(t) => t,
                        None => return Err(SemanticErr::UndeclaredVar(instance_name.clone()))
                    };

                    return Ok(instance_field.eval_child_type(ss, struct_type)?)
                }

                //stuct head is more complicated

                let struct_head = &s.0;
                let head_type = match ss.get_var_type_from_name(struct_head.get_first_id().data()) {
                    Some(t) => t,
                    None => return Err(SemanticErr::UndeclaredVar(struct_head.get_first_id()))
                };

                return Ok(instance_field.eval_child_type(ss, head_type)?)
            }

            Variable::Id(var_name) => {
                return match ss.get_var_type_from_name(var_name.data()) {
                    Some(t) => Ok(t),
                    None => Err(SemanticErr::UndeclaredVar(var_name.clone()))
                }
            }
        }
    }

    pub fn eval_child_type(&self, ss: &ScopeStack, parent: ValueType) -> Result<ValueType, SemanticErr> {
        match self {
            Variable::Id(lexeme) => {
                match parent {
                    ValueType::CustomType(custom) => {
                        if let CustomType::CustomStruct(parent_struct) = custom {
                            match parent_struct.get_field_type(lexeme) {
                                Some(t) => return Ok(t),
                                None => return Err(SemanticErr::NoStructField(parent_struct.clone(), lexeme.clone()))
                            }
                        }

                        return Err(SemanticErr::NotAStruct(lexeme.clone()))
                    }

                    ValueType::Array(item_type, _) => {
                        return Ok(*item_type.clone())
                    }

                    _ => return Err(SemanticErr::AccessError(lexeme.clone()))
                }
            }

            Variable::Array(array_head, index) => {
                let index_type = index.eval_type(ss)?;
                if !matches!(index_type, ValueType::U16) {
                    return Err(SemanticErr::WrongType(ValueType::U16, index_type))
                }


            }
        }
    }*/

    pub fn eval_type(&self, ss: &mut ScopeStack, parent_struct_type: Option<CustomStruct>) -> Result<ValueType, SemanticErr> {
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
                if !matches!(index_type, ValueType::U16) {
                    return Err(SemanticErr::WrongType(ValueType::U16, index_type))
                }

                
                let array_type = array_head.eval_type(ss, parent_struct_type)?;
                if let ValueType::Array(item_type, _) = array_type {
                    return Ok(*item_type)
                }
                return Err(SemanticErr::NotAnArray(array_head.get_first_id()))
            }

            Variable::StructField(s) => {
                let instance_head = &s.0;
                let instance_field = &s.1;

                let instance_type = instance_head.eval_type(ss, parent_struct_type)?;
                if let ValueType::CustomType(CustomType::CustomStruct(head_type)) = instance_type {
                    return instance_field.eval_type(ss, Some(head_type))
                }

                return Err(SemanticErr::NotAStruct(instance_head.get_first_id()))
            }
        }
    }
}