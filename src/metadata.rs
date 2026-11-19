//! `Phonemetadata` (`PhoneMetadata`, `PhoneNumberDesc`, `NumberFormat`) and the metadata source.
//!
//! The data is libphonenumber's own runtime metadata: the files `PhoneNumberMetadataProto_<key>` that the JVM loads
//! through `ObjectInputStream` and `PhoneMetadataCollection.readExternal`. They are embedded from `resources/` and
//! decoded here the way `readExternal` reads them, so every pattern, length and format is the one Java uses.

use std::collections::HashMap;
use std::sync::OnceLock;

mod files {
    include!(concat!(env!("OUT_DIR"), "/metadata_files.rs"));
}

/// `Phonemetadata.NumberFormat`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NumberFormat {
    /// The pattern a national significant number must match in full for this format to apply.
    pub pattern: String,
    /// How the groups of `pattern` are written, with `$1`, `$2`, … for the groups.
    pub format: String,
    /// Patterns the number's leading digits must match, the most specific last; empty means any.
    pub leading_digits_pattern: Vec<String>,
    /// How the national prefix is written in front of the first group when formatting nationally.
    pub national_prefix_formatting_rule: String,
    /// Whether the metadata sets `national_prefix_formatting_rule`.
    pub has_national_prefix_formatting_rule: bool,
    /// Whether a number formatted nationally may leave the national prefix out.
    pub national_prefix_optional_when_formatting: bool,
    /// How a domestic carrier code is written when formatting with one.
    pub domestic_carrier_code_formatting_rule: String,
    /// Whether the metadata sets `domestic_carrier_code_formatting_rule`.
    pub has_domestic_carrier_code_formatting_rule: bool,
}

/// `Phonemetadata.PhoneNumberDesc`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhoneNumberDesc {
    /// The pattern a national significant number of this type matches.
    pub national_number_pattern: String,
    /// Whether the metadata sets `national_number_pattern`.
    pub has_national_number_pattern: bool,
    /// The lengths a national significant number of this type can have.
    pub possible_length: Vec<i32>,
    /// The lengths only dialable within a local area.
    pub possible_length_local_only: Vec<i32>,
    /// An example number of this type.
    pub example_number: String,
    /// Whether the metadata sets `example_number`.
    pub has_example_number: bool,
}

/// `Phonemetadata.PhoneMetadata`. Descriptions that the file leaves out are `None` (Java's `null`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhoneMetadata {
    /// What every number of the region matches.
    pub general_desc: Option<PhoneNumberDesc>,
    /// Fixed-line numbers.
    pub fixed_line: Option<PhoneNumberDesc>,
    /// Mobile numbers.
    pub mobile: Option<PhoneNumberDesc>,
    /// Toll-free numbers.
    pub toll_free: Option<PhoneNumberDesc>,
    /// Premium-rate numbers.
    pub premium_rate: Option<PhoneNumberDesc>,
    /// Shared-cost numbers.
    pub shared_cost: Option<PhoneNumberDesc>,
    /// Personal numbers.
    pub personal_number: Option<PhoneNumberDesc>,
    /// Voice-over-IP numbers.
    pub voip: Option<PhoneNumberDesc>,
    /// Pager numbers.
    pub pager: Option<PhoneNumberDesc>,
    /// Universal access numbers.
    pub uan: Option<PhoneNumberDesc>,
    /// Emergency numbers (short-number metadata).
    pub emergency: Option<PhoneNumberDesc>,
    /// Voicemail access numbers.
    pub voicemail: Option<PhoneNumberDesc>,
    /// Short codes (short-number metadata).
    pub short_code: Option<PhoneNumberDesc>,
    /// Standard-rate short numbers (short-number metadata).
    pub standard_rate: Option<PhoneNumberDesc>,
    /// Carrier-specific short numbers (short-number metadata).
    pub carrier_specific: Option<PhoneNumberDesc>,
    /// SMS-only short numbers (short-number metadata).
    pub sms_services: Option<PhoneNumberDesc>,
    /// Numbers that cannot be dialled from abroad.
    pub no_international_dialling: Option<PhoneNumberDesc>,
    /// The region code, `US`, or `001` for a non-geographical entity.
    pub id: String,
    /// The country calling code, `1` for `US`.
    pub country_code: i32,
    /// The pattern of the prefixes dialled to call abroad from the region.
    pub international_prefix: String,
    /// The international prefix to write when `international_prefix` allows several.
    pub preferred_international_prefix: String,
    /// Whether the metadata sets `preferred_international_prefix`.
    pub has_preferred_international_prefix: bool,
    /// The prefix dialled before a national number within the region (the trunk prefix).
    pub national_prefix: String,
    /// Whether the metadata sets `national_prefix`.
    pub has_national_prefix: bool,
    /// How an extension is introduced when a number is formatted, if not ` ext. `.
    pub preferred_extn_prefix: String,
    /// Whether the metadata sets `preferred_extn_prefix`.
    pub has_preferred_extn_prefix: bool,
    /// The pattern of the national prefix (and carrier code) stripped when parsing.
    pub national_prefix_for_parsing: String,
    /// Whether the metadata sets `national_prefix_for_parsing`.
    pub has_national_prefix_for_parsing: bool,
    /// How the number is rewritten after `national_prefix_for_parsing` matched, with its groups.
    pub national_prefix_transform_rule: String,
    /// Whether the metadata sets `national_prefix_transform_rule`.
    pub has_national_prefix_transform_rule: bool,
    /// Whether mobile and fixed-line numbers share one pattern in the region.
    pub same_mobile_and_fixed_line_pattern: bool,
    /// The formats for national formatting, tried in order.
    pub number_format: Vec<NumberFormat>,
    /// The formats for international formatting; empty means `number_format`.
    pub intl_number_format: Vec<NumberFormat>,
    /// Whether this region is the main one of a calling code shared by several.
    pub main_country_for_code: bool,
    /// The leading digits that pick this region among those sharing its calling code.
    pub leading_digits: String,
    /// Whether the metadata sets `leading_digits`.
    pub has_leading_digits: bool,
    /// Whether mobile numbers can be ported to another carrier in the region.
    pub mobile_number_portable_region: bool,
}

