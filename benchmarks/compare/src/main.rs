use std::borrow::Cow;
use std::hint::black_box;
use std::time::Instant;

fn current(input: &str) -> Cow<'_, str> {
    current::Transliterator::default().transliterate(input)
}
fn baseline(input: &str) -> Cow<'_, str> {
    baseline::Transliterator::default().transliterate(input)
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
    let methods: [(&str, Convert); 6] = [
        ("current", current),
        ("baseline", baseline),
        ("AnyAscii", anyascii),
        ("deunicode", deunicode),
        ("Unidecode", unidecode),
        ("disarm", disarm),
    ];
    for (name, sample) in corpora {
        let input = sample.repeat(if short { 1 } else { 32768 / sample.len() });
        let expected = baseline(&input);
        assert_eq!(current(&input), expected, "{name}");
        if std::env::var_os("SAMPLES").is_some() {
            eprintln!("{name}: {sample:?}");
            for (library, convert) in methods {
                eprintln!("  {library}: {:?}", convert(sample));
            }
        }
        let sizes =
            methods.map(|(_, f)| ((0.016 / run(f, &input, 25) * 25.0) as usize).clamp(25, 50_000));
        let mut times = [[0f64; 7]; 6];
        for sample_index in 0..7 {
            for turn in 0..6 {
                let i = (turn + sample_index) % 6;
                times[i][sample_index] = run(methods[i].1, &input, sizes[i]);
            }
        }
        for (i, (library, convert)) in methods.into_iter().enumerate() {
            times[i].sort_by(f64::total_cmp);
            println!(
                "{name}\t{library}\t{:.2}\t{}",
                input.len() as f64 * sizes[i] as f64 / times[i][3] / 1048576.0,
                convert(&input) == expected
            );
        }
    }
}
