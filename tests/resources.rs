//! resources/ is a verbatim copy of libphonenumber's runtime and test metadata (scripts/sync-metadata.sh). Fails
//! when a file differs from upstream or upstream has one the copy lacks; passes without the upstream submodule.

use std::path::Path;

fn compare(copy: &Path, upstream: &Path, prefix: &str) {
    let mut names: Vec<String> = std::fs::read_dir(upstream)
        .expect("upstream dir")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(prefix))
        .collect();
    names.sort();
    assert!(!names.is_empty(), "no {prefix}* in {}", upstream.display());
    for name in names {
        let ours = std::fs::read(copy.join(&name)).unwrap_or_else(|_| panic!("resources lack {name}"));
        assert!(ours == std::fs::read(upstream.join(&name)).expect("upstream file"), "{name} differs from upstream");
    }
}

#[test]
fn resources_match_upstream() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let java = root.join("upstream/libphonenumber/java/libphonenumber");
    if !java.exists() {
        eprintln!("upstream/libphonenumber is not checked out; skipped");
        return;
    }
    let src = java.join("src/com/google/i18n/phonenumbers");
    let test = java.join("test/com/google/i18n/phonenumbers");
    compare(&root.join("resources/data"), &src.join("data"), "PhoneNumberMetadataProto_");
    compare(&root.join("resources"), &src, "CountryCodeToRegionCodeMap.java");
    compare(&root.join("resources/testing"), &test.join("data"), "PhoneNumberMetadataProtoForTesting_");
    compare(&root.join("resources/testing"), &test, "CountryCodeToRegionCodeMapForTesting.java");
}
