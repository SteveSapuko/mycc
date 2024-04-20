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

            Stmt::StructDeclr(_, _) => {}

            Stmt::FnDeclr(_, _, _, _) => {}

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

            Stmt::EnumDeclr(_, _) => {}
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