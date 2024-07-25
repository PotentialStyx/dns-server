use bytes::{Buf, Bytes};

use crate::parser::{AsciiError, BytesBuf, ParserError};

use crate::{OpCode, RecordClass, RecordType, ResCode};

use super::{Parsable, PartialResult};
use crate::{Domain, Header, Message, Question, ResourceRecord};

pub enum DomainErrorLocation {
    NameLengthTag,
    CompressedTag,
    TagParsing,
}

impl Parsable for Domain {
    type Error = ParserError;
    type ErrorLocation = DomainErrorLocation;

    fn parse(buf: &mut BytesBuf) -> PartialResult<Self, Self::ErrorLocation, Self::Error> {
        let mut result = vec![];
        loop {
            if buf.in_use.remaining() == 0 {
                let err = ParserError::NotEnoughBytes {
                    expected: 1,
                    recieved: 0,
                };

                if result.is_empty() {
                    return PartialResult::FullErr(err);
                } else {
                    return PartialResult::PartialOk(
                        Domain(result),
                        DomainErrorLocation::NameLengthTag,
                        err,
                    );
                }
            }

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

                let res = Domain::parse(&mut original);

                let mut compressed = match res {
                    PartialResult::FullErr(_) | PartialResult::PartialOk(_, _, _) => return res,
                    PartialResult::FullOk(value) => value,
                };

                result.append(&mut compressed.0);

                break;
            }

            // Conversion shouldn't fail as this will never target a less than 8 bit system.
            let len: usize = len.into();

            if buf.in_use.remaining() < len {
                return PartialResult::FullErr(ParserError::NotEnoughBytes {
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
                        let err = ParserError::InvalidAscii(AsciiError::NotAscii);

                        if result.is_empty() {
                            return PartialResult::FullErr(err);
                        } else {
                            return PartialResult::PartialOk(
                                Domain(result),
                                DomainErrorLocation::TagParsing,
                                err,
                            );
                        }
                    }
                }
                Err(err) => {
                    let err = ParserError::InvalidAscii(AsciiError::InvalidUtf8(err));

                    if result.is_empty() {
                        return PartialResult::FullErr(err);
                    } else {
                        return PartialResult::PartialOk(
                            Domain(result),
                            DomainErrorLocation::TagParsing,
                            err,
                        );
                    }
                }
            };

            result.push(part.to_string());

            buf.in_use.advance(len);
        }

        PartialResult::FullOk(Domain(result))
    }
}

