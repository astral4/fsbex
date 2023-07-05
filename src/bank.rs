use crate::header::{error::HeaderError, Header};
use crate::read::Reader;
use crate::stream::{LazyStream, Stream, StreamIntoIter};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Read,
};

pub(crate) struct Bank<R: Read> {
    header: Header,
    read: Reader<R>,
}

impl<R: Read> Bank<R> {
    fn new(source: R) -> Result<Self, DecodeError> {
        let mut read = Reader::new(source);
        let header = Header::parse(&mut read)?;
        Ok(Self { header, read })
    }

    fn process_streams<F, E>(mut self, f: F) -> Result<(), (E, u32)>
    where
        F: Fn(LazyStream<'_, R>) -> Result<(), E>,
    {
        for (info, index) in self.header.stream_info.iter().zip(0..) {
            f(LazyStream::new(
                index,
                self.header.format,
                self.header.flags,
                info,
                &mut self.read,
            ))
            .map_err(|e| (e, index))?;
        }
        Ok(())
    }
}

impl<R: Read> From<Bank<R>> for StreamIntoIter<R> {
    fn from(value: Bank<R>) -> Self {
        Self::new(
            value.header.format,
            value.header.flags,
            value.header.stream_info,
            value.read,
        )
    }
}

impl<R: Read> IntoIterator for Bank<R> {
    type IntoIter = StreamIntoIter<R>;
    type Item = Stream;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::from(self)
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
