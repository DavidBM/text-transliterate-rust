use std::borrow::Cow;
use std::error::Error;
use std::fmt;

#[rustfmt::skip]
#[path = "generated_glibc.rs"]
mod glibc;
#[cfg(all(feature = "script-data", any(test, not(feature = "all-scripts"))))]
#[rustfmt::skip]
#[path = "generated_scripts.rs"]
mod scripts;
#[cfg(feature = "all-scripts")]
#[rustfmt::skip]
#[path = "generated_scalars.rs"]
mod scalars;

#[derive(Clone, Copy, Debug)]
struct RuleTable {
    records: &'static [u8],
    output: &'static str,
    sequences: &'static [(&'static str, &'static str)],
    ascii_starts: &'static [u8],
}

impl RuleTable {
    fn scalar(&self, scalar: char) -> Option<&'static str> {
        let key = u32::from(scalar);
        const LATIN_START: u32 = 0x80;
        const LATIN_COUNT: u32 = 256;
        const LATIN_BYTES: usize = LATIN_COUNT as usize * 4;
        if (LATIN_START..LATIN_START + LATIN_COUNT).contains(&key) {
            let index = ((key - LATIN_START) * 4) as usize;
            let packed = u32::from_le_bytes(self.records[index..index + 4].try_into().ok()?);
            return self.replacement(packed);
        }

        let mut records = &self.records[LATIN_BYTES..];
        while !records.is_empty() {
            let middle = (records.len() / 16) * 8;
            let row = &records[middle..middle + 8];
            let point = u32::from_le_bytes(row[..4].try_into().ok()?);
            if point == key {
                let packed = u32::from_le_bytes(row[4..].try_into().ok()?);
                return self.replacement(packed);
            }
            records = if point < key {
                &records[middle + 8..]
            } else {
                &records[..middle]
            };
        }
        None
    }

    fn replacement(&self, packed: u32) -> Option<&'static str> {
        let start = (packed >> 8) as usize;
        let length = (packed & 0xff) as usize;
        self.output.get(start..start + length)
    }

    fn sequence(&self, first: char, text: &str) -> Option<(&'static str, &'static str)> {
        let (last, _) = self.sequences.last()?;
        if last.as_bytes()[0] < text.as_bytes()[0] {
            return None;
        }
        let offset = self
            .sequences
            .partition_point(|(input, _)| input.chars().next() < Some(first));
        self.sequences[offset..]
            .iter()
            .take_while(|(input, _)| input.starts_with(first))
            .find(|(input, _)| text.starts_with(input))
            .copied()
    }
}

#[cfg(feature = "all-scripts")]
#[inline]
fn packed_scalar(key: usize) -> &'static [u8; 8] {
    let entry = if key < scalars::DIRECT_LIMIT {
        let offset = key * 8;
        &scalars::PREFIX[offset..offset + 8]
    } else {
        let at = (key >> 6) * 2;
        let page = u16::from_le_bytes([scalars::PAGES[at], scalars::PAGES[at + 1]]);
        let offset = (usize::from(page) * 64 + (key & 63)) * 8;
        &scalars::BLOCKS[offset..offset + 8]
    };
    entry.try_into().expect("eight-byte packed entry")
}

#[cfg(feature = "all-scripts")]
fn long_replacement(packed: &[u8; 8]) -> Option<&'static str> {
    let index = usize::from(u16::from_le_bytes([packed[0], packed[1]]));
    (index != 0).then(|| scalars::OUTPUTS[index])
}

#[cfg(feature = "all-scripts")]
#[inline]
fn direct_scalar(scalar: char) -> Option<&'static str> {
    let packed = packed_scalar(scalar as usize);
    let length = packed[7] as usize;
    if length <= 7 {
        Some(std::str::from_utf8(&packed[..length]).expect("ASCII table"))
    } else {
        long_replacement(packed)
    }
}

