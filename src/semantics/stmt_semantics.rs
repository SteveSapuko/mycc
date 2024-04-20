use super::*;

impl Stmt {
    pub fn check_semantics(&self, ss: &mut ScopeStack) -> Result<(), SemanticErr> {
        match self {
            Stmt::ExprStmt(e) => {
                e.eval_type(ss)?;
            }

            Stmt::Block(body) => {
                ss.enter_scope();

                define_scope(ss, body)?;

                for s in body {
                    s.check_semantics(ss)?;
                }

                ss.leave_scope();
            }

            Stmt::VarDeclr(name, type_declr, value) => {
                if ss.used_ids.contains(&name.data()) {
                    return Err(SemanticErr::UsedId(name.clone()))
                }
                
                let var_type = match ValueType::from_declr(&type_declr, ss) {
                    Ok(t) => t,
                    Err(_) => return Err(SemanticErr::UnknownType(type_declr.get_id()))
                };

                ss.declare_var(name.data(), var_type.clone());

                if let Some(v) = value {
                    let value_type = v.eval_type(ss)?;
                    if value_type != var_type {
                        return Err(SemanticErr::WrongType(var_type, value_type, name.clone()))
                    }
                }
            }

            Stmt::StructDeclr(name, fields) => {
                
            }

            Stmt::FnDeclr(name, params, ret_type, body) => {
                let name_as_string = name.data();
                if ss.used_ids.contains(&name_as_string) {
                    return Err(SemanticErr::UsedId(name.clone()))
                }


                let mut param_names: Vec<String> = vec![];
                let mut param_types: Vec<ValueType> = vec![];

                for p in params.params.iter() {
                    let p_type = match ValueType::from_declr(&p.1, ss) {
                        Ok(t) => t,
                        Err(l) => return Err(SemanticErr::UnknownType(l))
                    };

                    if param_names.contains(&p.0.data()) {
                        return Err(SemanticErr::FnDuplicateParams(p.0.clone()))
                    }

                    param_names.push(p.0.data());
                    param_types.push(p_type);
                }

                let checked_ret_type = match ValueType::from_declr(ret_type, ss) {
                    Ok(t) => t,
                    Err(l) => return Err(SemanticErr::UnknownType(l))
                };

                ss.declare_fn(name_as_string, param_types.clone(), checked_ret_type.clone());

                let checked_params: Vec<(String, ValueType)> = param_names.into_iter().zip(param_types.into_iter()).collect();
                
                body.check_fn_body_semantics(checked_params, checked_ret_type, ss)?;
            }

            Stmt::ReturnStmt(op, value) => {
                let ret_type = match ss.get_nearest_ret_type() {
                    Some(t) => t,
                    None => return Err(SemanticErr::CantReturn(op.clone()))
                };

                let value_type = value.eval_type(ss)?;

                if ret_type != value_type {
                    return Err(SemanticErr::WrongType(ret_type, value_type, op.clone()))
                }
            }

            Stmt::BreakStmt(lexeme) => {
                if !ss.check_if_breakable() {
                    return Err(SemanticErr::CantBreak(lexeme.clone()))
                }
            }

            Stmt::IfStmt(cond, t_branch, f_branch) => {
                let cond_type = cond.eval_type(ss)?;

                if !matches!(cond_type, ValueType::U8) {
                    return Err(SemanticErr::WrongType(ValueType::U8, cond_type, cond.get_first_lexeme()))
                }

                t_branch.check_semantics(ss)?;

                if let Some(f) = f_branch {
                    f.check_semantics(ss)?;
                }
            }

            Stmt::LoopStmt(body) => {
                body.check_breakable_semantics(ss)?;
            }

            Stmt::WhileStmt(cond, body) => {
                let cond_type = cond.eval_type(ss)?;

                if !matches!(cond_type, ValueType::U8) {
                    return Err(SemanticErr::WrongType(ValueType::U8, cond_type, cond.get_first_lexeme()))
                }

                body.check_breakable_semantics(ss)?;
            }

            Stmt::EnumDeclr(_, _) => {
                
            }
        }
        
        Ok(())
    }

    pub fn check_fn_body_semantics(&self, args: Vec<(String, ValueType)>, ret_type: ValueType, ss: &mut ScopeStack) -> Result<(), SemanticErr>{
        if let Stmt::Block(body) = self {
            ss.enter_scope();
            ss.enter_returnable(ret_type);

            for a in args {
                ss.declare_var(a.0, a.1);
            }

            for s in body {
                s.check_semantics(ss)?;
            }

            ss.leave_scope();

            return Ok(())

        }

        unreachable!("FnDeclr only contains Stmt::Block")
    }

    pub fn check_breakable_semantics(&self, ss: &mut ScopeStack) -> Result<(), SemanticErr> {
        if let Stmt::Block(body) = self {
            ss.enter_scope();
            ss.enter_breakable();

            for s in body {
                s.check_semantics(ss)?;
            }

            ss.leave_scope();

            return Ok(())
        }

        unreachable!("Loops only contains Stmt::Block")
    }
}