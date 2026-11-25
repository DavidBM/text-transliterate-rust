#!/usr/bin/env python3
"""Regenerate the embedded tables and Cargo features from the vendored sources."""

from __future__ import annotations

import argparse
import bisect
from collections import defaultdict
from dataclasses import dataclass, field
import gzip
import io
import json
from pathlib import Path
import re
import struct
import tarfile
import unicodedata

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "data" / "source"
GENERATED = ROOT / "src" / "generated"
SYMBOL = re.compile(r"<U([0-9A-Fa-f]{4,8})>")
TOKEN = re.compile(r'"(?:/[^\n]|[^"/])*"|;|[^\s;"]+')
LOCALE = re.compile(r"[a-z]{2,3}_[A-Z]{2}(?:@[a-zA-Z0-9_-]+)?$")
TRANSLITERATION = re.compile(r"translit_(?:start|end)$")


@dataclass
class Locale:
    parent: str | None = None
    includes: list[str] = field(default_factory=list)
    rules: dict[str, list[str]] = field(default_factory=dict)


def uncomment(line):
    quoted = False
    escaped = False
    for index, char in enumerate(line):
        if escaped:
            escaped = False
        elif char == "/":
            escaped = True
        elif char == '"':
            quoted = not quoted
        elif char == "%" and not quoted:
            return line[:index].strip()
    return line.strip()


def tokenize(line):
    cursor = 0
    for match in TOKEN.finditer(line):
        if line[cursor:match.start()].strip():
            raise ValueError(f"unexpected token before {match[0]!r}")
        yield match[0]
        cursor = match.end()
    if line[cursor:].strip():
        raise ValueError("unexpected trailing source syntax")


def decode(token):
    if token.startswith('"'):
        if not token.endswith('"'):
            raise ValueError(f"unterminated string {token!r}")
        token = token[1:-1]
    result = []
    index = 0
    while index < len(token):
        char = token[index]
        if char == "/":
            result.append(token[index + 1])
            index += 2
        elif char == "<":
            symbol = SYMBOL.match(token, index)
            if not symbol:
                raise ValueError(f"unknown symbol at {token[index:]!r}")
            result.append(chr(int(symbol[1], 16)))
            index = symbol.end()
        else:
            result.append(char)
            index += 1
    return "".join(result)


def parse(text, name):
    ctype = False
    translit = False
    locale = None
    rest = ""
    for line_number, physical_line in enumerate(text.splitlines(), 1):
        if physical_line.endswith("/") and not physical_line.endswith("//"):
            rest += physical_line[:-1]
            continue
        line = uncomment(rest + physical_line)
        rest = ""
        if line == "LC_CTYPE":
            ctype = True
            locale = Locale()
        elif line == "END LC_CTYPE":
            if translit:
                raise ValueError(f"{name}:{line_number}: open transliteration section")
            return locale
        elif ctype and TRANSLITERATION.fullmatch(line):
            translit = line == "translit_start"
        elif ctype and line.startswith("copy"):
            locale.parent = re.fullmatch(r'copy\s+"([^"]+)"', line)[1]
        elif translit and line:
            try:
                tokens = list(tokenize(line))
                if tokens[0] == "include":
                    if tokens[2:] != [";", '""']:
                        raise ValueError("unexpected repertoire in include")
                    locale.includes.append(decode(tokens[1]))
                elif tokens[0] == "default_missing":
                    if [decode(token) for token in tokens[1:]] != ["?"]:
                        raise ValueError("unexpected default replacement")
                elif tokens[0] == "translit_ignore":
                    raise ValueError("this glibc release was expected to use explicit deletion rules")
                else:
                    key = decode(tokens[0])
                    choices = [[]]
                    for token in tokens[1:]:
                        if token == ";":
                            choices.append([])
                        else:
                            choices[-1].append(decode(token))
                    if not key or not all(choices):
                        raise ValueError("missing transliteration source or replacement")
                    # localedef's reverse insertion order gives an earlier
                    # rule precedence over a duplicate in the same file.
                    locale.rules.setdefault(key, ["".join(choice) for choice in choices])
            except (ValueError, IndexError) as error:
                raise ValueError(f"{name}:{line_number}: {line!r}: {error}") from error
    if ctype:
        raise ValueError(f"{name}: missing END LC_CTYPE")
    return None


def locale_sources(archive):
    with tarfile.open(archive, "r:gz") as source:
        return {
            item.name: source.extractfile(item).read().decode("utf-8")
            for item in source.getmembers()
        }


