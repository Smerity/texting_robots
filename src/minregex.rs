use lazy_static::lazy_static;
use regex::{Error, Regex, RegexBuilder};

#[derive(Debug, Clone)]
pub struct RobotRegex {
    regex: Regex,
}

impl RobotRegex {
    pub fn new(pattern: &str) -> Result<Self, Error> {
        // Replace any long runs of "*" with a single "*"
        // The two regexes "x.*y" and "x.*.*y" are equivalent but not simplified by the regex parser
        // Given that rules like "x***********y" exist this prevents memory blow-up in the regex
        lazy_static! {
            static ref STARKILLER_REGEX: Regex = Regex::new(r"\*+").unwrap();
        }
        let pat = STARKILLER_REGEX.replace_all(&pattern, "*");

        // Escape the pattern (except for the * and $ specific operators) for use in regular expressions
        let pat = regex::escape(&pat).replace("\\*", ".*").replace("\\$", "$");

        let rule = RegexBuilder::new(&pat)
            // Apply computation / memory limits against adversarial actors
            // This was previously 10KB but was upped to 42KB due to real domains with complex regexes
            .dfa_size_limit(42 * (1 << 10))
            .size_limit(42 * (1 << 10))
            .build()?;

        Ok(Self { regex: rule })
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }

    // Code is used in testing to ensure expected wildcard reduction
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        self.regex.as_str()
    }
}
