use bytes::BufMut;
use thiserror::Error;

use crate::serializer::{InfallibleSerializable, Serializable};
use crate::{Domain, Header, Message, Question, ResourceRecord};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SerializerError {
    #[error("Domain included invalid ascii: {0}")]
    InvalidAscii(String),
    #[error("Too many bytes, expected maximum of {expected_max:?} bytes, got {recieved:?}")]
    TooManyBytes {
        expected_max: usize,
        recieved: usize,
    },
    #[error(
        "Too many bytes, expected maximum of {expected_max:?} questions/records, got {recieved:?}"
    )]
    TooManyRecords {
        expected_max: usize,
        recieved: usize,
    },
    // Right now there are no unknown errors
    // #[error("Unknown error")]
    // Unknown,
}

impl Serializable for Domain {
    type Error = SerializerError;

    fn serialize(&self, buf: &mut bytes::BytesMut) -> Result<(), Self::Error>
    where
        Self: std::marker::Sized,
    {
        for part in &self.0 {
            if !part.is_ascii() {
                return Err(SerializerError::InvalidAscii(part.clone()));
            }

            let len = part.len();

            // Check that length is valid and won't be interpreted as a pointer
            if len > 0x3F {
                return Err(SerializerError::TooManyBytes {
                    expected_max: 0x3F,
                    recieved: len,
                });
            }

            // len has to fit into u8 due to previous check
            buf.put_u8(len as u8);

            // Reserve and insert segment
            buf.reserve(len);
            buf.put(part.as_bytes());
        }

        // Finally 0 marker (no parts left)
        buf.put_u8(0);

        Ok(())
    }
}

impl InfallibleSerializable for Header {
    fn serialize_infallible(&self, buf: &mut bytes::BytesMut)
    where
        Self: std::marker::Sized,
    {
        buf.reserve(12);

        buf.put_u16(self.id);

        // Many values are squeezed into this u16
        let mut chunk: u16 = 0;

        chunk |= Into::<u16>::into(self.rescode) & 0x0F;
        chunk |= ((self._z & 0x07) as u16) << 4;

        if self.recursion_available {
            chunk |= 0x80;
        }

        if self.should_recurse {
            chunk |= 0x0100;
        }

        if self.is_truncated {
            chunk |= 0x0200;
        }

        if self.is_authoritative {
            chunk |= 0x0400;
        }

        chunk |= (Into::<u16>::into(self.opcode) & 0x0F) << 11;

        if self.is_response {
            chunk |= 0x8000;
        }

        buf.put_u16(chunk);

        buf.put_u16(self.questions);
        buf.put_u16(self.answer_records);
        buf.put_u16(self.authority_records);
        buf.put_u16(self.additional_records);
    }
}

impl Serializable for Question {
    type Error = SerializerError;

    fn serialize(&self, buf: &mut bytes::BytesMut) -> Result<(), Self::Error>
    where
        Self: std::marker::Sized,
    {
        self.name.serialize(buf)?;

        buf.reserve(4);
        buf.put_u16(self.qtype.into());
        buf.put_u16(self.qclass.into());

        Ok(())
    }
}

impl Serializable for ResourceRecord {
    type Error = SerializerError;

    fn serialize(&self, buf: &mut bytes::BytesMut) -> Result<(), Self::Error>
    where
        Self: std::marker::Sized,
    {
        self.name.serialize(buf)?;

        buf.put_u16(self.rtype.into());
        buf.put_u16(self.rclass.into());

        buf.put_u32(self.ttl);

        let len_usize = self.data.len();
        let len: u16 = match len_usize.try_into() {
            Ok(len) => len,
            Err(_) => {
                return Err(SerializerError::TooManyBytes {
                    // Should be safe since I'm not targeting 8-bit targets
                    expected_max: u16::MAX as usize,
                    recieved: len_usize,
                });
            }
        };

        buf.reserve(2 + len_usize);

        buf.put_u16(len);
        buf.put(self.data.clone());

        Ok(())
    }
}

impl Serializable for Message {
    type Error = SerializerError;

    fn serialize(&self, buf: &mut bytes::BytesMut) -> Result<(), Self::Error>
    where
        Self: std::marker::Sized,
    {
        let mut header = self.header;

        header.questions =
            self.questions
                .len()
                .try_into()
                .map_err(|_| SerializerError::TooManyRecords {
                    expected_max: u16::MAX as usize,
                    recieved: self.questions.len(),
                })?;

        header.answer_records =
            self.answers
                .len()
                .try_into()
                .map_err(|_| SerializerError::TooManyRecords {
                    expected_max: u16::MAX as usize,
                    recieved: self.answers.len(),
                })?;

        header.authority_records =
            self.authorities
                .len()
                .try_into()
                .map_err(|_| SerializerError::TooManyRecords {
                    expected_max: u16::MAX as usize,
                    recieved: self.authorities.len(),
                })?;

        header.additional_records =
            self.additional
                .len()
                .try_into()
                .map_err(|_| SerializerError::TooManyRecords {
                    expected_max: u16::MAX as usize,
                    recieved: self.additional.len(),
                })?;

        header.serialize_infallible(buf);

        for question in &self.questions {
            question.serialize(buf)?;
        }

        for answer in &self.answers {
            answer.serialize(buf)?;
        }

        for authority in &self.authorities {
            authority.serialize(buf)?;
        }

        for additional in &self.additional {
            additional.serialize(buf)?;
        }

        Ok(())
    }
}
