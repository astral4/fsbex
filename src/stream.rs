use crate::encode::{encode, error::EncodeError};
use crate::header::{Codec, StreamInfo};
use crate::read::Reader;
use std::io::{Read, Seek, Write};

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

    fn write<W: Write + Seek>(self, sink: W) -> Result<(), EncodeError> {
        encode(self.codec, self.info, self.reader, sink)
    }
}

pub(crate) struct Stream {
    index: u32,
    info: StreamInfo,
    codec: Codec,
    data: Box<[u8]>,
}

impl Stream {
    pub(crate) fn new(index: u32, info: StreamInfo, codec: Codec, data: Box<[u8]>) -> Self {
        Self {
            index,
            info,
            codec,
            data,
        }
    }

    fn write<W: Write + Seek>(self, sink: W) -> Result<(), EncodeError> {
        let mut reader = Reader::new(&*self.data);
        encode(self.codec, &self.info, &mut reader, sink)
    }
}

pub(crate) struct StreamIntoIter<R: Read> {
    index: u32,
    info: Box<[StreamInfo]>,
    codec: Codec,
    reader: Reader<R>,
}

impl<R: Read> StreamIntoIter<R> {
    pub(crate) fn new(info: Box<[StreamInfo]>, codec: Codec, reader: Reader<R>) -> Self {
        Self {
            index: 0,
            info,
            codec,
            reader,
        }
    }
}

impl<R: Read> Iterator for StreamIntoIter<R> {
    type Item = Stream;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.info.get(self.index as usize).cloned().and_then(|info| {
            self.reader
                .take(u32::from(info.size) as usize)
                .ok()
                .map(|data| Stream::new(self.index, info, self.codec, data.into_boxed_slice()))
        });

        self.index += 1;

        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.info.len();
        (len, Some(len))
    }
}

impl<R: Read> ExactSizeIterator for StreamIntoIter<R> {
    fn len(&self) -> usize {
        self.info.len()
    }
}
