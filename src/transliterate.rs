use std::borrow::Cow;
use std::error::Error;
use std::fmt;

#[rustfmt::skip]
#[path = "generated_glibc.rs"]
mod glibc;
#[cfg(feature = "script-data")]
#[rustfmt::skip]
#[path = "generated_scripts.rs"]
mod scripts;

#[derive(Clone, Copy, Debug)]
struct RuleTable {
    records: &'static [u8],
    output: &'static str,
    sequences: &'static [(&'static str, &'static str)],
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
        if self.sequences.is_empty() {
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

    fn has_ascii_prefix(&self) -> bool {
        self.sequences
            .first()
            .is_some_and(|(input, _)| input.as_bytes()[0].is_ascii())
    }
}

#[cfg(feature = "script-data")]
fn supplemental(scalar: char) -> Option<&'static str> {
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
        let mut output = String::with_capacity(text.len());
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
        let check_ascii_prefix =
            self.profile.is_some_and(RuleTable::has_ascii_prefix) || glibc::BASE.has_ascii_prefix();
        while let Some(first) = text.chars().next() {
            if first.is_ascii() {
                let run = text.bytes().take_while(u8::is_ascii).count();
                if run == text.len() {
                    output.push_str(text);
                    break;
                }
                // A decomposed rule can start on the final ASCII character of
                // this run, such as the 'a' in a + COMBINING DIAERESIS.
                let copy = run - usize::from(check_ascii_prefix);
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
                let glibc = self.glibc_scalar(first);
                let direct = glibc.filter(|value| value.is_ascii());
                if let Some(value) = direct.or_else(|| supplemental(first)) {
                    output.push_str(value);
                } else if !glibc.is_some_and(|candidate| self.append_replacement(candidate, output))
                    && self.unmapped == Unmapped::Replace
                {
                    output.push('?');
                }
            }
            text = &text[first.len_utf8()..];
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