impl Parsable for Header {
    type Error = ParserError;
    type ErrorLocation = ();
    fn parse(buf: &mut BytesBuf) -> PartialResult<Self, Self::ErrorLocation, Self::Error>
    where
        Self: std::marker::Sized,
    {
        if buf.in_use.len() < 12 {
            return PartialResult::FullErr(ParserError::NotEnoughBytes {
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

        PartialResult::FullOk(Header {
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

pub enum QuestionErrorLocation {
    AfterDomain,
}

impl Parsable for Question {
    type Error = ParserError;
    type ErrorLocation = QuestionErrorLocation;

    fn parse(buf: &mut BytesBuf) -> PartialResult<Self, Self::ErrorLocation, Self::Error>
    where
        Self: std::marker::Sized,
    {
        let name = {
            let name_res = Domain::parse(buf);
            match name_res {
                // PartialOk from Domain turns into a FullErr because if the domain isn't parsable then nothing has been parsed and there is no reason to return an empty Question
                PartialResult::FullErr(err) | PartialResult::PartialOk(_, _, err) => {
                    return PartialResult::FullErr(err);
                }
                PartialResult::FullOk(value) => value,
            }
        };

        if buf.in_use.remaining() < 4 {
            return PartialResult::PartialOk(
                Question {
                    name,
                    qtype: RecordType::Unknown(0),
                    qclass: RecordClass::Unknown(0),
                },
                QuestionErrorLocation::AfterDomain,
                ParserError::NotEnoughBytes {
                    expected: 4,
                    recieved: buf.in_use.remaining(),
                },
            );
        }

        let qtype = buf.in_use.get_u16().into();
        let qclass = buf.in_use.get_u16().into();

        PartialResult::FullOk(Question {
            name,
            qtype,
            qclass,
        })
    }
}

pub enum ResourceRecordErrorLocation {
    AfterDomain,
    DataLength,
}

impl Parsable for ResourceRecord {
    type Error = ParserError;
    type ErrorLocation = ResourceRecordErrorLocation;

    fn parse(buf: &mut BytesBuf) -> PartialResult<Self, Self::ErrorLocation, Self::Error>
    where
        Self: std::marker::Sized,
    {
        let name = {
            let name_res = Domain::parse(buf);
            match name_res {
                // PartialOk from Domain turns into a FullErr because if the domain isn't parsable then nothing has been parsed and there is no reason to return an empty ResourceRecord
                PartialResult::FullErr(err) | PartialResult::PartialOk(_, _, err) => {
                    return PartialResult::FullErr(err);
                }
                PartialResult::FullOk(value) => value,
            }
        };

        if buf.in_use.remaining() < 8 {
            return PartialResult::PartialOk(
                ResourceRecord {
                    name,
                    rtype: RecordType::Unknown(0),
                    rclass: RecordClass::Unknown(0),
                    ttl: 0,
                    data: Bytes::new(),
                    domain_data: None,
                    after_ptr: None,
                },
                ResourceRecordErrorLocation::AfterDomain,
                ParserError::NotEnoughBytes {
                    expected: 8,
                    recieved: buf.in_use.remaining(),
                },
            );
        }

        let rtype: RecordType = buf.in_use.get_u16().into();
        let rclass: RecordClass = buf.in_use.get_u16().into();
        let ttl = buf.in_use.get_u32();

        let data_len: usize = buf.in_use.get_u16().into();

        if buf.in_use.remaining() < data_len {
            return PartialResult::PartialOk(
                ResourceRecord {
                    name,
                    rtype,
                    rclass,
                    ttl,
                    data: Bytes::new(),
                    domain_data: None,
                    after_ptr: None,
                },
                ResourceRecordErrorLocation::DataLength,
                ParserError::NotEnoughBytes {
                    expected: data_len,
                    recieved: buf.in_use.remaining(),
                },
            );
        }

        let data = buf.in_use.slice(0..data_len);

        // TODO: support these again
        let after_ptr = None;
        let domain_data = None;
        // match rtype {
        //     // TODO: deal with the case of a CNAME having more data after the name but the buffer isnt properly advanced
        //     // TODO: deal with name.len() >= data.len()
        //     // TODO: add tests for this
        //     RecordType::NS | RecordType::CNAME => Some(vec![Domain::parse(buf)?]),
        //     RecordType::MX => {
        //         buf.in_use.get_u16();
        //         Some(vec![Domain::parse(buf)?])
        //     }
        //     RecordType::SOA => {
        //         let ptr = buf.in_use.remaining();

        //         let mut tmp_buf = buf.clone();

        //         let domains = vec![Domain::parse(&mut tmp_buf)?, Domain::parse(&mut tmp_buf)?];

        //         after_ptr = Some(ptr - tmp_buf.in_use.remaining());

        //         buf.in_use.advance(data_len);
        //         Some(domains)
        //     }
        //     _ => {
        //         buf.in_use.advance(data_len);

        //         None
        //     }
        // };

        PartialResult::FullOk(ResourceRecord {
            name,
            rtype,
            rclass,
            ttl,
            data,
            domain_data,
            after_ptr,
        })
    }
}

// impl Parsable for Message {
//     type Error = ParserError;

//     fn parse(buf: &mut BytesBuf) -> Result<Self, Self::Error>
//     where
//         Self: std::marker::Sized,
//     {
//         let header = Header::parse(buf)?;

//         if header.is_truncated {
//             return Ok(Message {
//                 header,
//                 questions: vec![],
//                 answers: vec![],
//                 authorities: vec![],
//                 additional: vec![],
//             });
//         }

//         let mut questions = vec![];
//         for _ in 0..header.questions {
//             questions.push(Question::parse(buf)?);
//         }

//         let mut answers = vec![];
//         for _ in 0..header.answer_records {
//             answers.push(ResourceRecord::parse(buf)?);
//         }

//         let mut authorities = vec![];
//         for _ in 0..header.authority_records {
//             authorities.push(ResourceRecord::parse(buf)?);
//         }

//         let mut additional = vec![];
//         for _ in 0..header.additional_records {
//             additional.push(ResourceRecord::parse(buf)?);
//         }

//         Ok(Message {
//             header,
//             questions,
//             answers,
//             authorities,
//             additional,
//         })
//     }
// }
