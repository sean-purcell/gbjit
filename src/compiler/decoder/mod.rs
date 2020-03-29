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
    fill(ByteKmap::parse(&"ab'"), parse_alu);
    fill(ByteKmap::parse(&"a'b"), parse_ld);

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

fn parse_location(idx: u8) -> Location {
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
    use Condition::*;

    check_length(bytes, 2)?;

    let condition = match bytes[0] {
        0x20 => Nz,
        0x30 => Nc,
        0x28 => Z,
        0x38 => C,
        0x18 => Always,
        _ => unreachable!(),
    };

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

fn parse_ld_fullimm(bytes: &[u8]) -> DecodeResult {
    check_length(bytes, 3)?;

    let dst = {
        use Reg::*;
        match (bytes[0] >> 4) & 3 {
            0 => BC,
            1 => DE,
            2 => HL,
            3 => SP,
            _ => unreachable!(),
        }
    };

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

    let reg = {
        use Reg::*;
        match (bytes[0] >> 4) & 3 {
            0 => BC,
            1 => DE,
            2 => HL,
            3 => SP,
            _ => unreachable!(),
        }
    };

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

    let loc = parse_location(b >> 3);
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

    let (dst, cycles) = match parse_location(b >> 3) {
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

fn parse_ld(bytes: &[u8]) -> DecodeResult {
    use HalfWordId::*;

    check_length(bytes, 1)?;
    let b = bytes[0];

    let hl = RegAddr(Reg::HL);
    let src = match parse_location(b) {
        Location::Reg(r) => RegVal(r),
        Location::Mem => hl,
    };

    let dst = match parse_location(b >> 3) {
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

    let op = parse_location(b);
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
    }
}
