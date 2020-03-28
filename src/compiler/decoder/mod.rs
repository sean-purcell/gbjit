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
            if m[*i as usize].is_none() {
                m[*i as usize] = Some(parser);
            }
        }
    };

    fill(ByteKmap::single(0xCB), parse_cb);

    // Generated with:
    // minterms: 0,10,76,f3,fb
    fill(
        ByteKmap::parse(&"a'b'c'e'f'g'h' + a'b'cdfgh + a'bcde'fgh' + abcdf'gh"),
        parse_control,
    );

    fill(ByteKmap::parse(&"ab'"), parse_alu);
    fill(ByteKmap::parse(&"a'b"), parse_ld);

    m
}

fn check_length(bytes: &[u8], expected: usize) -> Result<(), DecodeError> {
    if bytes.len() != expected {
        Err(DecodeError::WrongByteCount)
    } else {
        Ok(())
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
        size: len as u8,
        cycles: 4,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
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
        size: 1,
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

    let op = parse_location(b);
    let cycles = if op == Location::Mem { 8 } else { 4 };

    Ok(Instruction {
        cmd: Command::AluHalf {
            cmd,
            op: AluOperand::Loc(op),
        },
        size: 1,
        cycles,
        alt_cycles: None,
        encoding: bytes.to_vec(),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn num_invalids() {
        let invalids = PARSERS.iter().filter(|x| x.is_none()).count();
        assert_eq!(invalids, 11);
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
    }
}
