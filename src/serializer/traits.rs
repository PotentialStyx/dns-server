use bytes::BytesMut;

pub trait Serializable {
    type Error;

    /// Serializes type into `data`
    fn serialize(&self, buf: &mut BytesMut) -> Result<(), Self::Error>
    where
        Self: std::marker::Sized;
}
