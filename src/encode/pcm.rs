use crate::{header::StreamInfo, read::Reader};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::{Error as IoError, Read, Write},
};

pub(super) fn encode<R: Read, W: Write, const BYTE_DEPTH: u16, const IS_INT: bool>(
    info: &StreamInfo,
    source: &mut Reader<R>,
    sink: W,
) -> Result<(), PcmError> {
    let mut sink = sink;

    write_header::<_, BYTE_DEPTH, IS_INT>(
        info.size.into(),
        u16::from(u8::from(info.channels)),
        info.sample_rate.into(),
        &mut sink,
    )
    .map_err(PcmError::factory(PcmErrorKind::CreateHeader))?;

    todo!()
}

fn write_header<W: Write, const BYTE_DEPTH: u16, const IS_INT: bool>(
    file_size: u32,
    channels: u16,
    sample_rate: u32,
    sink: &mut W,
) -> Result<(), IoError> {
    sink.write_all(b"RIFF")?;
    sink.write_all((file_size - 8).to_le_bytes().as_slice())?;
    sink.write_all(b"WAVE")?;
    sink.write_all(b"fmt ")?;
    sink.write_all(16u32.to_le_bytes().as_slice())?;
    sink.write_all((if IS_INT { 1u16 } else { 3u16 }).to_le_bytes().as_slice())?;
    sink.write_all(channels.to_le_bytes().as_slice())?;
    sink.write_all(sample_rate.to_le_bytes().as_slice())?;
    sink.write_all(
        (sample_rate * u32::from(channels) * u32::from(BYTE_DEPTH))
            .to_le_bytes()
            .as_slice(),
    )?;
    sink.write_all((channels * BYTE_DEPTH).to_le_bytes().as_slice())?;
    sink.write_all((BYTE_DEPTH * 8).to_le_bytes().as_slice())?;
    sink.write_all(b"data")?;
    sink.write_all((file_size - 40).to_le_bytes().as_slice())?;

    Ok(())
}

#[derive(Debug)]
pub(crate) struct PcmError {
    kind: PcmErrorKind,
    source: IoError,
}

#[derive(Debug)]
enum PcmErrorKind {
    CreateHeader,
}

impl PcmError {
    fn factory(kind: PcmErrorKind) -> impl FnOnce(IoError) -> Self {
        |source| Self { kind, source }
    }
}

impl Display for PcmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(match self.kind {
            PcmErrorKind::CreateHeader => "failed to encode PCM stream header",
        })
    }
}

impl Error for PcmError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}
