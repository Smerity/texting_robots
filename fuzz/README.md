To run:

`cargo fuzz run fuzz_target_1 -- -only_ascii=1 -max_len=512 -dict=keywords.dict`

For more involved:

`cargo fuzz run --jobs 8 fuzz_target_1 -- -only_ascii=1 -max_len=512 -dict=keywords.dict`

Note: `cargo fuzz` requires nightly (i.e. `rustup default nightly`)
