use crate::{header::StreamInfo, read::Reader};
use hound::{SampleFormat, SampleWriter16, WavSpec, WavWriter};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::{Read, Seek, Write},
};

pub(super) fn encode<R: Read, W: Write + Seek>(
    bit_depth: u16,
    format: SampleFormat,
    info: &StreamInfo,
    source: &mut Reader<R>,
    sink: W,
) -> Result<(), PcmError> {
    let mut encoder = WavWriter::new(
        sink,
        WavSpec {
            channels: u16::from(u8::from(info.channels)),
            sample_rate: info.sample_rate.into(),
            bits_per_sample: bit_depth,
            sample_format: format,
        },
    )
    .map_err(PcmError::from_hound(PcmErrorKind::CreateEncoder))?;

    todo!()
}

#[derive(Debug)]
pub(crate) struct PcmError {
    kind: PcmErrorKind,
    source: PcmErrorSource,
}

#[derive(Debug)]
enum PcmErrorKind {
    CreateEncoder,
}

#[derive(Debug)]
enum PcmErrorSource {
    Hound(hound::Error),
}

impl PcmError {
    fn from_hound(kind: PcmErrorKind) -> impl FnOnce(hound::Error) -> Self {
        |source| Self {
            kind,
            source: PcmErrorSource::Hound(source),
        }
    }
}

impl Display for PcmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(match self.kind {
            PcmErrorKind::CreateEncoder => "failed to create PCM stream encoder",
        })
    }
}

impl Error for PcmError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.source {
            PcmErrorSource::Hound(e) => Some(e),
        }
    }
}
