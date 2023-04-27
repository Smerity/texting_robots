/*!
Crate `texting_robots` is a library for parsing `robots.txt` files.
A key design goal of this crate is to have a thorough test suite tested
against real world data across millions of sites. While `robots.txt` is a
simple specification itself the scale and complexity of the web teases out
every possible edge case.

To read more about the `robots.txt` specification a good starting point is
[How Google interprets the robots.txt specification][google-spec].

This library cannot guard you against all possible edge cases but should
give you a strong starting point from which to ensure you and your code
constitute a positive addition to the internet at large.

[google-spec]: https://developers.google.com/search/docs/advanced/robots/robots_txt

# Installation

You can install the library by adding this entry:

```plain
[dependencies]
texting_robots = "0.2"
```

to your `Cargo.toml` dependency list.

# Overview of usage

This crate provides a simple high level usage through the `Robot` struct.

The `Robot` struct is responsible for consuming the `robots.txt` file,
processing the contents, and deciding whether a given URL is allow for
your bot or not. Additional information such as your bot's crawl delay
and any sitemaps that may exist are also available.

Given the many options and potential preferences Texting Robots does not
perform caching or a HTTP GET request of the `robots.txt` files themselves.
This step is up to the user of the library.

```rust
use texting_robots::{Robot, get_robots_url};

// If you want to fetch a URL we'll find the URL for `robots.txt`
let url = "https://www.rust-lang.org/learn";
let robots_url = get_robots_url(url);
// Then we fetch `robots.txt` from robots_url to parse as below

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
assert_eq!(r.delay, Some(10.0));

// Any listed sitemaps are available for any user agent who finds them
assert_eq!(r.sitemaps, vec!["https://www.example.com/site.xml"]);

// We can also check which pages Ferris is allowed to crawl
// Notice we can supply the full URL or a relative path?
assert_eq!(r.allowed("https://www.rust-lang.org/ocean"), true);
assert_eq!(r.allowed("/ocean"), true);
assert_eq!(r.allowed("/ocean/reef.html"), true);
// Sadly Ferris is allowed in the ocean but not in the rust
assert_eq!(r.allowed("/rust"), false);
// Ferris is also friendly but not very good with pythons
assert_eq!(r.allowed("/forest/tree/snake.py"), false);
```

# Crawling considerations

## Obtaining `robots.txt`

To obtain `robots.txt` requires performing an initial HTTP GET request to the
domain in question. When handling the HTTP status codes and how they impact `robots.txt`
the [suggestions made by Google are recommended][google-spec].

- 2xx (success): Attempt to process the resulting payload
- 3xx (redirection): Follow a reasonable number of redirects
- 4xx (client error): Assume there are no crawl restrictions except for:
  - 429 "Too Many Requests": Retry after a reasonable amount of time
  (potentially set by the "[Retry-After][mozilla-ra]" header)
- 5xx (server errors): Assume you should not crawl until fixed and/or interpret with care

Even when directed to "assume no crawl restrictions" it is likely reasonable and
polite to use a small fetch delay between requests.

### Always set a User Agent

For crawling `robots.txt` (and especially for crawling in general) you should
include a user agent in your request. Most crawling libraries offer adding the
user agent in a single line.

```ignore
ClientBuilder.new().user_agent("FerrisCrawler/0.1 (https://ferris.rust/about-this-robot)")...
```

Beyond respecting `robots.txt` providing a good user agent provides a line of
communication between you and the web master.

## Beyond the `robots.txt` specification and general suggestions

`texting_robots` provides much of what you need for safe and respectful
crawling but is not a full solution by itself.

As an example, the HTTP error code 429 ([Too Many Requests][mozilla-tmr]) must be
tracked when requesting pages on a given site. When a 429 is seen the crawler
should slow down, even if obeying the Crawl-Delay set in `robots.txt`, and
potentially using the delay set by the server's [Retry-After][mozilla-ra] header.

An even more complex example is that multiple domains may back on to the same
backend web server. This is a common scenario for specific products or services
that host thousands or millions of domains. How you rate limit fairly using the
`Crawl-Delay` is entirely up to the end user (and potentially the service when
using HTTP error code 429 to rate limit traffic).

To protect against adverse input the user of Texting Robots is also suggested to
follow [Google's recommendations][google-spec] and limit input to 500 kibibytes.
This is not yet done at the library level in case a larger input may be desired
but may be revisited depending on community feedback.

[mozilla-tmr]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/429
[mozilla-ra]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Retry-After

## Usage of Texting Robots in other languages

While not yet specifically supporting any languages other than Rust the
library was designed to support language integrations in the future. Battle
testing this intepretation of the `robots.txt` specification against the web is
easier done testing with friends!

A C API through Rust FFI should be relatively easy to provide given Texting Robots
only relies on strings, floats, and booleans. The lack of native fetching abilities
should ensure the library is portable across platforms, situations, and languages.

A proof of concept was performed in [WASI][wasi], the "WebAssembly System Interface",
showing that the library compiles happily and only experiences a 50% or 75% speed
penalty when used with the [Wasmer][wasmer] (LLVM backend) and [Wasmtime][wasmtime]
runtimes respectively. No optimizations have been done thus far and there's likely
low hanging fruit to reap.

See `wasi_poc.sh` for details.

[wasi]: https://wasi.dev/
[wasmer]: https://wasmer.io/
[wasmtime]: https://wasmtime.dev/

# Testing

To run the majority of core tests simply execute `cargo test`.

## Unit and Integration Tests

To check Texting Robot's behaviour against the `robots.txt` specification
almost all unit tests from [Google's C++ robots.txt parser][google-cpp] and
[Moz's reppy][moz-reppy] have been translated and included.

Certain aspects of the Google and Moz interpretation disagree with each other.
When this occurred the author deferred to as much common sense as they
were able to muster.

For a number of popular domains the `robots.txt` of the given domain was
saved and tests written against them.

[google-cpp]: https://github.com/google/robotstxt
[moz-reppy]: https://github.com/seomoz/reppy

## Common Crawl Test Harness

To ensure that the `robots.txt` parser will not panic in real world situations
over 34 million `robots.txt` responses were passed through Texting Robots.
While this test doesn't guarantee the `robots.txt` files were handled correctly
it does ensure the parser is unlikely to panic during practice.

Many problematic, invalid, outrageous, and even adversarial `robots.txt`
examples were discovered in this process.

For full details see [the Common Crawl testing harness][cc-test].

[cc-test]: https://github.com/Smerity/texting_robots_cc_test

## Fuzz Testing

In the local `fuzz` directory is a fuzz testing harness. The harness is not
particularly sophisticated but does utilize a low level of structure awareness
through utilizing [dictionary guided fuzzing][dgf]. The harness has already
revealed one low level unwrap panic.

To run:

```bash
cargo fuzz run fuzz_target_1 -- -max_len=512 -dict=keywords.dict
```

Note:

- `cargo fuzz` requires nightly (i.e. run `rustup default nightly` in the `fuzz` directory)
- If you have multiple processors you may wish to add `--jobs N` after `cargo run`

[dgf]: https://llvm.org/docs/LibFuzzer.html#dictionaries

## Code Coverage with Tarpaulin

This project uses [Tarpaulin](https://github.com/xd009642/tarpaulin) to perform
code coverage reporting. Given the relatively small surface area of the parser
and Robot struct the coverage is high. Unit testing is more important for ensuring
behavioural correctness however.

To get line numbers for uncovered code run:

```bash
cargo tarpaulin --ignore-tests -v
```

*/

