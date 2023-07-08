use super::pcm::PcmError;
use super::vorbis::VorbisError;
use crate::header::AudioFormat;
use std::{
    borrow::Cow,
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// Represents an error that can occur when encoding a sound bank stream.
#[derive(Debug)]
pub enum EncodeError {
    /// Encoding is not implemented for this audio format yet.
    UnsupportedFormat {
        /// The audio format of streams in the sound bank.
        format: AudioFormat,
    },
    /// Failed to encode a PCM stream.
    /// See [`PcmError`] for more information.
    Pcm(PcmError),
    /// Failed to encode a Vorbis stream.
    /// See [`VorbisError`] for more information.
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
