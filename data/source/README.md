# Embedded table sources

| Files | Source | Terms and notices |
| --- | --- | --- |
| `glibc-2.43-locales.tar.gz` | Unmodified locale source files from `localedata/locales` in the [GNU libc 2.43 release archive](https://ftp.gnu.org/gnu/glibc/glibc-2.43.tar.xz). The release archive SHA-256 is `d9c86c6b5dbddb43a3e08270c5844fc5177d19442cf5b8df4be7c07cd5fa3831`. | Original locale headers are preserved in the archive. They include the Free Software Foundation's locale-data notice. GNU libc's [LGPL text](LICENSE-GLIBC-LGPL-2.1) and [additional notices](GLIBC-NOTICES) are included. |
| `AnyAscii-table.tsv.gz` | [AnyAscii 0.3.3](https://github.com/anyascii/anyascii/tree/2c07da46be94346fecdc8f5a2c42fa7cf09fca00). The uncompressed TSV SHA-256 is `63d405125a149ed646b6f932be96414e2db4b9ff5c3cb1fac49f6386a6fb1fa9`. | [ISC](LICENSE-AnyAscii) |
| `Unicode-Scripts.txt.gz` | [Unicode 18.0.0 script property data](https://www.unicode.org/Public/18.0.0/ucd/Scripts.txt). The uncompressed file SHA-256 is `0071fd81b6aeae25f6e8bce8efec3066a6476a91b49bdb2f52dc76e817862a6a`. | [Unicode License V3](LICENSE-Unicode) |

Run `python3 tools/generate.py` from the repository root to regenerate the
`src/generated*` files and Cargo feature declarations from these snapshots.
Only development of the tables requires Python. Cargo uses the checked-in
generated files. No downloads, system locale data, or C compiler are needed.

The generator parses each glibc transliteration section, resolves inherited
rules, and stores a shared neutral table and deduplicated locale overrides. The
source archive retains the full order of every upstream replacement. Each
runtime row stores the first directly representable ASCII replacement or the
first candidate when it needs a further fold. The generator also compiles
ASCII overrides for canonically decomposed counterparts where the glibc rule
defines a decomposition as a candidate.

The generated scalar tables have a direct-index page for U+0080–U+017F plus
sorted records for the remaining Unicode scalars. Record offsets refer to a
deduplicated UTF-8 output pool. Sequence rules are sorted by their first
character, then longest match first. AnyAscii rows are grouped using Unicode
Script properties so Cargo can omit unselected script data.
