use texting_robots::{Robot};

#[cfg(test)]
mod tests {
    use super::*;

    fn read_file(filename: &str) -> String {
        let f = std::fs::File::open(filename).unwrap();
        let mut r = std::io::BufReader::new(f);
        let mut txt = String::new();
        std::io::Read::read_to_string(&mut r, &mut txt).unwrap();
        txt
    }

    #[test]
    fn test_real_robot_against_hn_robots() {
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
    fn test_real_robot_against_reddit() {
        let txt = read_file("testdata/reddit.robots.txt");

        let r = Robot::new("008", txt.as_bytes()).unwrap();
        assert!(!r.allowed("https://www.reddit.com/r/rust/"));

        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert!(r.allowed("https://www.reddit.com/"));
        assert!(r.allowed("https://www.reddit.com/r/rust/"));
        assert!(r.allowed("https://www.reddit.com/posts/2020/"));
        assert!(!r.allowed(&format!("https://www.reddit.com/login")));
        // RSS is allowed
        assert!(r.allowed(&format!("https://www.reddit.com/r/rust/.rss")));
        // Sitemaps are allowed
        assert!(r.allowed(&format!("https://www.reddit.com/sitemaps/2014.xml")));
        // JSON, XML, and "?feed=" are forbidden
        assert!(!r.allowed(&format!("https://www.reddit.com/r/rust/.json")));
        assert!(!r.allowed(&format!("https://www.reddit.com/r/rust/.xml")));
        assert!(!r.allowed(&format!("https://www.reddit.com/r/rust/?feed=simd")));
    }

    #[test]
    fn test_real_robot_against_twitter() {
        let txt = read_file("testdata/twitter.robots.txt");

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

    #[test]
    fn test_real_robot_against_quora() {
        // Quora's robots.txt is large, slow, and quite restrictive
        let txt = read_file("testdata/quora.robots.txt");

        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, None);
        assert!(r.allowed("https://quora.com/"));
        assert!(r.allowed("https://quora.com/about"));
        assert!(r.allowed("https://quora.com/about/"));
        assert!(r.allowed("https://www.quora.com/about/tos"));
        assert!(r.allowed("https://www.quora.com/challenges"));
        // They allow very little
        assert!(!r.allowed("https://www.quora.com/challenging"));
        assert!(!r.allowed("https://www.quora.com/What-is-the-saddest-part-of-being-a-programmer"));
    }

    #[test]
    fn test_real_robot_against_ebay() {
        let txt = read_file("testdata/ebay.robots.txt");

        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, None);
        assert!(r.allowed("https://www.ebay.com/"));
        assert!(r.allowed("https://www.ebay.com/signin"));
        assert!(r.allowed("https://www.ebay.com/p/578453454"));
        // Note: eBay's robots.txt has weird rules with trailing commas (e.g.) "/itm/*," and "/b/*,")
        // These do not block /itm/ and /b/ however
        assert!(r.allowed("https://www.ebay.com/b/HP-Z840-PC-Desktops-All-In-One-Computers/179/bn_89095575"));
        assert!(r.allowed("https://www.ebay.com/itm/124743368051"));
        assert!(!r.allowed("https://www.ebay.com/itm/124743368051,42"));
        //
        assert!(!r.allowed("https://www.ebay.com/rewards"));
        assert!(!r.allowed("https://www.ebay.com/tickets/"));
        assert!(!r.allowed("https://www.ebay.com/today/"));
        assert!(!r.allowed("https://www.ebay.com/usr/bobby/all-follows"));
        assert!(!r.allowed("https://www.ebay.com/usr/smerity/followers"));
        assert!(!r.allowed("https://www.ebay.com/e/products?test"));
    }

    #[test]
    fn test_real_robot_against_substack() {
        let txt = read_file("testdata/substack.robots.txt");

        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, None);
        assert!(!r.allowed("https://substack.com/sign-in/"));
        assert!(!r.allowed("https://substack.com/publish"));
        assert!(!r.allowed("https://substack.com/embed"));
    }
}