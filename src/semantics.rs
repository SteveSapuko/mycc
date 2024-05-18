use crate::expr::*;
use crate::stmt::*;
use crate::token::*;
use crate::types::*;
use crate::typed_ast::*;

pub mod semantic_err;
mod stmt_semantics;
mod expr_semantics;

use semantic_err::*;

pub struct ScopeStack {
    stack: Vec<ScopeStackOp>,
    used_ids: Vec<String>,
    global_used_ids: Vec<String>,
    pub defined_types: Vec<CustomType>,
    pub defined_functions: Vec<FnTemplate>,
}

enum ScopeStackOp {
    EnterScope(Vec<String>),
    EnterBreakable,
    EnterReturnable(ValueType),
    DeclrVar(String, ValueType),
}

pub fn generate_typed_ast(ast: Vec<Stmt>) -> Result<Vec<TypedStmt>, SemanticErr> {
    let mut ss = ScopeStack::new();
    let mut typed_ast: Vec<TypedStmt> = vec![];

    for stmt in ast {
        typed_ast.push(stmt.generate_typed_stmt(&mut ss)?);
    }
    
    Ok(typed_ast)
}

//ScopeStack helper functions
impl ScopeStack {
    pub fn new() -> Self {
        ScopeStack {
            stack: vec![],
            used_ids: vec![],
            global_used_ids: vec![],
            defined_types: vec![],
            defined_functions: vec![],
        }
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
        if target_name == "void" {
            return Some(ValueType::Void)
        }
        
        for var in self.stack.iter().rev() {
            if let ScopeStackOp::DeclrVar(var_name, var_type) = var {
                if *var_name == target_name {
                    return Some(var_type.clone())
                }
            }
        }
        return None
    }

    pub fn get_fn_from_name(&self, target_name: String) -> Option<FnTemplate> {
        for func in self.defined_functions.iter() {
            if func.name == target_name {
                return Some(func.clone())
            }
        }

        return None
    }

    pub fn get_nearest_ret_type(&self) -> Option<ValueType> {
        for stack_op in self.stack.iter().rev() {
            if let ScopeStackOp::EnterReturnable(ret_type) = stack_op {
                return Some(ret_type.clone())
            }
        }

        return None
    }

    pub fn check_if_breakable(&self) -> bool {
        for stack_op in self.stack.iter().rev() {
            if let ScopeStackOp::EnterBreakable = stack_op {
                return true
            }
        }

        return false
    }

    pub fn get_custom_type_from_name(&self, target: Lexeme) -> Result<CustomType, SemanticErr> {
        let target_name = target.data();

        for t in &self.defined_types {
            if t.name() == target_name {
                return Ok(t.clone())
            }
        }

        Err(SemanticErr::UnknownType(target))
    }

    pub fn get_custom_struct(&self, target_name: String) -> Option<StructTemplate> {
        for t in &self.defined_types {
            if let CustomType::CustomStruct(s) = &t {
                if s.name == target_name {
                    return Some(s.clone())
                }
            }
        }

        None
    }

    pub fn declare_var(&mut self, name: String, t: ValueType) {
        self.used_ids.push(name.clone());
        self.stack.push(ScopeStackOp::DeclrVar(name, t));
    }

    pub fn declare_custom_type(&mut self, t: CustomType) {
        self.global_used_ids.push(t.name());
        self.defined_types.push(t);
    }

    pub fn declare_fn(&mut self, name: String, parameters: Vec<ValueType>, ret_type: ValueType) {
        self.global_used_ids.push(name.clone());
        self.defined_functions.push(FnTemplate { name, parameters, ret_type });
    }

    pub fn enter_breakable(&mut self) {
        self.stack.push(ScopeStackOp::EnterBreakable);
    }

    pub fn enter_returnable(&mut self, ret_type: ValueType) {
        self.stack.push(ScopeStackOp::EnterReturnable(ret_type));
    }

    pub fn all_used_ids(&self) -> Vec<String> {
        let mut temp: Vec<String> = vec![];
        temp.append(&mut self.used_ids.clone());
        temp.append(&mut self.global_used_ids.clone());

        temp
    }
}


#[derive(Clone, Debug)]
pub struct FnTemplate {
    pub name: String,
    pub parameters: Vec<ValueType>,
    pub ret_type: ValueType,
}


impl StructTemplate {
    pub fn check_recursive(&self, ss: &ScopeStack, iteration: u8) -> Result<(), String> {
        if iteration == 100 {
            return Err(self.name.clone())
        }
        
        for f in self.fields.iter() {
            //println!("field: {} type: {:#?}", f.0, f.1);
            if let ValueType::CustomStruct(s) = &f.1 {
                let child_struct = ss.get_custom_struct(s.clone()).expect("should have been caught earlier");
                child_struct.check_recursive(ss, iteration + 1)?;
            }
        }

        Ok(())
    }
}