use bytes::BufMut;
use thiserror::Error;

use super::{Domain, Header, InfallibleSerializable, Serializable};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SerializerError {
    #[error("Domain included invalid ascii: {0}")]
    InvalidAscii(String),
    #[error("Too many bytes, expected maximum of {expected_max:?} bytes, got {recieved:?}")]
    TooManyBytes {
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
    fn serialize(&self, buf: &mut bytes::BytesMut)
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