#[cfg(feature = "all-scripts")]
fn supplemental(scalar: char) -> Option<&'static str> {
    if glibc::BASE.scalar(scalar).is_some_and(str::is_ascii) {
        scalars::SHADOWED.scalar(scalar)
    } else {
        direct_scalar(scalar)
    }
}

#[cfg(all(feature = "script-data", not(feature = "all-scripts")))]
fn supplemental(scalar: char) -> Option<&'static str> {
    compact_supplemental(scalar)
}

#[cfg(all(feature = "script-data", any(test, not(feature = "all-scripts"))))]
fn compact_supplemental(scalar: char) -> Option<&'static str> {
    let scalar_value = u32::from(scalar);
    let position = scripts::RANGES.partition_point(|&(first, _, _)| first <= scalar_value);
    let &(_, end, table) = scripts::RANGES.get(position.checked_sub(1)?)?;
    if scalar_value <= end {
        scripts::TABLES[table].as_ref()?.scalar(scalar)
    } else {
        None
    }
}

#[cfg(not(feature = "script-data"))]
fn supplemental(_: char) -> Option<&'static str> {
    None
}

/// The requested locale is unknown, or its Cargo feature is disabled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransliterationError {
    /// The locale identifier could not be resolved.
    UnknownLocale,
    /// The Cargo feature named in this variant is required for the locale.
    LocaleFeatureDisabled(&'static str),
}

impl fmt::Display for TransliterationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownLocale => formatter.write_str("unknown transliteration locale"),
            Self::LocaleFeatureDisabled(feature) => {
                write!(
                    formatter,
                    "transliteration locale requires Cargo feature {feature}"
                )
            }
        }
    }
}

impl Error for TransliterationError {}

/// The treatment of characters without a mapping to ASCII.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Unmapped {
    /// Write one question mark per unmapped Unicode scalar.
    #[default]
    Replace,
    /// Omit unmapped characters.
    Drop,
}

/// Transliterate text to ASCII using the neutral locale.
///
/// Already-ASCII input is borrowed. Unmapped characters become question marks.
/// Use [`Transliterator`] to select a locale or reuse an output buffer.
///
/// ```
/// assert_eq!(text_transliterate::transliterate("Straße"), "Strasse");
/// ```
pub fn transliterate(text: &str) -> Cow<'_, str> {
    Transliterator::default().transliterate(text)
}

/// A reusable Unicode-to-ASCII transliterator with locale-specific rules.
///
/// Construct it once and reuse it across strings. It borrows ASCII input and
/// supports writing into an existing string to reuse its capacity.
///
/// ```
/// use text_transliterate::Transliterator;
///
/// # #[cfg(feature = "de")] {
/// let german = Transliterator::new("de-DE")?;
/// assert_eq!(german.transliterate("Grüße aus Köln"), "Gruesse aus Koeln");
/// # }
/// # Ok::<(), text_transliterate::TransliterationError>(())
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct Transliterator {
    profile: Option<&'static RuleTable>,
    unmapped: Unmapped,
}

impl Transliterator {
    /// Select locale rules once. Accepts glibc locale names, short language
    /// codes, and language-region tags such as `de-CH`. Locale suffixes such
    /// as `.UTF-8` are ignored because Rust strings are UTF-8.
    pub fn new(locale: &str) -> Result<Self, TransliterationError> {
        let mut buffer = [0; 64];
        let mut length = 0;
        let mut skip_encoding = false;
        for byte in locale.bytes() {
            match byte {
                b'.' if !skip_encoding => {
                    skip_encoding = true;
                    continue;
                }
                b'@' => skip_encoding = false,
                b'_' | b'-' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' => {}
                _ => return Err(TransliterationError::UnknownLocale),
            }
            if !skip_encoding {
                let slot = buffer
                    .get_mut(length)
                    .ok_or(TransliterationError::UnknownLocale)?;
                *slot = if byte == b'-' {
                    b'_'
                } else {
                    byte.to_ascii_lowercase()
                };
                length += 1;
            }
        }
        let normalized = std::str::from_utf8(&buffer[..length])
            .map_err(|_| TransliterationError::UnknownLocale)?;
        Ok(Self {
            profile: glibc::profile(normalized)?,
            unmapped: Unmapped::Replace,
        })
    }

