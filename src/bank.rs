use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::{Read, Write},
};

use crate::header::{error::HeaderError, Codec, Header, StreamInfo};
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

    fn process_streams<F, E>(mut self, f: F) -> Result<(), ProcessError<E>>
    where
        F: Fn(LazyStream<'_, R>) -> Result<(), E>,
    {
        for (info, index) in self.header.stream_info.into_iter().zip(0..) {
            f(LazyStream {
                index,
                info,
                codec: self.header.codec,
                reader: &mut self.read,
            })
            .map_err(|e| ProcessError::new(index, e))?;
        }
        Ok(())
    }
}

struct LazyStream<'bank, R: Read> {
    index: u32,
    info: StreamInfo,
    codec: Codec,
    reader: &'bank mut Reader<R>,
}

impl<'bank, R: Read> LazyStream<'bank, R> {
    fn write<W: Write>(sink: W) -> Result<(), EncodeError> {
        todo!()
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

#[derive(Debug)]
struct ProcessError<E> {
    index: u32,
    source: E,
}

impl<E> ProcessError<E> {
    fn new(index: u32, source: E) -> Self {
        Self { index, source }
    }
}

impl<E> Display for ProcessError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&format!("failed to process stream at index {}", self.index))
    }
}

impl<E: 'static + Error> Error for ProcessError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

struct EncodeError {
    source: EncodeErrorSource,
}

enum EncodeErrorSource {}
