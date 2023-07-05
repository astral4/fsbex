use crate::{header::StreamInfo, read::Reader};
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::{Error as IoError, Read, Write},
};

pub(super) fn encode<R: Read, W: Write, const BYTE_DEPTH: u16, const IS_INT: bool>(
    info: &StreamInfo,
    source: &mut Reader<R>,
    sink: W,
) -> Result<(), PcmError> {
    let mut sink = sink;

    write_header::<_, BYTE_DEPTH, IS_INT>(
        info.size.into(),
        u16::from(u8::from(info.channels)),
        info.sample_rate.into(),
        &mut sink,
    )
    .map_err(PcmError::factory(PcmErrorKind::CreateHeader))?;

    todo!()
}

fn write_header<W: Write, const BYTE_DEPTH: u16, const IS_INT: bool>(
    file_size: u32,
    channels: u16,
    sample_rate: u32,
    sink: &mut W,
) -> Result<(), IoError> {
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

const MASK: u8 = 0b1000_0000;

fn uint_to_int<const SIZE: usize>(bytes: [u8; SIZE]) -> [u8; SIZE] {
    let mut bytes = bytes;
    bytes[SIZE - 1] ^= MASK;
    bytes
}

#[derive(Debug)]
pub(crate) struct PcmError {
    kind: PcmErrorKind,
    source: IoError,
}

#[derive(Debug)]
enum PcmErrorKind {
    CreateHeader,
}

impl PcmError {
    fn factory(kind: PcmErrorKind) -> impl FnOnce(IoError) -> Self {
        |source| Self { kind, source }
    }
}

impl Display for PcmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(match self.kind {
            PcmErrorKind::CreateHeader => "failed to encode PCM stream header",
        })
    }
}

impl Error for PcmError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
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
