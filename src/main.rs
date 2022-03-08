use bstr::{ByteSlice, BStr, B};

use nom::{IResult};
use nom::bytes::complete::{take_while, is_not};
use nom::character::is_alphabetic;
use nom::character::complete::{line_ending, not_line_ending};
use nom::combinator::{not, opt};
use nom::multi::many0;

#[derive(Debug, PartialEq)]
struct RobotLines<'a> {
    pub lines: Vec<&'a [u8]>,
}

fn is_not_line_ending(c: u8) -> bool {
    c != '\n' as u8 && c != '\r' as u8
}

fn line(input: &[u8]) -> IResult<&[u8], &[u8]> {
    //is_alphabetic)(input); //
    let (input, line) = take_while(is_not_line_ending)(input)?;
    if line.is_empty() {
        return Err(nom::Err::Error(nom::error::Error{ input: input, code: nom::error::ErrorKind::Eof }));
    }
    //let (input, line) = is_not("\n")(input)?;
    println!("line: {:?}", line.as_bstr());
    let (input, end) = opt(line_ending)(input)?;
    println!("end: {:?}", end);
    Ok((input, line))
}

fn robots_txt(input: &[u8]) -> IResult<&[u8], RobotLines> {
    let (input, lines) = many0(line)(input)?;
    println!("lines: {:?}", lines);
    Ok((input, RobotLines { lines: lines }))
}

fn main() {
    let txt = "Disallow: /path
Allow: /path/exception
Crawl-delay: 7";

    println!("{}", txt);
    println!("");

    let r = robots_txt(txt.as_bytes());
    match r {
        Ok((_, rlines)) => {
            for (idx, line) in rlines.lines.iter().enumerate() {
                println!("{}: {:?}", idx, line.as_bstr());
            }
        },
        Err(_) => {},
    }
}
