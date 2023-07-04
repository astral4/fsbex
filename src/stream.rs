use crate::encode::{encode, error::EncodeError};
use crate::header::{Codec, StreamInfo};
use crate::read::Reader;
use std::io::{Read, Write};

pub(crate) struct LazyStream<'bank, R: Read> {
    index: u32,
    info: &'bank StreamInfo,
    codec: Codec,
    reader: &'bank mut Reader<R>,
}

impl<'bank, R: Read> LazyStream<'bank, R> {
    pub(crate) fn new(
        index: u32,
        info: &'bank StreamInfo,
        codec: Codec,
        reader: &'bank mut Reader<R>,
    ) -> Self {
        Self {
            index,
            info,
            codec,
            reader,
        }
    }

    fn write<W: Write>(self, sink: W) -> Result<(), EncodeError> {
        encode(self.codec, &self.info, self.reader, sink)
    }
}
