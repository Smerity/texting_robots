use texting_robots::Robot;

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
        assert_eq!(r.delay, Some(30.0));
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
        assert!(!r.allowed("https://www.reddit.com/login"));
        // RSS is allowed
        assert!(r.allowed("https://www.reddit.com/r/rust/.rss"));
        // Sitemaps are allowed
        assert!(r.allowed("https://www.reddit.com/sitemaps/2014.xml"));
        // JSON, XML, and "?feed=" are forbidden
        assert!(!r.allowed("https://www.reddit.com/r/rust/.json"));
        assert!(!r.allowed("https://www.reddit.com/r/rust/.xml"));
        assert!(!r.allowed("https://www.reddit.com/r/rust/?feed=simd"));
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
        assert!(r.allowed(
            "https://twitter.com/search?q=%23Satoshi&src=typed_query&f=top"
        ));

        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(1.0));
        assert_eq!(r.sitemaps, vec!["https://twitter.com/sitemap.xml"]);
        assert!(!r.allowed("https://twitter.com/Smerity/following"));
        assert!(r.allowed("https://twitter.com/halvarflake"));
        // Note: They disallow any URL with a query parameter
        // Problematic as the default share URL includes query parameters
        assert!(r.allowed(
            "https://twitter.com/halvarflake/status/1501495664466927618"
        ));
        assert!(!r.allowed("https://twitter.com/halvarflake/status/1501495664466927618?s=20&t=7xv0WrBVxLVKo2OUCPn6OQ"));
        assert!(r.allowed(
            "https://twitter.com/search?q=%23Satoshi&src=typed_query&f=top"
        ));
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
        assert!(r.allowed(
            "https://www.ebay.com/b/HP-Z840-PC-Desktops-All-In-One-Computers/179/bn_89095575"
        ));
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

    // Note: robots.txt with exceptionally intensive regex-based rules
    #[test]
    fn test_real_robot_against_cnet() {
        let txt = read_file("testdata/cnet.robots.txt");

        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, None);
        // This robots.txt and URL combo has triggered "Compiled regex exceeds size limit of 10240 bytes."
        assert!(r.allowed("https://www.cnet.com/tech/mobile/homeland-security-details-new-tools-for-extracting-device-data-at-us-borders/"));
    }

    // Note: robots.txt with exceptionally intensive regex-based rules
    #[test]
    fn test_real_robot_against_ipwatchdog() {
        let txt = read_file("testdata/ipwatchdog.robots.txt");

        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        // This file starts off with a Crawl-Delay direction before any User-Agents are specified
        assert_eq!(r.delay, Some(120.0));
        assert!(!r.allowed("/2010/12/22/judge-kathleen-omalley-finally-confirmed-by-senate-for-cafc/id=13941/TEXT_IN_THE_MIDDLE_OF_THIS_%20%20http://inventivestep.net/2010/04/15/edward-dumont-nominated-to-federal-circuit/"));
    }

    // Note: robots.txt with exceptionally intensive regex-based rules
    #[test]
    fn test_real_robot_against_zillow() {
        let txt = read_file("testdata/zillow.robots.txt");

        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, None);
        // Testing against the "*/foreclosed/*" rule
        assert!(!r.allowed("/homes/sanfrancisco/cbd/foreclosed/2021-12-01/"));
        // Testing against the terrifying "/profiles/ProfileBorderTemplate,*myzillow*MyListingsTabulated.*postings*owners*OwnersProfileUpsell.*DirectLink.sdirect"
        assert!(!r.allowed("/profiles/ProfileBorderTemplate,BOB,TRIES,HARD,TO,LIKE,ROBOTS,myzillow,AND,SO,ON,MyListingsTabulated.BUT.IT.IS.HARD.postings/ETC/ETC/owners/ETC/OwnersProfileUpsell.AND.SO.ON.DirectLink.sdirect"));
    }

    // Note: robots.txt with exceptionally intensive regex-based rules
    #[test]
    fn test_real_robot_against_aviation_safety() {
        let txt = read_file("testdata/aviation-safety.net.robots.txt");

        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, None);
        // Testing against the terrifying "/database/types/Douglas-DC-3/database/*/*/*/*/*/*/*/*/*/*/*/*" rule
        // I guess they're not super aware that "/database/types/Douglas-DC-3/database/*" is equivalent..?
        // ¯\_(ツ)_/¯
        assert!(!r.allowed(
            "/database/types/Douglas-DC-3/database/a/b/c/d/e/f/g/h/i/j/k/l"
        ))
    }

    // This robots.txt contains many null bytes (\x00\x00\x00...)
    #[test]
    fn test_real_robot_against_sgppto() {
        let txt = read_file("testdata/sgppto.robots.txt");

        let r = Robot::new("SemrushBot", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(60.0));
        let r = Robot::new("SemrushBot-BA", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, None);
        assert!(r.allowed("/"));
        assert!(!r.allowed("/events/action~agenda/"));
    }
}
