# Changelog

## Unreleased

- Refactor non-zero integer to integer conversions (e.g. replace [`u32::from()`](https://doc.rust-lang.org/1.76.0/core/primitive.u32.html#method.from-7) and `NonZeroU32::into()` with [`NonZeroU32::get()`](https://doc.rust-lang.org/stable/core/num/struct.NonZeroU32.html#method.get))
- Forbid compilation for targets with pointers smaller than 32 bits

## 0.3.0 - 2023-08-19

### Changed

- Refactor PCM encoder internals
- Refactor code with [`tap::Pipe`](https://docs.rs/tap/1.0.1/tap/trait.Pipe.html)
- Adjust [`Display`](https://doc.rust-lang.org/stable/core/fmt/trait.Display.html) impls for [`EncodeError`](https://docs.rs/fsbex/latest/fsbex/encode/enum.EncodeError.html) and [`VorbisErrorKind`](https://docs.rs/fsbex/0.3.0/fsbex/encode/enum.VorbisErrorKind.html)

### Added

- Implement [`Display`](https://doc.rust-lang.org/stable/core/fmt/trait.Display.html) for [`AudioFormat`](https://docs.rs/fsbex/0.3.0/fsbex/enum.AudioFormat.html), [`PcmErrorKind`](https://docs.rs/fsbex/0.3.0/fsbex/encode/enum.PcmErrorKind.html), and [`VorbisErrorKind`](https://docs.rs/fsbex/0.3.0/fsbex/encode/enum.VorbisErrorKind.html)

### Removed

- **Breaking:** remove [`Stream::index()`](https://docs.rs/fsbex/0.2.2/fsbex/struct.Stream.html#method.index). Use [`Iterator::enumerate()`](https://doc.rust-lang.org/stable/core/iter/trait.Iterator.html#method.enumerate) to get the index instead.

### Fixed

## 0.2.2 - 2023-08-04

### Changed

- Adjust documentation of [`AudioFormat`](https://docs.rs/fsbex/0.2.2/fsbex/enum.AudioFormat.html)

### Fixed

- Fix reading stream names from file header

## 0.2.1 - 2023-08-03

### Fixed

- Remove incorrect conversion of integer PCM samples
- Fix PCM encoding error caused by incorrectly reading audio streams

## 0.2.0 - 2023-07-12

### Changed

- **Breaking:** make [`AudioFormat`](https://docs.rs/fsbex/0.2.0/fsbex/enum.AudioFormat.html), [`EncodeError`](https://docs.rs/fsbex/0.2.0/fsbex/encode/enum.EncodeError.html), [`PcmErrorKind`](https://docs.rs/fsbex/0.2.0/fsbex/encode/enum.PcmErrorKind.html) and [`VorbisErrorKind`](https://docs.rs/fsbex/0.2.0/fsbex/encode/enum.VorbisErrorKind.html) non-exhaustive

### Added

- Add [`LazyStreamError::index()`](https://docs.rs/fsbex/0.2.0/fsbex/struct.LazyStreamError.html#method.index)

## 0.1.0 - 2023-07-09

*First release.*