static EMPTY_DESC: PhoneNumberDesc = PhoneNumberDesc {
    national_number_pattern: String::new(),
    has_national_number_pattern: false,
    possible_length: Vec::new(),
    possible_length_local_only: Vec::new(),
    example_number: String::new(),
    has_example_number: false,
};

impl PhoneMetadata {
    /// `getGeneralDesc()`. Every description of libphonenumber's files is present; an absent one (where Java would
    /// throw a `NullPointerException`) reads as an empty description.
    pub fn general_desc(&self) -> &PhoneNumberDesc {
        self.general_desc.as_ref().unwrap_or(&EMPTY_DESC)
    }
}

/// A truncated or malformed metadata stream (Java's `IOException` from `readExternal`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataError(pub String);

/// `ObjectInputStream` over an `Externalizable` that writes primitives only: the stream header, then block-data
/// records whose payloads, concatenated, are `DataOutput` bytes.
struct ObjectInput {
    data: Vec<u8>,
    pos: usize,
}

impl ObjectInput {
    fn new(bytes: &[u8]) -> Result<Self, MetadataError> {
        if bytes.len() < 4 || bytes[0..4] != [0xAC, 0xED, 0x00, 0x05] {
            return Err(MetadataError("invalid stream header".into()));
        }
        let mut data = Vec::with_capacity(bytes.len());
        let mut i = 4;
        while i < bytes.len() {
            let (len, header) = match bytes[i] {
                0x77 => (*bytes.get(i + 1).ok_or_else(truncated)? as usize, 2),
                0x7A => {
                    let b = bytes.get(i + 1..i + 5).ok_or_else(truncated)?;
                    (u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize, 5)
                }
                tc => return Err(MetadataError(format!("unexpected type code 0x{tc:02X}"))),
            };
            let payload = bytes.get(i + header..i + header + len).ok_or_else(truncated)?;
            data.extend_from_slice(payload);
            i += header + len;
        }
        Ok(ObjectInput { data, pos: 0 })
    }

    fn take(&mut self, n: usize) -> Result<&[u8], MetadataError> {
        let slice = self.data.get(self.pos..self.pos + n).ok_or_else(truncated)?;
        self.pos += n;
        Ok(slice)
    }

    fn read_boolean(&mut self) -> Result<bool, MetadataError> {
        Ok(self.take(1)?[0] != 0)
    }

