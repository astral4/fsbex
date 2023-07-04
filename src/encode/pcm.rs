use crate::{header::StreamInfo, read::Reader};
use hound::SampleFormat;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::{Read, Write},
};

pub(super) fn encode<R: Read, W: Write>(
    bit_depth: u8,
    format: SampleFormat,
    info: &StreamInfo,
    source: &mut Reader<R>,
    sink: W,
) -> Result<(), PcmError> {
    todo!()
}

#[derive(Debug)]
pub(crate) struct PcmError {
    kind: PcmErrorKind,
    source: PcmErrorSource,
}

#[derive(Debug)]
enum PcmErrorKind {}

#[derive(Debug)]
enum PcmErrorSource {}

impl Display for PcmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        todo!()
    }
}

impl Error for PcmError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        todo!()
    }
}
