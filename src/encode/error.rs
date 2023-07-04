use super::vorbis::VorbisError;
use crate::read::ReadError;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Debug)]
pub(crate) struct EncodeError {
    kind: EncodeErrorKind,
    source: Option<EncodeErrorSource>,
}

#[derive(Debug)]
pub(super) enum EncodeErrorKind {
    Vorbis,
}

#[derive(Debug)]
enum EncodeErrorSource {
    Read(ReadError),
    Vorbis(VorbisError),
}

impl EncodeError {
    pub(super) fn new(kind: EncodeErrorKind) -> Self {
        Self { kind, source: None }
    }

    pub(super) fn factory(kind: EncodeErrorKind) -> impl FnOnce(ReadError) -> Self {
        |source| Self {
            kind,
            source: Some(EncodeErrorSource::Read(source)),
        }
    }

    pub(super) fn from_vorbis(source: VorbisError) -> Self {
        Self {
            kind: EncodeErrorKind::Vorbis,
            source: Some(EncodeErrorSource::Vorbis(source)),
        }
    }
}

impl From<VorbisError> for EncodeError {
    fn from(value: VorbisError) -> Self {
        Self {
            kind: EncodeErrorKind::Vorbis,
            source: Some(EncodeErrorSource::Vorbis(value)),
        }
    }
}

impl Display for EncodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&format!(
            "an error occurred while encoding a {} stream",
            match &self.kind {
                EncodeErrorKind::Vorbis => "Vorbis",
            }
        ))
    }
}

impl Error for EncodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.source {
            Some(source) => match source {
                EncodeErrorSource::Read(e) => Some(e),
                EncodeErrorSource::Vorbis(e) => Some(e),
            },
            None => None,
        }
    }
}
