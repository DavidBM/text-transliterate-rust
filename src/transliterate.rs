use crate::iconv::{Iconv, IconvError};
use libc::{freelocale, locale_t, newlocale, uselocale, LC_ALL_MASK};
use std::ffi::CString;
use std::ptr;

struct ThreadLocale {
    active: locale_t,
    previous: locale_t,
}

impl ThreadLocale {
    fn enter(name: &str) -> Result<Self, TransliterationError> {
        let invalid_locale = || TransliterationError::ErrorChangingThreadLocale;
        let name = CString::new(name).map_err(|_| invalid_locale())?;
        // A null base asks glibc to allocate a new locale owned by this guard.
        let active = unsafe { newlocale(LC_ALL_MASK, name.as_ptr(), ptr::null_mut()) };
        if active.is_null() {
            return Err(invalid_locale());
        }

        // `uselocale` changes only the calling thread and returns its previous locale.
        let previous = unsafe { uselocale(active) };
        if previous.is_null() {
            unsafe { freelocale(active) };
            return Err(invalid_locale());
        }
        Ok(Self { active, previous })
    }
}

impl Drop for ThreadLocale {
    fn drop(&mut self) {
        // The saved locale belongs to the caller. Restore it before freeing ours.
        unsafe {
            if !uselocale(self.previous).is_null() {
                freelocale(self.active);
            }
        }
    }
}

/// Transliterates text in the calling thread and restores its previous locale.
#[derive(Debug, Default)]
pub struct TextTransliterate;

impl TextTransliterate {
    pub fn new() -> TextTransliterate {
        TextTransliterate {}
    }

    pub fn transliterate<S: Into<String>>(
        &self,
        text: S,
        locale: S,
    ) -> Result<String, TransliterationError> {
        let text = text.into();
        let locale = locale.into();
        let _locale = ThreadLocale::enter(&locale)?;
        let mut iconv = Iconv::new("ascii//TRANSLIT//IGNORE", "utf-8")
            .map_err(|_| TransliterationError::IconvStartFailed)?;
        let mut output = Vec::new();
        iconv
            .convert(text.as_bytes(), &mut output, 0)
            .map_err(TransliterationError::IconvError)?;
        String::from_utf8(output).map_err(TransliterationError::IconvOutputNotUtf8)
    }
}

#[derive(Debug)]
pub enum TransliterationError {
    IconvError(IconvError),
    IconvStartFailed,
    ErrorChangingThreadLocale,
    IconvOutputNotUtf8(std::string::FromUtf8Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let tt = TextTransliterate::new();
        let result = tt.transliterate("ü  ä  ö  ß  Ü  Ä  Ö ç ñ 的", "de_DE.UTF-8");
        if let Ok(result) = result {
            assert_eq!("ue  ae  oe  ss  UE  AE  OE c n ?", result);
        } else {
            unreachable!()
        }
    }

    #[test]
    fn restores_the_callers_locale() {
        let _caller_locale = ThreadLocale::enter("C.UTF-8").unwrap();
        let current = || unsafe { uselocale(ptr::null_mut()) };
        let previous = current();
        let tt = TextTransliterate::new();
        assert_eq!(tt.transliterate("abc", "C").unwrap(), "abc");
        assert_eq!(current(), previous);
        assert!(matches!(
            tt.transliterate("abc", "invalid_locale_name"),
            Err(TransliterationError::ErrorChangingThreadLocale)
        ));
        assert_eq!(current(), previous);
    }

    #[test]
    fn converts_text_that_outgrows_the_initial_buffer() {
        let input = "¥".repeat(512);
        let result = TextTransliterate::new()
            .transliterate(input, "en_US.UTF-8".to_owned())
            .unwrap();
        assert_eq!(result, "JPY".repeat(512));
    }
}
