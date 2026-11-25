//! Utility to transliterate texts into ascii characters.
//! Conversions use the iconv functions provided by glibc on Linux.
//! You want to have glibc with the correct locales installed.
//!
//! For example. If you want to transliterate from german, you need to
//! have install the "de_DE.UTF-8" locale in your SO.
//!
//! You can do it with https://askubuntu.com/questions/76013/how-do-i-add-locale-to-ubuntu-server
//!
//! Iconv uses the thread locale in order to decide transliteration table to use.
//! The previous thread locale is restored after each conversion. Use
//! `TextTransliterateOffThread` to run conversions on a worker thread.

mod iconv;
mod transliterate;
mod transliterate_off_thread;

pub use transliterate::TextTransliterate;
pub use transliterate_off_thread::TextTransliterateOffThread;
