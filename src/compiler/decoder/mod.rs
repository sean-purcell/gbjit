use std::convert::TryInto;
use std::error::Error;
use std::fmt;

use lazy_static::lazy_static;

use super::instruction::*;

mod byte_count;
mod byte_kmap;

lazy_static! {
    static ref PARSERS: [Option<Parser>; 256] = generate_parsers();
}

type DecodeResult = Result<Instruction, DecodeError>;
type Parser = fn(&[u8]) -> DecodeResult;

#[derive(Debug, Copy, Clone)]
pub enum DecodeError {
    WrongByteCount,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DecodeError {}

pub fn bytes_required(first_byte: u8) -> u8 {
    match byte_count::bytes_required(first_byte) {
        0 => 1,
        x => x,
    }
}

pub fn decode(bytes: &[u8]) -> Result<Instruction, DecodeError> {
    if bytes.is_empty() {
        return Err(DecodeError::WrongByteCount);
    }
    match PARSERS[bytes[0] as usize] {
        Some(p) => p(bytes),
        None => Ok(Instruction::invalid(bytes[0])),
    }
}

pub fn decode_full(bytes: [u8; 3]) -> Instruction {
    let req = bytes_required(bytes[0]) as usize;
    decode(&bytes[0..req]).expect("Impossible error, correct number of bytes given")
}

fn generate_parsers() -> [Option<Parser>; 256] {
    use byte_kmap::ByteKmap;

    let mut m: [Option<Parser>; 256] = [None; 256];

    let mut fill = |kmap: ByteKmap, parser| {
        for i in kmap.enumerate().iter() {
            // Don't overwrite things.  This makes it easier to do things like the "HALT" cut out
            // in the ld block.
            if m[*i as usize].is_none() {
                m[*i as usize] = Some(parser);
            }
        }
    };

    // Use the kmaps to efficiently indicate the target set, but use them to fill a table so that
    // runtime is fast
    fill(
        ByteKmap::parse(&"a'b'c'e'f'g'h' + a'b'cdfgh + a'bcde'fgh' + abcdf'gh"),
        parse_control,
    );

    fill(ByteKmap::parse(&"a'b'def'g'h' + a'b'cf'g'h'"), parse_jr);
    fill(ByteKmap::parse(&"a'b'e'f'g'h"), parse_ld_fullimm);
    fill(ByteKmap::parse(&"a'b'c'f'gh'"), parse_ld_regaddr);
    fill(ByteKmap::parse(&"a'b'cf'gh'"), parse_ld_addrinc);
    fill(ByteKmap::parse(&"a'b'f'gh"), parse_incdec_full);
    fill(ByteKmap::parse(&"a'b'fg'"), parse_incdec_half);
    fill(ByteKmap::parse(&"a'b'fgh'"), parse_ld_halfimm);
    fill(ByteKmap::parse(&"a'b'c'fgh"), parse_rot_a);
    fill(ByteKmap::parse(&"a'b'cd'fgh"), parse_daacpl);
    fill(ByteKmap::single(0x08), parse_store_sp);
    fill(ByteKmap::parse(&"a'b'ef'g'h"), parse_add_hl);
    fill(ByteKmap::parse(&"ab'"), parse_alu);
    fill(ByteKmap::parse(&"a'b"), parse_ld);
    fill(ByteKmap::parse(&"abc'f'g'h' + abc'ef'g'"), parse_ret);
    fill(ByteKmap::parse(&"abce'f'g'h'"), parse_ld_ioimm);
    fill(ByteKmap::parse(&"abe'g'h"), parse_pushpop);
    fill(ByteKmap::parse(&"abce'f'gh'"), parse_ld_ioreg);
    fill(ByteKmap::parse(&"abc'd'e'f'g + abc'f'gh'"), parse_jp);
    fill(ByteKmap::parse(&"abc'fg'h' + abc'd'efg'"), parse_call);
    fill(ByteKmap::parse(&"abfgh'"), parse_alu_imm);
    fill(ByteKmap::parse(&"abfgh"), parse_rst);
    fill(ByteKmap::single(0xE8), parse_add_sp);
    fill(ByteKmap::single(0xF8), parse_hlsp_offset);
    fill(ByteKmap::single(0xE9), parse_jp_hl);
    fill(ByteKmap::single(0xF9), parse_ld_sphl);
    fill(ByteKmap::parse(&"abcef'gh'"), parse_ld_absolute);
    fill(ByteKmap::single(0xCB), parse_cb);

    m
}

fn check_length(bytes: &[u8], expected: usize) -> Result<(), DecodeError> {
    if bytes.len() != expected {
        Err(DecodeError::WrongByteCount)
    } else {
        Ok(())
    }
}

fn get_location(idx: u8) -> Location {
    use HalfReg::*;
    use Location::*;
    match idx & 7 {
        0 => Reg(B),
        1 => Reg(C),
        2 => Reg(D),
        3 => Reg(E),
        4 => Reg(H),
        5 => Reg(L),
        6 => Mem,
        7 => Reg(A),
        _ => unreachable!(),
    }
}

fn get_fullreg(idx: u8, sp: bool) -> Reg {
    use Reg::*;
    match idx & 3 {
        0 => BC,
        1 => DE,
        2 => HL,
        3 => {
            if sp {
                SP
            } else {
                AF
            }
        }
        _ => unreachable!(),
    }
}

fn get_condition(byte: u8, base: u8) -> Condition {
    use Condition::*;
    let mask = 0x18;
    if (byte & !mask) != base {
        Always
    } else {
        match byte & mask {
            0x00 => Nz,
            0x10 => Nc,
            0x08 => Z,
            0x18 => C,
            _ => unreachable!(),
        }
    }
}

fn get_alu_cmd(byte: u8) -> AluCommand {
    use AluCommand::*;
    match byte & 7 {
        0 => Add,
        1 => Adc,
        2 => Sub,
        3 => Sbc,
        4 => And,
        5 => Xor,
        6 => Or,
        7 => Cp,
        _ => unreachable!(),
    }
}

fn get_immediate(bytes: &[u8]) -> Result<u16, DecodeError> {
    if bytes.len() < 3 {
        Err(DecodeError::WrongByteCount)
    } else {
        Ok(u16::from_le_bytes(bytes[1..3].try_into().unwrap()))
    }
}

fn parse_cb(bytes: &[u8]) -> DecodeResult {
    use BitCommand::*;

    check_length(bytes, 2)?;

    let b = bytes[1];

    let section = b >> 6;
    let id = (b >> 3) & 7;

    let cmd = if section == 0 {
        match id {
            0 => Rlc,
            1 => Rrc,
            2 => Rl,
            3 => Rr,
            4 => Sla,
            5 => Sra,
            6 => Swap,
            7 => Srl,
            _ => unreachable!(),
        }
    } else if section == 1 {
        Bit(id)
    } else if section == 2 {
        Res(id)
    } else {
        Set(id)
    };

    let op = get_location(b);

    let cycles = if op == Location::Mem {
        if let Bit(_) = cmd {
            12
        } else {
            16
        }
    } else {
        8
    };

    Ok(Instruction {
        cmd: Command::BitHalf { cmd, op },
        cycles,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_control(bytes: &[u8]) -> DecodeResult {
    use ControlCommand::*;
    let op = match bytes[0] {
        0x00 => Nop,
        0x76 => Halt,
        0x10 => Stop,
        0x3f => Ccf,
        0x37 => Scf,
        0xf3 => Di,
        0xfb => Ei,
        _ => unreachable!(),
    };

    let len = if op == Stop { 2 } else { 1 };
    check_length(bytes, len)?;

    Ok(Instruction {
        cmd: Command::Control(op),
        cycles: 4,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_jr(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 2)?;

    let condition = get_condition(bytes[0], 0x20);

    let offset = bytes[1] as i8;

    Ok(Instruction {
        cmd: Command::Jump {
            target: JumpTarget::Relative(offset),
            condition,
        },
        cycles: 12,
        alt_cycles: Some(8),
        encoding: bytes.to_vec(),
    })
}

fn parse_jp(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 3)?;

    let b = bytes[0];

    let condition = get_condition(b, 0xc2);

    let target = JumpTarget::Absolute(get_immediate(bytes)?);

    let (cycles, alt_cycles) = if condition == Condition::Always {
        (16, None)
    } else {
        (16, Some(12))
    };

    Ok(Instruction {
        cmd: Command::Jump { target, condition },
        cycles,
        alt_cycles,
        encoding: bytes.to_vec(),
    })
}

fn parse_ret(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 1)?;

    let b = bytes[0];

    let condition = get_condition(b, 0xc0);
    let intenable = b == 0xd9;

    let (cycles, alt_cycles) = if condition == Condition::Always {
        (16, None)
    } else {
        (20, Some(8))
    };

    Ok(Instruction {
        cmd: Command::Ret {
            condition,
            intenable,
        },
        cycles,
        alt_cycles,
        encoding: bytes.to_vec(),
    })
}

fn parse_call(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 3)?;

    let b = bytes[0];

    let condition = get_condition(b, 0xc4);

    let target = get_immediate(bytes)?;

    let (cycles, alt_cycles) = if condition == Condition::Always {
        (24, None)
    } else {
        (24, Some(12))
    };

    Ok(Instruction {
        cmd: Command::Call { target, condition },
        cycles,
        alt_cycles,
        encoding: bytes.to_vec(),
    })
}

fn parse_jp_hl(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 1)?;

    Ok(Instruction {
        cmd: Command::Jump {
            target: JumpTarget::Hl,
            condition: Condition::Always,
        },
        cycles: 4,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_ld_fullimm(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 3)?;

    let dst = get_fullreg(bytes[0] >> 4, true);

    let val = get_immediate(bytes)?;

    Ok(Instruction {
        cmd: Command::LdFullImm { dst, val },
        cycles: 12,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_ld_absolute(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 3)?;

    let addr = HalfWordId::Addr(get_immediate(bytes)?);
    let a = HalfWordId::RegVal(HalfReg::A);

    let (src, dst) = if bytes[0] & 1 == 0 {
        (a, addr)
    } else {
        (addr, a)
    };

    Ok(Instruction {
        cmd: Command::LdHalf { src, dst },
        cycles: 16,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_ld_addrinc(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 1)?;

    let b = bytes[0];

    let inc = b & 0x10 == 0;
    let load = b & 0x8 != 0;

    Ok(Instruction {
        cmd: Command::LdAddrInc { inc, load },
        cycles: 8,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_incdec_full(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 1)?;
    let b = bytes[0];

    let reg = get_fullreg(bytes[0] >> 4, true);

    let inc = b & 0x8 == 0;

    Ok(Instruction {
        cmd: Command::IncDecFull { reg, inc },
        cycles: 8,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_incdec_half(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 1)?;
    let b = bytes[0];

    let loc = get_location(b >> 3);
    let inc = b & 1 == 0;

    let cycles = if loc == Location::Mem { 12 } else { 4 };

    Ok(Instruction {
        cmd: Command::IncDecHalf { loc, inc },
        cycles,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_ld_halfimm(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 2)?;
    let b = bytes[0];

    let src = HalfWordId::Imm(bytes[1]);

    let (dst, cycles) = match get_location(b >> 3) {
        Location::Reg(r) => (HalfWordId::RegVal(r), 8),
        Location::Mem => (HalfWordId::RegAddr(Reg::HL), 12),
    };

    Ok(Instruction {
        cmd: Command::LdHalf { src, dst },
        cycles,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_rot_a(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 1)?;
    let b = bytes[0];

    let cmd = {
        use BitCommand::*;
        match (b >> 3) & 3 {
            0 => Rlc,
            1 => Rrc,
            2 => Rl,
            3 => Rr,
            _ => unreachable!(),
        }
    };

    let op = Location::Reg(HalfReg::A);

    Ok(Instruction {
        cmd: Command::BitHalf { cmd, op },
        cycles: 4,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_daacpl(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 1)?;
    let b = bytes[0];

    let cmd = {
        match (b >> 3) & 1 {
            0 => Command::Daa,
            1 => Command::Cpl,
            _ => unreachable!(),
        }
    };

    Ok(Instruction {
        cmd,
        cycles: 4,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_store_sp(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 3)?;

    let addr = get_immediate(bytes)?;

    Ok(Instruction {
        cmd: Command::StoreSp { addr },
        cycles: 20,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_add_hl(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 1)?;

    let reg = get_fullreg(bytes[0] >> 4, true);

    Ok(Instruction {
        cmd: Command::AddHl(reg),
        cycles: 8,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_ld(bytes: &[u8]) -> DecodeResult {
    use HalfWordId::*;

    check_length(bytes, 1)?;
    let b = bytes[0];

    let hl = RegAddr(Reg::HL);
    let src = match get_location(b) {
        Location::Reg(r) => RegVal(r),
        Location::Mem => hl,
    };

    let dst = match get_location(b >> 3) {
        Location::Reg(r) => RegVal(r),
        Location::Mem => hl,
    };

    let cycles = if src == hl || dst == hl { 8 } else { 4 };

    Ok(Instruction {
        cmd: Command::LdHalf { src, dst },
        cycles,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_ld_regaddr(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 1)?;

    let b = bytes[0];

    let operand = HalfWordId::RegAddr(if b & 0x10 == 0 { Reg::BC } else { Reg::DE });
    let a = HalfWordId::RegVal(HalfReg::A);

    let load = (b & 0x8) != 0;

    let (src, dst) = match load {
        true => (operand, a),
        false => (a, operand),
    };

    Ok(Instruction {
        cmd: Command::LdHalf { src, dst },
        cycles: 8,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_ld_ioimm(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 2)?;
    let b = bytes[0];

    let addr = HalfWordId::IoImmAddr(bytes[1]);
    let reg = HalfWordId::RegVal(HalfReg::A);

    let (src, dst) = if b & 0x10 == 0 {
        (reg, addr)
    } else {
        (addr, reg)
    };

    Ok(Instruction {
        cmd: Command::LdHalf { src, dst },
        cycles: 12,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_ld_ioreg(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 1)?;
    let b = bytes[0];

    let addr = HalfWordId::IoRegAddr(HalfReg::C);
    let reg = HalfWordId::RegVal(HalfReg::A);

    let (src, dst) = if b & 0x10 == 0 {
        (reg, addr)
    } else {
        (addr, reg)
    };

    Ok(Instruction {
        cmd: Command::LdHalf { src, dst },
        cycles: 8,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_pushpop(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 1)?;
    let b = bytes[0];

    let reg = get_fullreg(b >> 4, false);
    let (cmd, cycles) = if b & 0x04 == 0 {
        (Command::Pop(reg), 12)
    } else {
        (Command::Push(reg), 16)
    };

    Ok(Instruction {
        cmd,
        cycles,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_alu(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 1)?;
    let b = bytes[0];

    let cmd = get_alu_cmd(b >> 3);

    let op = get_location(b);
    let cycles = if op == Location::Mem { 8 } else { 4 };

    Ok(Instruction {
        cmd: Command::AluHalf {
            cmd,
            op: AluOperand::Loc(op),
        },
        cycles,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_alu_imm(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 2)?;
    let b = bytes[0];

    let cmd = get_alu_cmd(b >> 3);

    let op = AluOperand::Imm(bytes[1]);

    Ok(Instruction {
        cmd: Command::AluHalf { cmd, op },
        cycles: 8,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_rst(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 1)?;
    let b = bytes[0];

    let idx = b & 0x38;
    Ok(Instruction {
        cmd: Command::Rst(idx),
        cycles: 16,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_add_sp(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 2)?;

    Ok(Instruction {
        cmd: Command::AddSp(bytes[1] as i8),
        cycles: 16,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_hlsp_offset(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 2)?;

    Ok(Instruction {
        cmd: Command::HlSpOffset(bytes[1] as i8),
        cycles: 12,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

fn parse_ld_sphl(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 1)?;

    Ok(Instruction {
        cmd: Command::LdSpHl,
        cycles: 8,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_unexpected_invalids() {
        let invalids = vec![211, 219, 221, 227, 228, 235, 236, 237, 244, 252, 253];
        for i in 0..=255 {
            if !invalids.contains(&i) {
                assert_eq!(
                    PARSERS[i as usize].is_some(),
                    true,
                    "Parser for {:#04x?} unexpectedly none",
                    i
                );
            }
        }
    }

    #[test]
    fn spot_check() {
        use Command::*;

        let test = |bytes, cmd| {
            let decoded = decode(bytes).unwrap();
            assert_eq!(decoded.cmd, cmd);
        };

        test(
            &[0xac],
            AluHalf {
                cmd: AluCommand::Xor,
                op: AluOperand::Loc(Location::Reg(HalfReg::H)),
            },
        );

        test(
            &[0x5e],
            LdHalf {
                src: HalfWordId::RegAddr(Reg::HL),
                dst: HalfWordId::RegVal(HalfReg::E),
            },
        );

        test(&[0x76], Control(ControlCommand::Halt));

        test(
            &[0x12],
            LdHalf {
                src: HalfWordId::RegVal(HalfReg::A),
                dst: HalfWordId::RegAddr(Reg::DE),
            },
        );

        test(
            &[0x18, 0xf0],
            Jump {
                target: JumpTarget::Relative(-16),
                condition: Condition::Always,
            },
        );

        test(
            &[0x21, 0xad, 0xde],
            LdFullImm {
                dst: Reg::HL,
                val: 0xdead,
            },
        );

        test(
            &[0x3a],
            LdAddrInc {
                inc: false,
                load: true,
            },
        );

        test(
            &[0x33],
            IncDecFull {
                reg: Reg::SP,
                inc: true,
            },
        );

        test(
            &[0x35],
            IncDecHalf {
                loc: Location::Mem,
                inc: false,
            },
        );

        test(
            &[0x1c],
            IncDecHalf {
                loc: Location::Reg(HalfReg::E),
                inc: true,
            },
        );

        test(
            &[0x26, 26],
            LdHalf {
                src: HalfWordId::Imm(26),
                dst: HalfWordId::RegVal(HalfReg::H),
            },
        );

        test(
            &[0x0f],
            BitHalf {
                cmd: BitCommand::Rrc,
                op: Location::Reg(HalfReg::A),
            },
        );

        test(&[0x27], Daa);

        test(&[0x08, 0xad, 0xde], Command::StoreSp { addr: 0xdead });

        test(&[0x39], Command::AddHl(Reg::SP));

        test(
            &[0xc8],
            Command::Ret {
                condition: Condition::Z,
                intenable: false,
            },
        );

        test(
            &[0xd9],
            Command::Ret {
                condition: Condition::Always,
                intenable: true,
            },
        );

        test(
            &[0xe0, 0x44],
            Command::LdHalf {
                src: HalfWordId::RegVal(HalfReg::A),
                dst: HalfWordId::IoImmAddr(0x44),
            },
        );

        test(
            &[0xf2],
            Command::LdHalf {
                src: HalfWordId::IoRegAddr(HalfReg::C),
                dst: HalfWordId::RegVal(HalfReg::A),
            },
        );

        test(&[0xf5], Command::Push(Reg::AF));

        test(
            &[0xd2, 0xad, 0xde],
            Command::Jump {
                target: JumpTarget::Absolute(0xdead),
                condition: Condition::Nc,
            },
        );

        test(
            &[0xd4, 0xad, 0xde],
            Command::Call {
                target: 0xdead,
                condition: Condition::Nc,
            },
        );

        test(
            &[0xee, 0x42],
            Command::AluHalf {
                cmd: AluCommand::Xor,
                op: AluOperand::Imm(0x42),
            },
        );

        test(&[0xcf], Command::Rst(0x08));

        test(&[0xe8, 0xf0], Command::AddSp(-16));

        test(&[0xf8, 0xf0], Command::HlSpOffset(-16));

        test(
            &[0xe9],
            Command::Jump {
                target: JumpTarget::Hl,
                condition: Condition::Always,
            },
        );

        test(&[0xf9], Command::LdSpHl);

        test(
            &[0xea, 0xad, 0xde],
            Command::LdHalf {
                src: HalfWordId::RegVal(HalfReg::A),
                dst: HalfWordId::Addr(0xdead),
            },
        );
    }

    #[test]
    fn cb_spot_check() {
        use BitCommand::*;
        use Command::*;
        use HalfReg::*;
        use Location::*;

        let test = |byte, cmd, op| {
            let decoded = decode(&[0xcb, byte]).unwrap();
            assert_eq!(decoded.cmd, BitHalf { cmd, op });
        };

        test(0x24, Sla, Reg(H));
        test(0x1e, Rr, Mem);
        test(0x68, Bit(5), Reg(B));
        test(0xb1, Res(6), Reg(C));
        test(0xff, Set(7), Reg(A));
    }
}