    /// Set how to handle characters outside the available tables.
    #[must_use]
    pub const fn with_unmapped(mut self, unmapped: Unmapped) -> Self {
        self.unmapped = unmapped;
        self
    }

    /// Transliterate `text`, borrowing it when it is already ASCII.
    pub fn transliterate<'a>(&self, text: &'a str) -> Cow<'a, str> {
        if text.is_ascii() {
            return Cow::Borrowed(text);
        }
        // A small allowance avoids reallocating after a short expansion.
        let padding = if cfg!(feature = "all-scripts") && self.profile.is_none() && text.len() < 128
        {
            16
        } else {
            0
        };
        let mut output = String::with_capacity(text.len() + padding);
        self.append(text, &mut output);
        Cow::Owned(output)
    }

    /// Append a transliteration to an existing buffer.
    pub fn transliterate_into(&self, text: &str, output: &mut String) {
        if text.is_ascii() {
            output.push_str(text);
        } else {
            output.reserve(text.len());
            self.append(text, output);
        }
    }

    fn glibc_scalar(&self, scalar: char) -> Option<&'static str> {
        self.profile
            .and_then(|table| table.scalar(scalar))
            .or_else(|| glibc::BASE.scalar(scalar))
    }

    fn append(&self, mut text: &str, output: &mut String) {
        #[cfg(feature = "all-scripts")]
        if self.profile.is_none() && glibc::BASE.sequences.is_empty() {
            if text.len() < 128 {
                self.append_default::<128>(text, output);
            } else {
                self.append_default::<512>(text, output);
            }
            return;
        }
        let local_starts = self.profile.map_or(&[][..], |table| table.ascii_starts);
        while let Some(first) = text.chars().next() {
            if first.is_ascii() {
                let run = text.bytes().take_while(u8::is_ascii).count();
                if run == text.len() {
                    output.push_str(text);
                    break;
                }
                // A decomposed rule can start on the final ASCII character,
                // but only if that character appears in a sequence table.
                let final_ascii = text.as_bytes()[run - 1];
                let starts_sequence = local_starts.contains(&final_ascii)
                    || glibc::BASE.ascii_starts.contains(&final_ascii);
                let copy = run - usize::from(starts_sequence);
                output.push_str(&text[..copy]);
                text = &text[copy..];
                if copy == run {
                    continue;
                }
            }

            let first = text
                .chars()
                .next()
                .expect("ASCII run ended before non-ASCII");
            let sequence = self
                .profile
                .and_then(|table| table.sequence(first, text))
                .or_else(|| glibc::BASE.sequence(first, text));
            if let Some((source, replacement)) = sequence {
                if self.append_replacement(replacement, output) {
                    text = &text[source.len()..];
                    continue;
                }
            }
            if first.is_ascii() {
                output.push(first);
            } else {
                self.append_scalar(first, output);
            }
            text = &text[first.len_utf8()..];
        }
    }

    #[cfg(feature = "all-scripts")]
    fn append_default<const CHUNK_SIZE: usize>(&self, mut text: &str, output: &mut String) {
        let mut chunk = [0; CHUNK_SIZE];
        let mut used = 0;
        while let Some(&first) = text.as_bytes().first() {
            if first.is_ascii() {
                let bytes = text.as_bytes();
                let count = if let Some(word) = bytes.first_chunk::<8>() {
                    let (count, prefix) = padded_ascii_prefix(*word);
                    chunk[used..used + 8].copy_from_slice(&prefix);
                    count
                } else {
                    let count = bytes.iter().take_while(|byte| byte.is_ascii()).count();
                    chunk[used..used + count].copy_from_slice(&bytes[..count]);
                    count
                };
                used += count;
                text = &text[count..];
            } else {
                let mut chars = text.chars();
                let scalar = chars.next().expect("nonempty text");
                text = chars.as_str();
                let packed = packed_scalar(scalar as usize);
                let length = usize::from(packed[7]);
                if length <= 7 {
                    chunk[used..used + 8].copy_from_slice(packed);
                    used += length;
                } else {
                    flush_ascii(output, &chunk[..used]);
                    used = 0;
                    if let Some(value) = long_replacement(packed) {
                        output.push_str(value);
                    } else {
                        self.append_indirect(glibc::BASE.scalar(scalar), output);
                    }
                }
            }
            if used > chunk.len() - 8 {
                flush_ascii(output, &chunk[..used]);
                used = 0;
            }
        }
        flush_ascii(output, &chunk[..used]);
    }

    fn append_scalar(&self, scalar: char, output: &mut String) {
        let local = self.profile.and_then(|table| table.scalar(scalar));
        #[cfg(feature = "all-scripts")]
        if local.is_none() {
            if let Some(replacement) = direct_scalar(scalar) {
                output.push_str(replacement);
            } else {
                self.append_indirect(glibc::BASE.scalar(scalar), output);
            }
            return;
        }
        let glibc = local.or_else(|| glibc::BASE.scalar(scalar));
        if let Some(value) = glibc
            .filter(|value| value.is_ascii())
            .or_else(|| supplemental(scalar))
        {
            output.push_str(value);
        } else {
            self.append_indirect(glibc, output);
        }
    }

    #[cold]
    fn append_indirect(&self, candidate: Option<&str>, output: &mut String) {
        if !candidate.is_some_and(|candidate| self.append_replacement(candidate, output))
            && self.unmapped == Unmapped::Replace
        {
            output.push('?');
        }
    }

    fn append_replacement(&self, replacement: &str, output: &mut String) -> bool {
        if replacement.is_ascii() {
            output.push_str(replacement);
            return true;
        }
        let start = output.len();
        for scalar in replacement.chars() {
            if scalar.is_ascii() {
                output.push(scalar);
            } else if let Some(value) = self
                .glibc_scalar(scalar)
                .filter(|value| value.is_ascii())
                .or_else(|| supplemental(scalar))
            {
                output.push_str(value);
            } else {
                output.truncate(start);
                return false;
            }
        }
        true
    }
}

