use core::fmt;

use bstr::{ByteSlice};

use nom::branch::alt;
use nom::{IResult};
use nom::bytes::complete::{take_while, tag_no_case};
use nom::character::complete::{line_ending};
use nom::combinator::{opt, eof};
use nom::multi::many0;

#[derive(Debug, PartialEq)]
struct RobotLines<'a> {
    pub lines: Vec<&'a [u8]>,
}

struct Allow<'a> {
    pub rule: &'a [u8],
}

struct Disallow<'a> {
    pub rule: &'a [u8],
}

struct CrawlDelay {
    pub delay: Option<u32>,
}

enum Line<'a> {
    Allow(Allow<'a>),
    Disallow(Disallow<'a>),
    CrawlDelay(CrawlDelay),
    Raw(&'a [u8]),
}

impl fmt::Debug for Line<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Allow(a) => f.debug_struct("Allow")
                .field("rule", &a.rule.as_bstr())
                .finish(),
                Line::Disallow(a) => f.debug_struct("Disallow")
                .field("rule", &a.rule.as_bstr())
                .finish(),
            Line::CrawlDelay(c) => f.debug_struct("CrawlDelay")
                .field("delay", &c.delay)
                .finish(),
            Line::Raw(r) => f.debug_struct("Raw")
                .field("text", &r.as_bstr())
                .finish(),
        }
    }
}

fn is_not_line_ending(c: u8) -> bool {
    c != b'\n' && c != b'\r'
}

fn line(input: &[u8]) -> IResult<&[u8], Line> {
    //is_alphabetic)(input); //
    let (input, line) = take_while(is_not_line_ending)(input)?;
    if line.is_empty() {
        return Err(nom::Err::Error(nom::error::Error{ input, code: nom::error::ErrorKind::Eof }));
    }
    //let (input, line) = is_not("\n")(input)?;
    println!("line: {:?}", line.as_bstr());
    let (input, _) = opt(line_ending)(input)?;
    Ok((input, Line::Raw(line)))
}

fn allow(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, _) = tag_no_case("allow: ")(input)?;
    let (input, line) = take_while(is_not_line_ending)(input)?;
    let (input, _) = opt(alt((line_ending, eof)))(input)?;
    Ok((input, Line::Allow(Allow{ rule: line })))
}

fn disallow(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, _) = tag_no_case("disallow: ")(input)?;
    let (input, line) = take_while(is_not_line_ending)(input)?;
    let (input, _) = opt(alt((line_ending, eof)))(input)?;
    Ok((input, Line::Disallow(Disallow{ rule: line })))
}

fn crawl_delay(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, _) = tag_no_case("crawl-delay: ")(input)?;
    let (input, line) = take_while(is_not_line_ending)(input)?;
    let (input, _) = opt(alt((line_ending, eof)))(input)?;

    let t= std::str::from_utf8(line).unwrap_or("1");
    let delay = match t.parse::<u32>() {
        Ok(d) => Some(d),
        Err(_) => None,
    };
    Ok((input, Line::CrawlDelay(CrawlDelay{ delay })))
}

fn robots_txt(input: &[u8]) -> IResult<&[u8], Vec<Line>> {
    let (input, lines) = many0(alt((allow, disallow, crawl_delay, line)))(input)?;
    Ok((input, lines))
}

fn main() {
    let txt = "Disallow: /path
Allow: /path/exception
Crawl-delay: 7";

    println!("{}", txt);
    println!();

    let r = robots_txt(txt.as_bytes());
    if let Ok((_, rlines)) = r {
        for (idx, line) in rlines.iter().enumerate() {
            println!("{}: {:?}", idx, line);
        }
    }
}
