# text-transliterate-rust

Proove of concept for text transliteration. It uses the OS iconv with //TRANSLITERATE//IGNORE for transforming character between locales. 

The locale must be avaialble in the OS. 

### Sync example

(you can see the tests for more results):

```rust
		let tt = TextTransliterate::new();
		let result = tt.transliterate("ü  ä  ö  ß  Ü  Ä  Ö ç ñ 的 😒", "de_DE.UTF-8");
		if let Ok(result) = result {
			assert_eq!("ue  ae  oe  ss  UE  AE  OE c n ? ?", result);
		} else {
			assert!(false);
		}
```

For using the correct locale (for example, to transliterate german letter correctly) it must use the function `uselocale` from C. This changes the locale of the thread. For avoiding you can use the "off-thread" version, that creates a new thread for executing the `uselocale` and `iconv`

### off thread example

```rust
		let mut tt = TextTransliterateAsync::new();
		let result = tt.transliterate("ü  ä  ö  ß  Ü  Ä  Ö ç ñ 的 😒", "de_DE.UTF-8");
		if let Ok(result) = result {
			assert_eq!("ue  ae  oe  ss  UE  AE  OE c n ? ?", result);
		} else {
			assert!(false);
		}
```

### Notes

1) The test results can change between machines. Keep in mind that.
2) The code depends of GNU libc
3) There is a unsafe code that can create problems:

```rust
			unsafe { 
				let locale = newlocale(LC_ALL_MASK, locale, null);
				let old_locale = uselocale(locale);

				//uselocale returns in some systems 0xffffffffffffffff instead of locale_t 0.
				//I'm starting to think that I should parse the locale transliteration files in rust...
				if !old_locale.is_null() && old_locale != 0xffffffffffffffff as *mut __locale_struct {
					freelocale(old_locale)
				}
			};
```

Use with precaution.

### License

Apache-2.0/MIT