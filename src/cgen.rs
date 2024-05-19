use crate::expr::*;
use crate::semantics::*;
use crate::typed_ast::*;
use crate::types::*;

use self::instruction::*;

pub mod instruction;
mod expr_cgen;
mod stmt_cgen;
mod known_at_compile;
mod value;

pub struct CodeGenerator {
    ast: Vec<TypedStmt>,
    output: Vec<AssemblyCommand>,
    symbol_table: SymbolTable,
    defined_types: Vec<CustomType>,
    fn_templates: Vec<FnTemplate>,
}

impl CodeGenerator {
    pub fn new(ast: Vec<TypedStmt>, defined_types: Vec<CustomType>, fn_templates: Vec<FnTemplate>) -> Self {
        CodeGenerator {
            ast: ast.clone(),
            output: vec![],
            symbol_table: SymbolTable { symbols: vec![], frame_size: 0 },
            defined_types,
            fn_templates,
        }
    }

    fn write_instruction(&mut self, inst: Instruction) {
        self.output.push(AssemblyCommand::Instruction(inst));
    }

    fn declare_var(&mut self, name: String, v_type: ValueType) {
        let var_size = v_type.size(&self);

        self.symbol_table.symbols.push(Symbol::Variable(name, v_type, self.symbol_table.frame_size));
        self.symbol_table.frame_size += var_size;
    }

    fn get_var(&self, target_name: String) -> (ValueType, u16) {
        for symbol in self.symbol_table.symbols.iter().rev() {
            if let Symbol::Variable(name, v_type, offset_from_base) = symbol {
                if name == &target_name {
                    return  (v_type.clone(), *offset_from_base)
                }
            }
        }

        unreachable!()
    }

    fn get_var_offset_from_bp(&self, var: &Expr) -> u16 {
        match var {
            Expr::Assign(e) => {
                let right = &e.1;
                if let Expr::Primary(p) = right {
                    if let PrimaryExpr::Variable(v) = &**p {
                        let head_name = v.get_first_lexeme().data();
                        let head_symbol = self.get_var(head_name);


                    }
                }

                //this function should only be called if the right side of an assign is a Variable
                unreachable!() 
            }

            _ => unreachable!()
        }
    }

    fn increase_sp_by(&mut self, n: u16) {
        /*
        ima n[0:7]
        add SPL
        amov SPL

        ima n[8:15]
        adc SPH
        amov SPH
        */

        self.write_instruction(Instruction::Ima(n as u8));
        self.write_instruction(Instruction::Add(SPL));
        self.write_instruction(Instruction::Amov(SPL));

        self.write_instruction(Instruction::Ima((n >> 8) as u8));
        self.write_instruction(Instruction::Adc(SPH));
        self.write_instruction(Instruction::Amov(SPH));
    }

    ///overwrites R0
    fn decrease_sp_by(&mut self, n: u16) {
        /*
        imr R0 n[0:7]
        rmov SPL
        sub R0
        amov SPL

        imr R0 n[8:15]
        rmov SPH
        sbc R0
        amov SPH
        */

        self.write_instruction(Instruction::Imr(R0, n as u8));
        self.write_instruction(Instruction::Rmov(SPL));
        self.write_instruction(Instruction::Sub(R0));
        self.write_instruction(Instruction::Amov(SPL));
        
        self.write_instruction(Instruction::Imr(R0, (n >> 8) as u8));
        self.write_instruction(Instruction::Rmov(SPH));
        self.write_instruction(Instruction::Sbc(R0));
        self.write_instruction(Instruction::Amov(SPH));
    }

    fn ld_bp_plus_to_reg(&mut self, reg: REG, n: u16) {
        /*
        ima n[0:7]
        add BPL
        amov MARL

        ima n[8:15]
        adc BPH
        amov MARH

        ld REG
        */

        self.write_instruction(Instruction::Ima(n as u8));
        self.write_instruction(Instruction::Add(BPL));
        self.write_instruction(Instruction::Amov(MARL));

        self.write_instruction(Instruction::Ima((n >> 8) as u8));
        self.write_instruction(Instruction::Adc(BPH));
        self.write_instruction(Instruction::Amov(MARH));

        self.write_instruction(Instruction::Ld(reg));
    }

    fn ld_sp_minus_to_reg(&mut self, reg: REG, n: u16) {
        /*
        imr REG n[0:7]
        rmov SPL
        sub REG
        amov MARL

        imr REG n[8:15]
        rmov SPH
        sbc REG
        amov MARH

        ld REG
        */

        self.write_instruction(Instruction::Imr(reg, n as u8));
        self.write_instruction(Instruction::Rmov(SPL));
        self.write_instruction(Instruction::Sub(reg));
        self.write_instruction(Instruction::Amov(MARL));
        
        self.write_instruction(Instruction::Imr(reg, (n >> 8) as u8));
        self.write_instruction(Instruction::Rmov(SPH));
        self.write_instruction(Instruction::Sbc(reg));
        self.write_instruction(Instruction::Amov(MARH));

        self.write_instruction(Instruction::Ld(reg));
    }
}

impl Variable {
    pub fn eval_offset_from_bp(&self, cg: &CodeGenerator, parent: Option<StructTemplate>) -> (ValueType, u16) {
        match self {
            Variable::Id(id) => {
                match parent {
                    None => {
                        cg.get_var(id.data())
                    }
                    
                    Some(parent_struct) => {
                        return parent_struct.get_field(id.data())
                    }
                }
            }

            Variable::StructField(s) => {
                let (head, tail) = &**s;
                match parent {
                    None => {
                        let (head_type, head_offest_from_bp) = head.eval_offset_from_bp(cg, parent);
                        
                        if let ValueType::CustomStruct(struct_name) = head_type {
                            let head_struct = cg.ss.get_custom_struct(struct_name).unwrap();
                            let (tail_type, tail_offset_from_head) = tail.eval_offset_from_bp(cg, Some(head_struct));

                            return (tail_type, head_offest_from_bp + tail_offset_from_head)
                        }
                        
                        unreachable!()
                    }
                    
                    Some(parent_struct) => {
                        let (head_type, head_offset_from_parent) = head.eval_offset_from_bp(cg, parent);

                        if let ValueType::CustomStruct(head_struct_name) = head_type {
                            let head_struct_template = cg.ss.get_custom_struct(head_struct_name).expect("semantics should have caught this");
                            let (tail_type, tail_offset_from_head) = tail.eval_offset_from_bp(cg, Some(head_struct_template));

                            return (tail_type, head_offset_from_parent + tail_offset_from_head)
                        }

                        unreachable!()
                    }
                }
            }

            Variable::Array(array_head, array_index) => {
                match parent {
                    None => {
                        let head_type = array_head.eval_offset_from_bp(cg, parent);

                        if let Expr::Primary(primary_expr) = array_index {
                            if let PrimaryExpr::NumLiteral(num_literal, _) = primary_expr {
                                
                            }
                        }
                        
                        unreachable!()
                    }

                    ()
                }
            }
        }
    }
}

pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
    pub frame_size: u16,
}
enum Symbol {
    EnterScope,
    Label(String),
    Variable(String, ValueType, u16),
}