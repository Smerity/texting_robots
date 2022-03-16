# Texting Robots

[![Workflow Status](https://github.com/Smerity/texting_robots/workflows/ci/badge.svg)](https://github.com/Smerity/texting_robots/actions?query=ci)

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

Given the many options and potential preferences Texting Robots does not
perform caching or a HTTP GET request. These are up to the user of the library.

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

## Testing

### Unit Tests

To check Texting Robot's behaviour against the `robots.txt` specification
unit tests from [Google's C++ robots.txt parser][google-cpp] and
[Moz's reppy][moz-reppy] have been translated and included.

Certain aspects of the Google and Moz interpretation disagree with each other.
When this has occurred the author deferred to as much common sense as they
were able to muster.

[google-cpp]: https://github.com/google/robotstxt
[moz-reppy]: https://github.com/seomoz/reppy

### Integration Tests

For a number of popular domains the `robots.txt` of the given domain has been
saved and tests written against them.

### Common Crawl Test Harness

To ensure that the `robots.txt` parser will not panic in real world situations
over 54 million `robots.txt` responses were passed through Texting Robots.
While this test doesn't guarantee the `robots.txt` files were handled correctly
it does ensure the parser is unlikely to panic during practice.

For full details see [the Common Crawl testing harness][cc-test].

[cc-test]: https://github.com/Smerity/texting_robots_cc_test

### Fuzz Testing

In the local `fuzz` directory is a fuzz testing harness. The harness is not
particularly sophisticated but does utilize a low level of structure awareness
by utilizing [dictionary guided fuzzing][dgf]. The harness has already revealed
one low level unwrap panic.

[dgf]: https://llvm.org/docs/LibFuzzer.html#dictionaries

## Additional considerations

### Obtaining `robots.txt`

To obtain `robots.txt` requires performing an initial HTTP GET request to the
domain in question. When handling the HTTP status codes and how they impact `robots.txt`
the [Google suggestions are recommended][google-spec].

- 2xx (success): Attempt to process the resulting payload
- 3xx (redirection): Follow a reasonable number of steps
- 4xx (client error): Assume there are no crawl restrictions
  - 429 "Too Many Requests": Retry after a reasonable amount of time
- 5xx (server errors): Assume you should not crawl until fixed and/or interpret carefully

Even when directed to "assume no crawl restrictions" it is likely reasonable and
polite to use a small fetch delay between requests.

### Beyond the `robots.txt` specification and general suggestions

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
This is not yet done at the library level in case a larger input may be desired.

[mozilla-tmr]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/429
[mozilla-ra]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Retry-After

