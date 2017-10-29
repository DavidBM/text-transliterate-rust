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
		let tt = TextTransliterateAsync::new();
		let result = tt.transliterate("ü  ä  ö  ß  Ü  Ä  Ö ç ñ 的 😒", "de_DE.UTF-8");
		if let Ok(result) = result {
			assert_eq!("ue  ae  oe  ss  UE  AE  OE c n ? ?", result);
		} else {
			assert!(false);
		}
```

Note: The test results can change between machines. Keep in mind that.
