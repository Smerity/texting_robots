use super::Robot;

#[cfg(test)]
mod tests {

    use super::*;

    // The majority of tests from https://github.com/seomoz/rep-cpp/blob/master/test/test-robots.cpp
    // are highly similar to those from reppy. A few are unique and worthy of inclusion however.

    #[test]
    fn test_repcpp_no_leading_user_agent() {
        let txt = "Disallow: /path
        Allow: /path/exception
        Crawl-delay: 5.2";

        let r = Robot::new("Agent", txt.as_bytes()).unwrap();
        assert!(r.allowed("https://example.com/path/exception"));
        assert!(!r.allowed("https://example.com/path"));
        assert_eq!(r.delay, Some(5.2));
    }

    #[test]
    fn test_repcpp_well_formed_crawl_delay() {
        let txt = "Disallow: /path
        Allow: /path/exception
        Crawl-delay: 5.2";

        let r = Robot::new("Agent", txt.as_bytes()).unwrap();
        assert!(r.allowed("https://example.com/path/exception"));
        assert!(!r.allowed("https://example.com/path"));
        assert_eq!(r.delay, Some(5.2));
    }

    #[test]
    fn test_repcpp_malformed_crawl_delay() {
        let txt = "User-agent: *
        Crawl-delay: word";

        let r = Robot::new("Agent", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, None);
    }

    #[test]
    fn test_repcpp_empty() {
        let txt = "";
        let r = Robot::new("Agent", txt.as_bytes()).unwrap();
        assert!(r.allowed("/"));
    }

    #[test]
    fn test_repcpp_accepts_full_url() {
        let txt = "User-Agent: agent
        Disallow: /path;params?query";
        let r = Robot::new("Agent", txt.as_bytes()).unwrap();
        assert!(!r.allowed(
            "http://userinfo@exmaple.com:10/path;params?query#fragment"
        ));
    }

    #[test]
    fn test_repcpp_leading_wildcard_allow() {
        let txt = "User-agent: meow
        Disallow: /
        Allow: ****/cats
        Allow: */kangaroos";
        let r = Robot::new("meow", txt.as_bytes()).unwrap();
        assert!(!r.allowed("/kangaroo/zebra/cat/page.html"));
        assert!(r.allowed("/cats.html"));
        assert!(r.allowed("/cats/page.html"));
        assert!(r.allowed("/get/more/cats/page.html"));
        assert!(r.allowed("/kangaroos/page.html"));
        assert!(r.allowed("/heaps/of/kangaroos/page.html"));
        assert!(r.allowed("/kangaroosandkoalas/page.html"));
    }

    // Redundant but included for completeness (matching repcpp tests)
    #[test]
    fn test_repcpp_leading_wildcard_disallow() {
        let txt = "User-agent: meow
        Allow: /
        Disallow: ****/cats
        Disallow: */kangaroos";
        let r = Robot::new("meow", txt.as_bytes()).unwrap();
        assert!(r.allowed("/kangaroo/zebra/cat/page.html"));
        assert!(!r.allowed("/cats.html"));
        assert!(!r.allowed("/cats/page.html"));
        assert!(!r.allowed("/get/more/cats/page.html"));
        assert!(!r.allowed("/kangaroos/page.html"));
        assert!(!r.allowed("/heaps/of/kangaroos/page.html"));
        assert!(!r.allowed("/kangaroosandkoalas/page.html"));
    }
}
