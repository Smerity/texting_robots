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

    #[test]
    fn test_conflicting_patterns() {
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

        let txt = "allow: /page
        disallow: /*.htm";
        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/page.htm"));
        assert!(!r.allowed("http://lotr.com/page.htm"));

        // Skipping the "page.php5" example as I don't understand / agree

        let txt = "allow: /$
        disallow: /";
        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert!(!r.allowed("http://example.com/page.htm"));
    }

    #[test]
    fn test_robot_against_hn_robots() {
        let txt = "User-Agent: *
        Disallow: /x?
        Disallow: /r?
        Disallow: /vote?
        Disallow: /reply?
        Disallow: /submitted?
        Disallow: /submitlink?
        Disallow: /threads?
        Crawl-delay: 30";
        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(30));
        assert!(r.allowed("https://news.ycombinator.com/item?id=30611367"));
        assert!(!r.allowed("https://news.ycombinator.com/threads?id=Smerity"));
        assert!(r.allowed("https://news.ycombinator.com/user?id=Smerity"));
    }

    #[test]
    fn test_robot_against_twitter() {
        let f = std::fs::File::open("testdata/twitter.robots.txt").unwrap();
        let mut r = std::io::BufReader::new(f);
        let mut txt = String::new();
        std::io::Read::read_to_string(&mut r, &mut txt).unwrap();

        let r = Robot::new("GOOGLEBOT", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, None);
        assert!(!r.allowed("https://twitter.com/Smerity/following"));
        assert!(r.allowed("https://twitter.com/halvarflake"));
        assert!(!r.allowed("https://twitter.com/search?q=%22Satoshi%20Nakamoto%22&src=trend_click"));
        // They allow hash tag search specifically for some reason..?
        assert!(r.allowed("https://twitter.com/search?q=%23Satoshi&src=typed_query&f=top"));

        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(1));
        assert_eq!(r.sitemaps, vec!["https://twitter.com/sitemap.xml"]);
        assert!(!r.allowed("https://twitter.com/Smerity/following"));
        assert!(r.allowed("https://twitter.com/halvarflake"));
        // Note: They disallow any URL with a query parameter
        // Problematic as the default share URL includes query parameters
        assert!(r.allowed("https://twitter.com/halvarflake/status/1501495664466927618"));
        assert!(!r.allowed("https://twitter.com/halvarflake/status/1501495664466927618?s=20&t=7xv0WrBVxLVKo2OUCPn6OQ"));
        assert!(r.allowed("https://twitter.com/search?q=%23Satoshi&src=typed_query&f=top"));
        assert!(!r.allowed("/oauth"));
    }

    // From reppy tests
    #[test]
    fn test_robot_no_leading_user_agent() {
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
    fn test_robot_honours_default() {
        let txt = "User-agent: *
        Disallow: /tmp

        User-agent: other-agent
        Allow: /tmp";
        let r = Robot::new("agent", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/tmp"));
        assert!(r.allowed("/path"));
    }

    #[test]
    fn test_robot_honours_specific_user_agent() {
        let txt = "User-agent: *
        Disallow: /tmp

        User-agent: agent
        Allow: /tmp";
        let r = Robot::new("agent", txt.as_bytes()).unwrap();
        assert!(r.allowed("/tmp"));
        assert!(r.allowed("/path"));
    }

    #[test]
    fn test_robot_grouping() {
        let txt = "User-agent: one
        User-agent: two
        Disallow: /tmp";
        let r = Robot::new("one", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/tmp"));
        let r = Robot::new("two", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/tmp"));
    }

    #[test]
    fn test_robot_grouping_unknown_keys() {
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

    #[test]
    fn test_robot_separates_agents() {
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
    fn test_robot_finds_and_exposes_sitemaps() {
        let txt = "            Sitemap: http://a.com/sitemap.xml
        Sitemap: http://b.com/sitemap.xml";
        let r = Robot::new("agent", txt.as_bytes()).unwrap();
        assert_eq!(r.sitemaps, vec!["http://a.com/sitemap.xml", "http://b.com/sitemap.xml"]);
    }

    #[test]
    fn test_robot_case_insensitivity() {
        let txt = "User-agent: Agent
        Disallow: /path";
        let r = Robot::new("agent", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/path"));
        let r = Robot::new("AGeNT", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/path"));
    }
}