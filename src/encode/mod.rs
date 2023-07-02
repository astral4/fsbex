use crate::header::{Codec, StreamInfo};
use std::io::Write;

pub(crate) mod error;

pub(crate) fn encode<W: Write>(
    codec: Codec,
    info: StreamInfo,
    sink: W,
) -> Result<(), error::EncodeError> {
    todo!()
}
