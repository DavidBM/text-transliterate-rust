use crate::iconv::Iconv;
use crate::iconv::IconvError;
use crate::locale_ffi::{__locale_struct, freelocale, newlocale, uselocale, LC_ALL_MASK, };
use std::ffi::CString;
use std::ptr;

/// Transliterates text in the same thread where it is called.
/// Be aware this calls the unsafe ffi function uselocale from C.
/// If you don't know if that is safe, preffer to use TextTransliterateOffThread instead
#[derive(Debug, Default)]
pub struct TextTransliterate;

impl TextTransliterate {
    pub fn new() -> TextTransliterate {
        TextTransliterate {}
    }

    fn set_thread_locale<S: Into<String>>(&self, locale: S) -> Result<(), &'static str> {
        let locale = locale.into();

        if let Ok(locale) = CString::new(locale) {
            let locale = locale.as_ptr();

            let null: *mut __locale_struct = ptr::null_mut();

            unsafe {
                let locale = newlocale(LC_ALL_MASK, locale, null);
                let old_locale = uselocale(locale);

                //uselocale returns in some systems 0xffff_ffff_ffff_ffff instead of locale_t 0.
                if !old_locale.is_null()
                    && old_locale != 0xffff_ffff_ffff_ffff as *mut __locale_struct
                {
                    freelocale(old_locale)
                }
            };

            Ok(())
        } else {
            Err("Not able to decode locale text")
        }
    }

    pub fn transliterate<S: Into<String>>(
        &self,
        text: S,
        locale: S,
    ) -> Result<String, TransliterationError> {
        let text = text.into();
        let locale = locale.into();

        if self.set_thread_locale(locale).is_ok() {
            let iconv = Iconv::new("ascii//TRANSLIT//IGNORE", "utf-8");

            if let Ok(mut iconv) = iconv {
                let mut buf = Vec::new();
                let result = iconv.convert(&text.as_bytes(), &mut buf, 0);

                if let Err(error) = result {
                    return Err(TransliterationError::IconvError(error));
                }
                let output_utf8 = String::from_utf8(buf);

                match output_utf8 {
                    Ok(output) => Ok(output),
                    Err(error) => Err(TransliterationError::IconvOutputNotUtf8(error)),
                }
            } else {
                Err(TransliterationError::IconvStartFailed)
            }
        } else {
            Err(TransliterationError::ErrorChangingThreadLocale)
        }
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
        let result = tt.transliterate("ü  ä  ö  ß  Ü  Ä  Ö ç ñ 的 😒", "de_DE.UTF-8");
        if let Ok(result) = result {
            assert_eq!("ue  ae  oe  ss  UE  AE  OE c n ? ?", result);
        } else {
            unreachable!()
        }
    }
}