def create_source_archive(glibc):
    path = SOURCE / "glibc-2.43-locales.tar.gz"
    with path.open("wb") as compressed, gzip.GzipFile(
        fileobj=compressed, filename="", mode="wb", mtime=0
    ) as stream, tarfile.open(fileobj=stream, mode="w") as output:
        for path in sorted((glibc / "localedata" / "locales").iterdir()):
            if path.is_file():
                source = path.read_bytes()
                item = tarfile.TarInfo(path.name)
                item.size = len(source)
                output.addfile(item, io.BytesIO(source))


def replacement_for_ascii(options):
    for replacement in options:
        if replacement.isascii():
            return replacement
    return options[0]


def compile_glibc(locales):
    def resolved(name, visiting=frozenset()):
        if name in visiting:
            raise ValueError(f"cyclic locale references at {name}")
        locale = locales[name]
        next_visiting = visiting | {name}
        rules = resolved(locale.parent, next_visiting) if locale.parent else {}
        for inherited in locale.includes:
            rules.update(resolved(inherited, next_visiting))
        rules.update(locale.rules)
        return rules

    names = sorted(name for name in locales if LOCALE.fullmatch(name))
    base = resolved("C")
    tables = {"base": {}}
    profiles = defaultdict(dict)
    profile_sets = {}
    rule_count = sum(len(locale.rules) for locale in locales.values())
    # All locale data are read. Locale layers are represented as deltas from the
    # shared table so selectors embed each shared rule just once.
    for key, options in base.items():
        tables["base"][key] = replacement_for_ascii(options)

    for name in names:
        full = resolved(name)
        # Most profiles inherit the same combining data. Use the shared table
        # for neutral behavior and keep only each locale's distinct rules.
        differences = {
            key: replacement_for_ascii(options)
            for key, options in full.items()
            if options != base.get(key)
        }
        differences = {
            key: value for key, value in differences.items()
            if value != tables["base"].get(key)
        }
        for key, replacement in list(differences.items()):
            if not replacement.isascii() or len(key) != 1:
                continue
            decomposed = unicodedata.normalize("NFD", key)
            if decomposed != key and decomposed in full[key]:
                differences.setdefault(decomposed, replacement)
        if not differences:
            continue
        identity = tuple(sorted(differences.items()))
        table = profile_sets.get(identity)
        if table is None:
            table = name.lower().replace("@", "_").replace("-", "_")
            profile_sets[identity] = table
            tables[table] = differences
        profiles[name.split("_", 1)[0]][name] = table
    print(f"glibc: {rule_count} source rules, {len(names)} locale names, {len(tables)} distinct lookup tables")
    return tables, profiles, names


SCRIPT_GROUPS = {
    "latin": {"Latin"},
    "cyrillic": {"Cyrillic"},
    "greek": {"Greek"},
    "arabic": {"Arabic", "Syriac", "Thaana", "Nko", "Mandaic", "Samaritan"},
    "hebrew": {"Hebrew"},
    "han": {"Han", "Bopomofo"},
    "kana": {"Hiragana", "Katakana"},
    "hangul": {"Hangul"},
    "indic": {
        "Bengali", "Brahmi", "Devanagari", "Gujarati", "Gurmukhi", "Kannada",
        "Malayalam", "Oriya", "Sinhala", "Tamil", "Telugu",
    },
    "southeast-asian": {"Buginese", "Javanese", "Khmer", "Lao", "Myanmar", "Thai", "Tibetan"},
    "ethiopic": {"Ethiopic"},
    "armenian": {"Armenian"},
    "georgian": {"Georgian"},
    "symbols": {"Common", "Inherited", "Unknown"},
}


