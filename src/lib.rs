//! Fast, local transliteration from Unicode to ASCII with embedded locale rules.
//!
//! The default build includes all locale and supplementary script data.
//! Select a smaller build with Cargo features such as `de` or
//! `script-cyrillic`. Use [Transliterator] to parse the locale once and
//! optionally reuse an output buffer across calls.

#![forbid(unsafe_code)]

mod transliterate;
#[cfg(feature = "off-thread")]
mod transliterate_off_thread;

pub use transliterate::{TextTransliterate, TransliterationError, Transliterator, Unmapped};
#[cfg(feature = "off-thread")]
pub use transliterate_off_thread::{TextTransliterateOffThread, TransliterateFailure};