use core::fmt;

use bstr::ByteSlice;

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

use thiserror::Error;
use url::{ParseError, Position, Url};

mod minregex;
use minregex::MinRegex as RobotRegex;

#[cfg(test)]
mod test;

#[cfg(test)]
mod test_repcpp;

#[cfg(test)]
mod test_get_robots_url;

mod parser;
use crate::parser::{robots_txt_parse, Line};

#[derive(Error, Debug)]
pub enum Error {
    /// On any parsing error encountered parsing `robots.txt` this error will
    /// be returned.
    ///
    /// Note: Parsing errors should be rare as the parser is highly forgiving.
    #[error("Failed to parse robots.txt")]
    InvalidRobots,
}

fn percent_encode(input: &str) -> String {
    // Paths outside ASCII must be percent encoded
    const FRAGMENT: &AsciiSet =
        &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
    utf8_percent_encode(input, FRAGMENT).to_string()
}

/// Construct the URL for `robots.txt` when given a base URL from the
/// target domain.
///
/// # Errors
///
/// If there are any issues in parsing the URL, a [ParseError][pe] from the
/// [URL crate](url) will be returned.
///
/// ```rust
/// use texting_robots::get_robots_url;
///
/// let robots_url = get_robots_url("https://example.com/abc/file.html").unwrap();
/// assert_eq!(robots_url, "https://example.com/robots.txt");
/// ```
///
/// [pe]: ParseError
pub fn get_robots_url(url: &str) -> Result<String, ParseError> {
    let parsed = Url::parse(url);
    match parsed {
        Ok(mut url) => {
            if url.cannot_be_a_base() {
                return Err(ParseError::SetHostOnCannotBeABaseUrl);
            }

            if url.scheme() != "http" && url.scheme() != "https" {
                // EmptyHost isn't optimal but I'd prefer to re-use errors
                return Err(ParseError::EmptyHost);
            }

            // Setting username to "" removes the username and password
            if !url.username().is_empty() {
                url.set_username("").unwrap();
            }
            if url.password().is_some() {
                url.set_password(None).unwrap();
            }

            match url.join("/robots.txt") {
                Ok(robots_url) => Ok(robots_url.to_string()),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

#[allow(dead_code)]
pub struct Robot {
    // Rules are stored in the form of (regex rule, allow/disallow)
    // where the regex rule is ordered by original pattern length
    rules: Vec<(RobotRegex, bool)>,
    /// The delay in seconds between requests.
    /// If `Crawl-Delay` is set in `robots.txt` it will return `Some(f32)`
    /// and otherwise `None`.
    pub delay: Option<f32>,
    /// Any sitemaps found in the `robots.txt` file are added to this vector.
    /// According to the `robots.txt` specification a sitemap found in `robots.txt`
    /// is accessible and available to any bot reading `robots.txt`.
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

impl Robot {
    /// Construct a new Robot object specifically processed for the given user agent.
    /// The user agent extracts all relevant rules from `robots.txt` and stores them
    /// internally. If the user agent isn't found in `robots.txt` we default to `*`.
    ///
    /// Note: The agent string is lowercased before comparison, as required by the
    /// `robots.txt` specification.
    ///
    /// # Errors
    ///
    /// If there are difficulties parsing, which should be rare as the parser is quite
    /// forgiving, then an [InvalidRobots](Error::InvalidRobots) error is returned.
    pub fn new(agent: &str, txt: &[u8]) -> Result<Self, anyhow::Error> {
        // Replace '\x00' with '\n'
        // This shouldn't be necessary but some websites are strange ...
        let txt = txt
            .iter()
            .map(|x| if *x == 0 { b'\n' } else { *x })
            .collect::<Vec<u8>>();

        // Parse robots.txt using the nom library
        let lines = match robots_txt_parse(&txt) {
            Ok((_, lines)) => lines,
            Err(e) => {
                let err = anyhow::Error::new(Error::InvalidRobots)
                    .context(e.to_string());
                return Err(err);
            }
        };

        // All agents are case insensitive in `robots.txt`
        let agent = agent.to_lowercase();
        let mut agent = agent.as_str();

        // Collect all sitemaps
        // Why? "The sitemap field isn't tied to any specific user agent and may be followed by all crawlers"
        let sitemaps = lines
            .iter()
            .filter_map(|x| match x {
                Line::Sitemap(url) => match String::from_utf8(url.to_vec()) {
                    Ok(url) => Some(url),
                    Err(_) => None,
                },
                _ => None,
            })
            .collect();

        // Filter out any lines that aren't User-Agent, Allow, Disallow, or CrawlDelay
        // CONFLICT: reppy's "test_robot_grouping_unknown_keys" test suggests these lines should be kept
        let lines: Vec<Line> = lines
            .iter()
            .filter(|x| !matches!(x, Line::Sitemap(_) | Line::Raw(_)))
            .copied()
            .collect();

        // Check if our crawler is explicitly referenced, otherwise we're catch all agent ("*")
        let references_our_bot = lines.iter().any(|x| match x {
            Line::UserAgent(ua) => {
                agent.as_bytes() == ua.as_bstr().to_ascii_lowercase()
            }
            _ => false,
        });
        if !references_our_bot {
            agent = "*";
        }

        // Collect only the lines relevant to this user agent
        // If there are no User-Agent lines then we capture all
        let mut capturing = false;
        if lines.iter().filter(|x| matches!(x, Line::UserAgent(_))).count()
            == 0
        {
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
                if idx == lines.len() {
                    break;
                }
                line = lines[idx];
            }

            if capturing {
                subset.push(line);
            }
            idx += 1;
        }

        // Collect the crawl delay
        let mut delay = subset
            .iter()
            .filter_map(|x| match x {
                Line::CrawlDelay(Some(d)) => Some(d),
                _ => None,
            })
            .copied()
            .next();

        // Special note for crawl delay:
        // Some robots.txt files have it at the top, before any User-Agent lines, to apply to all
        if delay.is_none() {
            for line in lines.iter() {
                if let Line::CrawlDelay(Some(d)) = line {
                    delay = Some(*d);
                }
                if let Line::UserAgent(_) = line {
                    break;
                }
            }
        }

        // Prepare the regex patterns for matching rules
        let mut rules = vec![];
        for line in subset
            .iter()
            .filter(|x| matches!(x, Line::Allow(_) | Line::Disallow(_)))
        {
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

            let rule = RobotRegex::new(&pat);

            let rule = match rule {
                Ok(rule) => rule,
                Err(e) => {
                    let err = anyhow::Error::new(e)
                        .context(format!("Invalid robots.txt rule: {}", pat));
                    return Err(err);
                }
            };
            rules.push((rule, is_allowed));
        }

        Ok(Robot { rules, delay, sitemaps })
    }

    fn prepare_url(raw_url: &str) -> String {
        // Try to get only the path + query of the URL
        if raw_url.is_empty() {
            return "/".to_string();
        }
        // Note: If this fails we assume the passed URL is valid
        // i.e. We assume the user has passed us a valid relative URL
        let parsed = Url::parse(raw_url);
        let url = match parsed.as_ref() {
            // The Url library performs percent encoding
            Ok(url) => url[Position::BeforePath..].to_string(),
            Err(_) => percent_encode(raw_url),
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

        // Filter to only rules matching the URL
        let mut matches: Vec<&_> = self
            .rules
            .iter()
            .filter(|(rule, _)| rule.is_match(&url))
            .collect();

        // Sort according to the longest match and then by whether it's allowed
        // RobotRegex is sorted with preference going from longest to shortest
        // If there are two rules of equal length, allow and disallow, spec says allow
        matches.sort_by_key(|x| (&x.0, !x.1));

        match matches.first() {
            Some((_, is_allowed)) => *is_allowed,
            // If there are no rules we assume we're allowed
            None => true,
        }
    }

    /// Export the raw rules matching this agent
    ///
    /// Returns an iterator of two-element tuples with the raw pattern as found
    /// in the `robots.txt` file, and whether the pattern is allowed.
    pub fn rules(&self) -> impl Iterator<Item = (&str, bool)> + '_ {
        self.rules.iter().map(|(regex, allowed)| (regex.as_str(), *allowed))
    }
}
