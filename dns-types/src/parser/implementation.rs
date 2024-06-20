use core::str::Utf8Error;

use bytes::{Buf, Bytes};
use thiserror::Error;

use crate::{OpCode, RecordClass, RecordType, ResCode};

use crate::parser::Parsable;
use crate::{Domain, Header, Message, Question, ResourceRecord};

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
pub struct BytesBuf {
    original: Bytes,
    pub(crate) in_use: Bytes,
}

impl BytesBuf {
    pub fn new(data: Vec<u8>) -> BytesBuf {
        let data: Bytes = data.into();
        BytesBuf {
            original: data.clone(),
            in_use: data,
        }
    }

    pub fn from_bytes(data: Bytes) -> BytesBuf {
        BytesBuf {
            original: data.clone(),
            in_use: data,
        }
    }

    /// Get a copy of the original `Bytes` object at ptr=0
    pub fn get_original(&self) -> Bytes {
        self.original.clone()
    }
}

impl Parsable for Domain {
    type Error = ParserError;
    fn parse(buf: &mut BytesBuf) -> Result<Self, Self::Error> {
        let mut result = vec![];
        loop {
            if buf.in_use.remaining() == 0 {
                return Err(ParserError::NotEnoughBytes {
                    expected: 1,
                    recieved: 0,
                });
            }

            // Conversion shouldn't fail as this will never target a less than 8 bit system.
            let len = buf.in_use.get_u8();

            if len == 0 {
                break;
            }

            if (len >> 6) == 0b11 {
                let left = len as u16 ^ 0xC0; // 0b11000000
                let right = buf.in_use.get_u8() as u16;

                let ptr = ((left << 8) + right) as usize;

                let mut original = BytesBuf::from_bytes(buf.get_original());
                original.in_use.advance(ptr);

                let mut compressed = Domain::parse(&mut original)?;
                result.append(&mut compressed.0);

                break;
            }

            let len: usize = len.into();

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
    fn parse(buf: &mut BytesBuf) -> Result<Self, Self::Error>
    where
        Self: std::marker::Sized,
    {
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
        let opcode: OpCode = ((chunk >> 11) & 0x0F).into();
        let is_authoritative = (chunk >> 10) & 0b1 == 1;
        let is_truncated = (chunk >> 9) & 0b1 == 1;
        let should_recurse = (chunk >> 8) & 0b1 == 1;
        let recursion_available = (chunk >> 7) & 0b1 == 1;
        let _z = ((chunk >> 4) & 0b111) as u8;
        let rescode: ResCode = (chunk & 0x0F).into();

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

    fn parse(buf: &mut BytesBuf) -> Result<Self, Self::Error>
    where
        Self: std::marker::Sized,
    {
        let name = Domain::parse(buf)?;

        if buf.in_use.remaining() < 4 {
            return Err(ParserError::NotEnoughBytes {
                expected: 4,
                recieved: buf.in_use.remaining(),
            });
        }

        let qtype = buf.in_use.get_u16().into();
        let qclass = buf.in_use.get_u16().into();

        Ok(Question {
            name,
            qtype,
            qclass,
        })
    }
}

impl Parsable for ResourceRecord {
    type Error = ParserError;

    fn parse(buf: &mut BytesBuf) -> Result<Self, Self::Error>
    where
        Self: std::marker::Sized,
    {
        let name = Domain::parse(buf)?;

        if buf.in_use.remaining() < 8 {
            return Err(ParserError::NotEnoughBytes {
                expected: 8,
                recieved: buf.in_use.remaining(),
            });
        }

        let rtype: RecordType = buf.in_use.get_u16().into();
        let rclass: RecordClass = buf.in_use.get_u16().into();
        let ttl = buf.in_use.get_u32();

        let data_len: usize = buf.in_use.get_u16().into();

        if buf.in_use.remaining() < data_len {
            return Err(ParserError::NotEnoughBytes {
                expected: data_len,
                recieved: buf.in_use.remaining(),
            });
        }

        let data = buf.in_use.slice(0..data_len);

        let domain_data = match rtype {
            // TODO: add tests for this
            RecordType::NS | RecordType::CNAME => Some(Domain::parse(buf)?),
            _ => {
                buf.in_use.advance(data_len);

                None
            }
        };

        Ok(ResourceRecord {
            name,
            rtype,
            rclass,
            ttl,
            data,
            domain_data,
        })
    }
}

impl Parsable for Message {
    type Error = ParserError;

    fn parse(buf: &mut BytesBuf) -> Result<Self, Self::Error>
    where
        Self: std::marker::Sized,
    {
        let header = Header::parse(buf)?;

        let mut questions = vec![];
        for _ in 0..header.questions {
            questions.push(Question::parse(buf)?);
        }

        let mut answers = vec![];
        for _ in 0..header.answer_records {
            answers.push(ResourceRecord::parse(buf)?);
        }

        let mut authorities = vec![];
        for _ in 0..header.authority_records {
            authorities.push(ResourceRecord::parse(buf)?);
        }

        let mut additional = vec![];
        for _ in 0..header.additional_records {
            additional.push(ResourceRecord::parse(buf)?);
        }

        Ok(Message {
            header,
            questions,
            answers,
            authorities,
            additional,
        })
    }
}
