# text-transliterate

Convert Unicode text to ASCII using embedded GNU libc locale rules and
supplementary script mappings. The conversion runs in safe Rust and uses the
same tables on every platform.

```rust
use text_transliterate::transliterate;

assert_eq!(transliterate("Straße"), "Strasse");
```

The function runs synchronously and uses the neutral locale. Use
`Transliterator` to choose a locale or reuse an output buffer.

```rust
use text_transliterate::Transliterator;

let german = Transliterator::new("de-DE")?;
assert_eq!(german.transliterate("Grüße aus Köln"), "Gruesse aus Koeln");
assert_eq!(german.transliterate("a\u{0308}"), "ae");

let mut buffer = String::with_capacity(256);
german.transliterate_into("Straße", &mut buffer);
assert_eq!(buffer, "Strasse");
# Ok::<(), text_transliterate::TransliterationError>(())
```

Reuse a `Transliterator` to avoid resolving the locale on each call.
`transliterate` borrows ASCII input and `transliterate_into` can reuse caller-owned
capacity. `Transliterator::default()` uses the neutral locale. Use
`with_unmapped(Unmapped::Drop)` to omit characters without a mapping. The default
writes a question mark for each one.

## Select your tables

The default build includes all locale profiles and supplementary scripts.
Use `default-features = false` to select only what you need.

```toml
[dependencies]
text-transliterate = { version = "3", default-features = false, features = ["de", "ru"] }
```

The shared glibc table is always available. Language features enable distinct
locale overrides and any associated supplementary scripts. For example, `de`
enables the German rules, `ru` enables Cyrillic romanization, and `ja` enables
Japanese glibc rules plus the kana and Han tables. A request for an omitted
locale override returns the required feature name.

You can also select scripts directly. The available features are `script-latin`,
`script-cyrillic`, `script-greek`, `script-arabic`, `script-hebrew`, `script-han`,
`script-kana`, `script-hangul`, `script-indic`, `script-southeast-asian`,
`script-ethiopic`, `script-armenian`, `script-georgian`, `script-symbols`, and
`script-other`. `all-scripts` enables every supplemental table.

Locale names use glibc language and region identifiers. The API accepts names
such as `de`, `de-CH`, `de_DE.UTF-8`, and `sr_RS@latin`. Locale selection matters.
German maps `ä` to `ae`, and the neutral profile maps it to `a`.

## Coverage

The bundled glibc 2.43 data provides shared folding rules and locale-specific
overrides. It includes ordered replacements, deletion rules, and sequences
such as Ukrainian `зг` to `zgh`. The generator resolves locale inheritance and
chooses the first direct ASCII replacement. A non-ASCII replacement can be
folded to ASCII when the characters it contains are covered by the enabled
tables.

AnyAscii provides supplementary per-character approximations for scripts that
lack direct glibc ASCII mappings. It supports text such as `Привет` to `Privet`,
`你好` to `NiHao`, and `かな` to `kana`. Scripts and characters can share
representations or need context, such as choosing a reading for Han characters.
The output is an ASCII approximation for search, slugs, and legacy systems.

## Performance comparison

The default build uses a packed lookup for the neutral locale. The benchmark
compares the public `transliterate` function with the original v3 engine, the first
packed implementation, AnyAscii, deunicode, Unidecode, and disarm. It prints
input MiB/s and whether each library produced matching bytes. The German
locale case compares the two packed implementations directly so the German
sequence rules are part of the measurement.

```sh
cargo run --release --locked --manifest-path benchmarks/compare/Cargo.toml
SHORT=1 cargo run --release --locked --manifest-path benchmarks/compare/Cargo.toml
VERIFY=1 cargo run --release --locked --manifest-path benchmarks/compare/Cargo.toml
```

The first run uses repeated samples of about 32 KiB. The second uses the
individual phrases. The last checks every Unicode scalar against the previous
engine through both output APIs, across all distinct locale profiles.

## Upgrading from 2.x

`TextTransliterate::new().transliterate(text, locale)` still returns an owned
String. The off-thread API and Cargo feature have been removed. Error variants
describe locale selection, and output can differ from a host's glibc conversion
when supplementary scripts are enabled.

## Table sources and licensing

The original Rust source is available under MIT or Apache-2.0. The pinned data,
upstream notices, and regeneration instructions are included alongside it.
[Table sources and licenses](data/source/README.md)
