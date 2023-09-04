#![no_main]

use libfuzzer_sys::fuzz_target;
use rizin_rs::wrapper::abbrev;

fuzz_target!(|input: &[u8]| {
    let _ = abbrev(input);
});
