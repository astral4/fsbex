use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Read,
};

use crate::header::{error::HeaderError, Header};
use crate::read::Reader;

struct Bank<R: Read> {
    header: Header,
    read: Reader<R>,
}

impl<R: Read> Bank<R> {
    fn new(source: R) -> Result<Self, DecodeError> {
        let mut read = Reader::new(source);
        let header = Header::parse(&mut read)?;
        Ok(Self { header, read })
    }
}

#[derive(Debug)]
struct DecodeError {
    inner: Box<HeaderError>,
}

impl From<HeaderError> for DecodeError {
    fn from(value: HeaderError) -> Self {
        Self {
            inner: Box::new(value),
        }
    }
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.inner.fmt(f)
    }
}

impl Error for DecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.inner.source()
    }
}
