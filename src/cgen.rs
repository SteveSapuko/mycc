use crate::expr::*;
use crate::semantics::*;
use crate::stmt::Stmt;
use crate::stmt::TypeDeclr;
use crate::types::*;

use self::instruction::*;

pub mod instruction;
mod expr_cgen;
mod stmt_cgen;
mod known_at_compile;
mod value;

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
    pub fn get_offset_from_bp(&self, cg: &CodeGenerator, parent: Option<StructTemplate>) -> u16 {
        match self {
            Variable::Id(id) => {
                match parent {
                    Some(parent_struct) => {
                        return parent_struct.get_field_offset_from_head(id.data())
                    }

                    None => {
                        return 0
                    }
                }
            }

            Variable::StructField(s) => {
                let (head, child) = &**s;
                match parent {
                    Some(parent_struct) => {
                        let head_offset_from_parent = parent_struct.get_field(field)
                    }

                    None => {
                        let head_first_id = head.get_first_lexeme().data();
                        let (head_type, head_offest_from_bp) = cg.get_var(head_first_id);
                        
                        if let ValueType::CustomStruct(struct_name) = head_type {
                            let head_struct = cg.ss.get_custom_struct(struct_name).unwrap();
                            let child_offset_from_head = child.get_offset_from_bp(cg, Some(head_struct));

                            return head_offest_from_bp + child_offset_from_head
                        }
                        
                        unreachable!()
                    }
                }

                ()
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
    Variable(String, ValueType, u16),
    Function(FnTemplate),
}