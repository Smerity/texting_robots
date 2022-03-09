use super::robots_txt;

use super::Line;
use super::Line::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robots_txt_broken_into_lines() {
        let txt = "User-Agent: SmerBot
Disallow: /path
Allow:    /path/exception   # ONLY THIS IS ALLOWED
Crawl-delay: 60 # Very slow delay

sitemap: https://example.com/sitemap.xml";

        let lines = robots_txt(txt.as_bytes()).unwrap().1;

        let result: Vec<Line> = vec![
            UserAgent(b"SmerBot"), Disallow(b"/path"), Allow(b"/path/exception"),
            CrawlDelay(Some(60)), Raw(b""), Sitemap(b"https://example.com/sitemap.xml")
        ];

        assert_eq!(lines, result);
    }

    #[test]
    fn test_crawl_delay() {
        // Test correct retrieval
        let good_text = "    crawl-delay  : 60";
        match robots_txt(good_text.as_bytes()) {
            Ok((_, lines)) => {
                assert_eq!(lines.len(), 1);
                assert_eq!(lines[0], CrawlDelay(Some(60)));
            },
            Err(_) => panic!("Crawl-Delay not correctly retrieved")
        };
        // Test invalid result
        let bad_text = "Crawl-delay: wait";
        let r = robots_txt(bad_text.as_bytes());
        if let Ok((_, lines)) = &r {
            assert_eq!(lines.len(), 1);
            if let Raw(_) = lines[0] {}
            else {
                panic!("Invalid Crawl-Delay not correctly handled")
            }
        }
    }
}