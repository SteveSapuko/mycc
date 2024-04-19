use crate::expr::*;
use crate::stmt::*;
use crate::token::*;
use crate::types::*;

pub mod semantic_err;
mod stmt_semantics;
mod expr_semantics;

use semantic_err::*;
use stmt_semantics::*;
use expr_semantics::*;

pub struct ScopeStack {
    stack: Vec<ScopeStackOp>,
    used_ids: Vec<String>,
}

enum ScopeStackOp {
    EnterScope(Vec<String>),
    EnterBreakable,
    EnterReturnable(ValueType),
    DeclrVar(String, ValueType),
    DeclrCustomType(CustomType),
    DeclrFn(String, ValueType),
}

pub fn check_semantics(ast: &Vec<Stmt>) -> Result<(), SemanticErr> {
    let mut ss = ScopeStack::new();

    for s in ast {
        s.check_semantics(&mut ss)?;
    }

    Ok(())
}

//ScopeStack helper functions
impl ScopeStack {
    pub fn new() -> Self {
        ScopeStack { stack: vec![], used_ids: vec![] }
    }
    
    pub fn enter_scope(&mut self) {
        self.stack.push(ScopeStackOp::EnterScope(self.used_ids.clone()));
        self.used_ids.clear();
    }

    pub fn leave_scope(&mut self) {
        let mut ptr = self.stack.len() - 1;
        
        while !matches!(self.stack[ptr], ScopeStackOp::EnterScope(_)) {
            self.stack.pop();
            ptr -= 1;
        }

        if let ScopeStackOp::EnterScope(ids) = &self.stack[ptr] {
            self.used_ids = ids.clone();
            return
        }

        unreachable!()
    }

    pub fn get_var_type_from_name(&self, target_name: String) -> Option<ValueType> {
        for var in self.stack.iter().rev() {
            if let ScopeStackOp::DeclrVar(var_name, var_type) = var {
                if *var_name == target_name {
                    return Some(var_type.clone())
                }
            }
        }
        return None
    }

    pub fn get_type_from_string(&self, target: Lexeme) -> Result<CustomType, SemanticErr> {
        let defined_types = self.defined_types();
        let target_name = target.data();

        for t in defined_types {
            if t.name() == target_name {
                return Ok(t)
            }
        }

        Err(SemanticErr::UnknownType(target))
    }

    fn defined_types(&self) -> Vec<CustomType> {
        let mut defined_types: Vec<CustomType> = vec![];

        for op in self.stack.iter().rev() {
            if let ScopeStackOp::DeclrCustomType(custom) = op {
                defined_types.push(custom.clone());
            }
        }

        defined_types
    }

    pub fn declare_var(&mut self, name: String, t: ValueType) {
        self.used_ids.push(name.clone());
        self.stack.push(ScopeStackOp::DeclrVar(name, t));
    }

    pub fn declare_custom_type(&mut self, t: CustomType) {
        self.used_ids.push(t.name());
        self.stack.push(ScopeStackOp::DeclrCustomType(t));
    }

    pub fn declare_fn(&mut self, name: String, ret_type: ValueType) {
        self.used_ids.push(name.clone());
        self.stack.push(ScopeStackOp::DeclrFn(name, ret_type));
    }

    pub fn enter_breakable(&mut self) {
        self.stack.push(ScopeStackOp::EnterBreakable);
    }

    pub fn enter_returnable(&mut self, ret_type: ValueType) {
        self.stack.push(ScopeStackOp::EnterReturnable(ret_type));
    }
}