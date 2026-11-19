# libphonenumber

<!-- languages:begin -->
[English](README.md) · **Русский** · [中文](README.zh-CN.md) · [Español](README.es.md)
<!-- languages:end -->
<!-- translation of README.md: 6aa119596861 -->

**Неофициальный порт на Rust библиотеки Google [libphonenumber](https://github.com/google/libphonenumber) (Java).
Проект не связан с Google и не одобрен ею.** Пакет на crates.io называется `libphonenumber`; `use libphonenumber::...`.

Он портирует те части libphonenumber 9.0.41 (Java, `com.googlecode.libphonenumber:libphonenumber`), до которых
доходит Datafaker 2.7.0: метаданные, `PhoneNumberUtil.parse`, проверку, типы номеров, проверки возможности номера и
форматирование. Результаты — те же, что у libphonenumber. Метаданные регионов — из libphonenumber 9.0.41.

`upstream/` — исходный код оригинала, закреплённый на портированном выпуске (`v9.0.41`) как подмодуль и доступный
только для чтения (`git submodule update --init`).

Его используют [mock-server](https://github.com/oxidestack/mock-server) — MockServer на Rust, который должен вести себя
в точности как настоящий, — и порт на Rust провайдера телефонов Datafaker, который генерирует номера через
libphonenumber. MockServer, за которым он следует, поставляет libphonenumber 9.0.41 (его сборка закрепляет эту версию
поверх 9.0.33 из Datafaker; MockServer 8.0.0 поставлял 9.0.38, выпуск которой отличается от 9.0.41 только
метаданными).

Статус: до первого выпуска. Крейт ещё не опубликован на crates.io; первым выпуском станет `0.1.0`.

## Установка

Ещё не опубликован. Когда `0.1.0` появится на crates.io, зависимость будет такой:

```toml
[dependencies]
libphonenumber = "0.1"
```

До тех пор зависьте от репозитория: `libphonenumber = { git = "https://github.com/oxidestack/libphonenumber" }`.
Крейту нужен Rust 1.85 или новее.

## Пример

```rust
use libphonenumber::{PhoneNumberFormat, PhoneNumberType, PhoneNumberUtil};

let util = PhoneNumberUtil::get_instance();
let number = util.parse("044 668 18 00", Some("CH")).expect("a Swiss number");

assert!(util.is_valid_number(&number));
assert_eq!(util.get_number_type(&number), PhoneNumberType::FixedLine);
assert_eq!(util.format(&number, PhoneNumberFormat::E164), "+41446681800");
assert_eq!(util.format(&number, PhoneNumberFormat::International), "+41 44 668 18 00");
```

## Объём

- **Метаданные.** В `resources/data/` лежат файлы времени выполнения libphonenumber
  `PhoneNumberMetadataProto_<region or calling code>`, скопированные без изменений скриптом
  `scripts/sync-metadata.sh`: `PhoneMetadataCollection`, которую JVM читает через `ObjectInputStream` и
  `readExternal`. `build.rs` встраивает их, а `metadata.rs` декодирует блочные данные потока и modified UTF-8 так же,
  как их читает `readExternal`, поэтому каждый шаблон, длина и формат — те же, что использует Java. Таблица кодов
  страны читается из `CountryCodeToRegionCodeMap.java` во время сборки. Встроены все 254 файла (234 КБ):
  `parse("+…")` доходит до любого кода страны, так что меньшего безопасного набора нет. Во время выполнения ничего не
  читается ни с диска, ни из сети. Тестовые метаданные оригинала (`resources/testing/`) тоже встроены — для
  `PhoneNumberUtil::for_testing()`.
- **`PhoneNumberUtil`** (`util.rs`):
  - `parse`, `parse_into`, `parse_and_keep_raw_input` — с вводом RFC 3966, добавочными номерами, префиксами IDD и
    знаком плюс, национальными префиксами и кодами операторов, буквенными номерами и не-ASCII цифрами, и с теми же
    типами и сообщениями `NumberParseException`;
  - `is_valid_number`, `is_valid_number_for_region`, `get_number_type`, `get_region_code_for_number`;
  - `format` (E164, INTERNATIONAL, NATIONAL, RFC3966), `format_by_pattern`, форматы с кодом оператора;
  - семейство проверок возможности номера (`is_possible_number*`, `ValidationResult`), `truncate_too_long_number`;
  - поиск по регионам и кодам страны, поддерживаемые регионы, типы и коды страны (в порядке обхода `HashMap`/`HashSet`
    в Java), номера-примеры, `get_length_of_geographical_area_code`, `get_length_of_national_destination_code`,
    `is_alpha_number`, `can_be_internationally_dialled`, `is_mobile_number_portable_region`;
  - вспомогательные функции уровня пакета, которые вызывают тесты оригинала (`normalize`, `extract_possible_number`,
    `maybe_strip_extension`, `maybe_extract_country_code`, …).
- **`PhoneNumber`** и `NumberParseException` (`phonenumber.rs`), с равенством `exactlySameAs` и `toString()`.

Не портировано, потому что Datafaker до этого не доходит: `AsYouTypeFormatter`, `findNumbers`/`PhoneNumberMatcher`,
`ShortNumberInfo` и его метаданные, альтернативные форматы, `formatOutOfCountryCallingNumber`,
`formatOutOfCountryKeepingAlphaChars`, `formatInOriginalFormat`, `formatNumberForMobileDialing`, `isNumberMatch`,
библиотеки геокодера, операторов и часовых поясов, а также загрузка метаданных (`MetadataLoader`, ресурсы из
class path).

## Отличия

- Регулярные выражения Java выполняются крейтом `regex` (`jregex.rs`). Шаблоны, которые использует libphonenumber,
  читаются в обоих одинаково: `\d` в метаданных становится `[0-9]` (в Java он ASCII); possessive-квантификатор
  `[+＋]*+` в `VALID_PHONE_NUMBER` здесь жадный, что совпадает с теми же строками, потому что ничто после него не может
  совпасть со знаком плюс; `.` в `VALID_ALPHA_PHONE_PATTERN` в Java исключает любой символ конца строки, и здесь это
  записано явно. Строки замены следуют `Matcher.appendReplacement` (`$n` читается цифра за цифрой, пока группа
  существует).
- Таблицы Unicode (`\p{Nd}`, `\p{L}`, свёртка регистра в `(?i)`, `Character.digit`) — от крейтов `regex` и
  `unicode-general-category`, а не от JDK; они различаются только для символов, добавленных в Unicode после версии
  JDK.
- Позиции в строках — смещения в байтах, а не индексы UTF-16; длины, которые Java сравнивает на возможно не-ASCII
  вводе (`MAX_INPUT_STRING_LENGTH`, `MIN_LENGTH_FOR_NSN`), считаются в единицах UTF-16, как их считает Java.
- `null` из Java становится `Option` там, где метод его возвращает или принимает (`get_region_code_for_number`,
  `get_metadata_for_region`, регион по умолчанию в `parse`). Где Java бросает непроверяемое исключение
  (`MissingMetadataException`, `IllegalArgumentException` для недопустимого региона во внутреннем поиске, неправильная
  строка замены), порт паникует с тем же сообщением.
- `getExampleNumberForType(type)`, перегрузка без региона, — это `get_example_number_for_any_region`.

## Документация

- Руководства на английском, русском, китайском и испанском: <https://oxidestack.github.io/libphonenumber/>, включая
  [обновление](docs/i18n/ru/guide/upgrading.md) между выпусками.
- Документация API: на docs.rs, когда крейт будет опубликован; до тех пор — `cargo doc --open`.
- Что изменилось в каждом выпуске: [CHANGELOG.md](CHANGELOG.md).

## Тесты

- `tests/upstream_*.rs`: собственные тесты libphonenumber для портированного кода (`PhoneNumberUtilTest` на тестовых
  метаданных, `PhonenumberTest`, `ExampleNumbersTest`); [docs/test-coverage.md](docs/test-coverage.md) сопоставляет
  каждый из них.
- `tests/vectors.rs`: 13 624 строки для сравнения, записанные с libphonenumber 9.0.41 скриптом `scripts/record.sh`
  (`scripts/Vectors.java`): номера-примеры каждого типа и региона во всех форматах и с некоторыми искажениями,
  случайные строки цифр, необычный ввод и запросы по регионам.
- `tests/resources.rs`: `resources/` — точная копия метаданных оригинала.

```sh
git submodule update --init   # upstream/libphonenumber, read by some tests
cargo test
make lint public-check deny   # clippy, the publication gate, cargo-deny
make site                     # the documentation site
scripts/sync-metadata.sh       # after moving upstream/ to another release
scripts/record.sh [--corpus]   # needs a JDK and libphonenumber-9.0.41.jar in ~/.m2 (Maven local repository)
```

## Помощь

Вопрос о том, как работает крейт, можно сначала задать [DeepWiki](https://deepwiki.com/oxidestack/libphonenumber): он
отвечает по коду и документации этого репозитория, а где он расходится с документацией, права документация. Вопросы,
которые он не решил, — в [Discussions](https://github.com/oxidestack/libphonenumber/discussions), ошибки — в
[issues](https://github.com/oxidestack/libphonenumber/issues).

## Лицензия

Apache-2.0, как у libphonenumber ([LICENSE](LICENSE), [NOTICE](NOTICE)); документация — CC-BY-4.0
([LICENSE-docs](LICENSE-docs)). Участие в разработке: [CONTRIBUTING.md](CONTRIBUTING.md) (подпись DCO),
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), [SECURITY.md](SECURITY.md), [GOVERNANCE.md](GOVERNANCE.md).
