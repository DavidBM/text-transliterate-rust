use std::borrow::Cow;
use std::fmt;
use text_transliterate::{
    transliterate, try_transliterate, CustomRuleError, TextTransliterate, TransliterationError,
    Transliterator, Unmapped, UnmappedCharacter,
};

#[test]
fn borrowed_ascii_is_unchanged() {
    let input = "Hello, earth!\n\0";
    assert!(matches!(
        transliterate(input),
        Cow::Borrowed(same) if same.as_ptr() == input.as_ptr()
    ));
}

#[test]
fn pure_function_uses_neutral_rules() {
    assert_eq!(transliterate("Straße, ä\u{10ffff}"), "Strasse, a?");
}

#[test]
fn strict_conversion_distinguishes_deletions_from_missing_mappings() {
    assert!(matches!(
        try_transliterate("ASCII"),
        Ok(Cow::Borrowed("ASCII"))
    ));
    assert_eq!(try_transliterate("Straße\u{0308}").unwrap(), "Strasse");
    let error = UnmappedCharacter {
        character: '\u{10ffff}',
        byte_offset: "Straße\u{0308}".len(),
    };
    let input = "Straße\u{0308}\u{10ffff}\u{10fffe}";
    assert_eq!(try_transliterate(input), Err(error));
    assert_eq!(
        Transliterator::default()
            .with_unmapped(Unmapped::Drop)
            .try_transliterate(input),
        Err(error)
    );
}

struct FixedBuffer {
    bytes: [u8; 16],
    len: usize,
}

