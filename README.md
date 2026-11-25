# text-transliterate-rust

Transliterate Unicode text to ASCII using glibc's `iconv` on Linux. Install the
locale you want to use. The host's locale data determines the output.

## On the calling thread

```rust
use text_transliterate::TextTransliterate;

let converter = TextTransliterate::new();
let result = converter.transliterate("ü ä ö ß", "de_DE.UTF-8");
assert_eq!(result.unwrap(), "ue ae oe ss");
```

The converter restores the caller's previous thread locale after each call.

## On a worker thread

```rust
use text_transliterate::TextTransliterateOffThread;

let mut converter = TextTransliterateOffThread::default();
let result = converter.transliterate("ü ä ö ß", "de_DE.UTF-8");
assert_eq!(result.unwrap(), "ue ae oe ss");
```

Use `async_transliterate` to await the worker result without blocking the caller.

## Tests

Install `de_DE.UTF-8`, `ja_JP.UTF-8`, `zh_CN.UTF-8`, and `en_US.UTF-8` before
running `cargo test`. On a glibc system with locale sources installed, you can
also generate them locally and set `LOCPATH` to the output directory.

## License

MIT or Apache-2.0.