def compile_supplement():
    groups = {script: group for group, scripts in SCRIPT_GROUPS.items() for script in scripts}
    ranges = []
    with gzip.open(SOURCE / "Unicode-Scripts.txt.gz", "rt", encoding="utf-8") as stream:
        for physical_line in stream:
            line = physical_line.split("#", 1)[0].strip()
            if not line:
                continue
            span, script = (value.strip() for value in line.split(";"))
            first, _, last = span.partition("..")
            ranges.append((int(first, 16), int(last or first, 16), groups.get(script, "other")))
    ranges.sort()
    starts = [first for first, _, _ in ranges]
    tables = defaultdict(dict)
    covered_ranges = {}
    with gzip.open(SOURCE / "AnyAscii-table.tsv.gz", "rt", encoding="utf-8") as stream:
        for line in stream:
            source, value = line.removesuffix("\n").split("\t", 1)
            point = ord(source)
            if point < 128:
                continue
            assert value.isascii()
            offset = bisect.bisect_right(starts, point) - 1
            if offset >= 0 and point <= ranges[offset][1]:
                first, last, group = ranges[offset]
                covered_ranges[first] = (first, last, group)
            else:
                group = "symbols"
                covered_ranges[point] = (point, point, group)
            tables[group][source] = value
    combined = []
    for first, last, group in sorted(covered_ranges.values()):
        if combined and first == combined[-1][1] + 1 and group == combined[-1][2]:
            combined[-1] = (combined[-1][0], last, group)
        else:
            combined.append((first, last, group))
    print(f"AnyAscii: {sum(map(len, tables.values()))} mappings across {len(tables)} script groups")
    return dict(tables), combined


def rust_string(value):
    escaped = json.dumps(value, ensure_ascii=False)
    return re.sub(r"\\u([0-9a-f]{4})", r"\\u{\1}", escaped)


def emit_table(name, mapping):
    prefix = GENERATED / name
    latin = [0xFFFFFFFF] * 256
    records = bytearray()
    output = bytearray()
    offsets = {}
    sequences = {}
    for key, replacement in sorted(mapping.items()):
        if len(key) != 1:
            # The engine can copy an ASCII run up to its last character.
            if key[0].isascii() and key[1].isascii():
                raise ValueError(f"unsupported multi-ASCII sequence prefix in {key!r}")
            sequences[key] = replacement
            continue
        encoded = replacement.encode("utf-8")
        if replacement not in offsets:
            offsets[replacement] = len(output)
            output.extend(encoded)
        offset = offsets[replacement]
        assert offset < 0xFFFFFF and len(encoded) < 256
        packed = (offset << 8) | len(encoded)
        codepoint = ord(key)
        if 0x80 <= codepoint < 0x180:
            latin[codepoint - 0x80] = packed
        else:
            records.extend(struct.pack("<II", codepoint, packed))
    prefix.with_suffix(".keys").write_bytes(struct.pack("<256I", *latin) + records)
    prefix.with_suffix(".values").write_bytes(output)
    sequences = sorted(sequences.items(), key=lambda pair: (pair[0][0], -len(pair[0]), pair[0]))
    entries = [f"({rust_string(source)}, {rust_string(value)})" for source, value in sequences]
    return "[" + ", ".join(entries) + "]"


