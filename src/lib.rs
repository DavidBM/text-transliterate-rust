//! Fast, local transliteration from Unicode to ASCII with embedded locale rules.
//!
//! Use [transliterate()] for a synchronous conversion with the neutral locale.
//!
//! The default build includes all locale and supplementary script data.
//! Select a smaller build with Cargo features such as `de` or
//! `script-cyrillic`. Use [Transliterator] to reuse a locale, write to a sink,
//! or apply custom rules. Use [try_transliterate()] to reject unmapped input.
//! This crate uses `core` and `alloc` and has no runtime dependencies.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

mod transliterate;

pub use transliterate::{
    transliterate, try_transliterate, CustomRuleError, CustomTransliterator, TextTransliterate,
    TransliterationError, Transliterator, Unmapped, UnmappedCharacter,
};
