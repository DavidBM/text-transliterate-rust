use locale_ffi::{uselocale, newlocale, __locale_struct};
use std::ptr;
use std::ffi::CString;
use libc::{LC_ALL_MASK, free, c_void};
use iconv::{Iconv};

#[derive(Debug)]
pub struct TextTransliterate;

impl TextTransliterate {
	pub fn new() -> TextTransliterate {
		TextTransliterate {}
	}

	fn set_thread_locale<S: Into<String>>(&self, locale: S) -> Result<*mut __locale_struct, &'static str> {

		
		if let Ok(locale) = CString::new(locale.into()) {

			let locale = locale.as_ptr();

			let null: *mut __locale_struct = ptr::null_mut();
			
			let old_locale = unsafe { 
				let locale = newlocale(LC_ALL_MASK, locale, null);
				uselocale(locale)
			};

			Ok(old_locale)

		} else {
			Err("Not able to decode locale text")
		}
	}

	fn restore_thread_locale(&self, old_locale: *mut __locale_struct) {
		unsafe { 
			let old_locale = uselocale(old_locale);
			free(old_locale as *mut c_void)
		};
	}

	pub fn transliterate<S: Into<String>>(&self, text: S, locale: S) -> Result<String, &'static str> {
		let old_locale = self.set_thread_locale(locale);

		if let Ok(old_locale) = old_locale {
			
			let iconv = Iconv::new("ascii//TRANSLIT//IGNORE", "utf-8");

			if let Ok(mut iconv) = iconv {
				
				let mut buf = Vec::new();
				let result = iconv.convert(&text.into().as_bytes(), &mut buf, 0);

				if let Err(_) = result {
					return Err("Error in transliteration");
				}
				let output_utf8 = String::from_utf8(buf);

				self.restore_thread_locale(old_locale);

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
			assert!(false);
		}
	}
}
