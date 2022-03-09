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
        # Agent C
        # will have the same crawl delay
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
}