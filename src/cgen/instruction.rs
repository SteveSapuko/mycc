pub const R0: REG = 0;
pub const R1: REG = 1;
pub const BPL: REG = 2;
pub const BPH: REG = 3;
pub const SPL: REG = 4;
pub const SPH: REG = 5;
pub const MARL: REG = 6;
pub const MARH: REG = 7;

pub type REG = u8;
pub type LABEL = String;

pub enum AssemblyCommand {
    Label(String),
    Comment(String),
    Instruction(Instruction),
}

pub enum Instruction {
    Add(REG),
    Sub(REG),
    Adc(REG),
    Sbc(REG),
    Ror(REG),
    Nor(REG),
    And(REG),

    Stc,
    Clc,
    Rmov(REG),
    Amov(REG),
    Jmp(LABEL),
    Bca(LABEL),
    Bnc(LABEL),
    Bze(LABEL),
    Bnz(LABEL),
    Bsi(LABEL),
    Bpa(LABEL),
    Acz,
    Ima(REG), //REG,
    Imr(REG, u8), //REG, VALUE
    Spc,
    Str(REG),
    Ld(REG),
    In(REG),
    Out(REG),
    Nop,
    Hlt,
    Push(REG),
    Pop(REG),
}