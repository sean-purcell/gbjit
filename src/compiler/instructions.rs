pub enum Reg {
    AF,
    BC,
    DE,
    HL,
    SP,
}

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

pub enum HalfWordId {
    RegVal(HalfReg),
    RegAddr(Reg),
    Addr(u16),
    IoAddr(u8),
    Imm(u8),
}

pub enum AluCommand {
    Add,
    Adc,
    Sub,
    Sbc,
    And,
    Xor,
    Or,
    Cp,
    Inc,
    Dec,
}

pub enum AluOperand {
    Reg(HalfReg),
    Imm(u8),
    Mem,
}

pub enum BitOperand {
    Reg(HalfReg),
    Mem,
}

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

pub enum ControlCommand {
    Nop,
    Halt,
    Stop,
    Ccf,
    Scf,
    Di,
    Ei,
}

pub enum Condition {
    Always,
    Z,
    Nz,
    C,
    Nc,
}

pub enum JumpTarget {
    Imm(u16),
    Hl,
    Relative(i8),
}

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
    HalfAlu {
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
    HalfBit {
        cmd: BitCommand,
        op: BitOperand,
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

pub struct Instruction {
    pub cmd: Command,
    pub size: u8,
    pub cycles: u8,
    pub alt_cycles: Option<u8>,
}
