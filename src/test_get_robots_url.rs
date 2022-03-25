use super::get_robots_url;

#[cfg(test)]
mod tests {

    use super::*;

    use url::ParseError;

    #[test]
    fn test_get_robots_url_varying_paths() {
        let urls = vec![
            "https://twitter.com/",
            "https://twitter.com/sitemap.xml",
            "https://twitter.com/halvarflake",
            "https://twitter.com/halvarflake/status/1501495664466927618",
            "https://twitter.com/halvarflake/status/1501495664466927618?s=20&t=7xv0WrBVxLVKo2OUCPn6OQ",
        ];
        let expected = "https://twitter.com/robots.txt";
        for url in urls {
            assert_eq!(get_robots_url(url).unwrap(), expected);
        }

        let urls = vec![
            "https://news.ycombinator.com/",
            "https://news.ycombinator.com/threads?id=pg",
            "https://news.ycombinator.com/item?id=22238335",
        ];
        let expected = "https://news.ycombinator.com/robots.txt";
        for url in urls {
            assert_eq!(get_robots_url(url).unwrap(), expected);
        }

        let urls = vec![
            "http://en.wikipedia.org",
            "http://en.wikipedia.org/",
            "http://en.wikipedia.org/wiki/",
            "http://en.wikipedia.org/wiki/Gravity_hill",
            "http://en.wikipedia.org/wiki/Gravity_hill?action=edit",
        ];
        let expected = "http://en.wikipedia.org/robots.txt";
        for url in urls {
            assert_eq!(get_robots_url(url).unwrap(), expected);
        }
    }

    #[test]
    fn test_get_robots_url_has_wrong_scheme() {
        let urls = vec!["ipfs://etc/", "ftp://linux-isos.org/"];
        let expected = ParseError::EmptyHost;

        for url in urls {
            let result = get_robots_url(url);
            assert!(result.is_err());
            assert_eq!(result, Err(expected));
        }
    }

    #[test]
    fn test_get_robots_url_cannot_be_base() {
        let urls = vec!["mailto:ferris@rust.com", "/rust/v1/index.html"];

        for url in urls {
            let result = get_robots_url(url);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_get_robots_url_removes_username_and_passwd() {
        // Test in the style of Reppy's robots URL tests
        let url =
            "http://user:pass@example.com:8080/path;params?query#fragment";
        let expected = "http://example.com:8080/robots.txt";
        assert_eq!(get_robots_url(url).unwrap(), expected);
    }

    /// REPPY ROBOTS URL TESTS
    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn test_reppy_robots_url_http() {
        let url = "http://user@example.com:80/path;params?query#fragment";
        let expected = "http://example.com/robots.txt";
        assert_eq!(get_robots_url(url).unwrap(), expected);
    }

    #[test]
    fn test_reppy_robots_url_https() {
        // This test is modified
        // This tries to connect to https on port 80
        // Reppy removes the port from the robots.txt URL
        // We assume it remains the same
        let url = "https://user@example.com:80/path;params?query#fragment";
        let expected = "https://example.com:80/robots.txt";
        assert_eq!(get_robots_url(url).unwrap(), expected);
    }

    #[test]
    fn test_reppy_robots_url_non_default_port() {
        let url = "http://user@example.com:8080/path;params?query#fragment";
        let expected = "http://example.com:8080/robots.txt";
        assert_eq!(get_robots_url(url).unwrap(), expected);
    }

    #[test]
    fn test_reppy_robots_url_invalid_port() {
        let url = "http://:::cnn.com/";
        let expected = ParseError::EmptyHost;
        let result = get_robots_url(url);

        assert!(result.is_err());
        assert_eq!(result, Err(expected));
    }
}
