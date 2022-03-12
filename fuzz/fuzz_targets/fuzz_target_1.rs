#![no_main]
extern crate libfuzzer_sys;

extern crate texting_robots;
use texting_robots::{Robot};

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _r = Robot::new("*", data);
});