    fn read_int(&mut self) -> Result<i32, MetadataError> {
        let b = self.take(4)?;
        Ok(i32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// `readUTF`: a two-byte length and modified UTF-8 (UTF-16 code units, NUL as `C0 80`).
    fn read_utf(&mut self) -> Result<String, MetadataError> {
        let b = self.take(2)?;
        let len = u16::from_be_bytes([b[0], b[1]]) as usize;
        let bytes = self.take(len)?.to_vec();
        let mut units = Vec::with_capacity(len);
        let mut i = 0;
        while i < bytes.len() {
            let c = bytes[i] as u16;
            if c < 0x80 {
                units.push(c);
                i += 1;
            } else if c & 0xE0 == 0xC0 {
                let c2 = *bytes.get(i + 1).ok_or_else(malformed)? as u16;
                units.push(((c & 0x1F) << 6) | (c2 & 0x3F));
                i += 2;
            } else if c & 0xF0 == 0xE0 {
                let c2 = *bytes.get(i + 1).ok_or_else(malformed)? as u16;
                let c3 = *bytes.get(i + 2).ok_or_else(malformed)? as u16;
                units.push(((c & 0x0F) << 12) | ((c2 & 0x3F) << 6) | (c3 & 0x3F));
                i += 3;
            } else {
                return Err(malformed());
            }
        }
        Ok(String::from_utf16_lossy(&units))
    }

    fn read_number_format(&mut self) -> Result<NumberFormat, MetadataError> {
        let mut f = NumberFormat { pattern: self.read_utf()?, format: self.read_utf()?, ..NumberFormat::default() };
        for _ in 0..self.read_int()? {
            f.leading_digits_pattern.push(self.read_utf()?);
        }
        if self.read_boolean()? {
            f.national_prefix_formatting_rule = self.read_utf()?;
            f.has_national_prefix_formatting_rule = true;
        }
        if self.read_boolean()? {
            f.domestic_carrier_code_formatting_rule = self.read_utf()?;
            f.has_domestic_carrier_code_formatting_rule = true;
        }
        f.national_prefix_optional_when_formatting = self.read_boolean()?;
        Ok(f)
    }

    fn read_desc(&mut self) -> Result<Option<PhoneNumberDesc>, MetadataError> {
        if !self.read_boolean()? {
            return Ok(None);
        }
        let mut d = PhoneNumberDesc::default();
        if self.read_boolean()? {
            d.national_number_pattern = self.read_utf()?;
            d.has_national_number_pattern = true;
        }
        for _ in 0..self.read_int()? {
            d.possible_length.push(self.read_int()?);
        }
        for _ in 0..self.read_int()? {
            d.possible_length_local_only.push(self.read_int()?);
        }
        if self.read_boolean()? {
            d.example_number = self.read_utf()?;
            d.has_example_number = true;
        }
        Ok(Some(d))
    }

    fn read_optional_utf(&mut self) -> Result<(String, bool), MetadataError> {
        Ok(if self.read_boolean()? { (self.read_utf()?, true) } else { (String::new(), false) })
    }

    fn read_metadata(&mut self) -> Result<PhoneMetadata, MetadataError> {
        let mut m = PhoneMetadata {
            general_desc: self.read_desc()?,
            fixed_line: self.read_desc()?,
            mobile: self.read_desc()?,
            toll_free: self.read_desc()?,
            premium_rate: self.read_desc()?,
            shared_cost: self.read_desc()?,
            personal_number: self.read_desc()?,
            voip: self.read_desc()?,
            pager: self.read_desc()?,
            uan: self.read_desc()?,
            emergency: self.read_desc()?,
            voicemail: self.read_desc()?,
            short_code: self.read_desc()?,
            standard_rate: self.read_desc()?,
            carrier_specific: self.read_desc()?,
            sms_services: self.read_desc()?,
            no_international_dialling: self.read_desc()?,
            ..PhoneMetadata::default()
        };
        m.id = self.read_utf()?;
        m.country_code = self.read_int()?;
        m.international_prefix = self.read_utf()?;
        (m.preferred_international_prefix, m.has_preferred_international_prefix) = self.read_optional_utf()?;
        (m.national_prefix, m.has_national_prefix) = self.read_optional_utf()?;
        (m.preferred_extn_prefix, m.has_preferred_extn_prefix) = self.read_optional_utf()?;
        (m.national_prefix_for_parsing, m.has_national_prefix_for_parsing) = self.read_optional_utf()?;
        (m.national_prefix_transform_rule, m.has_national_prefix_transform_rule) = self.read_optional_utf()?;
        m.same_mobile_and_fixed_line_pattern = self.read_boolean()?;
        for _ in 0..self.read_int()? {
            m.number_format.push(self.read_number_format()?);
        }
        for _ in 0..self.read_int()? {
            m.intl_number_format.push(self.read_number_format()?);
        }
        m.main_country_for_code = self.read_boolean()?;
        (m.leading_digits, m.has_leading_digits) = self.read_optional_utf()?;
        m.mobile_number_portable_region = self.read_boolean()?;
        Ok(m)
    }
}

fn truncated() -> MetadataError {
    MetadataError("unexpected end of stream".into())
}

fn malformed() -> MetadataError {
    MetadataError("malformed input".into())
}

/// `PhoneMetadataCollection.readExternal` over a serialized metadata file.
pub fn parse_metadata_collection(bytes: &[u8]) -> Result<Vec<PhoneMetadata>, MetadataError> {
    let mut input = ObjectInput::new(bytes)?;
    let size = input.read_int()?;
    let mut out = Vec::with_capacity(size.max(0) as usize);
    for _ in 0..size {
        out.push(input.read_metadata()?);
    }
    Ok(out)
}

/// The metadata of one `PhoneNumberUtil`: region metadata keyed by region code, non-geographical metadata keyed by
/// calling code (`CompositeMetadataContainer`), and the calling-code-to-regions map.
#[derive(Debug)]
pub struct MetadataSet {
    pub(crate) regions: HashMap<String, PhoneMetadata>,
    pub(crate) non_geo: HashMap<i32, PhoneMetadata>,
    pub(crate) country_code_to_region_codes: Vec<(i32, Vec<String>)>,
}

impl MetadataSet {
    /// Builds a set from serialized files (named by their file-name suffix, `US`, `800`) and the calling-code map in
    /// the order `CountryCodeToRegionCodeMap` lists it. As `CompositeMetadataContainer`, metadata with the id `001`
    /// is keyed by calling code, the rest by id.
    pub fn from_files<'a>(
        files: impl IntoIterator<Item = (&'a str, &'a [u8])>,
        country_code_to_region_codes: Vec<(i32, Vec<String>)>,
    ) -> Result<Self, MetadataError> {
        let mut regions = HashMap::new();
        let mut non_geo = HashMap::new();
        for (_key, bytes) in files {
            for m in parse_metadata_collection(bytes)? {
                if m.id == "001" {
                    non_geo.insert(m.country_code, m);
                } else {
                    regions.insert(m.id.clone(), m);
                }
            }
        }
        Ok(MetadataSet { regions, non_geo, country_code_to_region_codes })
    }

    fn embedded(files: &'static [(&'static str, &'static [u8])], map: &'static [(i32, &'static [&'static str])]) -> Self {
        let map = map.iter().map(|(c, r)| (*c, r.iter().map(|s| s.to_string()).collect())).collect();
        MetadataSet::from_files(files.iter().copied(), map).expect("embedded metadata")
    }

