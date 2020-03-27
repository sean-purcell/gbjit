#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Reg {
    AF,
    BC,
    DE,
    HL,
    SP,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum HalfReg {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum HalfWordId {
    RegVal(HalfReg),
    RegAddr(Reg),
    Addr(u16),
    IoAddr(u8),
    Imm(u8),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AluCommand {
    Add,
    Adc,
    Sub,
    Sbc,
    And,
    Xor,
    Or,
    Cp,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Location {
    Reg(HalfReg),
    Mem,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AluOperand {
    Loc(Location),
    Imm(u8),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum BitCommand {
    Rlc,
    Rl,
    Rrc,
    Rr,
    Sla,
    Sra,
    Srl,
    Bit(u8),
    Set(u8),
    Res(u8),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ControlCommand {
    Nop,
    Halt,
    Stop,
    Ccf,
    Scf,
    Di,
    Ei,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Condition {
    Always,
    Z,
    Nz,
    C,
    Nc,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum JumpTarget {
    Imm(u16),
    Hl,
    Relative(i8),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Command {
    LdHalf {
        src: HalfWordId,
        dst: HalfWordId,
    },
    LdAddrInc {
        inc: bool,
        load: bool,
    },
    LdFullIm {
        dst: Reg,
        val: u16,
    },
    LdSpHl,
    Push(Reg),
    Pop(Reg),
    AluHalf {
        cmd: AluCommand,
        op: AluOperand,
    },
    Daa,
    Cpl,
    AddHl(Reg),
    IncReg(Reg),
    DecReg(Reg),
    AddSp(i8),
    HlSpOffset(i8),
    BitHalf {
        cmd: BitCommand,
        op: Location,
    },
    Control(ControlCommand),
    Jump {
        target: JumpTarget,
        condition: Condition,
    },
    Call {
        target: u16,
        condition: Condition,
    },
    Ret {
        condition: Condition,
        enable: bool,
    },
    Rst(u8),
}

#[derive(PartialEq, Debug, Clone)]
pub struct Instruction {
    pub cmd: Command,
    pub size: u8,
    pub cycles: u8,
    pub alt_cycles: Option<u8>,
    pub encoding: Vec<u8>,
}
