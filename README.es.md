# libphonenumber

<!-- languages:begin -->
[English](README.md) · [Русский](README.ru.md) · [中文](README.zh-CN.md) · **Español**
<!-- languages:end -->
<!-- translation of README.md: 6aa119596861 -->

**Un port no oficial a Rust de [libphonenumber](https://github.com/google/libphonenumber) (Java) de Google. No está
afiliado a Google ni cuenta con su respaldo.** El paquete de crates.io es `libphonenumber`; `use libphonenumber::...`.

Porta las partes de libphonenumber 9.0.41 (Java, `com.googlecode.libphonenumber:libphonenumber`) que usa Datafaker
2.7.0: los metadatos, `PhoneNumberUtil.parse`, la validación, los tipos de número, las comprobaciones de número
posible y el formateo. Los resultados son los de libphonenumber. Los metadatos de regiones que incluye son los de
libphonenumber 9.0.41.

`upstream/` es el código fuente original, fijado en la versión portada (`v9.0.41`) como submódulo y de solo lectura
(`git submodule update --init`).

Lo usan [mock-server](https://github.com/oxidestack/mock-server), un MockServer en Rust que debe comportarse
exactamente como el real, y el port a Rust del proveedor de teléfonos de Datafaker, que genera números con
libphonenumber. El MockServer al que sigue incluye libphonenumber 9.0.41 (su compilación fija esa versión por encima de
la 9.0.33 de Datafaker; MockServer 8.0.0 incluía la 9.0.38, cuya versión difiere de la 9.0.41 solo en los metadatos).

Estado: previo a la primera versión. El crate todavía no está publicado en crates.io; su primera versión será `0.1.0`.

## Instalación

Todavía no publicado. Cuando `0.1.0` esté en crates.io, la dependencia será:

```toml
[dependencies]
libphonenumber = "0.1"
```

Hasta entonces, dependa del repositorio: `libphonenumber = { git = "https://github.com/oxidestack/libphonenumber" }`.
El crate necesita Rust 1.85 o posterior.

## Ejemplo

```rust
use libphonenumber::{PhoneNumberFormat, PhoneNumberType, PhoneNumberUtil};

let util = PhoneNumberUtil::get_instance();
let number = util.parse("044 668 18 00", Some("CH")).expect("a Swiss number");

assert!(util.is_valid_number(&number));
assert_eq!(util.get_number_type(&number), PhoneNumberType::FixedLine);
assert_eq!(util.format(&number, PhoneNumberFormat::E164), "+41446681800");
assert_eq!(util.format(&number, PhoneNumberFormat::International), "+41 44 668 18 00");
```

## Alcance

- **Los metadatos.** `resources/data/` contiene los archivos de ejecución de libphonenumber
  `PhoneNumberMetadataProto_<region or calling code>`, copiados tal cual por `scripts/sync-metadata.sh`: la
  `PhoneMetadataCollection` que la JVM lee mediante `ObjectInputStream` y `readExternal`. `build.rs` los incrusta, y
  `metadata.rs` decodifica los datos en bloque del flujo y el modified UTF-8 como los lee `readExternal`, de modo que
  cada patrón, longitud y formato es el que usa Java. La tabla de códigos de país se lee de
  `CountryCodeToRegionCodeMap.java` al compilar. Se incrustan los 254 archivos (234 KB): `parse("+…")` alcanza
  cualquier código de país, así que no hay un conjunto seguro más pequeño. En ejecución no se lee nada del disco ni de
  la red. Los metadatos de prueba del original (`resources/testing/`) también se incrustan, para
  `PhoneNumberUtil::for_testing()`.
- **`PhoneNumberUtil`** (`util.rs`):
  - `parse`, `parse_into`, `parse_and_keep_raw_input`, con entrada RFC 3966, extensiones, prefijos IDD y de signo más,
    prefijos nacionales y códigos de operador, números alfabéticos y dígitos no ASCII, y los mismos tipos y mensajes
    de `NumberParseException`;
  - `is_valid_number`, `is_valid_number_for_region`, `get_number_type`, `get_region_code_for_number`;
  - `format` (E164, INTERNATIONAL, NATIONAL, RFC3966), `format_by_pattern`, los formatos con código de operador;
  - la familia de número posible (`is_possible_number*`, `ValidationResult`), `truncate_too_long_number`;
  - búsquedas por región y código de país, regiones, tipos y códigos de país admitidos (en el orden de iteración de
    `HashMap`/`HashSet` de Java), números de ejemplo, `get_length_of_geographical_area_code`,
    `get_length_of_national_destination_code`, `is_alpha_number`, `can_be_internationally_dialled`,
    `is_mobile_number_portable_region`;
  - las funciones auxiliares de ámbito de paquete que llaman las pruebas del original (`normalize`,
    `extract_possible_number`, `maybe_strip_extension`, `maybe_extract_country_code`, …).
- **`PhoneNumber`** y `NumberParseException` (`phonenumber.rs`), con la igualdad `exactlySameAs` y `toString()`.

No se porta, porque Datafaker no lo usa: `AsYouTypeFormatter`, `findNumbers`/`PhoneNumberMatcher`, `ShortNumberInfo` y
sus metadatos, los formatos alternativos, `formatOutOfCountryCallingNumber`, `formatOutOfCountryKeepingAlphaChars`,
`formatInOriginalFormat`, `formatNumberForMobileDialing`, `isNumberMatch`, las bibliotecas de geocodificación,
operadores y zonas horarias, y la carga de metadatos (`MetadataLoader`, recursos del class path).

## Diferencias

- Las expresiones regulares de Java se ejecutan con el crate `regex` (`jregex.rs`). Los patrones que usa
  libphonenumber se leen igual en ambos: `\d` en los metadatos pasa a ser `[0-9]` (el de Java es ASCII); el posesivo
  `[+＋]*+` de `VALID_PHONE_NUMBER` aquí es voraz, lo que coincide con las mismas cadenas porque nada de lo que sigue
  puede coincidir con un signo más; el `.` de `VALID_ALPHA_PHONE_PATTERN` en Java excluye todo terminador de línea, y
  aquí se escribe explícitamente. Las cadenas de reemplazo siguen `Matcher.appendReplacement` (`$n` se lee dígito a
  dígito mientras exista el grupo).
- Las tablas Unicode (`\p{Nd}`, `\p{L}`, el plegado de mayúsculas de `(?i)`, `Character.digit`) son las de los crates
  `regex` y `unicode-general-category`, no las del JDK; solo difieren en caracteres añadidos a Unicode después de la
  versión del JDK.
- Las posiciones en cadenas son desplazamientos en bytes en lugar de índices UTF-16; las longitudes que Java compara
  sobre entradas posiblemente no ASCII (`MAX_INPUT_STRING_LENGTH`, `MIN_LENGTH_FOR_NSN`) se cuentan en unidades UTF-16,
  como las cuenta Java.
- El `null` de Java se convierte en `Option` donde un método lo devuelve o lo acepta (`get_region_code_for_number`,
  `get_metadata_for_region`, la región por defecto de `parse`). Donde Java lanza una excepción no comprobada
  (`MissingMetadataException`, `IllegalArgumentException` por una región no válida en una búsqueda interna, una cadena
  de reemplazo mal formada), el port entra en pánico con el mismo mensaje.
- `getExampleNumberForType(type)`, la sobrecarga sin región, es `get_example_number_for_any_region`.

## Documentación

- Las guías, en inglés, ruso, chino y español: <https://oxidestack.github.io/libphonenumber/>, con la
  [actualización](docs/i18n/es/guide/upgrading.md) entre versiones.
- La documentación de la API: en docs.rs cuando el crate se publique; hasta entonces, `cargo doc --open`.
- Qué cambió en cada versión: [CHANGELOG.md](CHANGELOG.md).

## Pruebas

- `tests/upstream_*.rs`: las pruebas propias de libphonenumber para el código portado (`PhoneNumberUtilTest` sobre los
  metadatos de prueba, `PhonenumberTest`, `ExampleNumbersTest`); [docs/test-coverage.md](docs/test-coverage.md)
  relaciona cada una.
- `tests/vectors.rs`: 13 624 filas diferenciales grabadas de libphonenumber 9.0.41 por `scripts/record.sh`
  (`scripts/Vectors.java`): números de ejemplo de cada tipo y región en cada formato y algunas distorsiones, cadenas de
  dígitos aleatorias, entradas inusuales y las consultas por región.
- `tests/resources.rs`: `resources/` es una copia exacta de los metadatos del original.

```sh
git submodule update --init   # upstream/libphonenumber, read by some tests
cargo test
make lint public-check deny   # clippy, the publication gate, cargo-deny
make site                     # the documentation site
scripts/sync-metadata.sh       # after moving upstream/ to another release
scripts/record.sh [--corpus]   # needs a JDK and libphonenumber-9.0.41.jar in ~/.m2 (Maven local repository)
```

## Ayuda

Una pregunta sobre cómo funciona el crate puede ir primero a [DeepWiki](https://deepwiki.com/oxidestack/libphonenumber):
responde a partir del código y la documentación de este repositorio, y donde no coincide con la documentación, la
documentación tiene razón. Las preguntas que no resuelva van a
[Discussions](https://github.com/oxidestack/libphonenumber/discussions), y los errores a
[issues](https://github.com/oxidestack/libphonenumber/issues).

## Licencia

Apache-2.0, como libphonenumber ([LICENSE](LICENSE), [NOTICE](NOTICE)); la documentación, CC-BY-4.0
([LICENSE-docs](LICENSE-docs)). Para contribuir: [CONTRIBUTING.md](CONTRIBUTING.md) (sign-off DCO),
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), [SECURITY.md](SECURITY.md), [GOVERNANCE.md](GOVERNANCE.md).
