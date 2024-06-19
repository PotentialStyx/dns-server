use bytes::BufMut;
use thiserror::Error;

use super::{Domain, Serializable};

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
