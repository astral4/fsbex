use crate::read::{ReadError, Reader};
use bilge::prelude::*;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Read,
    num::NonZeroU32,
    ops::Mul,
};

struct Header {}

impl Header {
    fn parse<R: Read>(reader: &mut Reader<R>) -> Result<Self, HeaderError> {
        match reader.take() {
            Ok(data) if data == FSB5_MAGIC => Ok(()),
            Err(e) => Err(HeaderError::new_with_source(HeaderErrorKind::Magic, e)),
            _ => Err(HeaderError::new(HeaderErrorKind::Magic)),
        }?;

        let version = reader
            .le_u32()
            .map_err(HeaderError::factory(HeaderErrorKind::Version))
            .and_then(Version::parse)?;

        let total_subsongs: NonZeroU32 = reader
            .le_u32()
            .map_err(HeaderError::factory(HeaderErrorKind::TotalSubsongs))?
            .try_into()
            .map_err(|_| HeaderError::new(HeaderErrorKind::ZeroSubsongs))?;

        let sample_header_size = reader
            .le_u32()
            .map_err(HeaderError::factory(HeaderErrorKind::SampleHeaderSize))?;

        let name_table_size = reader
            .le_u32()
            .map_err(HeaderError::factory(HeaderErrorKind::NameTableSize))?;

        let sample_data_size = reader
            .le_u32()
            .map_err(HeaderError::factory(HeaderErrorKind::SampleDataSize))?;

        let codec = reader
            .le_u32()
            .map_err(HeaderError::factory(HeaderErrorKind::Codec))?;

        let base_header_size = match version {
            Version::V0 => 60,
            Version::V1 => 64,
        };

        reader
            .advance_to(base_header_size)
            .map_err(HeaderError::factory(HeaderErrorKind::Metadata))?;

        for index in 0..total_subsongs.into() {
            let mode = match reader.le_u64() {
                Ok(n) => RawSampleMode::from(n).parse(index),
                Err(e) => Err(StreamError::new_with_source(index, StreamErrorKind::SampleMode, e)),
            }?;

            if mode.has_chunks {
                let chunk = match reader.le_u32() {
                    Ok(n) => RawSampleChunk::from(n).parse(0),
                    Err(e) => Err(ChunkError::new_with_source(0, ChunkErrorKind::Flag, e)),
                }
                .map_err(|e| e.into_stream_err(index))?;
            }
        }

        todo!()
    }
}

const FSB5_MAGIC: [u8; 4] = *b"FSB5";

enum Version {
    V0,
    V1,
}

impl Version {
    fn parse(num: u32) -> Result<Self, HeaderError> {
        match num {
            0 => Ok(Self::V0),
            1 => Ok(Self::V1),
            version => Err(HeaderError::new(HeaderErrorKind::UnknownVersion { version })),
        }
    }
}

#[bitsize(64)]
#[derive(FromBits)]
struct RawSampleMode {
    has_chunks: bool,
    sample_rate: u4,
    channels: u2,
    data_offset: u27,
    num_samples: u30,
}

#[derive(Debug, PartialEq)]
struct SampleMode {
    has_chunks: bool,
    sample_rate: NonZeroU32,
    channels: u8,
    data_offset: NonZeroU32,
    num_samples: NonZeroU32,
}

impl RawSampleMode {
    fn parse(self, stream_index: u32) -> Result<SampleMode, StreamError> {
        let sample_rate = match self.sample_rate().value() {
            0 => Ok(4000),
            1 => Ok(8000),
            2 => Ok(11000),
            3 => Ok(11025),
            4 => Ok(16000),
            5 => Ok(22050),
            6 => Ok(24000),
            7 => Ok(32000),
            8 => Ok(44100),
            9 => Ok(48000),
            10 => Ok(96000),
            flag => Err(StreamError::new(
                stream_index,
                StreamErrorKind::SampleRateFlag { flag },
            )),
        }?
        .try_into()
        .unwrap();

        let channels = match self.channels().value() {
            0 => 1,
            1 => 2,
            2 => 6,
            3 => 8,
            _ => unreachable!(),
        };

        let data_offset = self
            .data_offset()
            .value()
            .mul(32)
            .try_into()
            .map_err(|_| StreamError::new(stream_index, StreamErrorKind::ZeroDataOffset))?;

        let num_samples = self
            .num_samples()
            .value()
            .try_into()
            .map_err(|_| StreamError::new(stream_index, StreamErrorKind::ZeroSamples))?;

        Ok(SampleMode {
            has_chunks: self.has_chunks(),
            sample_rate,
            channels,
            data_offset,
            num_samples,
        })
    }
}

