#![allow(dead_code)]

use std::convert::TryInto;

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
    InvalidEncoding,
    WrongByteCount,
}

fn invalid_encoding(_bytes: &[u8]) -> Result<Instruction, DecodeError> {
    Err(DecodeError::InvalidEncoding)
}

pub fn bytes_required(first_byte: u8) -> Result<u8, DecodeError> {
    match byte_count::bytes_required(first_byte) {
        0 => Err(DecodeError::InvalidEncoding),
        x => Ok(x),
    }
}

pub fn decode(bytes: &[u8]) -> Result<Instruction, DecodeError> {
    if bytes.is_empty() {
        return Err(DecodeError::WrongByteCount);
    }
    match PARSERS[bytes[0] as usize] {
        Some(p) => p(bytes),
        None => Err(DecodeError::InvalidEncoding),
    }
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
    fill(ByteKmap::parse(&"abc'd'e'f'g + abc'f'gh'"), parse_jp);
    fill(ByteKmap::parse(&"abce'f'gh'"), parse_ld_ioreg);
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

fn parse_cb(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 2)?;
    unimplemented!();
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

fn parse_jp(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 3)?;

    let b = bytes[0];

    let condition = get_condition(b, 0xc2);

    let target = JumpTarget::Absolute(u16::from_le_bytes(bytes[1..3].try_into().unwrap()));

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

fn parse_ld_fullimm(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 3)?;

    let dst = get_fullreg(bytes[0] >> 4, true);

    let val = u16::from_le_bytes(bytes[1..3].try_into().unwrap());

    Ok(Instruction {
        cmd: Command::LdFullImm { dst, val },
        cycles: 12,
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

    let addr = u16::from_le_bytes(bytes[1..3].try_into().unwrap());

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
    let cmd = {
        use AluCommand::*;
        match (b >> 3) & 7 {
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
    };

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

        let xac = decode(&[0xac]).unwrap();
        assert_eq!(
            xac.cmd,
            AluHalf {
                cmd: AluCommand::Xor,
                op: AluOperand::Loc(Location::Reg(HalfReg::H))
            }
        );

        let x5e = decode(&[0x5e]).unwrap();
        assert_eq!(
            x5e.cmd,
            LdHalf {
                src: HalfWordId::RegAddr(Reg::HL),
                dst: HalfWordId::RegVal(HalfReg::E),
            }
        );

        let x76 = decode(&[0x76]).unwrap();
        assert_eq!(x76.cmd, Control(ControlCommand::Halt));

        let x12 = decode(&[0x12]).unwrap();
        assert_eq!(
            x12.cmd,
            LdHalf {
                src: HalfWordId::RegVal(HalfReg::A),
                dst: HalfWordId::RegAddr(Reg::DE),
            }
        );

        let x18 = decode(&[0x18, 0xf0]).unwrap();
        assert_eq!(
            x18.cmd,
            Jump {
                target: JumpTarget::Relative(-16),
                condition: Condition::Always
            }
        );

        let x21 = decode(&[0x21, 0xad, 0xde]).unwrap();
        assert_eq!(
            x21.cmd,
            LdFullImm {
                dst: Reg::HL,
                val: 0xdead
            }
        );

        let x3a = decode(&[0x3a]).unwrap();
        assert_eq!(
            x3a.cmd,
            LdAddrInc {
                inc: false,
                load: true,
            }
        );

        let x33 = decode(&[0x33]).unwrap();
        assert_eq!(
            x33.cmd,
            IncDecFull {
                reg: Reg::SP,
                inc: true,
            }
        );

        let x35 = decode(&[0x35]).unwrap();
        assert_eq!(
            x35.cmd,
            IncDecHalf {
                loc: Location::Mem,
                inc: false,
            }
        );

        let x1c = decode(&[0x1c]).unwrap();
        assert_eq!(
            x1c.cmd,
            IncDecHalf {
                loc: Location::Reg(HalfReg::E),
                inc: true,
            }
        );

        let x26 = decode(&[0x26, 26]).unwrap();
        assert_eq!(
            x26.cmd,
            LdHalf {
                src: HalfWordId::Imm(26),
                dst: HalfWordId::RegVal(HalfReg::H),
            }
        );

        let x0f = decode(&[0x0f]).unwrap();
        assert_eq!(
            x0f.cmd,
            BitHalf {
                cmd: BitCommand::Rrc,
                op: Location::Reg(HalfReg::A),
            }
        );

        let x27 = decode(&[0x27]).unwrap();
        assert_eq!(x27.cmd, Daa);

        let x08 = decode(&[0x08, 0xad, 0xde]).unwrap();
        assert_eq!(x08.cmd, Command::StoreSp { addr: 0xdead });

        let x39 = decode(&[0x39]).unwrap();
        assert_eq!(x39.cmd, Command::AddHl(Reg::SP));

        let xc8 = decode(&[0xc8]).unwrap();
        assert_eq!(
            xc8.cmd,
            Command::Ret {
                condition: Condition::Z,
                intenable: false
            }
        );

        let xd9 = decode(&[0xd9]).unwrap();
        assert_eq!(
            xd9.cmd,
            Command::Ret {
                condition: Condition::Always,
                intenable: true
            }
        );

        let xe0 = decode(&[0xe0, 0x44]).unwrap();
        assert_eq!(
            xe0.cmd,
            Command::LdHalf {
                src: HalfWordId::RegVal(HalfReg::A),
                dst: HalfWordId::IoImmAddr(0x44),
            }
        );

        let xf2 = decode(&[0xf2]).unwrap();
        assert_eq!(
            xf2.cmd,
            Command::LdHalf {
                src: HalfWordId::IoRegAddr(HalfReg::C),
                dst: HalfWordId::RegVal(HalfReg::A),
            }
        );

        let xf5 = decode(&[0xf5]).unwrap();
        assert_eq!(xf5.cmd, Command::Push(Reg::AF));

        let xd2 = decode(&[0xd2, 0xad, 0xde]).unwrap();
        assert_eq!(
            xd2.cmd,
            Command::Jump {
                target: JumpTarget::Absolute(0xdead),
                condition: Condition::Nc
            }
        );
    }
}
