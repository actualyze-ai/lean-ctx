#[test]
fn slice_at_multibyte_boundary_does_not_panic() {
    // '→' is 3 bytes (E2 86 92). Place it so byte 200 falls inside it.
    let mut s = "a".repeat(198);
    s.push('→'); // bytes 198..201
    s.push_str("tail");

    // Byte 200 is inside '→'; floor_char_boundary should snap back to 198.
    let end = s.floor_char_boundary(200);
    assert_eq!(end, 198);
    // Slicing at the floor boundary must not panic.
    let _ = &s[..end];

    // ceil_char_boundary should advance to 201.
    let start = s.ceil_char_boundary(200);
    assert_eq!(start, 201);
    let _ = &s[start..];
}

#[test]
fn hash_fast_does_not_panic_on_multibyte_at_prefix_boundary() {
    // Construct a string > 16KB where byte 4096 falls inside a multi-byte char.
    let mut s = "a".repeat(4094);
    s.push('→'); // 3-byte char at bytes 4094..4097, so byte 4096 is inside it
    while s.len() <= 16 * 1024 {
        s.push('x');
    }
    let _hash = lean_ctx::server::helpers::hash_fast(&s);
}

#[test]
fn hash_fast_does_not_panic_on_multibyte_at_suffix_boundary() {
    // Construct a string > 16KB where the suffix start falls inside a multi-byte char.
    let target_len = 20_000;
    let suffix_start = target_len - 4096; // = 15904
    let mut s = "a".repeat(suffix_start - 1);
    s.push('→'); // 3-byte char, some byte of which aligns with suffix_start
    while s.len() < target_len {
        s.push('x');
    }
    let _hash = lean_ctx::server::helpers::hash_fast(&s);
}
