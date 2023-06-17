use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::{Error as IoError, ErrorKind, Read},
    num::NonZeroUsize,
};

pub(crate) struct Reader<R: Read>(R);

impl<R: Read> Reader<R> {
    pub(crate) fn new(reader: R) -> Self {
        Self(reader)
    }

    fn read_to_array<const LEN: usize>(&mut self, buf: &mut [u8; LEN]) -> ParseResult<()> {
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
                ErrorKind::Interrupted => self.read_to_array(buf),
                ErrorKind::UnexpectedEof => Err(ParseError::Incomplete(Needed::Unknown)),
                _ => Err(ParseError::Failure(e)),
            },
        }
    }

    fn read_to_slice(&mut self, buf: &mut [u8]) -> ParseResult<()> {
        match self.0.read(buf) {
            Ok(n) => {
                let buf_len = buf.len();
                if n == buf_len {
                    Ok(())
                } else {
                    Err(ParseError::Incomplete(Needed::Size(
                        NonZeroUsize::new(buf_len - n)
                            .expect("n is guaranteed to not equal buf_len"),
                    )))
                }
            }
            Err(e) => match e.kind() {
                ErrorKind::Interrupted => self.read_to_slice(buf),
                ErrorKind::UnexpectedEof => Err(ParseError::Incomplete(Needed::Unknown)),
                _ => Err(ParseError::Failure(e)),
            },
        }
    }

    pub(crate) fn take<const LEN: usize>(&mut self) -> ParseResult<[u8; LEN]> {
        let mut buf = [0; LEN];
        Self::read_to_array(self, &mut buf)?;
        Ok(buf)
    }

    pub(crate) fn skip(&mut self, amount: usize) -> ParseResult<()> {
        let mut buf = vec![0u8; amount];
        Self::read_to_slice(self, buf.as_mut_slice())
    }

    pub(crate) fn u8(&mut self) -> ParseResult<u8> {
        let mut buf = [0; 1];
        Self::read_to_array(self, &mut buf)?;
        Ok(buf[0])
    }

    pub(crate) fn le_u32(&mut self) -> ParseResult<u32> {
        let mut buf = [0; 4];
        Self::read_to_array(self, &mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub(crate) fn le_u64(&mut self) -> ParseResult<u64> {
        let mut buf = [0; 8];
        Self::read_to_array(self, &mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    pub(crate) fn le_i32(&mut self) -> ParseResult<i32> {
        let mut buf = [0; 4];
        Self::read_to_array(self, &mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }
}

type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub(crate) enum ParseError {
    Failure(IoError),
    Incomplete(Needed),
}

#[cfg(test)]
impl PartialEq for ParseError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Failure(first), Self::Failure(second)) => first.kind() == second.kind(),
            (Self::Incomplete(first), Self::Incomplete(second)) => first == second,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Needed {
    Size(NonZeroUsize),
    Unknown,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Failure(_) => f.write_str("failed to parse data due to I/O error"),
            Self::Incomplete(needed) => match needed {
                Needed::Size(size) => f.write_str(&format!(
                    "incomplete data: needed {size} more bytes to parse"
                )),
                Needed::Unknown => f.write_str("incomplete data"),
            },
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Failure(err) => Some(err),
            Self::Incomplete(_) => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Needed, ParseError, ParseResult, Reader};
    use std::{
        io::{Error as IoError, ErrorKind, Read, Result as IoResult},
        num::NonZeroUsize,
    };

    #[test]
    fn take_bytes() {
        let data = b"abc123";
        let mut reader = Reader::new(data.as_slice());

        assert_eq!(reader.take(), Ok([97]));
        assert_eq!(reader.take(), Ok([98, 99]));
        assert_eq!(reader.take(), Ok([49, 50, 51]));
        assert_eq!(reader.take(), Ok([]));
        assert_eq!(
            reader.take::<1>(),
            Err(ParseError::Incomplete(Needed::Size(
                NonZeroUsize::new(1).unwrap()
            )))
        );
    }

    #[test]
    fn skip_bytes() {
        let data = b"abc123";
        let mut reader = Reader::new(data.as_slice());

        assert_eq!(reader.skip(1), Ok(()));
        assert_eq!(reader.skip(2), Ok(()));
        assert_eq!(reader.skip(3), Ok(()));
        assert_eq!(reader.skip(0), Ok(()));
        assert_eq!(
            reader.skip(1),
            Err(ParseError::Incomplete(Needed::Size(
                NonZeroUsize::new(1).unwrap()
            )))
        );
    }

    #[test]
    fn parse_single_number() {
        let data = b"\x00\x00\x00\x00\x00\x00";
        let mut reader = Reader::new(data.as_slice());

        assert_eq!(reader.le_u32(), Ok(0));
    }

    #[test]
    fn parse_multiple_numbers() {
        let data = b"\x11\x00\x00\x00\x34\x12\x00\x00\x66\x66\x66\x66\xFF\xFF\xFF\xFF";
        let mut reader = Reader::new(data.as_slice());

        assert_eq!(reader.le_u32(), Ok(17));
        assert_eq!(reader.le_u32(), Ok(4660));
        assert_eq!(reader.le_u32(), Ok(1_717_986_918));
        assert_eq!(reader.le_u32(), Ok(u32::MAX));
    }

    #[test]
    fn parse_multiple_number_types() {
        let data = b"\x11\x00\x00\x00\x00\x00\x00\xFF\xFF\x22";
        let mut reader = Reader::new(data.as_slice());

        assert_eq!(reader.le_u32(), Ok(17));
        assert_eq!(reader.u8(), Ok(0));
        assert_eq!(reader.le_i32(), Ok(-65536));
        assert_eq!(reader.u8(), Ok(34));
    }

    #[test]
    fn handle_incomplete_data() {
        let data = b"\x00\x00";
        let mut reader = Reader::new(data.as_slice());

        assert_eq!(
            reader.le_u32(),
            Err(ParseError::Incomplete(Needed::Size(
                NonZeroUsize::new(2).unwrap()
            )))
        );
    }

    impl<R: Read> Reader<R> {
        fn unit(&mut self) -> ParseResult<()> {
            let mut buf = [0; 0];
            Self::read_to_array(self, &mut buf)
        }
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
        let mut buf = [0; 0];
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
        let mut buf = [0; 0];
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

    struct UnsupportedReader;

    impl Read for UnsupportedReader {
        fn read(&mut self, _buf: &mut [u8]) -> IoResult<usize> {
            Err(IoError::from(ErrorKind::Unsupported))
        }
    }

    #[test]
    fn unsupported_reader_works() {
        let mut buf = [0; 0];
        let mut reader = UnsupportedReader;

        assert!(reader
            .read(&mut buf)
            .is_err_and(|e| e.kind() == ErrorKind::Unsupported));
    }

    #[test]
    fn handle_misc_io_error() {
        let mut reader = Reader::new(UnsupportedReader);

        assert_eq!(
            reader.unit(),
            Err(ParseError::Failure(IoError::from(ErrorKind::Unsupported)))
        );
    }
}
