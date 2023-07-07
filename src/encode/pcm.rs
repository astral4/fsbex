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

    // Stream samples are encoded as little-endian and, for the int format, signed ints.
    // However, 1) samples can be stored as big-endian, and 2) int samples are stored as unsigned ints.
    // When this happens, the samples have to be converted. Otherwise (i.e. for little-endian float samples),
    // the stream data can be directly copied from reader to writer.

    if !IS_INT && order == Order::LittleEndian {
        // There could be more data after the stream, so a limit is placed on the number of bytes read.
        return copy(&mut source.limit(stream_size), &mut sink)
            .map(|_| sink)
            .map_err(PcmError::from_io(PcmErrorKind::EncodeStream));
    }

    while source.position() - start_pos < stream_size {
        let mut sample = source
            .take_const::<BYTE_DEPTH_USIZE>()
            .map_err(PcmError::from_read(PcmErrorKind::DecodeSample))?;

        // endianness doesn't matter when samples are 1 byte wide
        if BYTE_DEPTH != 1 && order == Order::BigEndian {
            sample.reverse();
        }

        if IS_INT {
            // Samples are converted from unsigned to signed int values by shifting down.
            // For example, u8::MAX becomes i8::MAX and 25u8 becomes -103i8.
            sample = uint_to_int(sample);
        }

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

const fn uint_to_int<const SIZE: usize>(bytes: [u8; SIZE]) -> [u8; SIZE] {
    const MASK: u8 = 0b1000_0000;

    let mut bytes = bytes;
    bytes[SIZE - 1] ^= MASK;
    bytes
}

#[derive(Debug)]
pub(crate) struct PcmError {
    kind: PcmErrorKind,
    source: PcmErrorSource,
}

#[derive(Debug)]
enum PcmErrorKind {
    CreateHeader,
    EncodeStream,
    DecodeSample,
    EncodeSample,
    FinishStream,
}

#[derive(Debug)]
enum PcmErrorSource {
    Io(IoError),
    Read(ReadError),
}

impl PcmError {
    fn from_io(kind: PcmErrorKind) -> impl FnOnce(IoError) -> Self {
        |source| Self {
            kind,
            source: PcmErrorSource::Io(source),
        }
    }

    fn from_read(kind: PcmErrorKind) -> impl FnOnce(ReadError) -> Self {
        |source| Self {
            kind,
            source: PcmErrorSource::Read(source),
        }
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

#[cfg(test)]
mod test {
    use super::uint_to_int;

    #[test]
    fn convert_uint_to_int() {
        const U8_MIDDLE: u8 = u8::MAX / 2 + 1;
        const U16_MIDDLE: u16 = u16::MAX / 2 + 1;
        const U32_MIDDLE: u32 = u32::MAX / 2 + 1;

        // u8 + i8 values

        let before = u8::MAX;
        let after = u8::from_le_bytes(uint_to_int(before.to_le_bytes()));
        assert_eq!(i64::from(after), i64::from(before) - i64::from(U8_MIDDLE));

        let before = u8::MIN;
        let after = i8::from_le_bytes(uint_to_int(before.to_le_bytes()));
        assert_eq!(i64::from(after), i64::from(before) - i64::from(U8_MIDDLE));

        let before = 193u8;
        let after = u8::from_le_bytes(uint_to_int(before.to_le_bytes()));
        assert_eq!(i64::from(after), i64::from(before) - i64::from(U8_MIDDLE));

        let before = 42u8;
        let after = i8::from_le_bytes(uint_to_int(before.to_le_bytes()));
        assert_eq!(i64::from(after), i64::from(before) - i64::from(U8_MIDDLE));

        // u16 + i16 values

        let before = u16::MAX;
        let after = u16::from_le_bytes(uint_to_int(before.to_le_bytes()));
        assert_eq!(i64::from(after), i64::from(before) - i64::from(U16_MIDDLE));

        let before = u16::MIN;
        let after = i16::from_le_bytes(uint_to_int(before.to_le_bytes()));
        assert_eq!(i64::from(after), i64::from(before) - i64::from(U16_MIDDLE));

        let before = 44022u16;
        let after = u16::from_le_bytes(uint_to_int(before.to_le_bytes()));
        assert_eq!(i64::from(after), i64::from(before) - i64::from(U16_MIDDLE));

        let before = 1001u16;
        let after = i16::from_le_bytes(uint_to_int(before.to_le_bytes()));
        assert_eq!(i64::from(after), i64::from(before) - i64::from(U16_MIDDLE));

        // u32 + i32 values

        let before = u32::MAX;
        let after = u32::from_le_bytes(uint_to_int(before.to_le_bytes()));
        assert_eq!(i64::from(after), i64::from(before) - i64::from(U32_MIDDLE));

        let before = u32::MIN;
        let after = i32::from_le_bytes(uint_to_int(before.to_le_bytes()));
        assert_eq!(i64::from(after), i64::from(before) - i64::from(U32_MIDDLE));

        let before = 3_344_556_677u32;
        let after = u32::from_le_bytes(uint_to_int(before.to_le_bytes()));
        assert_eq!(i64::from(after), i64::from(before) - i64::from(U32_MIDDLE));

        let before = 7_654_321u32;
        let after = i32::from_le_bytes(uint_to_int(before.to_le_bytes()));
        assert_eq!(i64::from(after), i64::from(before) - i64::from(U32_MIDDLE));
    }
}
