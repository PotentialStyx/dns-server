use bytes::BytesMut;

pub trait Serializable {
    type Error;

    /// Serializes type into `data`
    fn serialize(&self, buf: &mut BytesMut) -> Result<(), Self::Error>
    where
        Self: std::marker::Sized;
}

pub trait InfallibleSerializable {
    /// Serializes type into `data`
    fn serialize_infallible(&self, buf: &mut BytesMut)
    where
        Self: std::marker::Sized;
}

impl<T: InfallibleSerializable> Serializable for T {
    type Error = ();

    fn serialize(&self, buf: &mut BytesMut) -> Result<(), Self::Error>
    where
        Self: std::marker::Sized,
    {
        self.serialize_infallible(buf);
        Ok(())
    }
}
