use super::*;

impl Stmt {
    pub fn generate_typed_stmt(&self, ss: &mut ScopeStack, in_local_scope: bool) -> Result<TypedStmt, SemanticErr> {
        match self {
            Stmt::Block(body) => {
                let mut typed_body: Vec<TypedStmt> = vec![];

                for s in body {
                    typed_body.push(s.generate_typed_stmt(ss, true)?);
                }

                return Ok(TypedStmt::Block(typed_body))
            }

            Stmt::VarDeclr(name, type_declr, value) => {
                if ss.global_used_ids.contains(&name.data()) {
                    return Err(SemanticErr::UsedId(name.clone()))
                }

                let var_type = ValueType::from_declr(type_declr, &ss.defined_types)?;

                let final_init_value: Option<TypedExpr>;

                if let Some(init_expr) = value {
                    let typed_init_expr = init_expr.generate_typed_expr(ss)?;
                    let init_value_type = typed_init_expr.final_type();

                    if init_value_type != var_type {
                        return Err(SemanticErr::WrongType(var_type, init_value_type, name.clone()))
                    }
                    
                    final_init_value = Some(typed_init_expr);
                } else {
                    final_init_value = None;
                }

                ss.declare_var(name.data(), var_type.clone());

                return Ok(TypedStmt::VarDeclr(name.data(), var_type, final_init_value))
            }

            Stmt::ExprStmt(expr) => {
                return Ok(TypedStmt::ExprStmt(expr.generate_typed_expr(ss)?))
            }

            Stmt::FnDeclr(fn_name, params, ret_type, body) => {
                if in_local_scope {
                    return Err(SemanticErr::CantDeclareThisInLocalScope(fn_name.clone()))
                }
                
                if ss.all_used_ids().contains(&fn_name.data()) {
                    return Err(SemanticErr::UsedId(fn_name.clone()))
                }

                let typed_params = params.generate_typed_params(ss, None)?;
                let typed_ret_type = ValueType::from_declr(ret_type, &ss.defined_types)?;
                
                ss.enter_returnable(typed_ret_type.clone());
                let typed_body = body.generate_typed_stmt(ss, true)?;

                let fn_template = FnTemplate {
                    name: fn_name.data(),
                    parameters: typed_params.items.iter().map(|x| x.1.clone()).collect::<Vec<ValueType>>(),
                    ret_type: typed_ret_type,
                };

                ss.declare_fn(fn_template.clone());

                return Ok(TypedStmt::FnDeclr(fn_template, Box::new(typed_body)))

            }

            //struct declaration is done before
            Stmt::StructDeclr(struct_name, _) => {
                if in_local_scope {
                    return Err(SemanticErr::CantDeclareThisInLocalScope(struct_name.clone()))
                }

                Ok(TypedStmt::CustomTypeDeclr)
            },

            Stmt::EnumDeclr(name, _) => {
                if in_local_scope {
                    return Err(SemanticErr::CantDeclareThisInLocalScope(name.clone()))
                }

                Ok(TypedStmt::CustomTypeDeclr)
            }

            Stmt::ReturnStmt(op, expr) => {
                let nearest_ret_type = match ss.get_nearest_ret_type() {
                    Some(t) => t,
                    None => return Err(SemanticErr::CantReturn(op.clone()))
                };

                let typed_expr = expr.generate_typed_expr(ss)?;

                if typed_expr.final_type() != nearest_ret_type {
                    return Err(SemanticErr::WrongType(nearest_ret_type, typed_expr.final_type(), op.clone()))
                }

                return Ok(TypedStmt::ReturnStmt(typed_expr))
            }

            Stmt::BreakStmt(op) => {
                if !ss.check_if_breakable() {
                    return Err(SemanticErr::CantBreak(op.clone()))
                }

                return Ok(TypedStmt::BreakStmt)
            }

            Stmt::IfStmt(condition, t_branch, f_branch) => {
                let typed_condition = condition.generate_typed_expr(ss)?;
                
                if typed_condition.final_type() != ValueType::U8 {
                    return Err(SemanticErr::WrongType(ValueType::U8, typed_condition.final_type(), condition.get_first_lexeme()))
                }

                let typed_t_branch = t_branch.generate_typed_stmt(ss, true)?;
                
                let typed_f_branch: Option<Box<TypedStmt>>;

                if let Some(f_body) = f_branch {
                    let typed_f_body = f_body.generate_typed_stmt(ss, true)?;

                    typed_f_branch = Some(Box::new(typed_f_body));
                } else {
                    typed_f_branch = None;
                }

                return Ok(TypedStmt::IfStmt(typed_condition, Box::new(typed_t_branch), typed_f_branch))
            }

            Stmt::WhileStmt(cond, body) => {
                let typed_cond = cond.generate_typed_expr(ss)?;

                if typed_cond.final_type() != ValueType::U8 {
                    return Err(SemanticErr::WrongType(ValueType::U8, typed_cond.final_type(), cond.get_first_lexeme()))
                }

                let typed_body = body.generate_typed_stmt(ss, true)?;

                return Ok(TypedStmt::WhileStmt(typed_cond, Box::new(typed_body)))
            }

            Stmt::LoopStmt(body) => {
                let typed_body = body.generate_typed_stmt(ss, true)?;
                return Ok(TypedStmt::LoopStmt(Box::new(typed_body)))
            }
        }
    
    }
}

impl Parameters {
    pub fn generate_typed_params(&self, ss: &ScopeStack, being_defined: Option<&Vec<String>>) -> Result<TypedParameters, SemanticErr> {
        let mut used_param_names: Vec<String> = vec![];
        let mut typed_params: Vec<(String, ValueType)> = vec![];

        for param in &self.params {
            let (param_name, type_declr) = param;

            if used_param_names.contains(&param_name.data()) {
                return Err(SemanticErr::DuplicateParams(param_name.clone()))
            }
            used_param_names.push(param_name.data());

            let param_type = match being_defined {
                Some(t) => ValueType::from_declr_new_struct(&type_declr, &ss.defined_types, t)?,

                None => ValueType::from_declr(&type_declr, &ss.defined_types)?
            };
                

            typed_params.push((param_name.data(), param_type));
        }
        
        Ok(TypedParameters {items: typed_params})
    }
}