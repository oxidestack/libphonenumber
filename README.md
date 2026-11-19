# libphonenumber

<!-- languages:begin -->
**English** · [Русский](README.ru.md) · [中文](README.zh-CN.md) · [Español](README.es.md)
<!-- languages:end -->

**An unofficial Rust port of Google's [libphonenumber](https://github.com/google/libphonenumber) (Java). It is not
affiliated with or endorsed by Google.** The crates.io package is `libphonenumber`; `use libphonenumber::...`.

It ports the parts of libphonenumber 9.0.41 (Java, `com.googlecode.libphonenumber:libphonenumber`) that Datafaker
2.7.0 reaches: the metadata, `PhoneNumberUtil.parse`, validation, number types, possible-number checks and
formatting. Results are libphonenumber's own. The region metadata it carries is the one of libphonenumber 9.0.41.

`upstream/` is the original source, pinned to the ported release (`v9.0.41`) as a submodule and read-only
(`git submodule update --init`).

It is used by [mock-server](https://github.com/oxidestack/mock-server), a Rust MockServer that must behave exactly as the
real one, and by the Rust port of Datafaker's phone provider, which generates numbers through libphonenumber. The
MockServer it follows ships libphonenumber 9.0.41 (its build pins that over Datafaker's 9.0.33; MockServer 8.0.0 shipped
9.0.38, whose release differs from 9.0.41 only in metadata).

Status: pre-release. The crate is not published on crates.io yet; its first release will be `0.1.0`.

## Install

Not published yet. Once `0.1.0` is on crates.io, the dependency will be:

```toml
[dependencies]
libphonenumber = "0.1"
```

Until then, depend on the repository: `libphonenumber = { git = "https://github.com/oxidestack/libphonenumber" }`. The
crate needs Rust 1.85 or later.

## Example

```rust
use libphonenumber::{PhoneNumberFormat, PhoneNumberType, PhoneNumberUtil};

let util = PhoneNumberUtil::get_instance();
let number = util.parse("044 668 18 00", Some("CH")).expect("a Swiss number");

assert!(util.is_valid_number(&number));
assert_eq!(util.get_number_type(&number), PhoneNumberType::FixedLine);
assert_eq!(util.format(&number, PhoneNumberFormat::E164), "+41446681800");
assert_eq!(util.format(&number, PhoneNumberFormat::International), "+41 44 668 18 00");
```

## Scope

- **The metadata.** `resources/data/` holds libphonenumber's runtime files `PhoneNumberMetadataProto_<region or
  calling code>`, copied verbatim by `scripts/sync-metadata.sh`: the `PhoneMetadataCollection` the JVM reads through
  `ObjectInputStream` and `readExternal`. `build.rs` embeds them, and `metadata.rs` decodes the stream's block data
  and modified UTF-8 the way `readExternal` reads it, so every pattern, length and format is the one Java uses.
  The calling-code map is read out of `CountryCodeToRegionCodeMap.java` at build time. All 254 files are embedded
  (234 KB): `parse("+…")` reaches any calling code, so there is no smaller safe set. Nothing is read from disk or
  network at run time. Upstream's test metadata (`resources/testing/`) is embedded too, for
  `PhoneNumberUtil::for_testing()`.
- **`PhoneNumberUtil`** (`util.rs`):
  - `parse`, `parse_into`, `parse_and_keep_raw_input`, with RFC 3966 input, extensions, IDD and plus-sign prefixes,
    national prefixes and carrier codes, alpha numbers and non-ASCII digits, and the same `NumberParseException`
    types and messages;
  - `is_valid_number`, `is_valid_number_for_region`, `get_number_type`, `get_region_code_for_number`;
  - `format` (E164, INTERNATIONAL, NATIONAL, RFC3966), `format_by_pattern`, the carrier-code formats;
  - the possible-number family (`is_possible_number*`, `ValidationResult`), `truncate_too_long_number`;
  - region and calling-code lookups, supported regions/types/calling codes (in Java's `HashMap`/`HashSet` iteration
    order), example numbers, `get_length_of_geographical_area_code`, `get_length_of_national_destination_code`,
    `is_alpha_number`, `can_be_internationally_dialled`, `is_mobile_number_portable_region`;
  - the package-private helpers the upstream tests call (`normalize`, `extract_possible_number`,
    `maybe_strip_extension`, `maybe_extract_country_code`, …).
- **`PhoneNumber`** and `NumberParseException` (`phonenumber.rs`), with `exactlySameAs` equality and `toString()`.

Not ported, because Datafaker does not reach them: `AsYouTypeFormatter`, `findNumbers`/`PhoneNumberMatcher`,
`ShortNumberInfo` and its metadata, alternate formats, `formatOutOfCountryCallingNumber`,
`formatOutOfCountryKeepingAlphaChars`, `formatInOriginalFormat`, `formatNumberForMobileDialing`, `isNumberMatch`,
the geocoder, carrier and time-zone libraries, and metadata loading (`MetadataLoader`, class-path resources).

## Differences

- Java's regular expressions run on the `regex` crate (`jregex.rs`). The patterns libphonenumber uses read alike in
  both: `\d` in metadata becomes `[0-9]` (Java's is ASCII); the possessive `[+＋]*+` of `VALID_PHONE_NUMBER` is
  greedy here, which matches the same strings because nothing after it can match a plus sign; Java's `.` in
  `VALID_ALPHA_PHONE_PATTERN` excludes every line terminator, as written out here. Replacement strings follow
  `Matcher.appendReplacement` (`$n` read digit by digit while the group exists).
- Unicode tables (`\p{Nd}`, `\p{L}`, case folding of `(?i)`, `Character.digit`) are those of the `regex` and
  `unicode-general-category` crates, not of the JDK; they differ only for characters added to Unicode after the
  JDK's version.
- String positions are byte offsets instead of UTF-16 indices; lengths that Java compares on possibly non-ASCII
  input (`MAX_INPUT_STRING_LENGTH`, `MIN_LENGTH_FOR_NSN`) are counted in UTF-16 units as Java counts them.
- Java's `null` becomes `Option` where a method returns or accepts it (`get_region_code_for_number`,
  `get_metadata_for_region`, the default region of `parse`). Where Java throws an unchecked exception
  (`MissingMetadataException`, `IllegalArgumentException` for an invalid region in an internal lookup, a malformed
  replacement), the port panics with the same message.
- `getExampleNumberForType(type)`, the overload without a region, is `get_example_number_for_any_region`.

## Documentation

- The guides, in English, Russian, Chinese and Spanish: <https://oxidestack.github.io/libphonenumber/>, with
  [upgrading](docs/guide/upgrading.md) between releases.
- The API documentation: on docs.rs once the crate is published; until then, `cargo doc --open`.
- What changed in each release: [CHANGELOG.md](CHANGELOG.md).

## Tests

- `tests/upstream_*.rs`: libphonenumber's own tests of the ported code (`PhoneNumberUtilTest` on the test
  metadata, `PhonenumberTest`, `ExampleNumbersTest`); [docs/test-coverage.md](docs/test-coverage.md) maps each one.
- `tests/vectors.rs`: 13,624 differential rows recorded from libphonenumber 9.0.41 by `scripts/record.sh`
  (`scripts/Vectors.java`): example numbers of every type and region in every format and some distortions, random
  digit strings, odd inputs and the region queries.
- `tests/resources.rs`: `resources/` is a verbatim copy of upstream's metadata.

```sh
git submodule update --init   # upstream/libphonenumber, read by some tests
cargo test
make lint public-check deny   # clippy, the publication gate, cargo-deny
make site                     # the documentation site
scripts/sync-metadata.sh       # after moving upstream/ to another release
scripts/record.sh [--corpus]   # needs a JDK and libphonenumber-9.0.41.jar in ~/.m2 (Maven local repository)
```

## Getting help

A question about how the crate works can go to [DeepWiki](https://deepwiki.com/oxidestack/libphonenumber) first: it
answers from this repository's code and documentation, and where it disagrees with the documentation, the
documentation is right. Questions it does not settle go to
[Discussions](https://github.com/oxidestack/libphonenumber/discussions), bugs to
[issues](https://github.com/oxidestack/libphonenumber/issues).

## License

Apache-2.0, as libphonenumber ([LICENSE](LICENSE), [NOTICE](NOTICE)); documentation CC-BY-4.0
([LICENSE-docs](LICENSE-docs)). Contributing: [CONTRIBUTING.md](CONTRIBUTING.md) (DCO sign-off),
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), [SECURITY.md](SECURITY.md), [GOVERNANCE.md](GOVERNANCE.md).
