use texting_robots::Robot;

#[cfg(not(tarpaulin_include))]
fn main() {
    let f = std::fs::File::open("testdata/twitter.robots.txt").unwrap();
    let mut r = std::io::BufReader::new(f);
    let mut txt = String::new();
    std::io::Read::read_to_string(&mut r, &mut txt).unwrap();

    use std::time::Instant;
    let before = Instant::now();
    const ITERATIONS: u32 = 100_000;
    for _ in 0..ITERATIONS {
        let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
        assert_eq!(r.delay, Some(1.0));
        assert_eq!(r.sitemaps, vec!["https://twitter.com/sitemap.xml"]);
    }
    for foo in vec![1] {
        println!("{foo}");
    }
    println!(
        "Elapsed time: {:.2?} / {} = {:.2?} per parsed robots.txt",
        before.elapsed(),
        ITERATIONS,
        before.elapsed() / ITERATIONS
    );

    let before = Instant::now();
    let r = Robot::new("BobBot", txt.as_bytes()).unwrap();
    for _ in 0..ITERATIONS {
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
        // Round it out to ten for benchmarking purposes
        assert!(r.allowed(
            "https://twitter.com/smerity/status/1501495664466927618"
        ));
        assert!(r.allowed("https://twitter.com/halvarflake/follower"));
        assert!(r.allowed("https://twitter.com/explore"));
        assert!(r.allowed("https://twitter.com/settings/account"));
    }
    println!(
        "Elapsed time: {:.2?} / {} = {:.2?} per allow check",
        before.elapsed(),
        ITERATIONS * 10,
        before.elapsed() / ITERATIONS / 10 // As there are 10 allow checks per loop
    );
}