impl fmt::Write for FixedBuffer {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.len + text.len();
        self.bytes
            .get_mut(self.len..end)
            .ok_or(fmt::Error)?
            .copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

#[test]
fn writing_accepts_a_fixed_buffer_and_propagates_sink_failure() {
    let mut buffer = FixedBuffer {
        bytes: [0; 16],
        len: 0,
    };
    let neutral = Transliterator::default();
    let sink: &mut dyn fmt::Write = &mut buffer;
    neutral.write_transliteration("Straße", sink).unwrap();
    assert_eq!(
        neutral.write_transliteration("This fragment is too long", sink),
        Err(fmt::Error)
    );
    assert_eq!(&buffer.bytes[..buffer.len], b"Strasse");
}

#[test]
fn custom_rules_validate_match_priority_and_compose_with_output_apis() {
    let neutral = Transliterator::default();
    assert_eq!(
        neutral
            .with_rules(&[("ß", "ss"), ("", "empty")])
            .unwrap_err(),
        CustomRuleError::EmptyInput(1)
    );
    assert_eq!(
        neutral.with_rules(&[("ß", "ä")]).unwrap_err(),
        CustomRuleError::NonAsciiOutput(0)
    );
    let input = String::from("Brand™");
    let replacement = String::from("Acme");
    let rules = [
        ("Brand", "brand"),
        (&input, &replacement),
        ("Brand™", "duplicate"),
        ("-", ""),
        ("Acme", "ACME"),
    ];
    let custom = neutral.with_rules(&rules).unwrap();
    assert_eq!(custom.transliterate("Brand™-Straße"), "AcmeStrasse");
    assert_eq!(custom.transliterate("Brand-Acme"), "brandACME");
    assert_eq!(
        custom.try_transliterate("Brand™\u{10ffff}"),
        Err(UnmappedCharacter {
            character: '\u{10ffff}',
            byte_offset: input.len(),
        })
    );
    let mut output = String::from("prefix ");
    custom
        .write_transliteration("Brand™-Straße", &mut output)
        .unwrap();
    assert_eq!(output, "prefix AcmeStrasse");
}

#[test]
fn appends_and_reuses_an_output_buffer() {
    let neutral = Transliterator::new("C.UTF-8").unwrap();
    let mut output = String::with_capacity(128);
    let capacity = output.capacity();
    output.push_str("prefix: ");
    neutral.transliterate_into("Ãß æ € ⒈", &mut output);
    assert_eq!(output, "prefix: Ass ae EUR 1.");
    assert_eq!(output.capacity(), capacity);
}

#[cfg(feature = "all-scripts")]
#[test]
fn packed_output_handles_chunk_boundaries_and_long_replacements() {
    let neutral = Transliterator::default();
    let text = "1234567ä你好🚀\u{0308}\u{10ffff}abc".repeat(96);
    let expected = "1234567aNiHao:rocket:?abc".repeat(96);
    assert_eq!(neutral.transliterate(&text), expected);

    let mut appended = String::from("prefix:");
    neutral.transliterate_into(&text, &mut appended);
    assert_eq!(&appended[7..], expected);
    let mut written = String::from("prefix:");
    neutral.write_transliteration(&text, &mut written).unwrap();
    assert_eq!(written, appended);
}

#[cfg(feature = "all-scripts")]
#[test]
fn mixed_widths_and_unmapped_policy_across_output_boundaries() {
    let neutral = Transliterator::default();
    let dropping = neutral.with_unmapped(Unmapped::Drop);
    for prefix_len in (0..=10).chain([
        63, 64, 65, 119, 120, 121, 127, 128, 129, 503, 504, 505, 511, 512, 513,
    ]) {
        let prefix = "A".repeat(prefix_len);
        for suffix_len in 0..=8 {
            let suffix = "z".repeat(suffix_len);
            let input = format!("{prefix}ß你🚀\u{0308}\u{10ffff}\0{suffix}");
            let expected = format!("{prefix}ssNi:rocket:?\0{suffix}");
            let expected_drop = format!("{prefix}ssNi:rocket:\0{suffix}");
            assert_eq!(neutral.transliterate(&input), expected);
            assert_eq!(dropping.transliterate(&input), expected_drop);

            let mut output = String::from("existing\0");
            dropping.transliterate_into(&input, &mut output);
            assert_eq!(output, format!("existing\0{expected_drop}"));
        }
    }
}

#[test]
fn handles_invalid_locales_and_unmapped_characters() {
    assert_eq!(
        Transliterator::new("not_a_locale").unwrap_err(),
        TransliterationError::UnknownLocale
    );
    assert!(Transliterator::new("de_DE.\0UTF-8").is_err());
    assert!(Transliterator::new("").is_err());
    let neutral = Transliterator::default();
    let input = "a\u{10ffff}b";
    assert_eq!(neutral.transliterate(input), "a?b");
    assert_eq!(
        neutral.with_unmapped(Unmapped::Drop).transliterate(input),
        "ab"
    );
}

#[test]
fn resolves_named_locales_and_keeps_generic_rules() {
    let neutral = Transliterator::new("en-US").unwrap();
    assert_eq!(neutral.transliterate("ä Ä ß Straße"), "a A ss Strasse");
    assert_eq!(neutral.transliterate("a\u{0308} é e\u{0301}"), "a e e");
    assert!(Transliterator::new("tr_TR.UTF-8").is_ok());
}

#[cfg(feature = "de")]
#[test]
fn locale_overrides_handle_composed_and_decomposed_german() {
    for name in ["de", "de-DE", "de_AT.UTF-8", "DE-ch", "de_DE.utf8"] {
        let german = Transliterator::new(name).unwrap();
        assert_eq!(
            german.transliterate("ÄÖÜ äöü ß A\u{308}O\u{308}U\u{308} a\u{308}o\u{308}u\u{308}"),
            "AEOEUE aeoeue ss AEOEUE aeoeue"
        );
    }
    let converter = TextTransliterate::new();
    assert_eq!(converter.transliterate("Grüße", "de").unwrap(), "Gruesse");
    let german = Transliterator::new("de-DE").unwrap();
    assert_eq!(
        german.transliterate("Das Ma\u{0308}dchen antwortet. x\u{0308} a\u{0308} Straße"),
        "Das Maedchen antwortet. x ae Strasse"
    );
    let rules = [("a\u{0308}", "custom")];
    let custom = german.with_rules(&rules).unwrap();
    assert_eq!(
        custom.try_transliterate("a\u{0308} ä").unwrap(),
        "custom ae"
    );
}

#[cfg(not(feature = "de"))]
#[test]
fn an_omitted_locale_override_reports_its_feature() {
    assert_eq!(
        TextTransliterate::new().transliterate("ä", "de"),
        Err(TransliterationError::LocaleFeatureDisabled("de"))
    );
}

#[cfg(feature = "uk")]
#[test]
fn longer_glibc_sequences_win_over_scalar_mappings() {
    let ukrainian = Transliterator::new("uk-UA").unwrap();
    assert_eq!(ukrainian.transliterate("зг ЗГ з г"), "zgh ZGH z h");
}

#[cfg(feature = "scn")]
#[test]
fn native_multichar_rules_can_match_precomposed_input() {
    assert_eq!(
        Transliterator::new("scn-IT")
            .unwrap()
            .transliterate("ḌḌ Ḍḍ ḍḍ"),
        "DDH Ddh ddh"
    );
}

#[cfg(all(
    feature = "script-cyrillic",
    feature = "script-greek",
    feature = "script-han",
    feature = "script-hangul",
    feature = "script-kana",
    feature = "script-arabic",
    feature = "script-indic"
))]
#[test]
fn supplements_romanisable_scripts_without_a_glibc_ascii_rule() {
    let neutral = Transliterator::default();
    for sample in [
        "Привет",
        "Ελλάδα",
        "你好",
        "한글",
        "かな",
        "العربية",
        "नमस्ते",
    ] {
        let output = neutral.transliterate(sample);
        assert!(!output.is_empty() && !output.contains('?') && output.is_ascii());
    }
    assert_eq!(neutral.transliterate("Привет"), "Privet");
    assert_eq!(neutral.transliterate("你好"), "NiHao");
    assert_eq!(neutral.transliterate("かな"), "kana");
}

#[cfg(not(feature = "script-han"))]
#[test]
fn unselected_script_data_does_not_affect_output() {
    assert_eq!(Transliterator::default().transliterate("你好"), "??");
}

#[cfg(feature = "script-cyrillic")]
#[test]
fn cyrillic_script_can_be_enabled_independently() {
    let russian = Transliterator::new("ru").unwrap();
    assert_eq!(russian.transliterate("Привет"), "Privet");
}

#[cfg(not(feature = "script-cyrillic"))]
#[test]
fn an_omitted_cyrillic_script_is_not_romanized_by_the_neutral_profile() {
    assert_eq!(Transliterator::default().transliterate("Ж"), "?");
}
