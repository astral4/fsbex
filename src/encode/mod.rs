use crate::header::{Codec, StreamInfo};
use crate::read::Reader;
use hound::SampleFormat;
use std::io::{Read, Seek, Write};

pub(crate) mod error;
mod pcm;
mod vorbis;
mod vorbis_lookup;

pub(crate) fn encode<R: Read, W: Write + Seek>(
    codec: Codec,
    info: &StreamInfo,
    source: &mut Reader<R>,
    sink: W,
) -> Result<(), error::EncodeError> {
    match codec {
        Codec::Pcm8 => pcm::encode(8, SampleFormat::Int, info, source, sink)?,
        Codec::Pcm16 => pcm::encode(16, SampleFormat::Int, info, source, sink)?,
        Codec::Pcm24 => pcm::encode(24, SampleFormat::Int, info, source, sink)?,
        Codec::Pcm32 => pcm::encode(32, SampleFormat::Int, info, source, sink)?,
        Codec::PcmFloat => pcm::encode(32, SampleFormat::Float, info, source, sink)?,
        Codec::Vorbis => vorbis::encode(info, source, sink)?,
        _ => todo!(),
    }

    Ok(())
}
