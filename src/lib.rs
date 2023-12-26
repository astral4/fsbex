//! # fsbex
//!
//! `fsbex` is a library for extracting audio from FMOD sound banks. Only FSB version 5 is supported for now.
//!
//! ## Example
//!
//! Parsing a sound bank, then writing streams to files:
//!
//! ```no_run
//! use fsbex::{Bank, AudioFormat};
//! use std::{
//!     error::Error,
//!     io::{BufReader, BufWriter},
//!     fs::File,
//! };
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     // open file for reading sound bank
//!     let file = BufReader::new(File::open("example.fsb")?);
//!     let bank = Bank::new(file)?;
//!
//!     // report number of streams contained within the sound bank
//!     println!("{} streams within this sound bank", bank.num_streams());
//!
//!     // check stream audio format
//!     if bank.format() != AudioFormat::Vorbis {
//!         return Err("expected Vorbis format".into());
//!     }
//!
//!     // iterate over streams
//!     for (index, stream) in bank.into_iter().enumerate() {
//!         // check stream name
//!         let file_name = if let Some(name) = stream.name() {
//!             format!("{name}.ogg")
//!         } else {
//!             format!("stream_{index}.ogg")
//!         };
//!
//!         // write stream data to file
//!         let output_file = BufWriter::new(File::create(file_name)?);
//!         stream.write(output_file)?;
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Supported formats
//!
//! `fsbex` supports encoding stream data for the following formats:
//! - PCM (8, 16, 24, 32-bit integer)
//! - PCM (32-bit float)
//! - Vorbis

mod bank;
pub mod encode;
mod header;
mod read;
mod stream;

pub use bank::{Bank, DecodeError, LazyStreamError};
pub use header::{AudioFormat, Loop};
pub use stream::{LazyStream, Stream, StreamIntoIter};

// Decoding and encoding involves casting values from u32 to usize.
// To ensure correct conversions, only compilation targets where usize is at least 32 bits are allowed.
#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("only targets with 32 or 64-bit wide pointers are supported");
