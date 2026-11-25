use std::borrow::Cow;
use text_transliterate::{TextTransliterate, TransliterationError, Transliterator, Unmapped};

#[test]
fn borrowed_ascii_is_unchanged() {
    let input = "Hello, earth!\n\0";
    assert!(matches!(
        Transliterator::default().transliterate(input),
        Cow::Borrowed(same) if same.as_ptr() == input.as_ptr()
    ));
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
