use super::*;
use crate::stmt::*;
use crate::expr::*;
use crate::types::*;
use crate::typed_ast::*;

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
                if ss.all_used_ids().contains(&name.data()) {
                    return Err(SemanticErr::UsedId(name.clone()))
                }

                let var_type = match ValueType::from_declr(type_declr, &ss.defined_types) {
                    Ok(t) => t,
                    Err(l) => return Err(SemanticErr::UnknownType(l))
                };

                if let Some(init_value) = value {
                    
                }
            }
        }
        
        ()
    }
}