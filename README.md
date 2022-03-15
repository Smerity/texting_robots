# Texting Robots

[![Workflow Status](https://github.com/Smerity/texting_robots/workflows/ci/badge.svg)](https://github.com/Smerity/texting_robots/actions?query=ci)

Crate `texting_robots` is a library for parsing `robots.txt` files.
A key design goal of this crate is to have a thorough test suite tested
against real world data across millions of sites. While `robots.txt` is a
simple specification itself the web teases out every possible edge case.

To read more about the `robots.txt` specification a good starting point is
[How Google interprets the robots.txt specification][1].

[1]: https://developers.google.com/search/docs/advanced/robots/robots_txt

## Installation

Soon you'll be able to install the library by adding this entry:

```plain
[dependencies]
texting_robots = "0.1"
```

to your `Cargo.toml` dependency list.

## Overview of usage

This crate provides a simple high level usage through the `Robot` struct.

The `Robot` struct is responsible for consuming the `robots.txt` file,
processing the contents, and deciding whether a given URL is allow for
your bot or not. Additional information such as your bot's crawl delay
and any sitemaps that may exist are also available.

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

## Additional considerations

`texting_robots` provides much of what you need for safe and respectful
crawling but is not a full solution by itself.

As an example, the HTTP error code 429 ([Too Many Requests][2]) must be
tracked when requesting pages on a given site. When a 429 is seen the crawler
should slow down, even if obeying the Crawl-Delay set in `robots.txt`, and
potentially using the delay set by the server's [Retry-After][3] header.

An even more complex example is that multiple domains may back on to the same
backend web server. This is a common scenario for specific products or services
that host thousands or millions of domains. How you rate limit fairly using the
`Crawl-Delay` is entirely up to the end user (and potentially the service when
using HTTP error code 429 to rate limit traffic).

This library cannot guard you against all possible edge cases but should
give you a strong starting point from which to ensure you and your code
constitute a positive addition to the internet at large.

[2]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/429
[3]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Retry-After
