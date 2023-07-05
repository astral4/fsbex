use crate::encode::{encode, error::EncodeError};
use crate::header::{AudioFormat, StreamInfo};
use crate::read::Reader;
use std::io::{Read, Write};

pub(crate) struct LazyStream<'bank, R: Read> {
    index: u32,
    format: AudioFormat,
    flags: u32,
    info: &'bank StreamInfo,
    reader: &'bank mut Reader<R>,
}

impl<'bank, R: Read> LazyStream<'bank, R> {
    pub(crate) fn new(
        index: u32,
        format: AudioFormat,
        flags: u32,
        info: &'bank StreamInfo,
        reader: &'bank mut Reader<R>,
    ) -> Self {
        Self {
            index,
            format,
            flags,
            info,
            reader,
        }
    }

    pub(crate) fn write<W: Write>(self, sink: W) -> Result<(), EncodeError> {
        encode(self.format, self.flags, self.info, self.reader, sink)
    }
}

pub(crate) struct Stream {
    index: u32,
    format: AudioFormat,
    flags: u32,
    info: StreamInfo,
    data: Box<[u8]>,
}

impl Stream {
    pub(crate) fn new(
        index: u32,
        format: AudioFormat,
        flags: u32,
        info: StreamInfo,
        data: Box<[u8]>,
    ) -> Self {
        Self {
            index,
            format,
            flags,
            info,
            data,
        }
    }

    pub(crate) fn write<W: Write>(self, sink: W) -> Result<(), EncodeError> {
        let mut reader = Reader::new(&*self.data);
        encode(self.format, self.flags, &self.info, &mut reader, sink)
    }
}

pub(crate) struct StreamIntoIter<R: Read> {
    index: u32,
    format: AudioFormat,
    flags: u32,
    info: Box<[StreamInfo]>,
    reader: Reader<R>,
}

impl<R: Read> StreamIntoIter<R> {
    pub(crate) fn new(
        format: AudioFormat,
        flags: u32,
        info: Box<[StreamInfo]>,
        reader: Reader<R>,
    ) -> Self {
        Self {
            index: 0,
            format,
            flags,
            info,
            reader,
        }
    }
}

impl<R: Read> Iterator for StreamIntoIter<R> {
    type Item = Stream;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.info.get(self.index as usize).cloned().and_then(|info| {
            self.reader.take(u32::from(info.size) as usize).ok().map(|data| {
                Stream::new(self.index, self.format, self.flags, info, data.into_boxed_slice())
            })
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
