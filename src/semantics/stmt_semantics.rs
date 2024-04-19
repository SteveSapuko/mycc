use super::*;

impl Stmt {
    pub fn check_semantics(&self, ss: &mut ScopeStack) -> Result<(), SemanticErr> {
        match self {
            Stmt::ExprStmt(e) => {
                e.eval_type(ss)?;
            }

            Stmt::VarDeclr(name, type_declr, value) => {
                if ss.used_ids.contains(&name.data()) {
                    return Err(SemanticErr::UsedId(name.clone()))
                }
                
                let var_type = match ValueType::from_declr(&type_declr, &ss.defined_types()) {
                    Ok(t) => t,
                    Err(_) => return Err(SemanticErr::UnknownType(type_declr.get_id()))
                };

                ss.declare_var(name.data(), var_type.clone());

                if let Some(v) = value {
                    let value_type = v.eval_type(ss)?;
                    if value_type != var_type {
                        return Err(SemanticErr::WrongType(var_type, value_type))
                    }
                }
            }

            Stmt::StructDeclr(name, fields) => {
                if ss.used_ids.contains(&name.data()) {
                    return Err(SemanticErr::UsedId(name.clone()))
                }

                let mut field_names: Vec<String> = vec![];
                let mut field_types: Vec<ValueType> = vec![];
                
                //TODO allow forward declaration
                for f in fields.params.iter() {
                    let f_name = &f.0;
                    let f_type_declr = &f.1;

                    if field_names.contains(&f_name.data()) {
                        return Err(SemanticErr::StructDuplicateFields(name.clone(), f_name.clone()))
                    }

                    field_names.push(f_name.data());
                    
                    let f_type = match ValueType::from_declr(f_type_declr, &ss.defined_types()){
                        Ok(t) => t,
                        Err(l) => return Err(SemanticErr::UnknownType(l))
                    };
                    
                    field_types.push(f_type);
                }

                let mut struct_fields: Vec<(String, ValueType)> = vec![];

                for f_name in field_names.into_iter().enumerate() {
                    let i = f_name.0;
                    let f_name = f_name.1;

                    struct_fields.push((f_name, field_types[i].clone()))
                }
                

                ss.declare_custom_type(CustomType::CustomStruct(CustomStruct { name: name.data(), fields: struct_fields }))
            }

            _ => panic!("stmt_semantics")
        }
        
        Ok(())
    }
}