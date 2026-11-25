//! Fast, local transliteration from Unicode to ASCII with embedded locale rules.
//!
//! Use [transliterate()] for a synchronous conversion with the neutral locale.
//!
//! The default build includes all locale and supplementary script data.
//! Select a smaller build with Cargo features such as `de` or
//! `script-cyrillic`. Use [Transliterator] to parse the locale once and
//! optionally reuse an output buffer across calls.

#![forbid(unsafe_code)]

mod transliterate;

pub use transliterate::{
    transliterate, TextTransliterate, TransliterationError, Transliterator, Unmapped,
};
