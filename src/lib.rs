//! Utility to transliterate texts into ascii characters. 
//! This utility depends of the iconv utility installed in the SO.
//! You want to have glibc with the correct locales installed. 
//! 
//! For example. If you want to transliterate from german, you need to
//! have install the "de_DE.UTF-8" locale in your SO.
//! 
//! You can do it with https://askubuntu.com/questions/76013/how-do-i-add-locale-to-ubuntu-server
//! 

mod iconv;
mod locale_ffi;
mod transliterate;
mod transliterate_off_thread;

pub use transliterate::TextTransliterate;
pub use transliterate_off_thread::TextTransliterateOffThread;
