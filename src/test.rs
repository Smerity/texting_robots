use super::{Robot, robots_txt_parse};

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
            UserAgent(b"SmerBot"), Disallow(b"/path"), Allow(b"/path/exception"),
            CrawlDelay(Some(60)), Raw(b""), Sitemap(b"https://example.com/sitemap.xml")
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
                assert_eq!(lines[0], CrawlDelay(Some(60)));
            },
            Err(_) => panic!("Crawl-Delay not correctly retrieved")
        };
        // Test invalid result
        let bad_text = "Crawl-delay: wait";
        let r = robots_txt_parse(bad_text.as_bytes());
        if let Ok((_, lines)) = &r {
            assert_eq!(lines.len(), 1);
            if let Raw(_) = lines[0] {}
            else {
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
        User-Agent: *
        CRAWL-Delay : 3600";

        let r = Robot::new("A", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(42));
        let r = Robot::new("B", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(420));
        let r = Robot::new("C", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(420));
        let r = Robot::new("D", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(3600));
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
            "https://ja.example.org/テスト-サイトマップ.xml"
        ];

        let r = Robot::new("otherbot", txt.as_bytes()).unwrap();
        assert_eq!(r.sitemaps, sitemaps);
        let r = Robot::new("blah", txt.as_bytes()).unwrap();
        assert_eq!(r.sitemaps, sitemaps);
    }

    /// From fuzzer
    //

    #[test]
    fn test_fuzzed_long_regex_rule() {
        let statements: Vec<&str> =  vec!["Allow:", "Disallow:", "Sitemap:", "Crawl-Delay:", "User-Agent:"];
        for statement in statements {
            let crash: Vec<u8> = [statement.as_bytes(), &vec!['A' as u8; 4096]].concat();
            let _r = Robot::new("BobBot", &crash);
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

    #[test]
    fn test_reppy_no_leading_user_agent() {
        let txt = "Disallow: /path
        Allow: /path/exception
        Crawl-delay: 7";

        let r = Robot::new("agent", txt.as_bytes()).unwrap();
        assert!(r.allowed("/path/exception"));
        assert!(!r.allowed("/path"));
        assert!(r.allowed("/"));
        assert_eq!(r.delay, Some(7));
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
        assert_eq!(r.delay, Some(1));
        assert_eq!(u.delay, Some(2));
    }

    #[test]
    fn test_reppy_finds_and_exposes_sitemaps() {
        let txt = "            Sitemap: http://a.com/sitemap.xml
        Sitemap: http://b.com/sitemap.xml";
        let r = Robot::new("agent", txt.as_bytes()).unwrap();
        assert_eq!(r.sitemaps, vec!["http://a.com/sitemap.xml", "http://b.com/sitemap.xml"]);
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

    #[test]
    fn test_reppy_skips_malformed_line() {
        // Note: This conflicts with Google as they allow "Disallow /path"
        let txt = "User-Agent: agent
        Disallow /no/colon/in/this/line";
        let r = Robot::new("agent", txt.as_bytes()).unwrap();
        assert!(r.allowed("/no/colon/in/this/line"));
    }

    // TODO: Allow for HTTP status code consideration
    // See: Robots.fetch examples

    // TODO: Add a way for collecting the robots.txt URL from a target URL

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

        let targets = vec!["/", "/index.html", "/server.html", "/services/fast.html",
                           "/services/slow.html", "/orgo.gif", "/org/about.html", "/org/plans.html",
                           "/%7Ejim/jim.html", "/~mak/mak.html"];

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

    /*
    #[test]
    fn test_google_allows_disallow_with_no_colon() {
        // This stands in conflict to reppy's "test_reppy_skips_malformed_line"
        let txt = "user-agent FooBot
        disallow /\n";
        let r = Robot::new("FooBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/"));
    }
    */

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
        assert!(r.allowed("http://foo.bar/foo/bar?qux=taz&baz=http://foo.bar?tar&par"));

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
            let txt = format!("user-agent: FooBot
            disallow: /
            allow: {}", r);
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
            assert_eq!(lines.iter().filter(|x| matches!(x, UserAgent(_) | Allow(_) | Disallow(_))).count(), 4);
        }

        // Add an extra newline at the very end
        let txt = "User-Agent: foo
        Allow: /some/path
        User-Agent: bar


        Disallow: /\n";
        let (buffer, lines) = robots_txt_parse(txt.as_bytes()).unwrap();
        assert!(buffer.is_empty());
        assert_eq!(lines.len(), 6);
        assert_eq!(lines.iter().filter(|x| matches!(x, UserAgent(_) | Allow(_) | Disallow(_))).count(), 4);

        // Mixed \n and \r\n
        let txt = "User-Agent: foo\nAllow: /some/path\r\nUser-Agent: bar\n\r\n\nDisallow: /\n";
        let (buffer, lines) = robots_txt_parse(txt.as_bytes()).unwrap();
        assert!(buffer.is_empty());
        assert_eq!(lines.len(), 6);
        assert_eq!(lines.iter().filter(|x| matches!(x, UserAgent(_) | Allow(_) | Disallow(_))).count(), 4);
    }

    #[test]
    fn test_google_utf8_bom_is_skipped() {
        for bom in vec![b"\xef\xbb\xbf".to_vec(), b"\xef\xbb".to_vec(), b"\xef".to_vec()] {
            let txt = b"User-Agent: foo\nAllow: /AnyValue\n".to_vec();
            let txt = [&bom[..], &txt[..]].concat();
            let (buffer, lines) = robots_txt_parse(&txt).unwrap();
            assert!(buffer.is_empty());
            assert_eq!(lines.len(), 2);
            assert_eq!(lines.iter().filter(|x| matches!(x, UserAgent(_) | Allow(_) | Disallow(_))).count(), 2);
        }

        // Broken BOM: Expect one broken line (i.e. "\x11\xbfUser-Agent")
        let txt = b"\xef\x11\xbfUser-Agent: foo\nAllow: /AnyValue\n";
        let (buffer, lines) = robots_txt_parse(txt).unwrap();
        assert!(buffer.is_empty());
        assert_eq!(lines, vec![Raw(b"\x11\xbfUser-Agent: foo"), Allow(b"/AnyValue")]);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines.iter().filter(|x| matches!(x, UserAgent(_) | Allow(_) | Disallow(_))).count(), 1);

        // BOM in middle of the file
        let txt = b"User-Agent: foo\n\xef\xbb\xbfAllow: /AnyValue\n";
        let (buffer, lines) = robots_txt_parse(txt).unwrap();
        assert!(buffer.is_empty());
        assert_eq!(lines, vec![UserAgent(b"foo"), Raw(b"\xef\xbb\xbfAllow: /AnyValue")]);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines.iter().filter(|x| matches!(x, UserAgent(_) | Allow(_) | Disallow(_))).count(), 1);
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
            ("http://www.example.com/a/b?c=d&e=f#fragment", "/a/b?c=d&e=f#fragment"),
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
}