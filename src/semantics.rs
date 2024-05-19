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

pub fn generate_typed_ast(ast: Vec<Stmt>) -> Result<(Vec<TypedStmt>, Vec<CustomType>), SemanticErr> {
    let mut ss = ScopeStack::new();
    let mut typed_ast: Vec<TypedStmt> = vec![];

    //generating custom types
    let mut being_defined: Vec<String> = vec![];

    //first pass, defining all enums
    for stmt in ast.iter() {
        if let Stmt::EnumDeclr(enum_name, variants) = stmt {
            if ss.global_used_ids.contains(&enum_name.data()) {
                return Err(SemanticErr::UsedId(enum_name.clone()))
            }

            let mut variants_list: Vec<String> = vec![];
            for variant_name in variants.iter() {
                if variants_list.contains(&variant_name.data()) {
                    return Err(SemanticErr::EnumDuplicateVariants(variant_name.clone()))
                }

                variants_list.push(variant_name.data());
            }

            let variants_with_index = variants_list.iter().enumerate()
            .map(|x| (x.1.clone(), x.0 as u8)).collect::<Vec<(String, u8)>>();

            let enum_template = EnumTemplate {
                name: enum_name.data(),
                variants: variants_with_index
            };

            ss.declare_custom_type(CustomType::CustomEnum(enum_template));
        }
    }

    //second pass, adding every custom struct's name to being_defined
    for stmt in ast.iter() {
        if let Stmt::StructDeclr(struct_name, _) = stmt {
            if ss.global_used_ids.contains(&struct_name.data()) || being_defined.contains(&struct_name.data()) {
                return Err(SemanticErr::UsedId(struct_name.clone()))
            }

            being_defined.push(struct_name.data());
        }
    }

    let mut struct_templates: Vec<StructTemplate> = vec![];
    //third pass, generating all struct templates
    for stmt in ast.iter() {
        if let Stmt::StructDeclr(struct_name, params) = stmt {
            let typed_params = params.generate_typed_params(&ss, Some(&being_defined))?;
            
            let template = StructTemplate {
                name: struct_name.data(),
                fields: typed_params.items.iter()
                .map(|x| (x.0.clone(), x.1.clone(), 0 as u16))
                .collect::<Vec<(String, ValueType, u16)>>()
            };

            struct_templates.push(template.clone());
            ss.declare_custom_type(CustomType::CustomStruct(template));
        }
    }

    //making sure no structs are recursive
    for template in struct_templates.iter() {
        template.check_recursive(&ss, 0)?;
    }

    let mut final_struct_templates: Vec<StructTemplate> = vec![];
    //calculating field offsets from head
    for template in struct_templates {
        let mut sum: u16 = 0;
        let mut new_fields: Vec<(String, ValueType, u16)> = vec![];

        for field in template.fields.iter() {
            new_fields.push((field.0.clone(), field.1.clone(), sum));
            sum += field.1.size(&ss.defined_types);
        }

        final_struct_templates.push(StructTemplate {
            name: template.name,
            fields: new_fields }
        );
    }

    //removing incomplete structs and adding structs with fields offset from head
    ss.remove_all_structs();
    for template in final_struct_templates {
        ss.declare_custom_type(CustomType::CustomStruct(template));
    }    

    //generating AST
    for stmt in ast.iter() {
        typed_ast.push(stmt.generate_typed_stmt(&mut ss, false)?);
    }
    
    Ok((typed_ast, ss.defined_types))
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

    pub fn remove_all_structs(&mut self) {
        let mut ptr: usize = 0;

        while ptr < self.defined_types.len() {
            if let CustomType::CustomStruct(_) = self.defined_types[ptr] {
                self.defined_types.remove(ptr);
                continue
            }
            ptr += 1;
        }
    }

    pub fn declare_var(&mut self, name: String, t: ValueType) {
        self.used_ids.push(name.clone());
        self.stack.push(ScopeStackOp::DeclrVar(name, t));
    }

    pub fn declare_custom_type(&mut self, t: CustomType) {
        self.global_used_ids.push(t.name());
        self.defined_types.push(t);
    }

    pub fn declare_fn(&mut self, template: FnTemplate) {
        self.global_used_ids.push(template.name.clone());
        self.defined_functions.push(template);
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
    pub fn check_recursive(&self, ss: &ScopeStack, iteration: u8) -> Result<(), SemanticErr> {
        if iteration == 100 {
            return Err(SemanticErr::RecursiveStruct(self.name.clone()))
        }
        
        for f in self.fields.iter() {
            //println!("field: {} type: {:#?}", f.0, f.1);
            if let ValueType::CustomStruct(s) = &f.1 {
                let child_struct = get_custom_struct(s.clone(), &ss.defined_types).expect("should have been caught earlier");
                child_struct.check_recursive(ss, iteration + 1)?;
            }
        }

        Ok(())
    }
}