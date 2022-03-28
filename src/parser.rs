use core::fmt;

use bstr::ByteSlice;

use nom::branch::{alt, Alt};
use nom::bytes::complete::{tag, tag_no_case, take_while};
use nom::character::complete::{space0, space1};
use nom::combinator::{eof, opt};
use nom::error::ParseError as NomParseError;
use nom::multi::many_till;
use nom::sequence::preceded;
use nom::IResult;

#[derive(PartialEq, Copy, Clone)]
pub enum Line<'a> {
    UserAgent(&'a [u8]),
    Allow(&'a [u8]),
    Disallow(&'a [u8]),
    Sitemap(&'a [u8]),
    CrawlDelay(Option<f32>),
    Raw(&'a [u8]),
}

#[cfg(not(tarpaulin_include))]
impl fmt::Debug for Line<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::UserAgent(ua) => {
                f.debug_tuple("UserAgent").field(&ua.as_bstr()).finish()
            }
            Line::Allow(a) => {
                f.debug_tuple("Allow").field(&a.as_bstr()).finish()
            }
            Line::Disallow(a) => {
                f.debug_tuple("Disallow").field(&a.as_bstr()).finish()
            }
            Line::CrawlDelay(c) => {
                f.debug_tuple("CrawlDelay").field(&c).finish()
            }
            Line::Sitemap(sm) => {
                f.debug_tuple("Sitemap").field(&sm.as_bstr()).finish()
            }
            Line::Raw(r) => f.debug_tuple("Raw").field(&r.as_bstr()).finish(),
        }
    }
}

fn is_not_line_ending(c: u8) -> bool {
    c != b'\n' && c != b'\r'
}

fn is_not_line_ending_or_comment(c: u8) -> bool {
    c != b'\n' && c != b'\r' && c != b'#'
}

fn is_carriage_return(c: u8) -> bool {
    c == b'\r'
}

fn consume_newline(input: &[u8]) -> IResult<&[u8], Option<&[u8]>> {
    let (input, _) = take_while(is_carriage_return)(input)?;
    let (input, output) = opt(tag(b"\n"))(input)?;
    Ok((input, output))
}

fn line(input: &[u8]) -> IResult<&[u8], Line> {
    let (input, line) = take_while(is_not_line_ending)(input)?;
    let (input, _) = consume_newline(input)?;
    Ok((input, Line::Raw(line)))
}

fn many_statement_builder<
    'a,
    O,
    E: NomParseError<&'a [u8]>,
    List: Alt<&'a [u8], O, E>,
>(
    input: &'a [u8],
    targets: List,
) -> IResult<&'a [u8], &'a [u8]>
where
    nom::Err<nom::error::Error<&'a [u8]>>: From<nom::Err<E>>,
{
    let (input, _) = preceded(space0, alt(targets))(input)?;
    // This accepts a colon with spaces ("Disallow: /a") or one or more spaces ("Disallow /")
    let (input, _) = alt((preceded(space0, tag(":")), space1))(input)?;
    let (input, line) = take_while(is_not_line_ending_or_comment)(input)?;
    let (input, _) =
        opt(preceded(tag("#"), take_while(is_not_line_ending)))(input)?;
    let (input, _) = consume_newline(input)?;
    let line = line.trim();
    Ok((input, line))
}

fn user_agent(input: &[u8]) -> IResult<&[u8], Line> {
    let matcher = (
        tag_no_case("user-agent"),
        tag_no_case("user agent"),
        tag_no_case("useragent"),
    );
    let (input, agent) = many_statement_builder(input, matcher)?;
    Ok((input, Line::UserAgent(agent)))
}

fn allow(input: &[u8]) -> IResult<&[u8], Line> {
    let matcher = (tag_no_case("allow"),);
    let (input, rule) = many_statement_builder(input, matcher)?;
    Ok((input, Line::Allow(rule)))
}

fn disallow(input: &[u8]) -> IResult<&[u8], Line> {
    let matcher = (
        tag_no_case("disallow"),
        tag_no_case("dissallow"),
        tag_no_case("dissalow"),
        tag_no_case("disalow"),
        tag_no_case("diasllow"),
        tag_no_case("disallaw"),
    );
    let (input, rule) = many_statement_builder(input, matcher)?;
    if rule.is_empty() {
        // "Disallow:" is equivalent to allow all
        // See: https://moz.com/learn/seo/robotstxt and RFC example
        return Ok((input, Line::Allow(b"/")));
    }
    Ok((input, Line::Disallow(rule)))
}

fn sitemap(input: &[u8]) -> IResult<&[u8], Line> {
    let matcher = (
        tag_no_case("sitemap"),
        tag_no_case("site-map"),
        tag_no_case("site map"),
    );
    let (input, url) = many_statement_builder(input, matcher)?;
    Ok((input, Line::Sitemap(url)))
}

fn crawl_delay(input: &[u8]) -> IResult<&[u8], Line> {
    let matcher = (
        tag_no_case("crawl-delay"),
        tag_no_case("crawl delay"),
        tag_no_case("crawldelay"),
    );
    let (input, time) = many_statement_builder(input, matcher)?;

    let time = match std::str::from_utf8(time) {
        Ok(time) => time,
        Err(_) => {
            return Err(nom::Err::Error(nom::error::Error {
                input,
                code: nom::error::ErrorKind::Fail,
            }))
        }
    };
    let delay = match time.parse::<f32>() {
        Ok(d) if d >= 0.0 => Some(d),
        Ok(_) | Err(_) => {
            return Err(nom::Err::Error(nom::error::Error {
                input,
                code: nom::error::ErrorKind::Digit,
            }))
        }
    };
    Ok((input, Line::CrawlDelay(delay)))
}

pub fn robots_txt_parse(input: &[u8]) -> IResult<&[u8], Vec<Line>> {
    // Remove BOM ("\xef\xbb\xbf", "\uFEFF") if present
    // TODO: Find a more elegant solution that shortcuts
    let (input, _) = opt(tag(b"\xef"))(input)?;
    let (input, _) = opt(tag(b"\xbb"))(input)?;
    let (input, _) = opt(tag(b"\xbf"))(input)?;
    // TODO: Google limits to 500KB of data - should that be done here?
    let matcher =
        alt((user_agent, allow, disallow, sitemap, crawl_delay, line));
    let (input, (lines, _)) = many_till(matcher, eof)(input)?;
    Ok((input, lines))
}
