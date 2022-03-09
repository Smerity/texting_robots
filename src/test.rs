use super::robots_txt;

use super::Line;
use super::Line::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let txt = "User-Agent: SmerBot
Disallow: /path
Allow: /path/exception
Crawl-delay: 60 # Very slow delay

sitemap: https://example.com/sitemap.xml";

        let lines = robots_txt(txt.as_bytes()).unwrap().1;

        let result: Vec<Line> = vec![
            UserAgent(b"SmerBot"), Disallow(b"/path"), Allow(b"/path/exception"),
            CrawlDelay(Some(60)), Raw(b""), Sitemap(b"https://example.com/sitemap.xml")
        ];

        assert_eq!(lines, result);
    }
}