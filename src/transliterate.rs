use crate::iconv::Iconv;
use crate::locale_ffi::{__locale_struct, freelocale, newlocale, uselocale, LC_ALL_MASK};
use std::ffi::CString;
use std::ptr;

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

                //uselocale returns in some systems 0xffffffffffffffff instead of locale_t 0.
                //I'm starting to think that I should parse the locale transliteration files in rust...
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
    ) -> Result<String, &'static str> {
        let text = text.into();
        let locale = locale.into();

        if self.set_thread_locale(locale).is_ok() {
            let iconv = Iconv::new("ascii//TRANSLIT//IGNORE", "utf-8");

            if let Ok(mut iconv) = iconv {
                let mut buf = Vec::new();
                let result = iconv.convert(&text.as_bytes(), &mut buf, 0);

                if result.is_err() {
                    return Err("Error in transliteration");
                }
                let output_utf8 = String::from_utf8(buf);

                if let Ok(output) = output_utf8 {
                    Ok(output)
                } else {
                    Err("Error in the transliteration")
                }
            } else {
                Err("Not possible initialize iconv")
            }
        } else {
            Err("Not possible to set the locale")
        }
    }
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