#[bitsize(32)]
#[derive(FromBits)]
struct RawSampleChunk {
    is_end: bool,
    size: u24,
    kind: u7,
}

struct SampleChunk {
    is_end: bool,
    size: u32,
    kind: SampleChunkKind,
}

enum SampleChunkKind {
    Channels,
    SampleRate,
    Loop,
    Comment,
    XmaSeekTable,
    DspCoefficients,
    Atrac9Config,
    XwmaConfig,
    VorbisSeekTable,
    PeakVolume,
    VorbisIntraLayers,
    OpusDataSize,
}

impl RawSampleChunk {
    fn parse(self, chunk_index: u32) -> Result<SampleChunk, ChunkError> {
        #[allow(clippy::enum_glob_use)]
        use SampleChunkKind::*;

        let kind = match self.kind().value() {
            1 => Ok(Channels),
            2 => Ok(SampleRate),
            3 => Ok(Loop),
            4 => Ok(Comment),
            6 => Ok(XmaSeekTable),
            7 => Ok(DspCoefficients),
            9 => Ok(Atrac9Config),
            10 => Ok(XwmaConfig),
            11 => Ok(VorbisSeekTable),
            13 => Ok(PeakVolume),
            14 => Ok(VorbisIntraLayers),
            15 => Ok(OpusDataSize),
            flag => Err(ChunkError::new(chunk_index, ChunkErrorKind::UnknownTypeFlag { flag })),
        }?;

        Ok(SampleChunk {
            is_end: self.is_end(),
            size: self.size().value(),
            kind,
        })
    }
}

#[derive(Debug)]
struct HeaderError {
    kind: HeaderErrorKind,
    source: Option<HeaderErrorSource>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HeaderErrorKind {
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
enum HeaderErrorSource {
    Read(ReadError),
    Stream(StreamError),
}

impl HeaderError {
    fn new(kind: HeaderErrorKind) -> Self {
        Self { kind, source: None }
    }

    fn new_with_source(kind: HeaderErrorKind, source: ReadError) -> Self {
        Self {
            kind,
            source: Some(HeaderErrorSource::Read(source)),
        }
    }

    fn factory(kind: HeaderErrorKind) -> impl FnOnce(ReadError) -> Self {
        move |source| Self::new_with_source(kind, source)
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
struct StreamError {
    index: u32,
    kind: StreamErrorKind,
    source: Option<StreamErrorSource>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StreamErrorKind {
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
    fn new(index: u32, kind: StreamErrorKind) -> Self {
        Self {
            index,
            kind,
            source: None,
        }
    }

    fn new_with_source(index: u32, kind: StreamErrorKind, source: ReadError) -> Self {
        Self {
            index,
            kind,
            source: Some(StreamErrorSource::Read(source)),
        }
    }

    fn factory(index: u32, kind: StreamErrorKind) -> impl FnOnce(ReadError) -> Self {
        move |source| Self::new_with_source(index, kind, source)
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
struct ChunkError {
    index: u32,
    kind: ChunkErrorKind,
    source: Option<ReadError>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChunkErrorKind {
    Flag,
    UnknownTypeFlag { flag: u8 },
}

impl ChunkError {
    fn new(index: u32, kind: ChunkErrorKind) -> Self {
        Self {
            index,
            kind,
            source: None,
        }
    }

    fn new_with_source(index: u32, kind: ChunkErrorKind, source: ReadError) -> Self {
        Self {
            index,
            kind,
            source: Some(source),
        }
    }

    fn factory(index: u32, kind: ChunkErrorKind) -> impl FnOnce(ReadError) -> Self {
        move |source| Self::new_with_source(index, kind, source)
    }

    fn into_stream_err(self, stream_index: u32) -> StreamError {
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

#[cfg(test)]
mod test {
    #[allow(clippy::enum_glob_use)]
    use super::{
        ChunkErrorKind::{self, *},
        Header, HeaderError,
        HeaderErrorKind::*,
        HeaderErrorSource, RawSampleChunk, RawSampleMode, SampleMode,
        StreamErrorKind::{self, *},
        StreamErrorSource, FSB5_MAGIC,
    };
    use crate::read::Reader;
    use std::num::NonZeroU32;

    #[test]
    fn read_magic() {
        let mut reader;

        reader = Reader::new(b"".as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == Magic));

        reader = Reader::new(b"abcd".as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == Magic));

        reader = Reader::new(FSB5_MAGIC.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == Version));
    }

    #[test]
    fn read_version() {
        let mut reader;

        let data = b"FSB5\x00";
        reader = Reader::new(data.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == Version));

        let data = b"FSB5\xFF\x00\x00\x00";
        reader = Reader::new(data.as_slice());
        assert!(
            Header::parse(&mut reader).is_err_and(|e| e.kind == UnknownVersion { version: 0xFF })
        );

        let data = b"FSB5\x00\x00\x00\x00";
        reader = Reader::new(data.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == TotalSubsongs));
    }

    #[test]
    fn read_total_subsongs() {
        let mut reader;

        let data = b"FSB5\x01\x00\x00\x00\x00";
        reader = Reader::new(data.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == TotalSubsongs));

        let data = b"FSB5\x01\x00\x00\x00\x00\x00\x00\x00";
        reader = Reader::new(data.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == ZeroSubsongs));

        let data = b"FSB5\x01\x00\x00\x00\x00\x00\xFF\xFF";
        reader = Reader::new(data.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == SampleHeaderSize));
    }

    #[test]
    fn read_sample_header_size() {
        let mut reader;

        let data = b"FSB5\x01\x00\x00\x000000\x00";
        reader = Reader::new(data.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == SampleHeaderSize));

        let data = b"FSB5\x01\x00\x00\x0000000000";
        reader = Reader::new(data.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == NameTableSize));
    }

