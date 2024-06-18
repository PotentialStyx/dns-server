use bytes::Bytes;

pub trait Parsable {
    type Error;
    /// Parses data in `data` into type, without incrementing it's pointer
    fn parse(data: &Bytes) -> Result<Self, Self::Error>
    where
        Self: std::marker::Sized;
}
