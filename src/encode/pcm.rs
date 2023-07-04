use crate::{header::StreamInfo, read::Reader};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::{Read, Write},
};

pub(super) fn encode<R: Read, W: Write, const BIT_DEPTH: u8, const IS_INT: bool>(
    info: &StreamInfo,
    source: &mut Reader<R>,
    sink: W,
) -> Result<(), PcmError> {
    todo!()
}

#[derive(Debug)]
pub(crate) struct PcmError {
    kind: PcmErrorKind,
}

#[derive(Debug)]
enum PcmErrorKind {
    CreateHeader,
}

#[derive(Debug)]
enum PcmErrorSource {}

impl Display for PcmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(match self.kind {
            PcmErrorKind::CreateHeader => "failed to create PCM stream header",
        })
    }
}

impl Error for PcmError {}
