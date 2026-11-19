//! `java.util.regex` semantics on the `regex` crate, for the patterns libphonenumber uses: `matches()`,
//! `lookingAt()`, `find()`, `replaceFirst`/`replaceAll` with Java's replacement syntax, and `RegexCache`.
//!
//! Metadata patterns are Java syntax restricted to what both engines read alike, except `\d`, which is ASCII in
//! Java and Unicode in `regex`; [`translate`] rewrites it (and `\D`). Neither engine backtracks differently on
//! these patterns: both pick the leftmost-first alternative, so groups and match bounds agree.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use regex::{Captures, Regex};

/// Rewrites Java-only syntax: `\d`/`\D` become ASCII classes.
pub(crate) fn translate(java: &str) -> String {
    let mut out = String::with_capacity(java.len() + 8);
    let mut chars = java.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('d') => out.push_str("[0-9]"),
            Some('D') => out.push_str("[^0-9]"),
            Some(n) => {
                out.push('\\');
                out.push(n);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// One Java `Pattern`, compiled for its three kinds of match.
#[derive(Debug)]
pub struct JPattern {
    source: String,
    find: Regex,
    looking_at: OnceLock<Regex>,
    matches: OnceLock<Regex>,
}

impl JPattern {
    /// Compiles Java pattern text (translated by [`translate`]); `flags` is a Rust inline-flag prefix such as `(?i)`.
    pub(crate) fn new(java: &str, flags: &str) -> JPattern {
        let source = format!("{flags}{}", translate(java));
        let find = Regex::new(&source).unwrap_or_else(|e| panic!("pattern {java:?}: {e}"));
        JPattern { source, find, looking_at: OnceLock::new(), matches: OnceLock::new() }
    }

    fn anchored<'a>(&self, cell: &'a OnceLock<Regex>, suffix: &str) -> &'a Regex {
        cell.get_or_init(|| Regex::new(&format!("^(?:{}){suffix}", self.source)).expect("anchored pattern"))
    }

    /// `matcher(s).matches()`.
    pub fn matches(&self, s: &str) -> bool {
        self.anchored(&self.matches, "$").is_match(s)
    }

    /// `matcher(s).lookingAt()`: the captures of the match at the start of `s`.
    pub fn looking_at<'s>(&self, s: &'s str) -> Option<Captures<'s>> {
        self.anchored(&self.looking_at, "").captures(s)
    }

    /// `matcher(s).find()`.
    pub fn find<'s>(&self, s: &'s str) -> Option<Captures<'s>> {
        self.find.captures(s)
    }

    /// `groupCount()`.
    pub fn group_count(&self) -> usize {
        self.find.captures_len() - 1
    }

    /// `matcher(s).replaceAll(replacement)`.
    pub fn replace_all(&self, s: &str, replacement: &str) -> String {
        self.replace(s, replacement, usize::MAX)
    }

    /// `matcher(s).replaceFirst(replacement)`.
    pub fn replace_first(&self, s: &str, replacement: &str) -> String {
        self.replace(s, replacement, 1)
    }

    fn replace(&self, s: &str, replacement: &str, limit: usize) -> String {
        let mut out = String::with_capacity(s.len());
        let mut last = 0;
        for caps in self.find.captures_iter(s).take(limit) {
            let m = caps.get(0).expect("group 0");
            out.push_str(&s[last..m.start()]);
            append_replacement(&mut out, &caps, replacement, self.group_count());
            last = m.end();
        }
        out.push_str(&s[last..]);
        out
    }
}

/// `Matcher.appendReplacement`'s replacement syntax: `\x` is a literal `x`; `$n` is group `n`, read digit by digit
/// while the number stays a valid group; an unmatched group appends nothing.
///
/// # Panics
/// On what Java throws for: a `$` without a group number, or a reference to a group that does not exist.
fn append_replacement(out: &mut String, caps: &Captures<'_>, replacement: &str, group_count: usize) {
    let mut chars = replacement.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => out.push(chars.next().expect("character to be escaped is missing")),
            '$' => {
                let first = chars.next().and_then(|d| d.to_digit(10)).expect("Illegal group reference");
                let mut group = first as usize;
                assert!(group <= group_count, "No group {group}");
                while let Some(d) = chars.peek().and_then(|d| d.to_digit(10)) {
                    let next = group * 10 + d as usize;
                    if next > group_count {
                        break;
                    }
                    group = next;
                    chars.next();
                }
                if let Some(m) = caps.get(group) {
                    out.push_str(m.as_str());
                }
            }
            _ => out.push(c),
        }
    }
}

/// `RegexCache`: compiled metadata patterns, shared by every caller.
#[derive(Debug, Default)]
pub(crate) struct RegexCache {
    cache: Mutex<HashMap<String, Arc<JPattern>>>,
}

impl RegexCache {
    /// `getPatternForRegex(regex)`.
    pub(crate) fn get(&self, regex: &str) -> Arc<JPattern> {
        let mut cache = self.cache.lock().expect("regex cache");
        cache.entry(regex.to_string()).or_insert_with(|| Arc::new(JPattern::new(regex, ""))).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digit_classes_are_ascii() {
        let p = JPattern::new("\\d{2}", "");
        assert!(p.matches("12"));
        assert!(!p.matches("١٢"));
        assert_eq!(translate("[\\d]\\D"), "[[0-9]][^0-9]");
    }

    #[test]
    fn replacement_reads_group_numbers_as_java_does() {
        let p = JPattern::new("(\\d)(\\d)", "");
        assert_eq!(p.replace_all("1234", "$2$1"), "2143");
        assert_eq!(p.replace_first("1234", "$10"), "1034");
        assert_eq!(p.replace_first("12", "\\$1"), "$1");
    }

    #[test]
    fn looking_at_and_matches() {
        let p = JPattern::new("0|1", "");
        assert!(p.looking_at("01").is_some());
        assert!(!p.matches("01"));
        assert!(p.looking_at("21").is_none());
    }
}
