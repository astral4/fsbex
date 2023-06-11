use std::{
    borrow::{Borrow, Cow},
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::{ErrorKind, Read},
    num::NonZeroUsize,
};

struct Reader<R: Read>(R);

impl<R: Read> Reader<R> {
    fn new(reader: R) -> Self {
        Self(reader)
    }

    fn read_to_buf<const LEN: usize>(&mut self, buf: &mut [u8; LEN]) -> ParseResult<()> {
        match self.0.read(buf) {
            Ok(n) => {
                if n == LEN {
                    Ok(())
                } else {
                    Err(ParseError::Incomplete(Needed::Size(
                        NonZeroUsize::new(LEN - n).expect("n is guaranteed to not equal LEN"),
                    )))
                }
            }
            Err(e) => match e.kind() {
                ErrorKind::Interrupted => self.read_to_buf::<LEN>(buf),
                ErrorKind::UnexpectedEof => Err(ParseError::Incomplete(Needed::Unknown)),
                _ => Err(ParseError::Failure),
            },
        }
    }

    fn le_u32(&mut self) -> ParseResult<u32> {
        let mut buf: [u8; 4] = Default::default();
        Self::read_to_buf::<4>(self, &mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }
}

type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
enum ParseError {
    Incomplete(Needed),
    Failure,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
enum Needed {
    Unknown,
    Size(NonZeroUsize),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let msg = match self {
            Self::Incomplete(needed) => match needed {
                Needed::Unknown => Cow::Borrowed("Incomplete data"),
                Needed::Size(size) => Cow::Owned(format!(
                    "Incomplete data: needed {size} more bytes to parse"
                )),
            },
            Self::Failure => Cow::Borrowed("Failed to parse data"),
        };

        f.write_str(msg.borrow())
    }
}

impl Error for ParseError {}

#[cfg(test)]
mod test {
    use super::{Needed, ParseError, ParseResult, Reader};
    use std::{
        io::{Error as IoError, ErrorKind, Read, Result as IoResult},
        num::NonZeroUsize,
    };

    #[test]
    fn parse_single_number() {
        let data = b"\x00\x00\x00\x00\x00\x00";
        let mut reader = Reader::new(&data[..]);

        assert_eq!(reader.le_u32(), Ok(0));
    }

    #[test]
    fn parse_multiple_numbers() {
        let data = b"\x11\x00\x00\x00\x34\x12\x00\x00\x66\x66\x66\x66\xFF\xFF\xFF\xFF";
        let mut reader = Reader::new(&data[..]);

        assert_eq!(reader.le_u32(), Ok(17));
        assert_eq!(reader.le_u32(), Ok(4660));
        assert_eq!(reader.le_u32(), Ok(1_717_986_918));
        assert_eq!(reader.le_u32(), Ok(u32::MAX));
    }

    #[test]
    fn handle_incomplete_data() {
        let data = b"\x00\x00";
        let mut reader = Reader::new(&data[..]);

        assert_eq!(
            reader.le_u32(),
            Err(ParseError::Incomplete(Needed::Size(
                NonZeroUsize::new(2).unwrap()
            )))
        );
    }

    struct InterruptReader(usize);

    impl Read for InterruptReader {
        fn read(&mut self, _buf: &mut [u8]) -> IoResult<usize> {
            if self.0 < 3 {
                self.0 += 1;
                Err(IoError::from(ErrorKind::Interrupted))
            } else {
                Ok(0)
            }
        }
    }

    #[test]
    fn interrupt_reader_works() {
        let mut buf: [u8; 0] = Default::default();
        let mut reader = InterruptReader(0);

        assert!(reader
            .read(&mut buf)
            .is_err_and(|e| e.kind() == ErrorKind::Interrupted));
        assert!(reader
            .read(&mut buf)
            .is_err_and(|e| e.kind() == ErrorKind::Interrupted));
        assert!(reader
            .read(&mut buf)
            .is_err_and(|e| e.kind() == ErrorKind::Interrupted));
        assert!(matches!(reader.read(&mut buf), Ok(0)));
    }

    impl<R: Read> Reader<R> {
        fn unit(&mut self) -> ParseResult<()> {
            let mut buf: [u8; 0] = Default::default();
            Self::read_to_buf::<0>(self, &mut buf)
        }
    }

    #[test]
    fn handle_interrupted_io() {
        let mut reader = Reader::new(InterruptReader(0));

        assert_eq!(reader.unit(), Ok(()));
    }

    struct EofReader;

    impl Read for EofReader {
        fn read(&mut self, _buf: &mut [u8]) -> IoResult<usize> {
            Err(IoError::from(ErrorKind::UnexpectedEof))
        }
    }

    #[test]
    fn eof_reader_works() {
        let mut buf: [u8; 0] = Default::default();
        let mut reader = EofReader;

        assert!(reader
            .read(&mut buf)
            .is_err_and(|e| e.kind() == ErrorKind::UnexpectedEof));
    }

    #[test]
    fn handle_unexpected_eof() {
        let mut reader = Reader::new(EofReader);

        assert_eq!(reader.unit(), Err(ParseError::Incomplete(Needed::Unknown)));
    }
}
