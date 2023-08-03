use crate::{
    header::StreamInfo,
    read::{ReadError, Reader},
};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::{copy, Error as IoError, Read, Write},
};

pub(super) fn encode<
    R: Read,
    W: Write,
    const BYTE_DEPTH: u16,
    const BYTE_DEPTH_USIZE: usize,
    const IS_INT: bool,
>(
    order: Order,
    info: &StreamInfo,
    source: &mut Reader<R>,
    sink: W,
) -> Result<W, PcmError> {
    // The byte depth value can't be cast to other types, so there are two const generic parameters for it.
    // This will no longer be necessary when the generic_const_exprs feature is stabilized.
    // The tracking issue is at https://github.com/rust-lang/rust/issues/76560.
    debug_assert_eq!(BYTE_DEPTH as usize, BYTE_DEPTH_USIZE);

    let mut sink = sink;

    // write the WAVE file header
    write_header::<_, BYTE_DEPTH, IS_INT>(
        info.size.into(),
        u16::from(u8::from(info.channels)),
        info.sample_rate.into(),
        &mut sink,
    )
    .map_err(PcmError::from_io(PcmErrorKind::CreateHeader))?;

    let start_pos = source.position();
    let stream_size = u32::from(info.size) as usize;

    // Stream samples are encoded as little-endian.
    // However, samples can be stored as big-endian; when this happens, the samples have to be converted.
    // Otherwise, the stream data can be directly copied from reader to writer.

    if !IS_INT || order == Order::LittleEndian {
        // There could be more data after the stream, so a limit is placed on the number of bytes read.
        return copy(&mut source.limit(stream_size), &mut sink)
            .map(|_| sink)
            .map_err(PcmError::from_io(PcmErrorKind::EncodeStream));
    }

    while source.position() - start_pos < stream_size {
        let mut sample = source
            .take_const::<BYTE_DEPTH_USIZE>()
            .map_err(PcmError::from_read(PcmErrorKind::DecodeSample))?;

        // This is optimized out when BYTE_DEPTH == 1
        sample.reverse();

        sink.write_all(sample.as_slice())
            .map_err(PcmError::from_io(PcmErrorKind::EncodeSample))?;
    }

    sink.flush()
        .map(|_| sink)
        .map_err(PcmError::from_io(PcmErrorKind::FinishStream))
}

fn write_header<W: Write, const BYTE_DEPTH: u16, const IS_INT: bool>(
    file_size: u32,
    channels: u16,
    sample_rate: u32,
    sink: &mut W,
) -> Result<(), IoError> {
    // WAVE file header information taken from:
    // [1]: https://www-mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/WAVE.html
    // [2]: http://soundfile.sapp.org/doc/WaveFormat/

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Order {
    LittleEndian,
    BigEndian,
}

/// Represents an error that can occur when encoding a PCM stream.
///
/// See [`PcmErrorKind`] for the different kinds of errors that can occur.
#[derive(Debug)]
pub struct PcmError {
    kind: PcmErrorKind,
    source: PcmErrorSource,
}

/// A variant of a [`PcmError`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PcmErrorKind {
    /// Failed to write the file header due to an underlying I/O error.
    CreateHeader,
    /// Failed to encode the entire stream via copying from reader to writer.
    EncodeStream,
    /// Failed to decode an audio sample from the stream data.
    DecodeSample,
    /// Failed to encode an audio sample to the writer.
    EncodeSample,
    /// Failed to flush the writer after encoding the entire stream.
    FinishStream,
}

#[derive(Debug)]
enum PcmErrorSource {
    Io(IoError),
    Read(ReadError),
}

impl PcmError {
    fn from_io(kind: PcmErrorKind) -> impl FnOnce(IoError) -> Self {
        move |source| Self {
            kind,
            source: PcmErrorSource::Io(source),
        }
    }

    fn from_read(kind: PcmErrorKind) -> impl FnOnce(ReadError) -> Self {
        move |source| Self {
            kind,
            source: PcmErrorSource::Read(source),
        }
    }

    /// Returns the [`PcmErrorKind`] associated with this error.
    #[must_use]
    pub fn kind(&self) -> PcmErrorKind {
        self.kind
    }
}

impl Display for PcmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(match self.kind {
            PcmErrorKind::CreateHeader => "failed to encode file header",
            PcmErrorKind::EncodeStream => "failed to encode full PCM stream",
            PcmErrorKind::DecodeSample => "failed to decode sample from PCM stream",
            PcmErrorKind::EncodeSample => "failed to encode sample",
            PcmErrorKind::FinishStream => "failed to finalize writing PCM stream data",
        })
    }
}

impl Error for PcmError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.source {
            PcmErrorSource::Io(e) => Some(e),
            PcmErrorSource::Read(e) => Some(e),
        }
    }
}
