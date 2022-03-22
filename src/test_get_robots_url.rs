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
}
