use crate::expr::*;
use crate::stmt::*;
use crate::token::*;
use crate::types::*;

pub mod semantic_err;
mod stmt_semantics;
mod expr_semantics;

use semantic_err::*;

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
    DeclrFn(FnTemplate),
}

pub fn check_semantics(ast: &Vec<Stmt>) -> Result<(), SemanticErr> {
    let mut ss = ScopeStack::new();

    let program = Stmt::Block(ast.clone());

    program.check_semantics(&mut ss)?;

    Ok(())
}

fn define_scope(ss: &mut ScopeStack, ast: &Vec<Stmt>) -> Result<(), SemanticErr> {
    //define enums first
    for s in ast {
        if let Stmt::EnumDeclr(name, variants) = s {
            if ss.used_ids.contains(&name.data()) {
                return Err(SemanticErr::UsedId(name.clone()))
            }

            let mut checked_variants: Vec<String> = vec![];

            for v in variants {
                if checked_variants.contains(&v.data()) {
                    return Err(SemanticErr::EnumDuplicateVariants(v.clone()))
                }
                checked_variants.push(v.data());
            }

            ss.declare_custom_type(CustomType::CustomEnum(CustomEnum { name: name.data(), variants: checked_variants }))
        }
    }

    let mut unfinished_structs: Vec<UnfinishedStruct> = vec![];
    let mut unresolved_ids: Vec<Lexeme> = vec![];

    //first pass of structs
    for s in ast {
        if let Stmt::StructDeclr(name, fields) = s {
            if ss.used_ids.contains(&name.data()) {
                return Err(SemanticErr::UsedId(name.clone()))
            }

            let mut field_names: Vec<Lexeme> = vec![];
            let mut field_types: Vec<MaybeType> = vec![];
            let mut needs_rechecking = false;

            for f in fields.params.iter() {
                let f_name = &f.0;
                let f_type_declr = &f.1;

                let field_names_as_strings: Vec<String> = field_names.iter().map(|x| x.data()).collect();

                if field_names_as_strings.contains(&f_name.data()) {
                    return Err(SemanticErr::StructDuplicateFields(name.clone(), f_name.clone()))
                }

                field_names.push(f_name.clone());
                
                let f_type = match ValueType::from_declr(f_type_declr, ss) {
                    Ok(t) => MaybeType::Resolved(t),
                    Err(id) => {
                        needs_rechecking = true;
                        unresolved_ids.push(id);
                        MaybeType::Unresolved(f_type_declr.clone())
                    }
                };
                
                field_types.push(f_type);
            }

            if needs_rechecking {
                let struct_fields: Vec<(Lexeme, MaybeType)> = field_names.into_iter().zip(field_types.into_iter()).collect();
                unfinished_structs.push(UnfinishedStruct { name: name.clone(), fields: struct_fields });
            }
            else {
                let field_names_as_strings: Vec<String> = field_names.iter().map(|x| x.data()).collect();
                let struct_fields: Vec<(String, ValueType)> =
                field_names_as_strings.into_iter().zip(field_types.into_iter().map(|x| x.unwrap())).collect();

                ss.declare_custom_type(CustomType::CustomStruct(StructTemplate { name: name.data(), fields: struct_fields }))
            }
        }
    }

    //println!("unfinished structs: {:#?}\nunresolved ids: {:#?}", &unfinished_structs, &unresolved_ids);

    //check that all of the unresolved types are being defined right now
    for id in unresolved_ids.iter() {
        if !unfinished_structs.iter().map(|x| x.name.data()).collect::<Vec<String>>()
        .contains(&id.data())
        && ss.get_custom_struct(id.data()).is_none() {
            return Err(SemanticErr::UnknownType(id.clone()))
        }
    }

    let mut finished_structs: Vec<StructTemplate> = vec![];

    for s in unfinished_structs.iter() {
        let mut fields: Vec<(String, ValueType)> = vec![];
        for field in s.fields.iter() {
            match &field.1 {
                MaybeType::Resolved(t) => fields.push((field.0.data(), t.clone())),
                
                MaybeType::Unresolved(declr) => {
                    if let TypeDeclr::Basic(id) = declr {
                        if id.data() == s.name.data() {
                            return Err(SemanticErr::RecursiveStruct(s.name.data()))
                        }
                    }
                    
                    let field_type = ValueType::from_declr_new_struct(&declr);
                    fields.push((field.0.data(), field_type));
                }
            }
        }

        finished_structs.push(StructTemplate { name: s.name.data(), fields })
    }



    for s in &finished_structs {
        ss.declare_custom_type(CustomType::CustomStruct(s.clone()));
    }

    //check for infinite struct recursion
    for s in finished_structs {
        match s.check_recursive(ss, 0) {
            Ok(_) => {},
            Err(name) => {
                return Err(SemanticErr::RecursiveStruct(name))
            }
        }
    }

    //define functions
    for s in ast {
        if let Stmt::FnDeclr(name, params, ret_type, body) = s {
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
        for stack_op in self.stack.iter().rev() {
            if let ScopeStackOp::DeclrFn(fn_template) = stack_op {
                if fn_template.name == target_name {
                    return Some(fn_template.clone())
                }
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

    pub fn defined_types(&self) -> Vec<CustomType> {
        let mut defined_types: Vec<CustomType> = vec![];

        for op in self.stack.iter().rev() {
            if let ScopeStackOp::DeclrCustomType(custom) = op {
                defined_types.push(custom.clone());
            }
        }

        defined_types
    }

    pub fn get_custom_struct(&self, target_name: String) -> Option<StructTemplate> {
        for t in &self.defined_types() {
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
        self.used_ids.push(t.name());
        self.stack.push(ScopeStackOp::DeclrCustomType(t));
    }

    pub fn declare_fn(&mut self, name: String, parameters: Vec<ValueType>, ret_type: ValueType) {
        self.used_ids.push(name.clone());
        self.stack.push(ScopeStackOp::DeclrFn(FnTemplate { name, parameters, ret_type }));
    }

    pub fn enter_breakable(&mut self) {
        self.stack.push(ScopeStackOp::EnterBreakable);
    }

    pub fn enter_returnable(&mut self, ret_type: ValueType) {
        self.stack.push(ScopeStackOp::EnterReturnable(ret_type));
    }
}


#[derive(Clone, Debug)]
pub struct FnTemplate {
    pub name: String,
    pub parameters: Vec<ValueType>,
    pub ret_type: ValueType,
}