def generate(locales, fallback, ranges):
    license_notice = "// Generated by tools/generate.py from the sources documented in data/source/README.md."
    tables, profiles, names = compile_glibc(locales)
    files = [license_notice, "use super::RuleTable;"]
    lines = []
    features = {"all-glibc": [], "all-scripts": []}
    for table, mapping in tables.items():
        entries = emit_table(f"glibc_{table}", mapping)
        if table != "base":
            languages = sorted(language for language, variants in profiles.items() if table in variants.values())
            predicates = ", ".join(f'feature = "{language}"' for language in languages)
            cfg = f"any({predicates})" if len(languages) > 1 else predicates
            lines.append(f"#[cfg({cfg})]")
        lines.append(f'pub(super) static {table.upper()}: RuleTable = RuleTable {{')
        lines.append(f'    records: include_bytes!("generated/glibc_{table}.keys"),')
        lines.append(f'    output: include_str!("generated/glibc_{table}.values"),')
        lines.append(f'    sequences: &{entries},')
        lines.append("};")

    aliases = {}
    for name in names:
        language = name.split("_", 1)[0]
        aliases[name.lower()] = (language, profiles.get(language, {}).get(name))
    grouped = defaultdict(list)
    for alias, (language, table) in aliases.items():
        grouped[(language, table)].append(alias)
    for language, variants in profiles.items():
        features["all-glibc"].append(language)
        features[language] = []
    files.extend(lines)
    files.extend(["", "use super::TransliterationError;", "", "pub(super) fn profile(locale: &str) -> Result<Option<&'static RuleTable>, TransliterationError> {", "    match locale {"])
    for (language, table), variants in sorted(grouped.items(), key=lambda item: (item[0][0], item[0][1] or "")):
        if language not in aliases:
            preferred = f"{language}_{language}"
            if preferred in variants or (
                not any(name == preferred for name in aliases)
                and min(name for name in aliases if name.startswith(f"{language}_")) in variants
            ):
                variants.append(language)
        pattern = " | ".join(rust_string(alias) for alias in sorted(variants))
        if table:
            files.append(f'        #[cfg(feature = "{language}")]')
            files.append(f'        {pattern} => Ok(Some(&{table.upper()})),')
            files.append(f'        #[cfg(not(feature = "{language}"))]')
            files.append(f'        {pattern} => Err(TransliterationError::LocaleFeatureDisabled("{language}")),')
        else:
            files.append(f"        {pattern} => Ok(None),")
    files.extend(['        "c" | "posix" | "und" | "i18n" => Ok(None),', "        _ => Err(TransliterationError::UnknownLocale),", "    }", "}"])
    (ROOT / "src" / "generated_glibc.rs").write_text("\n".join(files) + "\n", encoding="utf-8")

    files = [license_notice, "use super::RuleTable;", ""]
    script_names = sorted(fallback)
    indexes = {name: index for index, name in enumerate(script_names)}
    for name in script_names:
        feature = f"script-{name}"
        features["all-scripts"].append(feature)
        features[feature] = ["script-data"]
        file_stem = name.replace("-", "_")
        emit_table(f"script_{file_stem}", fallback[name])
        files.extend([
            f'#[cfg(feature = "{feature}")]',
            f"static {file_stem.upper()}: Option<RuleTable> = Some(RuleTable {{",
            f'    records: include_bytes!("generated/script_{file_stem}.keys"),',
            f'    output: include_str!("generated/script_{file_stem}.values"),',
            "    sequences: &[],",
            "});",
            f'#[cfg(not(feature = "{feature}"))]',
            f"static {file_stem.upper()}: Option<RuleTable> = None;",
            "",
        ])
    files.append(f"pub(super) static TABLES: [Option<RuleTable>; {len(script_names)}] = [")
    files.extend(f'    {name.replace("-", "_").upper()},' for name in script_names)
    files.append("];")
    files.append("pub(super) static RANGES: &[(u32, u32, usize)] = &[")
    files.extend(f"    (0x{first:X}, 0x{last:X}, {indexes[name]})," for first, last, name in ranges)
    files.append("];")
    (ROOT / "src" / "generated_scripts.rs").write_text("\n".join(files) + "\n", encoding="utf-8")

    script_aliases = {
        "arabic": {"ar", "fa", "ps", "sd", "ug", "ur"},
        "armenian": {"hy"},
        "cyrillic": {"be", "bg", "ce", "kk", "ky", "mk", "mn", "ru", "sr", "tg", "tk", "tt", "uk", "uz"},
        "ethiopic": {"am", "gez", "ti"},
        "georgian": {"ka"},
        "greek": {"el"},
        "han": {"cmn", "hak", "ja", "ko", "lzh", "nan", "zh", "yue"},
        "hangul": {"ko"},
        "hebrew": {"he", "yi"},
        "indic": {"as", "bn", "gu", "hi", "kn", "kok", "ml", "mr", "ne", "or", "pa", "sa", "si", "ta", "te"},
        "kana": {"ja"},
        "southeast-asian": {"bo", "dz", "km", "lo", "my", "shn", "th"},
    }
    features["script-data"] = []
    for name in names:
        language = name.split("_", 1)[0]
        features.setdefault(language, [])
    for group, languages in script_aliases.items():
        for language in languages:
            features.setdefault(language, []).append(f"script-{group}")
    lines = ["# BEGIN GENERATED FEATURES"]
    for feature, values in sorted(features.items()):
        lines.append(f"{feature} = {json.dumps(sorted(set(values)))}")
    lines.append("# END GENERATED FEATURES")
    manifest = ROOT / "Cargo.toml"
    source = manifest.read_text(encoding="utf-8")
    source, count = re.subn(
        r"# BEGIN GENERATED FEATURES.*?# END GENERATED FEATURES",
        "\n".join(lines),
        source,
        flags=re.DOTALL,
    )
    assert count == 1
    manifest.write_text(source, encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--vendor-glibc", type=Path, help="snapshot an extracted glibc-2.43 source tree")
    options = parser.parse_args()
    if options.vendor_glibc:
        create_source_archive(options.vendor_glibc)
    locale_files = locale_sources(SOURCE / "glibc-2.43-locales.tar.gz")
    locales = {
        name: locale
        for name, content in locale_files.items()
        if (locale := parse(content, name)) is not None
    }
    fallback, ranges = compile_supplement()
    generate(locales, fallback, ranges)


if __name__ == "__main__":
    main()
