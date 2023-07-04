use super::vorbis_lookup::VORBIS_LOOKUP;
use crate::header::StreamInfo;
use crate::read::{ReadError, Reader};
use lewton::{
    audio::{read_audio_packet_generic, PreviousWindowRight},
    header::{read_header_ident, read_header_setup, IdentHeader, SetupHeader},
    samples::Samples,
};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::{Error as IoError, Read, Write},
};
use vorbis_rs::{VorbisBitrateManagementStrategy, VorbisEncoder};

pub(super) fn encode<R: Read, W: Write>(
    info: &StreamInfo,
    source: &mut Reader<R>,
    sink: W,
) -> Result<(), VorbisError> {
    let crc32 = info
        .vorbis_crc32
        .ok_or_else(|| VorbisError::new(VorbisErrorKind::Crc32Lookup))?;

    let (id_header, setup_header) =
        init_headers(info.sample_rate.into(), info.channels.into(), crc32)?;

    let mut encoder = VorbisEncoder::new(
        0,
        [("", "")],
        info.sample_rate,
        info.channels,
        VorbisBitrateManagementStrategy::QualityVbr {
            target_quality: 1.0,
        },
        None,
        sink,
    )
    .map_err(VorbisError::from_vorbis(VorbisErrorKind::CreateEncoder))?;

    let start_pos = source.position();
    let stream_size = u32::from(info.size) as usize;
    let mut window = PreviousWindowRight::new();

    while source.position() - start_pos < stream_size {
        let packet_size = source
            .le_u16()
            .map_err(VorbisError::from_read(VorbisErrorKind::ReadPacket))?;

        if packet_size == u16::MIN || packet_size == u16::MAX {
            break;
        }

        let packet = source
            .take(packet_size as usize)
            .map_err(VorbisError::from_read(VorbisErrorKind::ReadPacket))?;

        let block: Block =
            read_audio_packet_generic(&id_header, &setup_header, packet.as_slice(), &mut window)
                .map_err(Into::into)
                .map_err(VorbisError::from_lewton(VorbisErrorKind::DecodePacket))?;

        encoder
            .encode_audio_block(block.0)
            .map_err(VorbisError::from_vorbis(VorbisErrorKind::EncodeBlock))?;
    }

    encoder
        .finish()
        .map(|_| ())
        .map_err(VorbisError::from_vorbis(VorbisErrorKind::FinishStream))
}

const MIN_BLOCK_SIZE_EXP2: u8 = 8;
const MAX_BLOCK_SIZE_EXP2: u8 = 11;

fn init_headers(
    sample_rate: u32,
    channels: u8,
    crc32: u32,
) -> Result<(IdentHeader, SetupHeader), VorbisError> {
    let id_header_data = init_id_header_data(sample_rate, channels)
        .expect("writing to an in-memory buffer is infallible");

    let id_header = read_header_ident(id_header_data.as_slice())
        .map_err(Into::into)
        .map_err(VorbisError::from_lewton(VorbisErrorKind::CreateHeaders))?;

    let setup_header_data = *VORBIS_LOOKUP
        .get(&crc32)
        .ok_or_else(|| VorbisError::new(VorbisErrorKind::Crc32Lookup))?;

    let setup_header = read_header_setup(
        setup_header_data,
        channels,
        (MIN_BLOCK_SIZE_EXP2, MAX_BLOCK_SIZE_EXP2),
    )
    .map_err(Into::into)
    .map_err(VorbisError::from_lewton(VorbisErrorKind::CreateHeaders))?;

    Ok((id_header, setup_header))
}

fn init_id_header_data(sample_rate: u32, channels: u8) -> Result<Vec<u8>, IoError> {
    const BLOCK_SIZES: u8 = (MAX_BLOCK_SIZE_EXP2 << 4) | (MIN_BLOCK_SIZE_EXP2);

    let mut data = Vec::with_capacity(30);

    data.write_all(&[1])?;
    data.write_all(b"vorbis")?;
    data.write_all(&[0; 4])?;
    data.write_all(&[channels])?;
    data.write_all(sample_rate.to_le_bytes().as_slice())?;
    data.write_all(&[0; 4])?;
    data.write_all(&[0; 4])?;
    data.write_all(&[0; 4])?;
    data.write_all(&[BLOCK_SIZES])?;
    data.write_all(&[1])?;

    Ok(data)
}

struct Block(Vec<Vec<f32>>);

impl Samples for Block {
    fn from_floats(floats: Vec<Vec<f32>>) -> Self {
        Self(floats)
    }

    fn num_samples(&self) -> usize {
        self.0[0].len()
    }

    fn truncate(&mut self, limit: usize) {
        for channel in &mut self.0 {
            if limit < channel.len() {
                channel.truncate(limit);
            }
        }
    }
}

#[derive(Debug)]
pub(super) struct VorbisError {
    kind: VorbisErrorKind,
    source: Option<VorbisErrorSource>,
}

#[derive(Debug)]
enum VorbisErrorKind {
    MissingCrc32,
    CreateHeaders,
    Crc32Lookup,
    CreateEncoder,
    ReadPacket,
    DecodePacket,
    EncodeBlock,
    FinishStream,
}

#[derive(Debug)]
enum VorbisErrorSource {
    Encode(vorbis_rs::VorbisError),
    Decode(lewton::VorbisError),
    Read(ReadError),
}

impl VorbisError {
    fn new(kind: VorbisErrorKind) -> Self {
        Self { kind, source: None }
    }

    fn from_vorbis(kind: VorbisErrorKind) -> impl FnOnce(vorbis_rs::VorbisError) -> Self {
        |source| Self {
            kind,
            source: Some(VorbisErrorSource::Encode(source)),
        }
    }

    fn from_lewton(kind: VorbisErrorKind) -> impl FnOnce(lewton::VorbisError) -> Self {
        |source| Self {
            kind,
            source: Some(VorbisErrorSource::Decode(source)),
        }
    }

    fn from_read(kind: VorbisErrorKind) -> impl FnOnce(ReadError) -> Self {
        |source| Self {
            kind,
            source: Some(VorbisErrorSource::Read(source)),
        }
    }
}

impl Display for VorbisError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(match self.kind {
            VorbisErrorKind::MissingCrc32 => {
                "File header did not contain CRC32 of Vorbis setup header"
            }
            VorbisErrorKind::CreateHeaders => "failed to create dummy Vorbis headers",
            VorbisErrorKind::Crc32Lookup => {
                "CRC32 of Vorbis setup header was not found in lookup table"
            }
            VorbisErrorKind::CreateEncoder => "failed to create Vorbis stream encoder",
            VorbisErrorKind::ReadPacket => "failed to read audio packet from Vorbis stream",
            VorbisErrorKind::DecodePacket => "failed to decode audio packet from Vorbis stream",
            VorbisErrorKind::EncodeBlock => "failed to encode block of samples",
            VorbisErrorKind::FinishStream => "failed to write all Vorbis stream data",
        })
    }
}

impl Error for VorbisError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.source {
            Some(source) => match source {
                VorbisErrorSource::Encode(e) => Some(e),
                VorbisErrorSource::Decode(e) => Some(e),
                VorbisErrorSource::Read(e) => Some(e),
            },
            None => None,
        }
    }
}
