use core::str::Utf8Error;

use bytes::{Buf, Bytes};
use thiserror::Error;

use super::Domain;

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

pub trait Parsable {
    /// Parses data in `data` into type, without incrementing it's pointer
    fn parse(data: &Bytes) -> Result<Self, ParserError>
    where
        Self: std::marker::Sized;
}

impl Parsable for Domain {
    fn parse(data: &Bytes) -> Result<Self, ParserError> {
        let mut result = vec![];
        let mut buf = BytesBuf::from_bytes(data.clone());
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
