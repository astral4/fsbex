use crate::encode::{encode, EncodeError};
use crate::header::{AudioFormat, StreamInfo};
use crate::read::Reader;
use std::io::{Read, Write};

/// An audio stream of data that has not been read yet.
///
/// [`LazyStream`] is accessible through the [`Bank::read_streams`] method.
/// See [`Stream`] for the version of an audio stream that immediately reads its data into memory.
/// Unlike [`Stream`], encoding can fail when reading/decoding stream data.
/// However, encoding for both [`LazyStream`] and [`Stream`] can fail due to I/O errors.
///
/// [`Bank::read_streams`]: crate::Bank::read_streams
#[derive(Debug)]
pub struct LazyStream<'bank, R: Read> {
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

    /// Encodes the stream data by writing audio samples to a writer.
    ///
    /// # Errors
    /// This function returns an error if the stream data could not be successfully written.
    /// See [`EncodeError`] for more information.
    pub fn write<W: Write>(self, sink: W) -> Result<W, EncodeError> {
        encode(self.format, self.flags, self.info, self.reader, sink)
    }
}

/// An audio stream of data that has already been read.
///
/// [`Stream`] is accessible through the [`Bank::into_iter`] method,
/// which converts a [`Bank`] into a [`StreamIntoIter`] that iterates over [`Stream`] instances.
///
/// See [`LazyStream`] for the version of an audio stream that does not immediately read its data into memory.
///
/// [`Bank::into_iter`]: crate::Bank::into_iter
/// [`Bank`]: crate::Bank
#[derive(Debug)]
pub struct Stream {
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

    /// Encodes the stream data by writing audio samples to a writer.
    ///
    /// # Errors
    /// This function returns an error if the stream data could not be successfully written.
    /// See [`EncodeError`] for more information.
    pub fn write<W: Write>(self, sink: W) -> Result<W, EncodeError> {
        let mut reader = Reader::new(&*self.data);
        encode(self.format, self.flags, &self.info, &mut reader, sink)
    }
}

/// An iterator over sound bank streams.
///
/// This type is returned from [`Bank::into_iter`].
/// When iterating, `Some(Stream)` is returned if a stream was successfully read from the sound bank, and `None` otherwise.
///
/// [`Bank::into_iter`]: crate::Bank::into_iter
#[derive(Debug)]
pub struct StreamIntoIter<R: Read> {
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
        let stream = self.info.get(self.index as usize).cloned().and_then(|info| {
            let size = u32::from(info.size) as usize;
            let start_pos = self.reader.position();

            let stream = self.reader.take(size).ok().map(|data| {
                Stream::new(self.index, self.format, self.flags, info, data.into_boxed_slice())
            });

            self.reader.advance_to(start_pos + size).ok()?;

            stream
        });

        self.index += 1;

        stream
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
