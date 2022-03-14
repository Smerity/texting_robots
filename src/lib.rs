/*!
Crate `texting_robots` is a library for parsing `robots.txt` files.
A key design goal of this crate is to have a thorough test suite tested
against real world data across millions of sites. While `robots.txt` is a
simple specification itself the web teases out every possible edge case.

To read more about the `robots.txt` specification a good starting point is
[How Google interprets the robots.txt specification][1].

[1]: (https://developers.google.com/search/docs/advanced/robots/robots_txt)

# Installation

Simply add a corresponding entry to your `Cargo.toml` dependency list:

```plain
[dependencies]
texting_robots = "0.1"
```

The examples in this documentation will show the rest.

# Overview of usage

This crate provides a simple high level usage through the `Robot` struct.

The `set` and `map` sub-modules contain types specific to sets and maps, such
as range queries and streams.

The `raw` module permits direct interaction with finite state transducers.
Namely, the states and transitions of a transducer can be directly accessed
with the `raw` module.

```rust
use texting_robots::Robot;

// A `robots.txt` file in String or byte format.
let txt = r"User-Agent: FerrisCrawler
Allow: /ocean
Disallow: /rust
Disallow: /forest*.py
Crawl-Delay: 10
User-Agent: *
Disallow: /
Sitemap: https://www.example.com/site.xml";

// Build the Robot for our friendly User-Agent
let r = Robot::new("FerrisCrawler", txt.as_bytes()).unwrap();

// Ferris has a crawl delay of one second per limb
// (Crabs have 10 legs so Ferris must wait 10 seconds!)
assert_eq!(r.delay, Some(10));

// We can also check which pages Ferris is allowed to crawl
// Notice we can supply the full URL or a relative path?
assert_eq!(r.allowed("https://www.rust-lang.org/ocean"), true);
assert_eq!(r.allowed("/ocean"), true);
assert_eq!(r.allowed("/ocean/reef.html"), true);
// Sadly Ferris is allowed in the ocean but not in the rust
assert_eq!(r.allowed("/rust"), false);
// Ferris is also friendly but not very good with pythons
assert_eq!(r.allowed("/forest/tree/snake.py"), false);

// The sitemap is available for any user agent
assert_eq!(r.sitemaps, vec!["https://www.example.com/site.xml"]);
```

# Additional considerations

`texting_robots` provides much of what you need for safe and respectful
crawling but is not a full solution by itself.

As an example, the HTTP error code 429 ([Too Many Requests][2]) must be
tracked when requesting pages on a given site. When a 429 is seen the crawler
should slow down, potentially setting the length of delay to the
[Retry-After][3] header if supplied by the server.

An even more complex example is that multiple domains may back on to the same
backend web server. This is a common scenario for specific products or services
as they may host thousands or millions of domains. How you consider to deploy
`Crawl-Delay` is entirely up to the end user (and potentially the service when
using HTTP error code 429 to rate limit traffic).

This library cannot guard you against all possible edge cases but it should
give you a strong starting point from which to ensure you and your code
constitute a positive addition to the internet at large.

[2]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/429
[3]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Retry-After
*/

use core::fmt;

use bstr::ByteSlice;

use nom::branch::alt;
use nom::sequence::preceded;
use nom::{IResult};
use nom::bytes::complete::{take_while, tag_no_case, tag};
use nom::character::complete::{line_ending, space0};
use nom::combinator::{opt, eof};
use nom::multi::{many_till};

use nom::lib::std::result::Result::Err;

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

use regex::{Regex, RegexBuilder};

use url::{Url, Position};

#[cfg(test)]
mod test;

fn percent_encode(input: &str) -> String {
    // Paths outside ASCII must be percent encoded
    const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
    utf8_percent_encode(input, FRAGMENT).to_string()
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum Line<'a> {
    UserAgent(&'a [u8]),
    Allow(&'a [u8]),
    Disallow(&'a [u8]),
    Sitemap(&'a [u8]),
    CrawlDelay(Option<u32>),
    Raw(&'a [u8]),
}

impl fmt::Debug for Line<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::UserAgent(ua) => f.debug_tuple("UserAgent")
                .field(&ua.as_bstr())
                .finish(),
            Line::Allow(a) => f.debug_tuple("Allow")
                .field(&a.as_bstr())
                .finish(),
                Line::Disallow(a) => f.debug_tuple("Disallow")
                .field(&a.as_bstr())
                .finish(),
            Line::CrawlDelay(c) => f.debug_tuple("CrawlDelay")
                .field(&c)
                .finish(),
            Line::Sitemap(sm) => f.debug_tuple("Sitemap")
                .field(&sm.as_bstr())
                .finish(),
            Line::Raw(r) => f.debug_tuple("Raw")
                .field(&r.as_bstr())
                .finish(),
        }
    }
}

fn is_not_line_ending(c: u8) -> bool {
    c != b'\n' && c != b'\r'
}

fn is_not_line_ending_or_comment(c: u8) -> bool {
    c != b'\n' && c != b'\r' && c != b'#'
}

fn line(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, line) = take_while(is_not_line_ending)(input)?;
    let (input, _) = opt(line_ending)(input)?;
    Ok((input, Line::Raw(line)))
}

fn statement_builder<'a>(input: &'a [u8], target: &str) -> IResult<&'a [u8], &'a [u8]> {
    let (input, _) = preceded(space0, tag_no_case(target))(input)?;
    // Note: Adding an opt(...) here would allow for "Disallow /path" to be accepted
    let (input, _) = preceded(space0, tag(":"))(input)?;
    let (input, line) = take_while(is_not_line_ending_or_comment)(input)?;
    let (input, _) = opt(preceded(tag("#"), take_while(is_not_line_ending)))(input)?;
    let (input, _) = opt(line_ending)(input)?;
    let line = line.trim();
    Ok((input, line))
}

