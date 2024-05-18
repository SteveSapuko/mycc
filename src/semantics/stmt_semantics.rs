use super::*;

impl Stmt {
    pub fn generate_typed_stmt(&self, ss: &mut ScopeStack) -> Result<TypedStmt, SemanticErr> {
        match self {
            Stmt::Block(body) => {
                let mut typed_body: Vec<TypedStmt> = vec![];

                for s in body {
                    typed_body.push(s.generate_typed_stmt(ss)?);
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
                if ss.all_used_ids().contains(&fn_name.data()) {
                    return Err(SemanticErr::UsedId(fn_name.clone()))
                }

                let typed_params = params.generate_typed_params(ss, None)?;
                let typed_ret_type = ValueType::from_declr(ret_type, &ss.defined_types)?;
                let typed_body = body.generate_typed_stmt(ss)?;

                let fn_template = FnTemplate {
                    name: fn_name.data(),
                    parameters: typed_params.items.iter().map(|x| x.1.clone()).collect::<Vec<ValueType>>(),
                    ret_type: typed_ret_type,
                };

                ss.declare_fn(fn_template.clone());

                return Ok(TypedStmt::FnDeclr(fn_template, Box::new(typed_body)))

            }

            //struct declaration is done before
            Stmt::StructDeclr(_, _) => Ok(TypedStmt::CustomTypeDeclr),


            
            _ => todo!()
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