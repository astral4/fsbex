use crate::parse::{ParseError, Reader};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Read,
};

#[derive(Debug, PartialEq)]
struct Header {}

impl Header {
    fn parse<R: Read>(reader: &mut Reader<R>) -> Result<Self, HeaderError> {
        match reader.take() {
            Ok(data) if data == FSB5_MAGIC => Ok(()),
            Err(e) => Err(HeaderError::new_with_source(HeaderErrorKind::Magic, e)),
            _ => Err(HeaderError::new(HeaderErrorKind::Magic)),
        }?;

        let version = match reader.le_u32() {
            Ok(n) => Version::parse(n),
            Err(e) => Err(HeaderError::new_with_source(HeaderErrorKind::Version, e)),
        }?;

        todo!()
    }
}

const FSB5_MAGIC: [u8; 4] = *b"FSB5";

enum Version {
    V0,
    V1,
}

impl Version {
    fn parse(num: u32) -> Result<Self, HeaderError> {
        match num {
            0 => Ok(Self::V0),
            1 => Ok(Self::V1),
            _ => Err(HeaderError::new(HeaderErrorKind::Version)),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
struct HeaderError {
    kind: HeaderErrorKind,
    source: Option<ParseError>,
}

#[derive(Debug, PartialEq)]
enum HeaderErrorKind {
    Magic,
    Version,
}

impl HeaderError {
    fn new(kind: HeaderErrorKind) -> Self {
        Self { kind, source: None }
    }
    fn new_with_source(kind: HeaderErrorKind, source: ParseError) -> Self {
        Self {
            kind,
            source: Some(source),
        }
    }
}

impl Display for HeaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.kind {
            HeaderErrorKind::Magic => f.write_str("no file signature found"),
            HeaderErrorKind::Version => f.write_str("invalid file format version"),
        }
    }
}

impl Error for HeaderError {
    #[allow(trivial_casts)]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| e as &dyn Error)
    }
}

#[cfg(test)]
mod test {
    use super::{Header, HeaderErrorKind, FSB5_MAGIC};
    use crate::parse::Reader;

    #[test]
    fn parse_magic() {
        let mut reader;

        reader = Reader::new(b"".as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == HeaderErrorKind::Magic));

        reader = Reader::new(b"abcd".as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == HeaderErrorKind::Magic));

        reader = Reader::new(FSB5_MAGIC.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == HeaderErrorKind::Version));
    }
}
