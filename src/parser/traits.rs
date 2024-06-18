use super::BytesBuf;

pub trait Parsable {
    type Error;
    /// Parses data in `data` into type, without incrementing it's pointer
    fn parse(buf: &mut BytesBuf) -> Result<Self, Self::Error>
    where
        Self: std::marker::Sized;
}
