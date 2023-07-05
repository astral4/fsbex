use super::pcm::PcmError;
use super::vorbis::VorbisError;
use crate::header::AudioFormat;
use std::{
    borrow::Cow,
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Debug)]
pub(crate) enum EncodeError {
    UnsupportedFormat { format: AudioFormat },
    Pcm(PcmError),
    Vorbis(VorbisError),
}

impl From<PcmError> for EncodeError {
    fn from(value: PcmError) -> Self {
        Self::Pcm(value)
    }
}

impl From<VorbisError> for EncodeError {
    fn from(value: VorbisError) -> Self {
        Self::Vorbis(value)
    }
}

impl Display for EncodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let msg = match self {
            Self::UnsupportedFormat { format } => {
                Cow::Owned(format!("encoding for {format:?} streams is currently unsupported"))
            }
            Self::Pcm(_) => Cow::Borrowed("failed to encode PCM stream"),
            Self::Vorbis(_) => Cow::Borrowed("failed to encode Vorbis stream"),
        };

        f.write_str(&msg)
    }
}

impl Error for EncodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnsupportedFormat { format: _ } => None,
            Self::Pcm(e) => Some(e),
            Self::Vorbis(e) => Some(e),
        }
    }
}
