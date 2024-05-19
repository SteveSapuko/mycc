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
        let var_size = v_type.size(&self.defined_types);

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
    
    ///needs to overwrite a register
    fn decrease_sp_by(&mut self, n: u16, overwrite: REG) {
        /*
        imr OVERWRITE n[0:7]
        rmov SPL
        sub OVERWRITE
        amov SPL

        imr OVERWRITE n[8:15]
        rmov SPH
        sbc OVERWRITE
        amov SPH
        */

        self.write_instruction(Instruction::Imr(overwrite, n as u8));
        self.write_instruction(Instruction::Rmov(SPL));
        self.write_instruction(Instruction::Sub(overwrite));
        self.write_instruction(Instruction::Amov(SPL));
        
        self.write_instruction(Instruction::Imr(overwrite, (n >> 8) as u8));
        self.write_instruction(Instruction::Rmov(SPH));
        self.write_instruction(Instruction::Sbc(overwrite));
        self.write_instruction(Instruction::Amov(SPH));
    }

    fn ld_bp_plus_n_to_reg(&mut self, reg: REG, n: u16) {
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


pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
    pub frame_size: u16,
}

pub enum Symbol {
    EnterScope,
    Variable(String, ValueType, u16),
}