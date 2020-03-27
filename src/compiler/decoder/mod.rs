use super::instruction::*;

mod byte_count;

#[derive(Debug, Copy, Clone)]
pub enum DecodeError {
    InvalidEncoding,
    WrongByteCount,
}

type Parser = fn(u8, &dyn FnOnce(u8) -> u8) -> Option<Instruction>;
const PARSERS: [(u8, u8, Parser); 0] = [];

fn bytes_required(first_byte: u8) -> Result<u8, DecodeError> {
    match byte_count::bytes_required(first_byte) {
        0 => Err(DecodeError::InvalidEncoding),
        x => Ok(x),
    }
}

fn decode(bytes: &[u8]) -> Result<Instruction, DecodeError> {
    unimplemented!()
}
