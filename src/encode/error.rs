use super::pcm::PcmError;
use super::vorbis::VorbisError;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Debug)]
pub(crate) enum EncodeError {
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
        f.write_str(&format!(
            "an error occurred while encoding a {} stream",
            match self {
                Self::Pcm(_) => "PCM",
                Self::Vorbis(_) => "Vorbis",
            }
        ))
    }
}

impl Error for EncodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Pcm(e) => Some(e),
            Self::Vorbis(e) => Some(e),
        }
    }
}
