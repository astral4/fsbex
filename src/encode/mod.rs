use crate::header::{Codec, StreamInfo};
use crate::read::Reader;
use std::io::{Read, Write};

pub(crate) mod error;
mod pcm;
mod vorbis;
mod vorbis_lookup;

pub(crate) fn encode<R: Read, W: Write>(
    codec: Codec,
    info: &StreamInfo,
    source: &mut Reader<R>,
    sink: W,
) -> Result<(), error::EncodeError> {
    match codec {
        Codec::Pcm8 => pcm::encode::<_, _, 1, true>(info, source, sink)?,
        Codec::Pcm16 => pcm::encode::<_, _, 2, true>(info, source, sink)?,
        Codec::Pcm24 => pcm::encode::<_, _, 3, true>(info, source, sink)?,
        Codec::Pcm32 => pcm::encode::<_, _, 4, true>(info, source, sink)?,
        Codec::PcmFloat => pcm::encode::<_, _, 4, false>(info, source, sink)?,
        Codec::Vorbis => vorbis::encode(info, source, sink)?,
        _ => todo!(),
    }

    Ok(())
}
