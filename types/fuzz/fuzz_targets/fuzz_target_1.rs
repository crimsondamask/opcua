#![no_main]
use libfuzzer_sys::fuzz_target;

use opcua_types::*;
use std::io::Cursor;

pub fn deserialize(data: &[u8], decoding_options: &DecodingOptions) -> Result<Variant, StatusCode> {
    // Decode this, don't expect panics or whatever
    let mut stream = Cursor::new(data);
    Variant::decode(&mut stream, &decoding_options)
}

fuzz_target!(|data: &[u8]| {
    let decoding_options = DecodingOptions::default();
    let _ = deserialize(data, &decoding_options);
});