#[cfg(feature = "all-scripts")]
fn flush_ascii(output: &mut String, chunk: &[u8]) {
    output.push_str(std::str::from_utf8(chunk).expect("transliteration tables contain ASCII"));
}

/// Returns the length of an ASCII prefix and replaces the following bytes
/// with zeros. Little-endian order puts each byte's high bit in input order.
#[cfg(feature = "all-scripts")]
#[inline]
fn padded_ascii_prefix(bytes: [u8; 8]) -> (usize, [u8; 8]) {
    let word = u64::from_le_bytes(bytes);
    let high_bits = word & 0x8080_8080_8080_8080;
    if high_bits == 0 {
        return (8, bytes);
    }
    let length = high_bits.trailing_zeros() / 8;
    let mask = u64::MAX.checked_shr(64 - length * 8).unwrap_or(0);
    (length as usize, (word & mask).to_le_bytes())
}

#[cfg(all(test, feature = "all-scripts"))]
#[path = "packed_tests.rs"]
mod packed_tests;

/// Compatibility entry point for callers migrating from the original API.
#[derive(Debug, Default)]
pub struct TextTransliterate;

impl TextTransliterate {
    pub fn new() -> Self {
        Self
    }

    pub fn transliterate<S: Into<String>>(
        &self,
        text: S,
        locale: S,
    ) -> Result<String, TransliterationError> {
        let text = text.into();
        let converter = Transliterator::new(&locale.into())?;
        match converter.transliterate(&text) {
            Cow::Borrowed(_) => Ok(text),
            Cow::Owned(output) => Ok(output),
        }
    }
}
