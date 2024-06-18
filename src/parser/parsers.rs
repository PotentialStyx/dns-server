use core::str::Utf8Error;

use bytes::{Buf, Bytes};
use thiserror::Error;

use crate::parser::{OpCode, ResCode};

use super::{Domain, Header, Parsable, Question};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParserError {
    #[error("Recieved invalid ascii and got error `{0}`")]
    InvalidAscii(AsciiError),
    #[error("Not enough bytes, expected {expected:?} bytes, got {recieved:?}")]
    NotEnoughBytes { expected: usize, recieved: usize },
    // Right now there are no unknown errors
    // #[error("Unknown error")]
    // Unknown,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum AsciiError {
    #[error("Not even utf-8 ({0})")]
    InvalidUtf8(Utf8Error),
    #[error("Was valid utf-8 but wasn't ascii")]
    NotAscii,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BytesBuf {
    original: Bytes,
    pub(crate) in_use: Bytes,
}

impl BytesBuf {
    pub(crate) fn new(data: Vec<u8>) -> BytesBuf {
        let data: Bytes = data.into();
        BytesBuf {
            original: data.clone(),
            in_use: data,
        }
    }

    pub(crate) fn from_bytes(data: Bytes) -> BytesBuf {
        BytesBuf {
            original: data.clone(),
            in_use: data,
        }
    }

    /// Get a copy of the original `Bytes` object at ptr=0
    pub(crate) fn get_original(&self) -> Bytes {
        self.original.clone()
    }
}

impl Parsable for Domain {
    type Error = ParserError;
    fn parse(buf: &BytesBuf) -> Result<Self, Self::Error> {
        let mut result = vec![];
        let mut buf = buf.clone();
        loop {
            if buf.in_use.remaining() == 0 {
                return Err(ParserError::NotEnoughBytes {
                    expected: 1,
                    recieved: 0,
                });
            }

            // Conversion shouldn't fail as this will never target a less than 8 bit system.
            let len = buf.in_use.get_u8() as usize;

            if len == 0 {
                break;
            }

            if (len >> 6) == 0b11 {
                todo!("domain ptr");
                // break;
            }

            if buf.in_use.remaining() < len {
                return Err(ParserError::NotEnoughBytes {
                    expected: len,
                    recieved: buf.in_use.remaining(),
                });
            }

            let name = buf.in_use.slice(0..len);

            let part = match core::str::from_utf8(&name) {
                Ok(part) => {
                    if part.is_ascii() {
                        part
                    } else {
                        return Err(ParserError::InvalidAscii(AsciiError::NotAscii));
                    }
                }
                Err(err) => return Err(ParserError::InvalidAscii(AsciiError::InvalidUtf8(err))),
            };

            result.push(part.to_string());

            buf.in_use.advance(len);
        }

        Ok(Domain(result))
    }
}

impl Parsable for Header {
    type Error = ParserError;
    fn parse(buf: &BytesBuf) -> Result<Self, Self::Error>
    where
        Self: std::marker::Sized,
    {
        let mut buf = buf.clone();

        if buf.in_use.len() < 12 {
            return Err(ParserError::NotEnoughBytes {
                expected: 12,
                recieved: buf.in_use.len(),
            });
        }

        // Id is just a normal u16
        let id = buf.in_use.get_u16();

        // The next few pieces of data are all stored in this same u16 value
        let chunk = buf.in_use.get_u16();

        let is_response = chunk >> 15 == 1;
        let opcode: OpCode = ((chunk >> 11) & 0xf).into();
        let is_authoritative = (chunk >> 10) & 0b1 == 1;
        let is_truncated = (chunk >> 9) & 0b1 == 1;
        let should_recurse = (chunk >> 8) & 0b1 == 1;
        let recursion_available = (chunk >> 7) & 0b1 == 1;
        let _z = ((chunk >> 4) & 0b111) as u8;
        let rescode: ResCode = (chunk & 0xF).into();

        let question_count: u16 = buf.in_use.get_u16();
        let answer_record_count: u16 = buf.in_use.get_u16();
        let authority_record_count: u16 = buf.in_use.get_u16();
        let additional_record_count: u16 = buf.in_use.get_u16();

        Ok(Header {
            id,
            is_response,
            opcode,
            is_authoritative,
            is_truncated,
            should_recurse,
            recursion_available,
            _z,
            rescode,
            questions: question_count,
            answer_records: answer_record_count,
            authority_records: authority_record_count,
            additional_records: additional_record_count,
        })
    }
}

impl Parsable for Question {
    type Error = ParserError;

    fn parse(buf: &BytesBuf) -> Result<Self, Self::Error>
    where
        Self: std::marker::Sized,
    {
        let buf = buf.clone();
        todo!()
    }
}
