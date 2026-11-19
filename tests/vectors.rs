//! Replays tests/vectors/phonenumbers.tsv, recorded from libphonenumber 9.0.41 by scripts/record.sh (the columns are
//! described in scripts/Vectors.java), and fails with every row that differs.

use libphonenumber::{PhoneNumber, PhoneNumberFormat, PhoneNumberType, PhoneNumberUtil};

fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('t') => out.push('\t'),
                Some('n') => out.push('\n'),
                Some(n) => out.push(n),
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\t', "\\t").replace('\n', "\\n")
}

/// `String.valueOf` of a `PhoneNumber` that may be `null`.
fn number_or_null(n: Option<PhoneNumber>) -> String {
    n.map_or("null".into(), |n| n.to_string())
}

/// `TreeSet<PhoneNumberType>.toString()`.
fn types(t: Vec<PhoneNumberType>) -> String {
    format!("[{}]", t.iter().map(|t| t.name()).collect::<Vec<_>>().join(", "))
}

fn list<T: ToString>(items: &[T]) -> String {
    format!("[{}]", items.iter().map(T::to_string).collect::<Vec<_>>().join(", "))
}

fn parse_row(u: &PhoneNumberUtil, input: &str, region: &str) -> Vec<String> {
    let region_opt = (region != "null").then_some(region);
    let n = match u.parse(input, region_opt) {
        Ok(n) => n,
        Err(e) => return vec![format!("ERR {}: {}", e.error_type(), e.message())],
    };
    let mut r = vec![n.to_string()];
    r.push(match u.parse_and_keep_raw_input(input, region_opt) {
        Ok(k) => k.to_string(),
        Err(e) => format!("ERR {}: {}", e.error_type(), e.message()),
    });
    r.push(u.is_valid_number(&n).to_string());
    r.push(u.is_valid_number_for_region(&n, region).to_string());
    r.push(u.get_number_type(&n).name().to_string());
    r.push(u.get_region_code_for_number(&n).unwrap_or_else(|| "null".into()));
    r.push(u.is_possible_number_with_reason(&n).name().to_string());
    for f in [PhoneNumberFormat::E164, PhoneNumberFormat::International, PhoneNumberFormat::National, PhoneNumberFormat::Rfc3966] {
        r.push(u.format(&n, f));
    }
    r.push(u.get_length_of_geographical_area_code(&n).to_string());
    r.push(u.get_length_of_national_destination_code(&n).to_string());
    r.push(u.can_be_internationally_dialled(&n).to_string());
    r
}

fn query(u: &PhoneNumberUtil, q: &str, arg: &str) -> Vec<String> {
    match q {
        "@supportedRegions" => vec![u.get_supported_regions().join(",")],
        "@globalNetworkCallingCodes" => vec![list(u.get_supported_global_network_calling_codes())],
        "@callingCodes" => vec![list(u.get_supported_calling_codes())],
        "@regionInfo" => vec![
            u.get_country_code_for_region(arg).to_string(),
            types(u.get_supported_types_for_region(arg)),
            u.get_ndd_prefix_for_region(arg, false).unwrap_or_else(|| "null".into()),
            u.get_ndd_prefix_for_region(arg, true).unwrap_or_else(|| "null".into()),
            u.is_nanpa_country(arg).to_string(),
            u.is_mobile_number_portable_region(arg).to_string(),
            number_or_null(u.get_example_number(arg)),
            number_or_null(u.get_invalid_example_number(arg)),
        ],
        "@callingCodeInfo" => {
            let code: i32 = arg.parse().expect("calling code");
            vec![
                u.get_region_code_for_country_code(code).to_string(),
                list(&u.get_region_codes_for_country_code(code)),
                types(u.get_supported_types_for_non_geo_entity(code)),
                number_or_null(u.get_example_number_for_non_geo_entity(code)),
            ]
        }
        "@exampleForType" => {
            let t = PhoneNumberType::ALL.into_iter().find(|t| t.name() == arg).expect("type");
            vec![number_or_null(u.get_example_number_for_any_region(t))]
        }
        _ => panic!("unknown query {q}"),
    }
}

#[test]
fn phonenumbers() {
    let u = PhoneNumberUtil::get_instance();
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/vectors/phonenumbers.tsv")).expect("vectors");
    let mut rows = 0;
    let mut failures = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        rows += 1;
        let cols: Vec<&str> = line.split('\t').collect();
        let region = unescape(cols[0]);
        let input = unescape(cols[1]);
        let actual = if input.starts_with('@') { query(u, &input, &region) } else { parse_row(u, &input, &region) };
        let actual: Vec<String> = actual.iter().map(|s| escape(s)).collect();
        if actual != cols[2..] {
            failures.push(format!("{line}\n  rust: {}", actual.join("\t")));
        }
    }
    assert!(rows > 13_000, "{rows} rows");
    assert!(failures.is_empty(), "{} of {rows} rows differ:\n{}", failures.len(), failures.iter().take(30).cloned().collect::<Vec<_>>().join("\n"));
}
