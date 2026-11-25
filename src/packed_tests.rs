use super::*;

#[test]
fn ascii_prefix_padding_never_copies_a_non_ascii_byte() {
    for length in 0..=8 {
        let mut input = [b'A'; 8];
        input[0] = 0;
        if length < input.len() {
            input[length] = 0xc3;
        }
        let (actual, padded) = padded_ascii_prefix(input);
        assert_eq!(actual, length);
        assert_eq!(&padded[..length], &input[..length]);
        assert!(padded[length..].iter().all(|byte| *byte == 0));
    }
}

#[test]
fn packed_tables_have_valid_pages_and_ascii_replacements() {
    assert_eq!(scalars::PREFIX.len(), scalars::DIRECT_LIMIT * 8);
    assert_eq!(scalars::PAGES.len(), (0x110000 / 64) * 2);
    assert_eq!(scalars::BLOCKS.len() % (64 * 8), 0);
    let pages = scalars::BLOCKS.len() / (64 * 8);
    for entry in scalars::PAGES.chunks(2) {
        let page = u16::from_le_bytes([entry[0], entry[1]]);
        assert!(usize::from(page) < pages, "invalid page {page}");
    }
    assert_eq!(scalars::OUTPUTS[0], "");
    for entry in scalars::PREFIX.chunks(8).chain(scalars::BLOCKS.chunks(8)) {
        let length = usize::from(entry[7]);
        if length == 255 {
            let index = usize::from(u16::from_le_bytes([entry[0], entry[1]]));
            assert!(index < scalars::OUTPUTS.len(), "invalid output {index}");
            assert!(index == 0 || scalars::OUTPUTS[index].len() > 7);
        } else {
            assert!(length <= 7);
            assert!(entry[..length].is_ascii());
        }
    }
    assert!(scalars::OUTPUTS.iter().all(|output| output.is_ascii()));
}

#[test]
fn packed_lookup_matches_compact_tables_for_every_scalar() {
    for scalar in (0..=0x10ffff).filter_map(char::from_u32) {
        let script = compact_supplemental(scalar);
        let neutral = glibc::BASE.scalar(scalar).filter(|value| value.is_ascii());
        assert_eq!(
            direct_scalar(scalar),
            neutral.or(script),
            "neutral mapping for {scalar:?}"
        );
        assert_eq!(
            supplemental(scalar),
            script,
            "script mapping for {scalar:?}"
        );
    }
}
