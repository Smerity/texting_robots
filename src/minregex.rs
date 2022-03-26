use std::cmp::Ordering;

use lazy_static::lazy_static;
use regex::{Error, Regex, RegexBuilder};

#[derive(Debug, Clone)]
pub struct RobotRegex {
    pattern: String,
    // The regex is only constructed if the pattern contains "*" or "$"
    regex: Option<Regex>,
}

impl Ord for RobotRegex {
    fn cmp(&self, other: &Self) -> Ordering {
        // We want to reverse the ordering (i.e. longest to shortest)
        // Hence we use other.cmp(self)
        other.pattern.len().cmp(&self.pattern.len())
    }
}

impl PartialOrd for RobotRegex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for RobotRegex {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
    }
}

impl Eq for RobotRegex {}

impl RobotRegex {
    pub fn new(pattern: &str) -> Result<Self, Error> {
        // If the pattern doesn't contain "*" or "$" it's just a "starts_with" check.
        // We avoid compiling the regex as it's slow and takes space
        if !(pattern.contains("*") || pattern.contains("$")) {
            return Ok(Self { pattern: pattern.to_string(), regex: None });
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

        // Escape the pattern (except for the * and $ specific operators) for use in regular expressions
        let pat = regex::escape(&pat).replace("\\*", ".*").replace("\\$", "$");

        let rule = RegexBuilder::new(&pat)
            // Apply computation / memory limits against adversarial actors
            // This was previously 10KB but was upped to 42KB due to real domains with complex regexes
            .dfa_size_limit(42 * (1 << 10))
            .size_limit(42 * (1 << 10))
            .build()?;

        Ok(Self { pattern: pattern.to_string(), regex: Some(rule) })
    }

    pub fn is_match(&self, text: &str) -> bool {
        match &self.regex {
            Some(r) => r.is_match(text),
            None => text.starts_with(&self.pattern),
        }
    }

    // Code is used in testing to ensure expected wildcard reduction
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match &self.regex {
            Some(r) => r.as_str(),
            None => &self.pattern.as_str(),
        }
    }
}
