use crate::parser::BytesBuf;

pub enum PartialResult<T, S, E> {
    FullErr(E),
    PartialOk(T, S, E),
    FullOk(T),
}

impl<T, S, E> From<PartialResult<T, S, E>> for Result<T, E> {
    fn from(value: PartialResult<T, S, E>) -> Self {
        match value {
            PartialResult::FullErr(err) | PartialResult::PartialOk(_, _, err) => Err(err),
            PartialResult::FullOk(value) => Ok(value),
        }
    }
}

impl<T, S, E> From<Result<T, E>> for PartialResult<T, S, E> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Err(err) => PartialResult::FullErr(err),
            Ok(value) => PartialResult::FullOk(value),
        }
    }
}

pub trait Parsable {
    type Error;
    type ErrorLocation;

    /// Parses data in `data` into type, while also incrementing it's pointer
    fn parse(buf: &mut BytesBuf) -> PartialResult<Self, Self::ErrorLocation, Self::Error>
    where
        Self: std::marker::Sized;
}
