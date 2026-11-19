# libphonenumber

<!-- languages:begin -->
[English](README.md) · [Русский](README.ru.md) · **中文** · [Español](README.es.md)
<!-- languages:end -->
<!-- translation of README.md: 6aa119596861 -->

**Google [libphonenumber](https://github.com/google/libphonenumber)（Java）的非官方 Rust 移植。本项目与 Google
没有关联，也未获得其认可。** crates.io 上的包名为 `libphonenumber`；`use libphonenumber::...`。

它移植了 libphonenumber 9.0.41（Java，`com.googlecode.libphonenumber:libphonenumber`）中 Datafaker 2.7.0
用到的部分：元数据、`PhoneNumberUtil.parse`、校验、号码类型、可能号码检查以及格式化。结果与 libphonenumber 本身一致。所带的地区元数据来自
libphonenumber 9.0.41。

`upstream/` 是原始源代码，以子模块形式固定在所移植的版本（`v9.0.41`），只读（`git submodule update --init`）。

使用它的有 [mock-server](https://github.com/oxidestack/mock-server)（一个必须与真正的 MockServer 行为完全一致的 Rust 版
MockServer），以及 Datafaker 电话号码生成器的 Rust 移植，后者通过 libphonenumber 生成号码。它所跟随的 MockServer 附带
libphonenumber 9.0.41（其构建用该版本覆盖了 Datafaker 的 9.0.33；MockServer 8.0.0 附带的是 9.0.38，该版本与 9.0.41
的区别仅在于元数据）。

状态：尚未发布。该 crate 还没有在 crates.io 上发布；首个版本将是 `0.1.0`。

## 安装

尚未发布。`0.1.0` 发布到 crates.io 之后，依赖将是：

```toml
[dependencies]
libphonenumber = "0.1"
```

在此之前，请直接依赖仓库：`libphonenumber = { git = "https://github.com/oxidestack/libphonenumber" }`。该 crate 需要
Rust 1.85 或更高版本。

## 示例

```rust
use libphonenumber::{PhoneNumberFormat, PhoneNumberType, PhoneNumberUtil};

let util = PhoneNumberUtil::get_instance();
let number = util.parse("044 668 18 00", Some("CH")).expect("a Swiss number");

assert!(util.is_valid_number(&number));
assert_eq!(util.get_number_type(&number), PhoneNumberType::FixedLine);
assert_eq!(util.format(&number, PhoneNumberFormat::E164), "+41446681800");
assert_eq!(util.format(&number, PhoneNumberFormat::International), "+41 44 668 18 00");
```

## 范围

- **元数据。** `resources/data/` 中存放 libphonenumber 的运行时文件 `PhoneNumberMetadataProto_<region or calling code>`，由
  `scripts/sync-metadata.sh` 原样复制：即 JVM 通过 `ObjectInputStream` 和 `readExternal` 读取的
  `PhoneMetadataCollection`。`build.rs` 将其嵌入，`metadata.rs` 按 `readExternal` 的方式解码流中的块数据和 modified
  UTF-8，因此每个模式、长度和格式都与 Java 使用的相同。国家/地区代码映射在构建时从 `CountryCodeToRegionCodeMap.java`
  中读取。全部 254 个文件都被嵌入（234 KB）：`parse("+…")` 可以到达任何国家/地区代码，所以不存在更小的安全子集。运行时不从磁盘或网络读取任何内容。
  上游的测试元数据（`resources/testing/`）也被嵌入，供 `PhoneNumberUtil::for_testing()` 使用。
- **`PhoneNumberUtil`**（`util.rs`）：
  - `parse`、`parse_into`、`parse_and_keep_raw_input`，支持 RFC 3966 输入、分机号、IDD 和加号前缀、国内前缀和运营商代码、字母号码和非
    ASCII 数字，并给出相同的 `NumberParseException` 类型和消息；
  - `is_valid_number`、`is_valid_number_for_region`、`get_number_type`、`get_region_code_for_number`；
  - `format`（E164、INTERNATIONAL、NATIONAL、RFC3966）、`format_by_pattern`、带运营商代码的格式；
  - 可能号码检查系列（`is_possible_number*`、`ValidationResult`）、`truncate_too_long_number`；
  - 地区和国家/地区代码查询，支持的地区/类型/国家/地区代码（按 Java `HashMap`/`HashSet` 的迭代顺序），示例号码，
    `get_length_of_geographical_area_code`、`get_length_of_national_destination_code`、`is_alpha_number`、
    `can_be_internationally_dialled`、`is_mobile_number_portable_region`；
  - 上游测试调用的包级私有辅助函数（`normalize`、`extract_possible_number`、`maybe_strip_extension`、
    `maybe_extract_country_code` 等）。
- **`PhoneNumber`** 和 `NumberParseException`（`phonenumber.rs`），包括 `exactlySameAs` 相等性和 `toString()`。

以下内容未移植，因为 Datafaker 用不到：`AsYouTypeFormatter`、`findNumbers`/`PhoneNumberMatcher`、`ShortNumberInfo`
及其元数据、备选格式、`formatOutOfCountryCallingNumber`、`formatOutOfCountryKeepingAlphaChars`、`formatInOriginalFormat`、
`formatNumberForMobileDialing`、`isNumberMatch`、地理编码、运营商和时区库，以及元数据加载（`MetadataLoader`、class path 资源）。

## 差异

- Java 的正则表达式在 `regex` crate 上运行（`jregex.rs`）。libphonenumber 使用的模式在两者中含义相同：元数据中的 `\d` 变为
  `[0-9]`（Java 的 `\d` 只匹配 ASCII）；`VALID_PHONE_NUMBER` 中的占有型 `[+＋]*+` 在这里是贪婪的，匹配的字符串相同，因为其后的内容不可能匹配加号；
  Java 中 `VALID_ALPHA_PHONE_PATTERN` 的 `.` 不匹配任何行终止符，这里将其显式写出。替换字符串遵循 `Matcher.appendReplacement`
  （只要分组存在，`$n` 就逐位读取）。
- Unicode 表（`\p{Nd}`、`\p{L}`、`(?i)` 的大小写折叠、`Character.digit`）来自 `regex` 和 `unicode-general-category` crate，
  而不是 JDK；它们只在 JDK 版本之后加入 Unicode 的字符上有所不同。
- 字符串位置是字节偏移，而不是 UTF-16 索引；Java 在可能为非 ASCII 的输入上比较的长度（`MAX_INPUT_STRING_LENGTH`、
  `MIN_LENGTH_FOR_NSN`）按 Java 的方式以 UTF-16 单元计数。
- 在方法返回或接受 Java `null` 的地方，它变为 `Option`（`get_region_code_for_number`、`get_metadata_for_region`、`parse`
  的默认地区）。在 Java 抛出非受检异常的地方（`MissingMetadataException`、内部查询中无效地区引发的
  `IllegalArgumentException`、格式错误的替换字符串），移植版以相同的消息 panic。
- 不带地区参数的重载 `getExampleNumberForType(type)` 对应 `get_example_number_for_any_region`。

## 文档

- 指南（英文、俄文、中文和西班牙文）：<https://oxidestack.github.io/libphonenumber/>，包括版本之间的
  [升级](docs/i18n/zh/guide/upgrading.md)。
- API 文档：crate 发布后位于 docs.rs；在此之前请使用 `cargo doc --open`。
- 每个版本的变更：[CHANGELOG.md](CHANGELOG.md)。

## 测试

- `tests/upstream_*.rs`：libphonenumber 针对所移植代码的自有测试（基于测试元数据的 `PhoneNumberUtilTest`、
  `PhonenumberTest`、`ExampleNumbersTest`）；[docs/test-coverage.md](docs/test-coverage.md) 列出了每一个的对应关系。
- `tests/vectors.rs`：由 `scripts/record.sh`（`scripts/Vectors.java`）从 libphonenumber 9.0.41 记录的 13,624
  行差分数据：每种类型和地区的示例号码在各种格式下的结果及部分变形、随机数字串、特殊输入以及地区查询。
- `tests/resources.rs`：`resources/` 是上游元数据的原样副本。

```sh
git submodule update --init   # upstream/libphonenumber, read by some tests
cargo test
make lint public-check deny   # clippy, the publication gate, cargo-deny
make site                     # the documentation site
scripts/sync-metadata.sh       # after moving upstream/ to another release
scripts/record.sh [--corpus]   # needs a JDK and libphonenumber-9.0.41.jar in ~/.m2 (Maven local repository)
```

## 获取帮助

关于本 crate 如何工作的问题，可以先问 [DeepWiki](https://deepwiki.com/oxidestack/libphonenumber)：它根据本仓库的代码和文档作答，
当它与文档不一致时，以文档为准。它解决不了的问题请到 [Discussions](https://github.com/oxidestack/libphonenumber/discussions)，
缺陷请提交到 [issues](https://github.com/oxidestack/libphonenumber/issues)。

## 许可证

Apache-2.0，与 libphonenumber 相同（[LICENSE](LICENSE)、[NOTICE](NOTICE)）；文档采用 CC-BY-4.0
（[LICENSE-docs](LICENSE-docs)）。参与贡献：[CONTRIBUTING.md](CONTRIBUTING.md)（DCO 签署）、
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)、[SECURITY.md](SECURITY.md)、[GOVERNANCE.md](GOVERNANCE.md)。
