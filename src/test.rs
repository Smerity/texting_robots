use super::{robots_txt_parse, Error, Robot};

use super::Line;
use super::Line::*;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parser_line_elements() {
        let txt = "User-Agent: SmerBot
Disallow: /path
Allow:    /path/exception   # ONLY THIS IS ALLOWED
Crawl-delay : 60 # Very slow delay

sitemap: https://example.com/sitemap.xml";

        let lines = robots_txt_parse(txt.as_bytes()).unwrap().1;

        let result: Vec<Line> = vec![
            UserAgent(b"SmerBot"),
            Disallow(b"/path"),
            Allow(b"/path/exception"),
            CrawlDelay(Some(60.0)),
            Raw(b""),
            Sitemap(b"https://example.com/sitemap.xml"),
        ];

        assert_eq!(lines, result);
    }

    #[test]
    fn test_parser_crawl_delay() {
        // Test correct retrieval
        let good_text = "    crawl-delay  : 60";
        match robots_txt_parse(good_text.as_bytes()) {
            Ok((_, lines)) => {
                assert_eq!(lines.len(), 1);
                assert_eq!(lines[0], CrawlDelay(Some(60.0)));
            }
            Err(_) => panic!("Crawl-Delay not correctly retrieved"),
        };
        // Test float (good)
        let good_text = "    crawl-delay  : 3.14";
        match robots_txt_parse(good_text.as_bytes()) {
            Ok((_, lines)) => {
                assert_eq!(lines.len(), 1);
                assert_eq!(lines[0], CrawlDelay(Some(3.14)));
            }
            Err(_) => panic!("Crawl-Delay not correctly retrieved"),
        };
        // Test float (good)
        let good_text = "    crawl-delay  : 0.0";
        match robots_txt_parse(good_text.as_bytes()) {
            Ok((_, lines)) => {
                assert_eq!(lines.len(), 1);
                assert_eq!(lines[0], CrawlDelay(Some(0.0)));
            }
            Err(_) => panic!("Crawl-Delay not correctly retrieved"),
        };
        // Test float (bad)
        let bad_text = "    crawl-delay  : -1.618";
        match robots_txt_parse(bad_text.as_bytes()) {
            Ok((_, lines)) => {
                assert_eq!(lines.len(), 1);
                assert!(!matches!(lines[0], CrawlDelay(_)));
            }
            Err(_) => panic!("Crawl-Delay not correctly retrieved"),
        };
        // Test invalid result
        let bad_text = "Crawl-delay: wait";
        let r = robots_txt_parse(bad_text.as_bytes());
        if let Ok((_, lines)) = &r {
            assert_eq!(lines.len(), 1);
            if let Raw(_) = lines[0] {
            } else {
                panic!("Invalid Crawl-Delay not correctly handled")
            }
        }
    }

    #[test]
    fn test_robot_all_user_agents() {
        let txt = "User-agent: *
        User-agent: BobBot
        User-AGENT: SmerBot";
        let r = Robot::new("SmerBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("/index.html"));
    }

    #[test]
    fn test_robot_retrieve_crawl_delay() {
        let txt = "User-Agent: A
        Crawl-Delay: 42
        # A B and the other Agent ...
        User-Agent: B
        User-Agent: C
        Crawl-Delay: 420
        User-Agent: D
        Crawl-Delay: -1.25
        User-Agent: E
        Crawl-Delay: 8
        User-Agent: *
        CRAWL-Delay : 3600
        User-Agent: Zero
        Crawl-Delay: 0";

        let r = Robot::new("A", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(42.0));
        let r = Robot::new("B", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(420.0));
        let r = Robot::new("C", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(420.0));
        let r = Robot::new("D", txt.as_bytes()).unwrap();
        // Note: D ends up with 8 as it falls through to E's value
        // I'm not in love with this but it's in line with the spec ...
        // It's the same as what occurs with the comment between A/B/C above
        assert_eq!(r.delay, Some(8.0));
        let r = Robot::new("Zero", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(0.0));
    }

    #[test]
    fn test_robot_crawl_delay_not_integer() {
        let txt = b"User-Agent: A
        Crawl-Delay:1.0
        User-Agent: B
        Crawl-Delay:4.2
        User-Agent: C
        Crawl-Delay: \x41\xc2\xc3\xb1\x42";

        let r = Robot::new("A", txt).unwrap();
        assert_eq!(r.delay, Some(1.0));
        // We assume the (not well specified) Crawl-Delay only allows integers
        // Converting floats is both complicated and not at all well defined
        let r = Robot::new("B", txt).unwrap();
        assert_eq!(r.delay, Some(4.2));
        let r = Robot::new("C", txt).unwrap();
        assert_eq!(r.delay, None);
    }

    #[test]
    fn test_robot_crawl_evil_utf8() {
        // Example of ill-formed UTF-8 code unit sequence from:
        // http://www.unicode.org/versions/Unicode6.2.0/ch03.pdf
        let txt = b"User-Agent: A
        Allow: \x41\xc2\xc3\xb1\x42
        Disallow: \x41\xc2\xc3\xb1\x42
        SiteMap: \x41\xc2\xc3\xb1\x42
        Crawl-Delay: \x41\xc2\xc3\xb1\x42
        Disallow: /bob/";

        let r = Robot::new("A", txt).unwrap();
        assert!(!r.allowed("/bob/"));
        assert_eq!(r.delay, None);
        assert!(r.sitemaps.is_empty());
    }

    #[test]
    fn test_robot_retrieve_sitemaps() {
        let txt = "user-agent: otherbot
        disallow: /kale

        sitemap: https://example.com/sitemap.xml
        Sitemap: https://cdn.example.org/other-sitemap.xml
        siteMAP: https://ja.example.org/テスト-サイトマップ.xml";
        let sitemaps = vec![
            "https://example.com/sitemap.xml",
            "https://cdn.example.org/other-sitemap.xml",
            "https://ja.example.org/テスト-サイトマップ.xml",
        ];

        let r = Robot::new("otherbot", txt.as_bytes()).unwrap();
        assert_eq!(r.sitemaps, sitemaps);
        let r = Robot::new("blah", txt.as_bytes()).unwrap();
        assert_eq!(r.sitemaps, sitemaps);
    }

    #[test]
    fn test_robot_excessive_crawl_delay() {
        let txt = "User-Agent: Y
        Crawl-Delay: 115792089237316195423570985008687907853269984665640564039457584007913129639936";
        let r = Robot::new("Y", txt.as_bytes()).unwrap();
        // In the past this was none as the crawl delay overflow integer
        // but since we've moved to floating point it's complicated ...
        assert!(r.delay.unwrap() > 3e38);
    }

    #[test]
    fn test_robot_starts_with_crawl_delay() {
        // Some domains, such as https://www.ipwatchdog.com/robots.txt, start with a
        // Crawl-Delay directive that applies to all User-Agents.
        // We assume if your Agent doesn't have a specific Crawl-Delay then this applies.
        let txt = "Crawl-Delay: 42
        User-Agent: *
        Disallow: /blah
        User-Agent: SpecialFriend
        Allow: /
        Crawl-Delay: 1";

        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(42.0));
        let r = Robot::new("SpecialFriend", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(1.0));
    }

    #[test]
    fn test_robot_handles_random_nulls() {
        let txt = "User-Agent: *
        \x00\x00Allow: /family\x00\x00
        Disallow: /family/photos\x00\x00\x00
        Crawl-Delay: 42";

        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("/family"));
        assert!(!r.allowed("/family/photos"));
        assert_eq!(r.delay, Some(42.0));
    }

    #[test]
    fn test_robot_doesnt_do_full_regex() {
        // This is added purely as paranoia after seeing so many full regular
        // expressions written in robots.txt files!
        // This is also a good sanity test to ensure we always escape the rule
        let pat = "/(Cat|Dog).html";
        let target = "/Cat.html";
        assert!(regex::Regex::new(pat).unwrap().is_match(target));

        let txt = "User-Agent: *
        Disallow: /
        Allow: /(Cat|Dog).html";
        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert!(r.allowed(pat));
        assert!(!r.allowed(target));
    }

    #[test]
    fn test_robot_errors_on_crazy_long_line() {
        let mut txt = b"Disallow: /".to_vec();
        let ending = b"AAAAAAAAAA".to_vec();
        // 10 bytes * 100_000 = 1MB
        for _ in 0..100_000 {
            txt.extend(&ending);
        }
        // A Disallow followed by a megabyte of "A" was a real world adversarial example
        let result = Robot::new("BobBot", &txt);
        let _expected = anyhow::Error::new(Error::InvalidRobots);
        assert!(matches!(result, _expected));
    }

    #[test]
    fn test_robot_handles_end_properly() {
        let txt = "User-Agent: *
        Disallow: /
        Disallow: /*/about
        Allow: /about$";
        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("https://quora.com/about"));
        assert!(!r.allowed("/about/"));
    }

    #[test]
    fn test_robot_debug_format() {
        let txt = "User-Agent: A
        Allow: /allow/
        Disallow: /disallow/
        Crawl-Delay: 42
        SiteMap: https://example.com/sitemap.xml";

        let r = Robot::new("A", txt.as_bytes()).unwrap();
        let s = format!("{:?}", r);
        // This isn't particularly complex but a reasonable sanity test
        // The majority of the properties of the robots.txt file should be presented
        assert!(s.contains("/allow/"));
        assert!(s.contains("/disallow/"));
        assert!(s.contains("Some(42.0)"));
        assert!(s.contains("https://example.com/sitemap.xml"));
    }

    /// From Common Crawl burn test
    //

    #[test]
    fn test_robot_handle_double_return_then_newline() {
        let txt = b"\r
        User-agent: *\r\r
        Disallow: /en-AU/party\r\r\r\n\n\r\n
        User-Agent: BobBot
        Disallow: /fi-FI/party\r\r\n
        Disallow: /en-US/party\r\r\n
        \r\n\r\r\r\n\n
        Crawl-Delay: 4";

        let r = Robot::new("RandomBot", txt).unwrap();
        assert!(!r.allowed("/en-AU/party"));

        let r = Robot::new("BobBot", txt).unwrap();
        assert_eq!(r.delay, Some(4.0));
        assert!(r.allowed("/en-AU/party"));
        assert!(!r.allowed("/fi-FI/party"));
        assert!(!r.allowed("/en-US/party"));
    }

    #[test]
    fn test_robot_crazy_long_regex() {
        // Inspired by https://www.diecastlegends.com/robots.txt
        // The only sane reason that a million stars in a row make sense
        let txt = "User-agent: *
        Disallow: /basket*
        # Longest string takes priority. This is necessary due to conflicting Allow rules:
        Disallow: /*?************************************************************************************donotindex=1*";

        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/basket"));
        assert!(!r.allowed("/basket/ball"));
        assert!(r.allowed("/example/file?xyz=42"));
        assert!(!r.allowed("/example/file?xyz=42&donotindex=1"));
    }

    #[test]
    fn test_robot_many_star_rule_simplifier() {
        let txt = "Disallow: /x***y/";
        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/x/y/"));
        assert_eq!(r.rules.len(), 1);
        let (rule, _) = &r.rules[0];
        assert_eq!(rule.as_str(), "/x*y/");
    }

    #[test]
    fn test_robot_starts_with_wildcard() {
        let txt = "Disallow: *";
        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/"));
        assert!(!r.allowed("/a"));

        let txt = "Allow: *
        Disallow: *y
        Disallow: */a/*.html";
        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("/"));
        assert!(r.allowed("/b"));
        assert!(!r.allowed("bob/a/home.html"));
        assert!(!r.allowed("/gray"));
    }

    #[test]
    fn test_robot_handles_starting_position() {
        let txt = "User-agent: *
        Allow: /ocean
        Disallow: /tooth$
        Disallow: /fish*$";
        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("/ocean"));
        assert!(!r.allowed("/fish"));
        assert!(r.allowed("/shark/tooth"));
        assert!(!r.allowed("/tooth"));
        assert!(r.allowed("/toothy"));
        // Without proper starting position handling this will match the /fish rule
        assert!(r.allowed("/shark/fish"));
        assert!(!r.allowed("/fish/fins"));
        assert!(!r.allowed("/fish"));
        assert!(!r.allowed("/fishy"));
    }

    /// From fuzzer
    //

    #[test]
    fn test_fuzzed_long_regex_rule() {
        let statements: Vec<&str> = vec!["Allow:*", "Disallow:*"];
        // Note: We don't do this for Sitemap / User-Agent / Crawl-Delay
        // For the first two it'd be an allowed input and the latter is ignored
        for statement in statements {
            let mut crash: Vec<u8> =
                [statement.as_bytes(), &vec!['A' as u8; 4096]].concat();
            // Add wildcards (*) and an end match ($) to trigger full regex mode
            // Compilation doesn't fail when using the two shortcut modes
            crash.extend(b"*$");
            crash[10] = '*' as u8;
            crash[30] = '*' as u8;
            let r = Robot::new("BobBot", &crash);
            assert!(r.is_err());
        }
    }

    /// URL Tests
    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn test_url_prepare_relative() {
        for (url, path) in vec![
            ("https://example.com/foo/bar/baz.html", "/foo/bar/baz.html"),
            ("https://example.com/", "/"),
            ("https://example.com/path", "/path"),
            ("https://example.com/path?q=Linux", "/path?q=Linux"),
        ] {
            assert_eq!(Robot::prepare_url(url), path);
            assert_eq!(Robot::prepare_url(path), path);
        }
    }

    /// REPPY TESTS
    ////////////////////////////////////////////////////////////////////////////////

    // From https://github.com/seomoz/rep-cpp/issues/34
    #[test]
    fn test_reppy_handles_leading_wildcard() {
        let txt = "User-agent: *
        Disallow: */test";
        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/test"));
        assert!(!r.allowed("/test/"));
        assert!(!r.allowed("/foo/test"));
        assert!(r.allowed("/foo"));
    }

    #[test]
    fn test_reppy_no_leading_user_agent() {
        let txt = "Disallow: /path
        Allow: /path/exception
        Crawl-delay: 7";

        let r = Robot::new("agent", txt.as_bytes()).unwrap();
        assert!(r.allowed("/path/exception"));
        assert!(!r.allowed("/path"));
        assert!(r.allowed("/"));
        assert_eq!(r.delay, Some(7.0));
    }

    #[test]
    fn test_reppy_honours_default() {
        let txt = "User-agent: *
        Disallow: /tmp

        User-agent: other-agent
        Allow: /tmp";
        let r = Robot::new("agent", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/tmp"));
        assert!(r.allowed("/path"));
    }

    #[test]
    fn test_reppy_honours_specific_user_agent() {
        let txt = "User-agent: *
        Disallow: /tmp

        User-agent: agent
        Allow: /tmp";
        let r = Robot::new("agent", txt.as_bytes()).unwrap();
        assert!(r.allowed("/tmp"));
        assert!(r.allowed("/path"));
    }

    #[test]
    fn test_reppy_grouping() {
        let txt = "User-agent: one
        User-agent: two
        Disallow: /tmp";
        let r = Robot::new("one", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/tmp"));
        let r = Robot::new("two", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/tmp"));
    }

    /*
    // Disabled as it conflicts with a Google unit test
    // There's also a legitimate interpretation where disallow takes precedence
    #[test]
    fn test_reppy_grouping_unknown_keys() {
        let txt = "User-agent: *
        Disallow: /content/2/
        User-agent: *
        Noindex: /gb.html
        Noindex: /content/2/
        User-agent: ia_archiver
        Disallow: /";
        let r = Robot::new("agent", txt.as_bytes()).unwrap();
        assert!(r.allowed("/foo"));
        let r = Robot::new("ia_archiver", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/bar"));
    }
    */

    #[test]
    fn test_reppy_separates_agents() {
        let txt = "User-agent: one
        Crawl-delay: 1

        User-agent: two
        Crawl-delay: 2";
        let r = Robot::new("one", txt.as_bytes()).unwrap();
        let u = Robot::new("two", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(1.0));
        assert_eq!(u.delay, Some(2.0));
    }

    #[test]
    fn test_reppy_finds_and_exposes_sitemaps() {
        let txt = "            Sitemap: http://a.com/sitemap.xml
        Sitemap: http://b.com/sitemap.xml";
        let r = Robot::new("agent", txt.as_bytes()).unwrap();
        assert_eq!(
            r.sitemaps,
            vec!["http://a.com/sitemap.xml", "http://b.com/sitemap.xml"]
        );
    }

    #[test]
    fn test_reppy_case_insensitivity() {
        let txt = "User-agent: Agent
        Disallow: /path";
        let r = Robot::new("agent", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/path"));
        let r = Robot::new("AGeNT", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/path"));
    }

    #[test]
    fn test_reppy_empty_allows_all() {
        let r = Robot::new("agent", b"").unwrap();
        assert!(r.sitemaps.is_empty());
        assert_eq!(r.delay, None);
        assert!(r.allowed("/"));
        assert!(r.allowed("/foo"));
        assert!(r.allowed("/foo/bar"));
    }

    #[test]
    fn test_reppy_comments() {
        let txt = "User-Agent: *  # comment saying it's the default agent
        Allow: /
        Disallow: /foo";
        let r = Robot::new("agent", txt.as_bytes()).unwrap();
        assert!(r.allowed("/"));
        assert!(!r.allowed("/foo"));
        assert!(!r.allowed("/foo/bar"));
    }

    #[test]
    fn test_reppy_accepts_full_url() {
        let txt = "User-Agent: *  # comment saying it's the default agent
        Allow: /
        Disallow: /foo";
        let r = Robot::new("agent", txt.as_bytes()).unwrap();
        assert!(r.allowed("https://example.com/"));
        assert!(!r.allowed("https://example.com/foo"));
        assert!(!r.allowed("https://example.com/foo/bar"));
        assert!(r.allowed("https://example.com/found"));
    }

    /* #[test]
    fn test_reppy_skips_malformed_line() {
        // Note: This conflicts with Google as they allow "Disallow /path"
        let txt = "User-Agent: agent
        Disallow /no/colon/in/this/line";
        let r = Robot::new("agent", txt.as_bytes()).unwrap();
        assert!(r.allowed("/no/colon/in/this/line"));
    } */

    // TODO: Allow for HTTP status code consideration
    // See: Robots.fetch examples

    // Ignored reppy tests:
    // - test_utf8_bom: Google considers any line with bom as malformed

    #[test]
    fn test_robot_rfc_example() {
        let txt = "# /robots.txt for http://www.fict.org/
        # comments to webmaster@fict.org

        User-agent: unhipbot
        Disallow: /

        User-agent: webcrawler
        User-agent: excite
        Disallow:

        User-agent: *
        Disallow: /org/plans.html
        Allow: /org/
        Allow: /serv
        Allow: /~mak
        Disallow: /";

        let targets = vec![
            "/",
            "/index.html",
            "/server.html",
            "/services/fast.html",
            "/services/slow.html",
            "/orgo.gif",
            "/org/about.html",
            "/org/plans.html",
            "/%7Ejim/jim.html",
            "/~mak/mak.html",
        ];

        let r = Robot::new("unhipbot", txt.as_bytes()).unwrap();
        assert!(r.allowed("/robots.txt"));
        for t in &targets {
            assert!(!r.allowed(t));
        }

        let r = Robot::new("webcrawler", txt.as_bytes()).unwrap();
        assert!(r.allowed("/robots.txt"));
        for t in &targets {
            assert!(r.allowed(t), "Allowed failed on {}", t);
        }

        let r = Robot::new("excite", txt.as_bytes()).unwrap();
        assert!(r.allowed("/robots.txt"));
        for t in &targets {
            assert!(r.allowed(t), "Allowed failed on {}", t);
        }

        let r = Robot::new("anything", txt.as_bytes()).unwrap();
        assert!(r.allowed("/robots.txt"));
        assert!(!r.allowed("/"));
        assert!(!r.allowed("/index.html"));
        assert!(r.allowed("/server.html"));
        assert!(r.allowed("/services/fast.html"));
        assert!(r.allowed("/services/slow.html"));
        assert!(!r.allowed("/orgo.gif"));
        assert!(r.allowed("/org/about.html"));
        assert!(!r.allowed("/org/plans.html"));
        assert!(!r.allowed("/%7Ejim/jim.html"));
        assert!(r.allowed("/~mak/mak.html"));
    }

    /// TEST FORGIVENESS
    /// Inspired by Google allowing a million variations of "disallow"
    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn test_forgiveness_disallow_all_but_no_colon() {
        let text = "user-agent: FooBot
        disallow /\n";
        let r = Robot::new("FooBot", text.as_bytes()).unwrap();
        assert!(!r.allowed("/"));
        assert!(!r.allowed("/foo"));
    }

    #[test]
    fn test_forgiveness_disallow_variations() {
        let text = "user-agent: FooBot
        disallow: /a
        dissallow: /b
        dissalow: /c
        disalow: /d
        diasllow: /e
        disallaw: /f\n";
        let r = Robot::new("FooBot", text.as_bytes()).unwrap();
        for path in vec!["/a", "/b", "/c", "/d", "/e", "/f"] {
            assert!(!r.allowed(path));
        }
    }

    #[test]
    fn test_forgiveness_ensure_not_too_forgiving() {
        let text = "user-agent: FooBot
        disallow:/a
        dissallow/b
        disallow    /c\n";
        let r = Robot::new("FooBot", text.as_bytes()).unwrap();
        assert!(!r.allowed("/a"));
        assert!(r.allowed("/b"));
        assert!(!r.allowed("/c"));
    }

    #[test]
    fn test_forgiveness_sitemap_variations() {
        let text = "user-agent: FooBot
        site-map: /a
        sitemap: /b
        site map: /c\n";
        let r = Robot::new("FooBot", text.as_bytes()).unwrap();
        assert_eq!(r.sitemaps, vec!["/a", "/b", "/c"]);
    }

    #[test]
    fn test_forgiveness_crawl_delay_variations() {
        let text = "user-agent: FooBot
        crawl-delay: 42
        user-agent: BobBot
        crawl delay: 420
        user-agent: EveBot
        crawldelay: 360\n";
        let r = Robot::new("FooBot", text.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(42.0));
        let r = Robot::new("BobBot", text.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(420.0));
        let r = Robot::new("EveBot", text.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(360.0));
    }

    #[test]
    fn test_forgiveness_user_agent_variations() {
        let text = "user-agent: FooBot
        disallow: /a
        user agent: BobBot
        disallow: /b
        useragent: EveBot
        disallow: /e\n";
        let r = Robot::new("FooBot", text.as_bytes()).unwrap();
        assert!(!r.allowed("/a"));
        let r = Robot::new("BobBot", text.as_bytes()).unwrap();
        assert!(!r.allowed("/b"));
        let r = Robot::new("EveBot", text.as_bytes()).unwrap();
        assert!(!r.allowed("/e"));
    }

    /// GOOGLE TESTS
    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn test_google_foo_bar() {
        let text = "foo: FooBot
        bar: /\n";
        let r = Robot::new("FooBot", text.as_bytes()).unwrap();
        assert!(r.allowed("/"));
        assert!(r.allowed("/foo"));
    }

    #[test]
    fn test_google_allows_disallow_with_no_colon() {
        // This stands in conflict to reppy's "test_reppy_skips_malformed_line"
        let txt = "user-agent FooBot
        disallow /\n";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/"));
    }

    #[test]
    fn test_google_grouping() {
        let txt = "allow: /foo/bar/

        user-agent: FooBot
        disallow: /
        allow: /x/
        user-agent: BarBot
        disallow: /
        allow: /y/


        allow: /w/
        user-agent: BazBot

        user-agent: FooBot
        allow: /z/
        disallow: /";

        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("http://foo.bar/x/b"));
        assert!(r.allowed("http://foo.bar/z/d"));
        assert!(!r.allowed("http://foo.bar/y/c"));
        // Line outside of groupings ignored
        assert!(!r.allowed("http://foo.bar/foo/bar/"));

        let r = Robot::new("BarBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("http://foo.bar/y/c"));
        assert!(r.allowed("http://foo.bar/w/a"));
        assert!(!r.allowed("http://foo.bar/z/d"));
        // Line outside of groupings ignored
        assert!(!r.allowed("http://foo.bar/foo/bar/"));

        let r = Robot::new("BazBot", txt.as_bytes()).unwrap();
        println!("{:?}", r);
        assert!(r.allowed("http://foo.bar/z/d"));
        // Line outside of groupings ignored
        assert!(!r.allowed("http://foo.bar/foo/bar/"));
    }

    #[test]
    fn test_google_grouping_other_rules() {
        // This test stands in conflict with reppy's "test_robot_grouping_unknown_keys"
        let txt = "User-agent: BarBot
        Sitemap: https://foo.bar/sitemap
        User-agent: *
        Disallow: /";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("http://foo.bar/"));
        let r = Robot::new("BarBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("http://foo.bar/"));

        let txt = "User-agent: FooBot
        Invalid-Unknown-Line: unknown
        User-agent: *
        Disallow: /\n";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("http://foo.bar/"));
        let r = Robot::new("BarBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("http://foo.bar/"));
    }

    #[test]
    fn test_google_lines_and_bots_are_case_insensitive() {
        let txt = "USER-AGENT: FooBot
        ALLOW: /x/
        DISALLOW: /

        user-agent: BarBot
        allow: /x/
        disallow: /

        uSeR-aGeNt: BAZBOT
        AlLoW: /x/
        dIsAlLoW: /";

        for bot in vec!["FooBot", "BarBot", "BazBot"] {
            let r = Robot::new(bot, txt.as_bytes()).unwrap();
            assert!(r.allowed("http://foo.bar/x/y"));
            assert!(!r.allowed("http://foo.bar/a/b"));
        }
    }

    #[test]
    fn test_google_global_groups_secondary() {
        // Empty robots.txt is handled in a separate test

        let global = "user-agent: *
        allow: /
        user-agent: FooBot
        disallow: /";
        let r = Robot::new("FooBot", global.as_bytes()).unwrap();
        assert!(!r.allowed("http://foo.bar/x/y"));
        let r = Robot::new("BarBot", global.as_bytes()).unwrap();
        assert!(r.allowed("http://foo.bar/x/y"));

        // If not specified you may assume full permission
        let specific = "user-agent: FooBot
        allow: /
        user-agent: BarBot
        disallow: /
        user-agent: BazBot
        disallow: /";
        let r = Robot::new("QuxBot", specific.as_bytes()).unwrap();
        assert!(r.allowed("http://foo.bar/x/y"));
    }

    #[test]
    fn test_google_allow_disallow_value_case_sensitive() {
        let txt = "user-agent: FooBot
        disallow: /x/";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("http://foo.bar/x/y"));

        let txt = "user-agent: FooBot
        disallow: /X/";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("http://foo.bar/x/y"));
    }

    #[test]
    fn test_google_longest_match() {
        let txt = "user-agent: FooBot
        disallow: /x/page.html
        allow: /x/";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("http://foo.bar/x/page.html"));

        let txt = "user-agent: FooBot
        allow: /x/page.html
        disallow: /x/";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("http://foo.bar/x/page.html"));
        assert!(!r.allowed("http://foo.bar/x/"));

        let txt = "user-agent: FooBot
        disallow: 
        allow: ";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("http://foo.bar/x/page.html"));

        let txt = "user-agent: FooBot
        disallow: /
        allow: /";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("http://foo.bar/x/page.html"));

        let txt = "user-agent: FooBot
        disallow: /x
        allow: /x/";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("http://foo.bar/x"));
        assert!(r.allowed("http://foo.bar/x/"));

        let txt = "user-agent: FooBot
        disallow: /x/page.html
        allow: /x/page.html";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("http://foo.bar/x/page.html"));

        let txt = "user-agent: FooBot
        allow: /page
        disallow: /*.html";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("http://foo.bar/page.html"));
        assert!(r.allowed("http://foo.bar/page"));

        // "Longest match wins"
        let txt = "user-agent: FooBot
        allow: /x/page.
        disallow: /*.html";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("http://foo.bar/x/page.html"));
        assert!(!r.allowed("http://foo.bar/x/y.html"));

        let txt = "User-agent: *
        Disallow: /x/
        User-agent: FooBot
        Disallow: /y/";
        // Most specific group for FooBot allows implicitly /x/page
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("http://foo.bar/x/page"));
        assert!(!r.allowed("http://foo.bar/y/page"));
    }

    #[test]
    fn test_google_encoding() {
        let txt = "User-agent: FooBot
        Disallow: /
        Allow: /foo/bar?qux=taz&baz=http://foo.bar?tar&par";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(r.allowed(
            "http://foo.bar/foo/bar?qux=taz&baz=http://foo.bar?tar&par"
        ));

        let txt = "User-agent: FooBot
        Disallow: /
        Allow: /foo/bar/ツ";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("http://foo.bar/foo/bar/ツ"));
        assert!(r.allowed("http://foo.bar/foo/bar/%E3%83%84"));
        assert!(r.allowed("/foo/bar/ツ"));
        assert!(r.allowed("/foo/bar/%E3%83%84"));

        let txt = "User-agent: FooBot
        Disallow: /
        Allow: /foo/bar/%E3%83%84";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("http://foo.bar/foo/bar/%E3%83%84"));
        // Google's test says this should fail but I think that's as they don't have percent encoding in pipeline
        assert!(r.allowed("http://foo.bar/foo/bar/ツ"));

        let txt = "User-agent: FooBot
        Disallow: /
        Allow: /foo/bar/%62%61%7A";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("http://foo.bar/foo/bar/baz"));
        assert!(r.allowed("http://foo.bar/foo/bar/%62%61%7A"));
    }

    #[test]
    fn test_google_special_characters() {
        let txt = "User-agent: FooBot
        Disallow: /foo/bar/quz
        Allow: /foo/*/qux";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("http://foo.bar/foo/bar/quz"));
        assert!(r.allowed("http://foo.bar/foo/quz"));
        assert!(r.allowed("http://foo.bar/foo//quz"));
        assert!(r.allowed("http://foo.bar/foo/bax/quz"));

        let txt = "User-agent: FooBot
        Disallow: /foo/bar$
        Allow: /foo/bar/qux";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("http://foo.bar/foo/bar"));
        assert!(r.allowed("http://foo.bar/foo/bar/qux"));
        assert!(r.allowed("http://foo.bar/foo/bar/"));
        assert!(r.allowed("http://foo.bar/foo/bar/baz"));

        let txt = "User-agent: FooBot
        # Disallow: /
        Disallow: /foo/quz#qux
        Allow: /";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("http://foo.bar/foo/bar"));
        assert!(!r.allowed("http://foo.bar/foo/quz"));
    }

    #[test]
    fn test_google_documentation_checks() {
        for r in vec!["/fish", "/fish*"] {
            let txt = format!(
                "user-agent: FooBot
            disallow: /
            allow: {}",
                r
            );
            let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
            assert!(!r.allowed("http://foo.bar/bar"));
            assert!(r.allowed("http://foo.bar/fish"));
            assert!(r.allowed("http://foo.bar/fish/salmon"));
            assert!(r.allowed("http://foo.bar/fishheads"));
            assert!(r.allowed("http://foo.bar/fishheads/yummy.html"));
            assert!(r.allowed("http://foo.bar/fish.html?id=anything"));
            assert!(!r.allowed("http://foo.bar/Fish.asp"));
            assert!(!r.allowed("http://foo.bar/catfish"));
            assert!(!r.allowed("http://foo.bar/?id=fish"));
        }

        let txt = "user-agent: FooBot
        disallow: /
        allow: /fish/";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("http://foo.bar/fish/"));
        assert!(r.allowed("http://foo.bar/fish/salmon"));
        assert!(r.allowed("http://foo.bar/fish/?salmon"));
        assert!(r.allowed("http://foo.bar/fish/salmon.html"));
        assert!(r.allowed("http://foo.bar/fish/?id=anything"));

        assert!(!r.allowed("http://foo.bar/fish"));
        assert!(!r.allowed("http://foo.bar/fish.html"));
        assert!(!r.allowed("http://foo.bar/Fish/Salmon.html"));

        let txt = "user-agent: FooBot
        disallow: /
        allow: /*.php";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("http://foo.bar/bar"));
        assert!(r.allowed("http://foo.bar/filename.php"));
        assert!(r.allowed("http://foo.bar/folder/filename.php"));
        assert!(r.allowed("http://foo.bar//folder/any.php.file.html"));
        assert!(r.allowed("http://foo.bar/filename.php/"));
        assert!(r.allowed("http://foo.bar/index?f=filename.php/"));
        assert!(!r.allowed("http://foo.bar/php/"));
        assert!(!r.allowed("http://foo.bar/index?php"));
        assert!(!r.allowed("http://foo.bar/windows.PHP"));

        let txt = "user-agent: FooBot
        disallow: /
        allow: /*.php$";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("http://foo.bar/bar"));
        assert!(r.allowed("http://foo.bar/filename.php"));
        assert!(r.allowed("http://foo.bar/folder/filename.php"));
        //
        assert!(!r.allowed("http://foo.bar/filename.php?parameters"));
        assert!(!r.allowed("http://foo.bar/filename.php/"));
        assert!(!r.allowed("http://foo.bar/filename.php5"));
        assert!(!r.allowed("http://foo.bar/php/"));
        assert!(!r.allowed("http://foo.bar/filename?php"));
        assert!(!r.allowed("http://foo.bar/aaaphpaaa"));
        assert!(!r.allowed("http://foo.bar//windows.PHP"));

        let txt = "user-agent: FooBot
        disallow: /
        allow: /fish*.php";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("http://foo.bar/bar"));
        assert!(r.allowed("http://foo.bar/fish.php"));
        assert!(r.allowed("http://foo.bar/fishheads/catfish.php?parameters"));
        assert!(!r.allowed("http://foo.bar/Fish.PHP"));
    }

    #[test]
    fn test_google_order_of_precedence() {
        // From https://developers.google.com/search/docs/advanced/robots/robots_txt : "Order of precedence for rules"
        let txt = "allow: /p
        disallow: /";
        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("/page"));
        assert!(r.allowed("http://example.com/page"));

        let txt = "allow: /folder
        disallow: /folder";
        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("/folder"));
        assert!(r.allowed("http://example.com/folder/page"));

        let txt = "allow: /page
        disallow: /*.htm";
        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/page.htm"));
        assert!(!r.allowed("http://example.com/page.htm"));

        // Skipping the "page.php5" example as I don't understand / agree

        let txt = "allow: /$
        disallow: /";
        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("http://example.com/"));
        assert!(!r.allowed("http://example.com/page.htm"));
    }

    #[test]
    fn test_google_lines_correctly_counted() {
        // Skipping "\r" only line ending - assuming "\r\n" or "\n"
        for line_ending in &["\n", "\r\n"] {
            let txt = "User-Agent: foo
            Allow: /some/path
            User-Agent: bar
            
            
            Disallow: /";
            let txt = txt.replace("\n", line_ending);
            let (buffer, lines) = robots_txt_parse(txt.as_bytes()).unwrap();
            assert!(buffer.is_empty());
            assert_eq!(lines.len(), 6);
            assert_eq!(
                lines
                    .iter()
                    .filter(|x| matches!(
                        x,
                        UserAgent(_) | Allow(_) | Disallow(_)
                    ))
                    .count(),
                4
            );
        }

        // Add an extra newline at the very end
        let txt = "User-Agent: foo
        Allow: /some/path
        User-Agent: bar


        Disallow: /\n";
        let (buffer, lines) = robots_txt_parse(txt.as_bytes()).unwrap();
        assert!(buffer.is_empty());
        assert_eq!(lines.len(), 6);
        assert_eq!(
            lines
                .iter()
                .filter(|x| matches!(x, UserAgent(_) | Allow(_) | Disallow(_)))
                .count(),
            4
        );

        // Mixed \n and \r\n
        let txt = "User-Agent: foo\nAllow: /some/path\r\nUser-Agent: bar\n\r\n\nDisallow: /\n";
        let (buffer, lines) = robots_txt_parse(txt.as_bytes()).unwrap();
        assert!(buffer.is_empty());
        assert_eq!(lines.len(), 6);
        assert_eq!(
            lines
                .iter()
                .filter(|x| matches!(x, UserAgent(_) | Allow(_) | Disallow(_)))
                .count(),
            4
        );
    }

    #[test]
    fn test_google_utf8_bom_is_skipped() {
        for bom in vec![
            b"\xef\xbb\xbf".to_vec(),
            b"\xef\xbb".to_vec(),
            b"\xef".to_vec(),
        ] {
            let txt = b"User-Agent: foo\nAllow: /AnyValue\n".to_vec();
            let txt = [&bom[..], &txt[..]].concat();
            let (buffer, lines) = robots_txt_parse(&txt).unwrap();
            assert!(buffer.is_empty());
            assert_eq!(lines.len(), 2);
            assert_eq!(
                lines
                    .iter()
                    .filter(|x| matches!(
                        x,
                        UserAgent(_) | Allow(_) | Disallow(_)
                    ))
                    .count(),
                2
            );
        }

        // Broken BOM: Expect one broken line (i.e. "\x11\xbfUser-Agent")
        let txt = b"\xef\x11\xbfUser-Agent: foo\nAllow: /AnyValue\n";
        let (buffer, lines) = robots_txt_parse(txt).unwrap();
        assert!(buffer.is_empty());
        assert_eq!(
            lines,
            vec![Raw(b"\x11\xbfUser-Agent: foo"), Allow(b"/AnyValue")]
        );
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines
                .iter()
                .filter(|x| matches!(x, UserAgent(_) | Allow(_) | Disallow(_)))
                .count(),
            1
        );

        // BOM in middle of the file
        let txt = b"User-Agent: foo\n\xef\xbb\xbfAllow: /AnyValue\n";
        let (buffer, lines) = robots_txt_parse(txt).unwrap();
        assert!(buffer.is_empty());
        assert_eq!(
            lines,
            vec![UserAgent(b"foo"), Raw(b"\xef\xbb\xbfAllow: /AnyValue")]
        );
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines
                .iter()
                .filter(|x| matches!(x, UserAgent(_) | Allow(_) | Disallow(_)))
                .count(),
            1
        );
    }

    #[test]
    fn test_google_url_prepare_get_path_params_query() {
        // Note: We skip part of the test as we assume the user passed in a URL with valid http/s, not "example.com"
        for (url, path) in vec![
            ("", "/"),
            ("https://example.com", "/"),
            ("https://example.com/", "/"),
            ("http://www.example.com/a", "/a"),
            ("http://www.example.com/a/", "/a/"),
            ("http://www.example.com/a/b?c=http://d.e/", "/a/b?c=http://d.e/"),
            (
                "http://www.example.com/a/b?c=d&e=f#fragment",
                "/a/b?c=d&e=f#fragment",
            ),
        ] {
            assert_eq!(Robot::prepare_url(url), path);
            assert_eq!(Robot::prepare_url(path), path);
        }
    }

    #[test]
    fn test_google_url_prepare_escape_pattern() {
        // For the complexity of whether to normalize percent encoding (i.e. "%AA" = "%aa") see:
        // https://github.com/servo/rust-url/issues/149
        // "the algorithm specified at https://url.spec.whatwg.org/#path-state ..."
        // "leaves existing percent-encoded sequences unchanged"
        for (start, end) in vec![
            ("http://www.example.com", "/"),
            ("/a/b/c", "/a/b/c"),
            ("/á", "/%C3%A1"),
            // According the above, percent encoded remain encoded the same as before
            ("/%aa", "/%aa"),
        ] {
            assert_eq!(Robot::prepare_url(start), end);
        }
    }

    // Ignored Google test:
    // - ID_VerifyValidUserAgentsToObey ensures agents are [A-Za-z_-]
    // - Skip "GoogleOnly_AcceptUserAgentUpToFirstSpace"
    //   -(i.e. "Googlebot-Images" being "Googlebot Images" and screwing "Googlebot")
    // - Skip "GoogleOnly_IndexHTMLisDirectory" (i.e. allow "/index.html" if "/" is allowed)
    // - Skip "GoogleOnly_LineTooLong" (though something equivalent makes sense)
    // - TODO: Test the path + params conversion

    #[test]
    fn test_exporting_of_rules() {
        let txt = "User-agent: FooBot
        Disallow: /foo/bar$
        Allow: /foo/*/qux

        User-agent: BarBot
        Disallow: ";

        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert_eq!(
            r.rules().collect::<Vec<_>>(),
            vec![("/foo/bar$", false), ("/foo/*/qux", true)]
        );

        let r = Robot::new("BarBot", txt.as_bytes()).unwrap();
        assert_eq!(r.rules().collect::<Vec<_>>(), vec![("/", true)]);

        let r = Robot::new("QuxBot", txt.as_bytes()).unwrap();
        assert_eq!(r.rules().collect::<Vec<_>>(), vec![]);
    }
}
