use crate::parse::{ParseError, Reader};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Read,
};

#[derive(Debug, PartialEq)]
struct Header {}

const FSB5_MAGIC: [u8; 4] = *b"FSB5";

impl Header {
    fn parse<R: Read>(reader: &mut Reader<R>) -> Result<Self, HeaderError> {
        if reader
            .take()
            .map_err(|e| HeaderError::new_with_source(HeaderErrorKind::Magic, e))?
            != FSB5_MAGIC
        {
            return Err(HeaderError::new(HeaderErrorKind::Magic));
        }

        todo!()
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
    use super::{Header, HeaderError, HeaderErrorKind};
    use crate::parse::{Needed, ParseError, Reader};
    use std::num::NonZeroUsize;

    #[test]
    fn parse_magic() {
        let mut reader;

        reader = Reader::new(b"".as_slice());
        assert_eq!(
            Header::parse(&mut reader),
            Err(HeaderError {
                kind: HeaderErrorKind::Magic,
                source: Some(ParseError::Incomplete(Needed::Size(
                    NonZeroUsize::new(4).unwrap()
                )))
            })
        );

        reader = Reader::new(b"abcd".as_slice());
        assert_eq!(
            Header::parse(&mut reader),
            Err(HeaderError {
                kind: HeaderErrorKind::Magic,
                source: None
            })
        );
    }
}
