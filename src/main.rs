use core::fmt;

use bstr::{BStr, ByteSlice};

use nom::branch::alt;
use nom::sequence::preceded;
use nom::{IResult};
use nom::bytes::complete::{take_while, tag_no_case, tag};
use nom::character::complete::{line_ending, space0};
use nom::combinator::{opt, eof};
use nom::multi::{many_till};

use nom::lib::std::result::Result::Err;

use regex::Regex;

use url::{Url, Position};

#[cfg(test)]
mod test;

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
    // TODO: Google limits to 500KB of data - should that be done here?
    let (input, (lines, _)) = many_till(
        alt((user_agent, allow, disallow, sitemap, crawl_delay, line)
    ), eof)(input)?;
    Ok((input, lines))
}

#[allow(dead_code)]
struct Robot<'a> {
    txt: &'a [u8],
    lines: Vec<Line<'a>>,
    subset: Vec<Line<'a>>,
    rules: Vec<(isize, bool, Regex)>,
    delay: Option<u32>,
    sitemaps: Vec<&'a BStr>,
}

impl fmt::Debug for Robot<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Robot")
            //.field("txt", &self.txt.as_bstr())
            //.field("lines", &self.lines)
            .field("subset", &self.subset)
            .field("rules", &self.rules)
            .field("delay", &self.delay)
            .field("sitemaps", &self.sitemaps)
            .finish()
    }
}

impl<'a> Robot<'a> {
    fn new(agent: &str, txt: &'a [u8]) -> Result<Self, &'static str> {
        // Parse robots.txt
        let lines = match robots_txt_parse(txt.as_bytes()) {
            Ok((_, lines)) => lines,
            Err(_) => return Err("Failed to parse robots.txt"),
        };
        //for (idx, line) in lines.iter().enumerate() { println!("{:02}: {:?}", idx, line); }

        let agent = agent.to_ascii_lowercase();
        let mut agent = agent.as_str();

        // Collect all sitemaps
        // Why? "The sitemap field isn't tied to any specific user agent and may be followed by all crawlers"
        let sitemaps = lines.iter().filter_map(|x| match x {
            Line::Sitemap(url) => Some((*url).as_bstr()),
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
            let pat = regex::escape(pat)
                .replace("\\*", ".*").replace("\\$", "$");

            let rule = regex::RegexBuilder::new(&pat)
                // Apply computation / memory limits against adversarial actors
                .dfa_size_limit(10 * (2 << 10)).size_limit(10 * (1 << 10))
                .build().unwrap();
            rules.push((original.len() as isize, is_allowed, rule));
        }

        Ok(Robot {
            txt,
            lines,
            subset,
            rules,
            delay,
            sitemaps,
        })
    }

    fn allowed(&self, raw_url: &str) -> bool {
        // Try to get only the path + query of the URL
        // Note: If this fails we assume the passed URL is valid
        let parsed = Url::parse(raw_url);
        let url = match parsed.as_ref() {
            Ok(url) => &url[Position::BeforePath..],
            // TODO: How do we handle this cleanly..?
            //Err(_) => return Err("Failed to parse URL"),
            Err(_) => raw_url,
        };
        if url == "/robots.txt" {
            return true;
        }

        let mut matches: Vec<&(isize, bool, Regex)> = self.rules.iter().filter(|(_, _, rule)| {
            rule.is_match(url)
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

fn main() {
    let txt = "User-Agent: SmerBot
Disallow: /path
Allow: /path/exception
Crawl-delay: 60 # Very slow delay
User-Agent: *
CRAWL-DELAY: 3600
Disallow: /
User-Agent: SmerBot
User-Agent: BobBot
Allow: /secrets/

sitemap: https://example.com/sitemap.xml";

    println!("Robots.txt:\n---\n{}\n", txt);

    let r = Robot::new("SmerBot", txt.as_bytes());
    println!("\n---\n{:?}\n---\n", &r);

    println!("Expanding regex pattern:");
    for (pat, examples) in vec![
            ("/fish", vec!["/fish", "/fish.html", "/fish/salmon.html", "/fish.php?id=anything", "/Fish.asp", "/catfish"]),
            ("/*.php$", vec!["/filename.php", "/folder/filename.php", "/filename.php?parameters", "/filename.php/"]),
            ("/fish*.php", vec!["/fish.php", "/fishheads/catfish.php?parameters", "/Fish.PHP"]),
        ] {
        let mut s = regex::escape(pat);
        s = s.replace("\\*", ".*").replace("\\$", "$");
        println!("\t- {} => {}", pat, s);
        let rb = regex::RegexBuilder::new(&s)
            .dfa_size_limit(10 * (2 << 10)).size_limit(10 * (1 << 10))
            .build().unwrap();
        for ex in examples {
            println!("\t\t{} => {:?}", ex, rb.find(ex));
        }
    }
}