fn user_agent(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, agent) = statement_builder(input, "user-agent")?;
    // TODO: Ensure the user agent is lowercased, perhaps zero copy by modifying in place
    Ok((input, Line::UserAgent(agent)))
}

fn allow(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, rule) = statement_builder(input, "allow")?;
    Ok((input, Line::Allow(rule)))
}

fn disallow(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, rule) = statement_builder(input, "disallow")?;
    if rule.is_empty() {
        // "Disallow:" is equivalent to allow all
        // See: https://moz.com/learn/seo/robotstxt and RFC example
        return Ok((input, Line::Allow(b"/")))
    }
    Ok((input, Line::Disallow(rule)))
}

fn sitemap(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, url) = statement_builder(input, "sitemap")?;
    Ok((input, Line::Sitemap(url)))
}

fn crawl_delay(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, time) = statement_builder(input, "crawl-delay")?;

    let time= std::str::from_utf8(time).unwrap_or("1");
    let delay = match time.parse::<u32>() {
        Ok(d) => Some(d),
        Err(_) => return Err(nom::Err::Error(nom::error::Error{ input, code: nom::error::ErrorKind::Digit })),
    };
    Ok((input, Line::CrawlDelay(delay)))
}

fn robots_txt_parse(input: &[u8]) -> IResult<&[u8], Vec<Line>> {
    // Remove BOM ("\xef\xbb\xbf", "\uFEFF") if present
    // TODO: Find a more elegant solution that shortcuts
    let (input, _) = opt(tag(b"\xef"))(input)?;
    let (input, _) = opt(tag(b"\xbb"))(input)?;
    let (input, _) = opt(tag(b"\xbf"))(input)?;
    // TODO: Google limits to 500KB of data - should that be done here?
    let (input, (lines, _)) = many_till(
        alt((user_agent, allow, disallow, sitemap, crawl_delay, line)
    ), eof)(input)?;
    Ok((input, lines))
}

#[allow(dead_code)]
pub struct Robot {
    rules: Vec<(isize, bool, Regex)>,
    pub delay: Option<u32>,
    pub sitemaps: Vec<String>,
}

impl fmt::Debug for Robot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Robot")
            .field("rules", &self.rules)
            .field("delay", &self.delay)
            .field("sitemaps", &self.sitemaps)
            .finish()
    }
}