    #[test]
    fn read_name_table_size() {
        let mut reader;

        let data = b"FSB5\x01\x00\x00\x0000000000\x00";
        reader = Reader::new(data.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == NameTableSize));

        let data = b"FSB5\x01\x00\x00\x00000000000000";
        reader = Reader::new(data.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == SampleDataSize));
    }

    #[test]
    fn read_sample_data_size() {
        let mut reader;

        let data = b"FSB5\x01\x00\x00\x00000000000000\x00";
        reader = Reader::new(data.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == SampleDataSize));

        let data = b"FSB5\x01\x00\x00\x000000000000000000";
        reader = Reader::new(data.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == Codec));
    }

    #[test]
    fn read_codec() {
        let mut reader;

        let data = b"FSB5\x01\x00\x00\x000000000000000000\x00";
        reader = Reader::new(data.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == Codec));

        let data = b"FSB5\x01\x00\x00\x0000000000000000000000";
        reader = Reader::new(data.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == Metadata));
    }

    impl HeaderError {
        fn is_stream_err_kind(&self, kind: StreamErrorKind) -> bool {
            match &self.source {
                Some(HeaderErrorSource::Stream(e)) => e.kind == kind,
                _ => false,
            }
        }
    }

    #[test]
    fn read_metadata() {
        const V0_HEADER_BASE: [u8; 12] = *b"FSB5\x00\x00\x00\x000000";
        const V1_HEADER_BASE: [u8; 12] = *b"FSB5\x01\x00\x00\x000000";

        let mut reader;

        let incomplete_data = b"FSB5\x01\x00\x00\x0000000000000000000000\x00";
        reader = Reader::new(incomplete_data.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == Metadata));

        let err_v1_data = {
            let mut buf = Vec::from(V1_HEADER_BASE);
            buf.append(&mut vec![0; 48]);
            buf
        };
        reader = Reader::new(&err_v1_data);
        assert!(Header::parse(&mut reader).is_err_and(|e| e.kind == Metadata));

        let ok_v0_data = {
            let mut buf = Vec::from(V0_HEADER_BASE);
            buf.append(&mut vec![0; 48]);
            buf
        };
        reader = Reader::new(&ok_v0_data);
        assert!(Header::parse(&mut reader).is_err_and(|e| e.is_stream_err_kind(SampleMode)));

        let ok_v1_data = {
            let mut buf = Vec::from(V1_HEADER_BASE);
            buf.append(&mut vec![0; 52]);
            buf
        };
        reader = Reader::new(&ok_v1_data);
        assert!(Header::parse(&mut reader).is_err_and(|e| e.is_stream_err_kind(SampleMode)));
    }

    #[test]
    fn read_stream_mode() {
        let data = b"FSB5\x01\x00\x00\x00\x01\x00\x00\x0000000000000000000000000000000000000000000000000000000000";
        let mut reader = Reader::new(data.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.is_stream_err_kind(SampleMode)));
    }

    #[test]
    fn derived_sample_mode_parsing_works() {
        #[allow(clippy::unusual_byte_groupings)]
        let data = 0b011010000101100111100000001011_111001101101001101000100110_11_1110_0;

        let mode = RawSampleMode::from(data);

        let has_chunks = (data & 0x01) == 1;
        assert_eq!(mode.has_chunks(), has_chunks);

        let sample_rate_flag = (data >> 1) & 0x0F;
        assert_eq!(u64::from(mode.sample_rate().value()), sample_rate_flag);

        let channels_flag = (data >> 5) & 0x03;
        assert_eq!(u64::from(mode.channels().value()), channels_flag);

        let data_offset = ((data >> 7) & 0x07FF_FFFF) << 5;
        assert_eq!(u64::from(mode.data_offset().value()) * 32, data_offset);

        let num_samples = (data >> 34) & 0x3FFF_FFFF;
        assert_eq!(u64::from(mode.num_samples().value()), num_samples);
    }

    #[test]
    #[allow(clippy::unusual_byte_groupings)]
    fn parse_sample_mode() {
        let data = 0b011010000101100111100000001011_111001101101001101000100110_11_1110_0;
        let mode = RawSampleMode::from(data);
        assert!(mode
            .parse(0)
            .is_err_and(|e| e.kind == SampleRateFlag { flag: 0b1110 }));

        let data = 0b011010000101100111100000001011_000000000000000000000000000_11_0000_0;
        let mode = RawSampleMode::from(data);
        assert!(mode.parse(0).is_err_and(|e| e.kind == ZeroDataOffset));

        let data = 0b000000000000000000000000000000_111001101101001101000100110_11_0000_0;
        let mode = RawSampleMode::from(data);
        assert!(mode.parse(0).is_err_and(|e| e.kind == ZeroSamples));

        let data = 0b000000000000000000000000000001_000000000000000000000000001_01_1000_0;
        let mode = RawSampleMode::from(data).parse(0).unwrap();
        assert_eq!(
            mode,
            SampleMode {
                has_chunks: false,
                sample_rate: NonZeroU32::new(44100).unwrap(),
                channels: 2,
                data_offset: NonZeroU32::new(32).unwrap(),
                num_samples: NonZeroU32::new(1).unwrap()
            }
        );
    }

    #[test]
    fn derived_sample_chunk_parsing_works() {
        #[allow(clippy::unusual_byte_groupings)]
        let data = 0b0001101_100001101110000000011001_0;

        let flags = RawSampleChunk::from(data);

        let is_end = (data & 0x01) == 1;
        assert_eq!(flags.is_end(), is_end);

        let size = (data >> 1) & 0x00FF_FFFF;
        assert_eq!(flags.size().value(), size);

        let kind = (data >> 25) & 0x7F;
        assert_eq!(u32::from(flags.kind().value()), kind);
    }

    impl HeaderError {
        fn is_chunk_err_kind(&self, kind: ChunkErrorKind) -> bool {
            match &self.source {
                Some(HeaderErrorSource::Stream(e)) => match &e.source {
                    Some(StreamErrorSource::Chunk(e)) => e.kind == kind,
                    _ => false,
                },
                _ => false,
            }
        }
    }

    #[test]
    fn parse_sample_chunk() {
        const DATA: &[u8; 72] = b"FSB5\x01\x00\x00\x00\x01\x00\x00\x000000000000000000000000000000000000000000000000000000\x010000000";

        let mut reader;

        reader = Reader::new(DATA.as_slice());
        assert!(Header::parse(&mut reader).is_err_and(|e| e.is_chunk_err_kind(Flag)));

        #[allow(clippy::items_after_statements)]
        fn test_invalid_flag(kind: u8) {
            let flag = u32::from(kind).swap_bytes() << 1;
            assert!(RawSampleChunk::from(flag).parse(0).is_err());

            let full = {
                let mut buf = Vec::from(*DATA);
                buf.append(flag.to_le_bytes().to_vec().as_mut());
                buf
            };
            let mut reader = Reader::new(full.as_slice());
            assert!(Header::parse(&mut reader)
                .is_err_and(|e| e.is_chunk_err_kind(UnknownTypeFlag { flag: kind })));
        }

        for flag in [0, 5, 8, 12] {
            test_invalid_flag(flag);
        }
        for flag in 16..128 {
            test_invalid_flag(flag);
        }
    }
}
