#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum Reg {
    AF,
    BC,
    DE,
    HL,
    SP,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum HalfReg {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum HalfWordId {
    RegVal(HalfReg),
    RegAddr(Reg),
    Addr(u16),
    IoImmAddr(u8),
    IoRegAddr(HalfReg),
    Imm(u8),
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
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

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum Location {
    Reg(HalfReg),
    Mem,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum AluOperand {
    Loc(Location),
    Imm(u8),
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum BitCommand {
    Rlc,
    Rl,
    Rrc,
    Rr,
    Sla,
    Sra,
    Swap,
    Srl,
    Bit(u8),
    Set(u8),
    Res(u8),
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum ControlCommand {
    Nop,
    Halt,
    Stop,
    Ccf,
    Scf,
    Di,
    Ei,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum Condition {
    Always,
    Z,
    Nz,
    C,
    Nc,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum JumpTarget {
    Absolute(u16),
    Hl,
    Relative(i8),
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum Command {
    LdHalf {
        src: HalfWordId,
        dst: HalfWordId,
    },
    LdAddrInc {
        inc: bool,
        load: bool,
    },
    LdFullImm {
        dst: Reg,
        val: u16,
    },
    StoreSp {
        addr: u16,
    },
    Push(Reg),
    Pop(Reg),
    AluHalf {
        cmd: AluCommand,
        op: AluOperand,
    },
    Daa,
    Cpl,
    AddHl(Reg),
    IncDecHalf {
        loc: Location,
        inc: bool,
    },
    IncDecFull {
        reg: Reg,
        inc: bool,
    },
    AddSp(i8),
    HlSpOffset(i8),
    LdSpHl,
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
        intenable: bool,
    },
    Rst(u8),
    Invalid,
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct Instruction {
    pub cmd: Command,
    pub cycles: u8,
    pub alt_cycles: Option<u8>,
    pub encoding: Vec<u8>,
}

impl Instruction {
    pub fn invalid(b: u8) -> Self {
        Instruction {
            cmd: Command::Invalid,
            cycles: 0,
            alt_cycles: None,
            encoding: [b].to_vec(),
        }
    }

    pub fn size(&self) -> u16 {
        self.encoding.len() as u16
    }
}
