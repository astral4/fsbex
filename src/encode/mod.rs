use crate::header::{AudioFormat, StreamInfo};
use crate::read::Reader;
use std::io::{Read, Write};

mod error;
mod pcm;
mod vorbis;
mod vorbis_lookup;

pub(crate) use error::EncodeError;
use pcm::Order;

pub(crate) fn encode<R: Read, W: Write>(
    format: AudioFormat,
    flags: u32,
    info: &StreamInfo,
    source: &mut Reader<R>,
    sink: W,
) -> Result<W, EncodeError> {
    Ok(match format {
        AudioFormat::Pcm8 => {
            pcm::encode::<_, _, 1, 1, true>(Order::LittleEndian, info, source, sink)?
        }
        AudioFormat::Pcm16 => {
            let order = if flags & 0x01 == 1 {
                Order::BigEndian
            } else {
                Order::LittleEndian
            };

            pcm::encode::<_, _, 2, 2, true>(order, info, source, sink)?
        }
        AudioFormat::Pcm24 => {
            pcm::encode::<_, _, 3, 3, true>(Order::LittleEndian, info, source, sink)?
        }
        AudioFormat::Pcm32 => {
            pcm::encode::<_, _, 4, 4, true>(Order::LittleEndian, info, source, sink)?
        }
        AudioFormat::PcmFloat => {
            pcm::encode::<_, _, 4, 4, false>(Order::LittleEndian, info, source, sink)?
        }
        AudioFormat::Vorbis => vorbis::encode(info, source, sink)?,
        _ => return Err(EncodeError::UnsupportedFormat { format }),
    })
}
