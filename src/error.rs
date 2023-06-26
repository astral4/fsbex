use crate::read::ReadError;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Debug)]
pub(crate) struct HeaderError {
    kind: HeaderErrorKind,
    source: Option<HeaderErrorSource>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HeaderErrorKind {
    Magic,
    Version,
    UnknownVersion { version: u32 },
    TotalSubsongs,
    ZeroSubsongs,
    SampleHeaderSize,
    NameTableSize,
    SampleDataSize,
    Codec,
    Metadata,
    Stream,
}

#[derive(Debug)]
pub(crate) enum HeaderErrorSource {
    Read(ReadError),
    Stream(StreamError),
}

impl HeaderError {
    pub(crate) fn new(kind: HeaderErrorKind) -> Self {
        Self { kind, source: None }
    }

    pub(crate) fn new_with_source(kind: HeaderErrorKind, source: ReadError) -> Self {
        Self {
            kind,
            source: Some(HeaderErrorSource::Read(source)),
        }
    }

    pub(crate) fn factory(kind: HeaderErrorKind) -> impl FnOnce(ReadError) -> Self {
        move |source| Self::new_with_source(kind, source)
    }

    pub(crate) fn kind(&self) -> HeaderErrorKind {
        self.kind
    }

    pub(crate) fn is_stream_err_kind(&self, kind: StreamErrorKind) -> bool {
        match &self.source {
            Some(HeaderErrorSource::Stream(e)) => e.kind == kind,
            _ => false,
        }
    }

    pub(crate) fn is_chunk_err_kind(&self, kind: ChunkErrorKind) -> bool {
        match &self.source {
            Some(HeaderErrorSource::Stream(e)) => match &e.source {
                Some(StreamErrorSource::Chunk(e)) => e.kind == kind,
                _ => false,
            },
            _ => false,
        }
    }
}

impl Display for HeaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        #[allow(clippy::enum_glob_use)]
        use HeaderErrorKind::*;

        match self.kind {
            Magic => f.write_str("no file signature found"),
            Version => f.write_str("failed to read file format version"),
            UnknownVersion { version } => {
                f.write_str(&format!("file format version was not recognized (0x{version:08x})"))
            }
            TotalSubsongs => f.write_str("failed to read number of subsongs"),
            ZeroSubsongs => f.write_str("number of subsongs was 0"),
            SampleHeaderSize => f.write_str("failed to read size of sample header"),
            NameTableSize => f.write_str("failed to read size of name table"),
            SampleDataSize => f.write_str("failed to read size of sample data"),
            Codec => f.write_str("failed to parse codec"),
            Metadata => f.write_str("failed to read (unused) metadata bytes"),
            Stream => f.write_str("failed to parse stream header"),
        }
    }
}

impl Error for HeaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.source {
            Some(source) => match source {
                HeaderErrorSource::Read(e) => Some(e),
                HeaderErrorSource::Stream(e) => Some(e),
            },
            None => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct StreamError {
    index: u32,
    kind: StreamErrorKind,
    source: Option<StreamErrorSource>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StreamErrorKind {
    SampleMode,
    SampleRateFlag { flag: u8 },
    ZeroDataOffset,
    ZeroSamples,
    Chunk,
}

#[derive(Debug)]
enum StreamErrorSource {
    Read(ReadError),
    Chunk(ChunkError),
}

impl StreamError {
    pub(crate) fn new(index: u32, kind: StreamErrorKind) -> Self {
        Self {
            index,
            kind,
            source: None,
        }
    }

    pub(crate) fn new_with_source(index: u32, kind: StreamErrorKind, source: ReadError) -> Self {
        Self {
            index,
            kind,
            source: Some(StreamErrorSource::Read(source)),
        }
    }

    pub(crate) fn factory(index: u32, kind: StreamErrorKind) -> impl FnOnce(ReadError) -> Self {
        move |source| Self::new_with_source(index, kind, source)
    }

    pub(crate) fn kind(&self) -> StreamErrorKind {
        self.kind
    }
}

impl From<StreamError> for HeaderError {
    fn from(value: StreamError) -> Self {
        Self {
            kind: HeaderErrorKind::Stream,
            source: Some(HeaderErrorSource::Stream(value)),
        }
    }
}

impl Display for StreamError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        #[allow(clippy::enum_glob_use)]
        use StreamErrorKind::*;

        match self.kind {
            SampleMode => f.write_str("failed to read sample mode"),
            SampleRateFlag { flag } => {
                f.write_str(&format!("sample rate flag was not recognized (0x{flag:02x})"))
            }
            ZeroDataOffset => f.write_str("sample data offset was 0"),
            ZeroSamples => f.write_str("number of samples was 0"),
            Chunk => f.write_str("failed to parse sample chunk"),
        }?;

        f.write_str(&format!(" - stream at index {}", self.index))
    }
}

impl Error for StreamError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.source {
            Some(source) => match source {
                StreamErrorSource::Read(e) => Some(e),
                StreamErrorSource::Chunk(e) => Some(e),
            },
            None => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ChunkError {
    index: u32,
    kind: ChunkErrorKind,
    source: Option<ReadError>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChunkErrorKind {
    Flag,
    UnknownTypeFlag { flag: u8 },
    Channels,
    SampleRate,
    ZeroSampleRate,
    LoopStart,
    LoopEnd,
    ZeroLengthLoop,
    DspCoefficients,
    VorbisLayers,
    TooManyVorbisLayers,
    WrongChunkSize,
}

impl ChunkError {
    pub(crate) fn new(index: u32, kind: ChunkErrorKind) -> Self {
        Self {
            index,
            kind,
            source: None,
        }
    }

    pub(crate) fn new_with_source(index: u32, kind: ChunkErrorKind, source: ReadError) -> Self {
        Self {
            index,
            kind,
            source: Some(source),
        }
    }

    pub(crate) fn factory(index: u32, kind: ChunkErrorKind) -> impl FnOnce(ReadError) -> Self {
        move |source| Self::new_with_source(index, kind, source)
    }

    pub(crate) fn into_stream_err(self, stream_index: u32) -> StreamError {
        StreamError {
            index: stream_index,
            kind: StreamErrorKind::Chunk,
            source: Some(StreamErrorSource::Chunk(self)),
        }
    }
}

impl Display for ChunkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        #[allow(clippy::enum_glob_use)]
        use ChunkErrorKind::*;

        match self.kind {
            Flag => f.write_str("failed to read chunk flag"),
            UnknownTypeFlag { flag } => {
                f.write_str(&format!("type of chunk flag was not recognized (0x{flag:02x})"))
            }
            Channels => f.write_str("failed to read number of channels"),
            SampleRate => f.write_str("failed to read sample rate"),
            ZeroSampleRate => f.write_str("sample rate was 0"),
            LoopStart => f.write_str("failed to read starting position of loop in sample"),
            LoopEnd => f.write_str("failed to read ending position of loop in sample"),
            ZeroLengthLoop => f.write_str("length of loop in sample was 0"),
            DspCoefficients => f.write_str("failed to read sample DSP coefficients"),
            VorbisLayers => {
                f.write_str("failed to read number of layers per channel in Vorbis sample")
            }
            TooManyVorbisLayers => {
                f.write_str("number of layers in Vorbis sample was greater than 255")
            }
            WrongChunkSize => {
                f.write_str("reported sample chunk size was smaller than actual size")
            }
        }?;

        f.write_str(&format!(" - chunk at index {}", self.index))
    }
}

impl Error for ChunkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.source {
            Some(source) => Some(source),
            None => None,
        }
    }
}