impl<'a> Robot {
    /// Construct a new Robot object specifically processed for the given user agent.
    /// The user agent extracts all relevant rules from `robots.txt` and stores them
    /// internally. If the user agent isn't found in `robots.txt` we default to `*`.
    ///
    /// Note: The agent string is lowercased before comparison, as required by the
    /// `robots.txt` specification.
    pub fn new(agent: &str, txt: &'a [u8]) -> Result<Self, anyhow::Error> {
        // Parse robots.txt using the nom library
        let lines = match robots_txt_parse(txt.as_bytes()) {
            Ok((_, lines)) => lines,
            Err(_) => return Err(anyhow::Error::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to parse robots.txt"
            ))),
        };

        // All agents are case insensitive in `robots.txt`
        let agent = agent.to_lowercase();
        let mut agent = agent.as_str();

        // Collect all sitemaps
        // Why? "The sitemap field isn't tied to any specific user agent and may be followed by all crawlers"
        let sitemaps = lines.iter().filter_map(|x| match x {
            Line::Sitemap(url) => {
                match String::from_utf8(url.to_vec()) {
                    Ok(url) => Some(url),
                    Err(_) => None,
                }
            },
            _ => None,
        }).collect();

        // Filter out any lines that aren't User-Agent, Allow, Disallow, or CrawlDelay
        // CONFLICT: reppy's "test_robot_grouping_unknown_keys" test suggests these lines should be kept
        let lines: Vec<Line<'a>> = lines.iter()
            .filter(|x| !matches!(x, Line::Sitemap(_) | Line::Raw(_)))
            .copied().collect();
        // Minimum needed to "win" Google's `test_google_grouping` test: remove blank lines
        //let lines: Vec<Line<'a>> = lines.iter()
        //    .filter(|x| !matches!(x, Line::Raw(b"")))
        //    .copied().collect();

        // Check if our crawler is explicitly referenced, otherwise we're catch all agent ("*")
        let references_our_bot = lines.iter().any(|x| match x {
            Line::UserAgent(ua) => {
                agent.as_bytes() == ua.as_bstr().to_ascii_lowercase()
            },
            _ => false,
        });
        if !references_our_bot {
            agent = "*";
        }

        // Collect only the lines relevant to this user agent
        // If there are no User-Agent lines then we capture all
        let mut capturing = false;
        if lines.iter().filter(|x| matches!(x, Line::UserAgent(_))).count() == 0 {
            capturing = true;
        }
        let mut subset = vec![];
        let mut idx: usize = 0;
        while idx < lines.len() {
            let mut line = lines[idx];

            // User-Agents can be given in blocks with rules applicable to all User-Agents in the block
            // On a new block of User-Agents we're either in it or no longer active
            if let Line::UserAgent(_) = line {
                capturing = false;
            }
            while idx < lines.len() && matches!(line, Line::UserAgent(_)) {
                // Unreachable should never trigger as we ensure it's always a UserAgent
                let ua = match line {
                    Line::UserAgent(ua) => ua.as_bstr(),
                    _ => unreachable!(),
                };
                if agent.as_bytes() == ua.as_bstr().to_ascii_lowercase() {
                    capturing = true;
                }
                idx += 1;
                // If it's User-Agent until the end just escape to avoid potential User-Agent capture
                if idx == lines.len() { break; }
                line = lines[idx];
            }

            if capturing {
                subset.push(line);
            }
            idx += 1;
        }

        // Collect the crawl delay
        let delay = subset.iter().filter_map(|x| match x {
            Line::CrawlDelay(Some(d)) => Some(d),
            _ => None,
        }).copied().next();

        // Prepare the regex patterns for matching rules
        let mut rules = vec![];
        for line in subset.iter()
                .filter(|x| matches!(x, Line::Allow(_) | Line::Disallow(_))) {
            let (is_allowed, original) = match line {
                Line::Allow(pat) => (true, *pat),
                Line::Disallow(pat) => (false, *pat),
                _ => unreachable!(),
            };
            let pat = match original.to_str() {
                Ok(pat) => pat,
                Err(_) => continue,
            };

            // Paths outside ASCII must be percent encoded
            let pat = percent_encode(pat);

            // Escape the pattern (except for the * and $ specific operators) for use in regular expressions
            let pat = regex::escape(&pat)
                .replace("\\*", ".*").replace("\\$", "$");

            let rule = RegexBuilder::new(&pat)
                // Apply computation / memory limits against adversarial actors
                .dfa_size_limit(10 * (2 << 10)).size_limit(10 * (1 << 10))
                .build();
            let rule = match rule {
                Ok(rule) => rule,
                Err(e) => return Err(anyhow::Error::new(e)),
            };
            rules.push((original.len() as isize, is_allowed, rule));
        }

        Ok(Robot {
            rules,
            delay,
            sitemaps,
        })
    }

    fn prepare_url(raw_url: &str) -> String {
        // Try to get only the path + query of the URL
        if raw_url.is_empty() {
            return "/".to_string()
        }
        // Note: If this fails we assume the passed URL is valid
        let parsed = Url::parse(raw_url);
        let url = match parsed.as_ref() {
            // The Url library performs percent encoding
            Ok(url) => url[Position::BeforePath..].to_string(),
            Err(_) => {
                percent_encode(raw_url)
            },
        };
        url
    }

    /// Check if the given URL is allowed for the agent by `robots.txt`.
    /// This function returns true or false according to the rules in `robots.txt`.
    ///
    /// The provided URL can be absolute or relative depending on user preference.
    ///
    /// # Example
    ///
    /// ```rust
    /// use texting_robots::Robot;
    ///
    /// let r = Robot::new("Ferris", b"Disallow: /secret").unwrap();
    /// assert_eq!(r.allowed("https://example.com/secret"), false);
    /// assert_eq!(r.allowed("/secret"), false);
    /// assert_eq!(r.allowed("/everything-else"), true);
    /// ```
    pub fn allowed(&self, url: &str) -> bool {
        let url = Self::prepare_url(url);
        if url == "/robots.txt" {
            return true;
        }

        //println!("{:?} {:?}", url, self.rules);
        let mut matches: Vec<&(isize, bool, Regex)> = self.rules.iter().filter(|(_, _, rule)| {
            rule.is_match(&url)
        }).collect();

        // Sort according to the longest match
        matches.sort_by_key(|x| (-x.0, !x.1));
        //println!("{:?}", matches);

        match matches.first() {
            Some((_, is_allowed, _)) => *is_allowed,
            // If there are no rules we assume we're allowed
            None => true,
        }
    }
}