    /// libphonenumber's metadata (`PhoneNumberMetadataProto_*`, `CountryCodeToRegionCodeMap`).
    pub fn default_metadata() -> &'static MetadataSet {
        static SET: OnceLock<MetadataSet> = OnceLock::new();
        SET.get_or_init(|| MetadataSet::embedded(files::DEFAULT_FILES, files::DEFAULT_MAP))
    }

    /// The metadata upstream's unit tests use (`PhoneNumberMetadataProtoForTesting_*`,
    /// `CountryCodeToRegionCodeMapForTesting`).
    pub fn testing_metadata() -> &'static MetadataSet {
        static SET: OnceLock<MetadataSet> = OnceLock::new();
        SET.get_or_init(|| MetadataSet::embedded(files::TESTING_FILES, files::TESTING_MAP))
    }

    /// The calling-code-to-regions map, in the order `CountryCodeToRegionCodeMap` lists it.
    pub fn country_code_to_region_codes(&self) -> &[(i32, Vec<String>)] {
        &self.country_code_to_region_codes
    }

    /// The metadata of a region code, if a file defines it.
    pub fn region(&self, region_code: &str) -> Option<&PhoneMetadata> {
        self.regions.get(region_code)
    }

    /// The metadata of a non-geographical calling code, if a file defines it.
    pub fn non_geographical(&self, country_calling_code: i32) -> Option<&PhoneMetadata> {
        self.non_geo.get(&country_calling_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_every_embedded_file() {
        let set = MetadataSet::default_metadata();
        assert_eq!(set.regions.len() + set.non_geo.len(), files::DEFAULT_FILES.len());
        let ee = set.region("EE").unwrap();
        assert_eq!(ee.country_code, 372);
        assert_eq!(ee.international_prefix, "00");
        assert!(ee.general_desc().national_number_pattern.starts_with("8\\d{9}|[4578]\\d{7}"));
        let us = set.region("US").unwrap();
        assert!(us.main_country_for_code);
        assert_eq!(set.non_geographical(800).unwrap().id, "001");
        assert_eq!(set.country_code_to_region_codes.iter().find(|(c, _)| *c == 1).unwrap().1[0], "US");
    }

    #[test]
    fn decodes_the_test_metadata() {
        let set = MetadataSet::testing_metadata();
        assert_eq!(set.regions.len() + set.non_geo.len(), files::TESTING_FILES.len());
        assert_eq!(set.region("GB").unwrap().country_code, 44);
    }

    #[test]
    fn rejects_a_bad_header() {
        assert_eq!(parse_metadata_collection(b"nope"), Err(MetadataError("invalid stream header".into())));
    }
}

