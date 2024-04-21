use crate::semantics::*;
use crate::stmt::Stmt;
use crate::stmt::TypeDeclr;
use crate::token::*;
use crate::types::*;

use self::instruction::*;

pub mod instruction;
mod expr_cgen;
mod stmt_cgen;

pub struct CodeGenerator {
    ast: Vec<Stmt>,
    output: Vec<AssemblyCommand>,
    symbol_table: SymbolTable,
    ss: ScopeStack,
}

impl CodeGenerator {
    pub fn new(ast: Vec<Stmt>) -> Self {
        CodeGenerator {
            ast: ast.clone(),
            output: vec![],
            symbol_table: SymbolTable { symbols: vec![], frame_size: 0 },
            ss: ScopeStack::new()}
    }

    pub fn cgen(&mut self) {
        define_scope(&mut self.ss, &self.ast).expect("should have been caught earlier");
        
        for stmt in self.ast.clone() {
            stmt.cgen(self);
        }
    }

    fn get_type_from_declr(&mut self, declr: &TypeDeclr) -> ValueType {
        ValueType::from_declr(declr, &mut self.ss).expect("should have been caught earlier")
    }

    fn write_instruction(&mut self, inst: Instruction) {
        self.output.push(AssemblyCommand::Instruction(inst));
    }

    fn declare_var(&mut self, name: String, v_type: ValueType) {
        let var_size = v_type.size(&self.ss);
        self.ss.declare_var(name.clone(), v_type.clone());

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
}





pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
    pub frame_size: u16,
}

enum Symbol {
    EnterScope,
    Variable(String, ValueType, u16),
    Function(FnTemplate),
}