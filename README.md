# text-transliterate-rust

Prove of concept (AKA, Not ready for production) for text transliteration. It uses the OS iconv with //TRANSLITERATE//IGNORE for transforming character between locales. 

The locale must be available in the OS. 

### Same thread execution (not recommended)

(you can see the tests for more results):

```rust
		let tt = TextTransliterate::new();
		let result = tt.transliterate("ü  ä  ö  ß  Ü  Ä  Ö ç ñ 的 😒", "de_DE.UTF-8");
		if let Ok(result) = result {
			assert_eq!("ue  ae  oe  ss  UE  AE  OE c n ? ?", result);
		}
```

For using the correct locale (for example, to transliterate German letter correctly) it must use the function `uselocale` from C. This changes the locale of the thread. For avoiding you can use the "off-thread" version, that creates a new thread for executing the `uselocale` and `iconv`

### off thread example (recommended)

```rust
        let mut tt = TextTransliterateOffThread::new();
        let result = tt.transliterate("ü  ä  ö  ß  Ü  Ä  Ö ç ñ 的 😒", "de_DE.UTF-8");
        if let Ok(result) = result {
            assert_eq!("ue  ae  oe  ss  UE  AE  OE c n ? ?", result);
        }
```

### Notes

1) The test results can change between machines. Keep in mind that.
2) The code depends of GNU libc
3) There is a unsafe code that can create problems:

### License

Apache-2.0/MIT
