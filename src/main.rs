use core::fmt;

use bstr::{ByteSlice};

use nom::branch::alt;
use nom::sequence::preceded;
use nom::{IResult};
use nom::bytes::complete::{take_while, tag_no_case, tag};
use nom::character::complete::{line_ending, space0};
use nom::combinator::{opt, eof};
use nom::multi::{many_till};

use nom::lib::std::result::Result::Err;

#[cfg(test)]
mod test;

#[derive(PartialEq, Eq)]
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
    let (input, _) = preceded(space0, tag(":"))(input)?;
    let (input, line) = take_while(is_not_line_ending_or_comment)(input)?;
    let (input, _) = opt(preceded(tag("#"), take_while(is_not_line_ending)))(input)?;
    let (input, _) = opt(line_ending)(input)?;
    let line = line.trim();
    Ok((input, line))
}

fn user_agent(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, agent) = statement_builder(input, "user-agent")?;
    Ok((input, Line::UserAgent(agent)))
}

fn allow(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, rule) = statement_builder(input, "allow")?;
    Ok((input, Line::Allow(rule)))
}

fn disallow(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, rule) = statement_builder(input, "disallow")?;
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

fn robots_txt(input: &[u8]) -> IResult<&[u8], Vec<Line>> {
    let (input, (lines, _)) = many_till(
        alt((user_agent, allow, disallow, sitemap, crawl_delay, line)
    ), eof)(input)?;
    Ok((input, lines))
}

fn main() {
    let txt = "User-Agent: SmerBot
Disallow: /path
Allow: /path/exception
Crawl-delay: 60 # Very slow delay

sitemap: https://example.com/sitemap.xml";

    println!("Robots.txt:\n---\n{}\n", txt);

    let r = robots_txt(txt.as_bytes());
    println!("Parsed:\n---\n{:?}\n---", &r);
    if let Ok((_, rlines)) = &r {
        for (idx, line) in rlines.iter().enumerate() {
            println!("{}: {:?}", idx, line);
        }
    }

    println!("\nExpanding regex pattern:");
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
            println!("\t\t{} => {}", ex, rb.is_match(ex));
        }
    }
}
