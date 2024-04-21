use super::*;
use crate::semantics::*;
use crate::token::*;
use crate::types::*;
use crate::expr::*;
use crate::stmt::*;

use super::expr_cgen::*;

impl Stmt {
    pub fn cgen(&self, cg: &mut CodeGenerator) {
        match self {
            Stmt::ExprStmt(expr) => expr.cgen(cg),

            Stmt::VarDeclr(name, type_declr, value) => {
                let v_type = cg.get_type_from_declr(type_declr);
                cg.declare_var(name.data(), v_type.clone());

                match value {
                    Some(expr) => {
                        expr.cgen(cg);
                    }

                    None => {
                        cg.increase_sp_by(v_type.size(&cg.ss));
                    }
                }
            }

            _ => todo!("stmt cgen")
        }
    }
}