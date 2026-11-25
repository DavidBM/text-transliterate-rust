use std::borrow::Cow;
use std::hint::black_box;
use std::sync::OnceLock;
use std::time::Instant;

fn current(input: &str) -> Cow<'_, str> {
    current::Transliterator::default().transliterate(input)
}
fn baseline(input: &str) -> Cow<'_, str> {
    baseline::Transliterator::default().transliterate(input)
}
fn packed_v1(input: &str) -> Cow<'_, str> {
    packed_v1::Transliterator::default().transliterate(input)
}
fn anyascii(input: &str) -> Cow<'_, str> {
    Cow::Owned(any_ascii::any_ascii(input))
}
fn deunicode(input: &str) -> Cow<'_, str> {
    Cow::Owned(deunicode::deunicode(input))
}
fn unidecode(input: &str) -> Cow<'_, str> {
    Cow::Owned(unidecode::unidecode(input))
}
fn disarm(input: &str) -> Cow<'_, str> {
    disarm::api::transliterate(input)
}
fn current_german(input: &str) -> Cow<'_, str> {
    static ENGINE: OnceLock<current::Transliterator> = OnceLock::new();
    ENGINE
        .get_or_init(|| current::Transliterator::new("de-DE").unwrap())
        .transliterate(input)
}
fn packed_v1_german(input: &str) -> Cow<'_, str> {
    static ENGINE: OnceLock<packed_v1::Transliterator> = OnceLock::new();
    ENGINE
        .get_or_init(|| packed_v1::Transliterator::new("de-DE").unwrap())
        .transliterate(input)
}
type Convert = for<'a> fn(&'a str) -> Cow<'a, str>;

fn run(convert: Convert, input: &str, iterations: usize) -> f64 {
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(convert(black_box(input)));
    }
    start.elapsed().as_secs_f64()
}
fn verify_all_scalars() {
    let locales = [
        "C",
        "aa_ER",
        "az_AZ",
        "be_BY@latin",
        "da_DK",
        "de_AT",
        "hr_HR",
        "hu_HU",
        "ja_JP",
        "ko_KR",
        "lb_LU",
        "mn_MN",
        "pa_PK",
        "scn_IT",
        "sr_RS",
        "tk_TM",
        "uk_UA",
        "uz_UZ",
        "uz_UZ@cyrillic",
        "vi_VN",
        "yi_US",
    ];
    for locale in locales {
        let previous = baseline::Transliterator::new(locale).unwrap();
        let next = current::Transliterator::new(locale).unwrap();
        for start in (0..=0x10ffff).step_by(4096) {
            let mut text = String::new();
            for scalar in (start..start + 4096).filter_map(char::from_u32) {
                text.push(scalar);
                text.push('|');
            }
            let expected = previous.transliterate(&text);
            let actual = next.transliterate(&text);
            assert!(actual == expected, "{locale}: U+{start:04X} block differs");
            let mut into = String::from("prefix:");
            next.transliterate_into(&text, &mut into);
            assert!(
                into[7..] == *expected,
                "{locale}: U+{start:04X} append block differs"
            );
        }
        println!("{locale}: exact scalar parity");
    }
}

fn main() {
    if std::env::var_os("VERIFY").is_some() {
        verify_all_scalars();
        return;
    }
    let short = std::env::var_os("SHORT").is_some();
    let corpora = [
        (
            "ascii",
            "All that glitters is not gold. A quick trip to the city. ",
        ),
        (
            "latin",
            "Déjà vu à Montréal. São Paulo, Zürich, Straße. À bientôt! ",
        ),
        ("cyrillic", "Привет мир. Москва. Это русский текст. "),
        ("han", "你好世界北京上海今天明天你好世界北京上海今天明天"),
        ("kana", "かなひらがなカタカナ こんにちはさようなら。"),
        (
            "mixed",
            "Déjà vu à Montréal, Привет, 你好世界, かな, مرحبا. ",
        ),
    ];
    let methods: [(&str, Convert); 7] = [
        ("current", current),
        ("baseline", baseline),
        ("packed-v1", packed_v1),
        ("AnyAscii", anyascii),
        ("deunicode", deunicode),
        ("Unidecode", unidecode),
        ("disarm", disarm),
    ];
    for (name, sample) in corpora {
        compare(name, sample, short, baseline, &methods);
    }
    compare(
        "german",
        "Grüße aus Köln. Die Straße endet in einer Sackgasse. Ma\u{0308}dchen und Äpfel. ",
        short,
        packed_v1_german,
        &[("current", current_german), ("packed-v1", packed_v1_german)],
    );
}

fn compare(name: &str, sample: &str, short: bool, expected: Convert, methods: &[(&str, Convert)]) {
    let input = sample.repeat(if short { 1 } else { 32768 / sample.len() });
    let expected = expected(&input);
    assert_eq!(methods[0].1(&input), expected, "{name}");
    if std::env::var_os("SAMPLES").is_some() {
        eprintln!("{name}: {sample:?}");
        for (library, convert) in methods {
            eprintln!("  {library}: {:?}", convert(sample));
        }
    }
    let sizes: Vec<_> = methods
        .iter()
        .map(|&(_, f)| ((0.016 / run(f, &input, 25) * 25.0) as usize).clamp(25, 50_000))
        .collect();
    let mut times: Vec<_> = methods.iter().map(|_| Vec::with_capacity(7)).collect();
    for sample_index in 0..7 {
        for turn in 0..methods.len() {
            let i = (turn + sample_index) % methods.len();
            times[i].push(run(methods[i].1, &input, sizes[i]));
        }
    }
    for (i, &(library, convert)) in methods.iter().enumerate() {
        times[i].sort_by(f64::total_cmp);
        println!(
            "{name}\t{library}\t{:.2}\t{}",
            input.len() as f64 * sizes[i] as f64 / times[i][3] / 1048576.0,
            convert(&input) == expected
        );
    }
}
