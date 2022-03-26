use std::cmp::Ordering;

use lazy_static::lazy_static;
use regex::{Error, Regex, RegexBuilder};

#[derive(Debug, Clone)]
pub struct RobotRegex {
    pattern: String,
    regex: Regex,
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

#[allow(dead_code)]
impl RobotRegex {
    pub fn new(pattern: &str) -> Result<Self, Error> {
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

        Ok(Self { pattern: pattern.to_string(), regex: rule })
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

#[derive(Debug, Clone)]
pub struct MinRegex {
    pattern: String,
    regex: String,
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
        // Replace any long runs of "*" with a single "*"
        // The two regexes "x.*y" and "x.*.*y" are equivalent but not simplified by the regex parser
        // Given that rules like "x***********y" exist this prevents memory blow-up in the regex
        lazy_static! {
            static ref STARKILLER_REGEX: Regex = Regex::new(r"\*+").unwrap();
        }
        let pat = STARKILLER_REGEX.replace_all(pattern, "*");

        Ok(Self { pattern: pattern.to_string(), regex: pat.to_string() })
    }

    // Following https://github.com/seomoz/rep-cpp/blob/master/src/directive.cpp
    fn matcher(pattern: &[u8], text: &[u8], depth: usize) -> bool {
        let mut pidx = 0;
        let mut tidx = 0;

        /* eprintln!(
            "pat: {:?} {}, txt: {:?} {}, depth: {}",
            String::from_utf8_lossy(pattern),
            pattern.len(),
            String::from_utf8_lossy(text),
            text.len(),
            depth
        ); */

        while pidx < pattern.len() - 1 && tidx < text.len() - 1 {
            //eprintln!("pidx: {}, tidx: {}", pidx, tidx);
            if pattern[pidx] == b'*' {
                pidx += 1;
                let mut temp = tidx;
                while temp < text.len() - 1 {
                    //eprintln!("Matching");
                    if Self::matcher(
                        &pattern[pidx..],
                        &text[temp..],
                        depth + 1,
                    ) {
                        return true;
                    }
                    temp += 1;
                }
                // Handle the case of a leading wildcard such as:
                // - "*/text" matching "/text"
                // - "/fish*.php" matching "/fish.php"
                // ERROR / ISSUE / WARNING / PROBLEM:
                // This needs to ensure that
                if pidx < pattern.len() - 1 {
                    /* eprintln!(
                        "Leading: {:?}, {:?}",
                        &pattern[pidx..],
                        &text[tidx..]
                    ); */
                    return Self::matcher(
                        &pattern[pidx..],
                        &text[tidx..],
                        depth + 1,
                    );
                }
                return false;
            } else if pattern[pidx] == b'$' {
                // The loop would have exited if we were at the end of both pattern and text
                return false;
            } else if pattern[pidx] != text[tidx] {
                return false;
            } else {
                pidx += 1;
                tidx += 1;
            }
        }

        /* eprintln!("Exit: pidx: {}, tidx: {}", pidx, tidx);
        eprintln!("Exit: {:?}, {:?}", &pattern[pidx..], &text[tidx..]); */

        // Return true only if we've consumed all of the pattern
        if pidx == pattern.len() - 1 {
            if pattern[pidx] == b'*' {
                // If the last pattern token is a wildcard, we can match the rest of the text
                true
            } else {
                // This differs from Reppy as otherwise we end up with a false match for:
                // test_robot_rfc_example with pattern "/org/" and text "/orgo.gif"
                pattern[pidx] == text[tidx]
            }
        } else if pidx + 1 == pattern.len() - 1 && pattern[pidx + 1] == b'$' {
            pattern[pidx] == text[tidx] && tidx == text.len() - 1
        } else if pattern[pidx] == b'$' {
            // If the last token is $ ensure we are at the end of the text
            tidx == text.len() - 1
        } else if pattern.len() - 1 == pidx + 1 && pattern[pidx + 1] == b'*' {
            // We differ to Reppy's as we allow '/fish*' to match '/fish'
            true
        } else {
            false
        }
    }

    pub fn is_match(&self, text: &str) -> bool {
        /* eprintln!("\nStarting on p:{} and t:{}", self.regex, text); */
        // Special case of "Allow: " or "Disallow: "
        if self.regex.is_empty() {
            return true;
        }
        let b = Self::matcher(self.regex.as_bytes(), text.as_bytes(), 0);
        /* eprintln!("Ending on {}", b); */
        b
    }

    // Code is used in testing to ensure expected wildcard reduction
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        self.regex.as_str()
    }
}
