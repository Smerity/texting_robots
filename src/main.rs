use texting_robots::Robot;

fn main() {
    let f = std::fs::File::open("testdata/twitter.robots.txt").unwrap();
    let mut r = std::io::BufReader::new(f);
    let mut txt = String::new();
    std::io::Read::read_to_string(&mut r, &mut txt).unwrap();

    use std::time::Instant;
    let before = Instant::now();
    const ITERATIONS: u32 = 1_000;
    for _ in 0..ITERATIONS {
        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(1));
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
    println!(
        "Elapsed time: {:.2?} / {} = {:.2?} per loop",
        before.elapsed(),
        ITERATIONS,
        before.elapsed() / ITERATIONS
    );
}
