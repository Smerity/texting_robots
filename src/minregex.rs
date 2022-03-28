use std::cmp::Ordering;

use bstr::ByteSlice;
use lazy_static::lazy_static;
use regex::{Error, Regex, RegexBuilder};

#[derive(Debug, Clone)]
pub struct MinRegex {
    pattern: String,
    // The regex is only constructed if the pattern contains "*" or "$"
    regex: Option<Regex>,
    starred: Option<String>,
}

impl Ord for MinRegex {
    fn cmp(&self, other: &Self) -> Ordering {
        // We want to reverse the ordering (i.e. longest to shortest)
        // Hence we use other.cmp(self)
        other.pattern.len().cmp(&self.pattern.len())
    }
}

impl PartialOrd for MinRegex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for MinRegex {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
    }
}

impl Eq for MinRegex {}

impl MinRegex {
    pub fn new(pattern: &str) -> Result<Self, Error> {
        // If the pattern doesn't contain "*" or "$" it's just a "starts_with" check.
        // We avoid compiling the regex as it's slow and takes space
        if !pattern.contains("$") && !pattern.contains("*") {
            return Ok(Self {
                pattern: pattern.to_string(),
                regex: None,
                starred: None,
            });
        }
        // TODO: We should ensure that "$" only appears at the end of the pattern
        // TODO: We could implement "$" w/o "*" using "starts_with" and "equal to".

        // Replace any long runs of "*" with a single "*"
        // The two regexes "x.*y" and "x.*.*y" are equivalent but not simplified by the regex parser
        // Given that rules like "x***********y" exist this prevents memory blow-up in the regex
        lazy_static! {
            static ref STARKILLER_REGEX: Regex = Regex::new(r"\*+").unwrap();
        }
        let pat = STARKILLER_REGEX.replace_all(pattern, "*");

        // If the pattern contains "$" we must do a proper regular expression to ensure it matches
        // Otherwise we can do a shortcut of ensuring each section is sequentially contained in the target
        // See: match_stars
        if !pattern.contains("$") {
            return Ok(Self {
                pattern: pattern.to_string(),
                regex: None,
                starred: Some(pat.to_string()),
            });
        }

        // Escape the pattern (except for the * and $ specific operators) for use in regular expressions
        let pat = regex::escape(&pat).replace("\\*", ".*").replace("\\$", "$");

        let rule = RegexBuilder::new(&pat)
            // Apply computation / memory limits against adversarial actors
            // This was previously 10KB but was upped to 42KB due to real domains with complex regexes
            .dfa_size_limit(42 * (1 << 10))
            .size_limit(42 * (1 << 10))
            .build()?;

        Ok(Self {
            pattern: pattern.to_string(),
            regex: Some(rule),
            starred: None,
        })
    }

    pub fn match_stars(&self, pattern: &[u8], text: &[u8]) -> bool {
        // Break the pattern into the parts between the "*"
        let parts = pattern.as_bytes().split(|&b| b == b'*');

        let mut starting_point = 0;

        for (idx, part) in parts.enumerate() {
            if idx == 0 && !text.is_empty() && text[0] != b'*' {
                // The first part is special if it doesn't start with a '*'
                // This must match at the very start
                if !text.starts_with(part) {
                    return false;
                }
                starting_point += part.len();
                continue;
            }

            match text[starting_point..].find(part) {
                Some(idx) => {
                    starting_point += idx + part.len();
                }
                None => return false,
            }
        }

        true
    }

    pub fn is_match(&self, text: &str) -> bool {
        match &self.regex {
            Some(r) => r.is_match(text),
            None => match &self.starred {
                Some(p) => self.match_stars(p.as_bytes(), text.as_bytes()),
                None => text.starts_with(&self.pattern),
            },
        }
    }

    // Code is used in testing to ensure expected wildcard reduction
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match &self.regex {
            Some(r) => r.as_str(),
            None => match &self.starred {
                Some(p) => p.as_str(),
                None => self.pattern.as_str(),
            },
        }
    }
}
