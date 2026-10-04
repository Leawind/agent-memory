//! Small utility functions. Base64 and SHA-256 are hand-written std-only (minimal-dependency
//! stance); time formatting goes through `chrono` — local wall-clock conversion needs the OS
//! time-zone database, which std lacks and the `time` crate refuses to touch in multithreaded
//! processes (the server is a multithreaded HTTP worker pool).

/// Standard-alphabet Base64 encode (RFC 4648, with padding). Std-only, mirroring the project's
/// minimal-dependency stance; used for the `=?base64?...?=` header sentinels and the opaque
/// resources/list pagination cursors.
pub fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = u32::from_be_bytes([0, b[0], b[1], b[2]]);
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// Standard-alphabet Base64 decode; `None` on malformed input (bad length, misplaced padding,
/// non-alphabet characters).
pub fn base64_decode(input: &str) -> Option<Vec<u8>> {
    fn val(c: u8) -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'a'..=b'z' => Some((c - b'a' + 26) as u32),
            b'0'..=b'9' => Some((c - b'0' + 52) as u32),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let bytes = input.as_bytes();
    if bytes.is_empty() {
        return Some(Vec::new());
    }
    if bytes.len() % 4 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks_exact(4) {
        let pad = chunk.iter().filter(|&&c| c == b'=').count();
        if pad > 2 || chunk[..4 - pad].contains(&b'=') {
            return None; // padding only allowed as the 1-2 final characters
        }
        let mut acc: u32 = 0;
        for &c in &chunk[..4 - pad] {
            acc = (acc << 6) | val(c)?;
        }
        acc <<= 6 * pad as u32;
        let decoded = [(acc >> 16) as u8, (acc >> 8) as u8, acc as u8];
        out.extend_from_slice(&decoded[..3 - pad]);
    }
    Some(out)
}

/// Percent-decoding core: every valid `%XX` escape becomes its byte; malformed input is handled
/// per-escape according to `strict` (error out, or keep the `%` literal and move on). `+` is never
/// a space — that is form encoding, not URI encoding.
fn percent_decode_bytes(s: &str, strict: bool) -> Result<Vec<u8>, String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 3 > bytes.len() {
                if strict {
                    return Err("truncated percent-escape".into());
                }
            } else {
                match u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    Ok(byte) => {
                        out.push(byte);
                        i += 3;
                        continue;
                    }
                    Err(_) if strict => return Err("malformed percent-escape".into()),
                    Err(_) => {}
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    Ok(out)
}

/// Strict percent-decode (the `memory://` resource URIs): a malformed escape or a non-UTF-8
/// result is an error — resource identities must not be silently mangled.
pub fn percent_decode_strict(s: &str) -> Result<String, String> {
    let bytes = percent_decode_bytes(s, true)?;
    String::from_utf8(bytes).map_err(|_| "percent-decoded value is not valid UTF-8".into())
}

/// Lenient percent-decode (the REST face's path and query segments): malformed escapes stay
/// literal and invalid UTF-8 decodes lossily — tiny_http hands the raw request line through, and
/// one bad segment must not sink an otherwise routable request.
pub fn percent_decode_lenient(s: &str) -> String {
    let bytes = percent_decode_bytes(s, false).unwrap_or_else(|_| s.as_bytes().to_vec());
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Format a Unix-seconds timestamp as UTC ISO 8601 (e.g. `2026-09-25T21:41:50Z`). Used by the
/// memory resource's `lastModified` annotation (protocol-level metadata, hence precise UTC).
pub fn format_utc_iso(epoch_secs: u64) -> String {
    time_string(chrono::Utc, epoch_secs, "%Y-%m-%dT%H:%M:%SZ")
}

/// Format a Unix-seconds timestamp as the server's local wall clock, minute precision
/// (`2026-10-04 21:41`). Agent-facing memory views use this instead of epoch: models cannot
/// convert epoch mentally, and a self-hosted box serves one operator whose wall clock is the
/// reference. Out-of-range values fall back to the raw epoch number (unreachable in practice).
pub fn format_local_compact(epoch_secs: u64) -> String {
    time_string(chrono::Local, epoch_secs, "%Y-%m-%d %H:%M")
}

fn time_string<Tz>(tz: Tz, epoch_secs: u64, pattern: &str) -> String
where
    Tz: chrono::TimeZone,
    Tz::Offset: std::fmt::Display,
{
    tz.timestamp_opt(epoch_secs as i64, 0)
        .single()
        .map(|t| t.format(pattern).to_string())
        .unwrap_or_else(|| epoch_secs.to_string())
}

/// SHA-256 digest (lowercase hex). Hand-written on std to avoid a hashing dependency — the only use is
/// token hashing (tokens are 256-bit random values with plenty of entropy; no slow hash needed against brute force).
pub fn sha256_hex(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    // Message padding: append 0x80, pad with zeros to ≡56 (mod 64), then append the 64-bit big-endian bit length
    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64) * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    for block in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in block.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        let parts = [a, b, c, d, e, f, g, hh];
        for (v, add) in h.iter_mut().zip(parts) {
            *v = v.wrapping_add(add);
        }
    }
    h.iter().map(|v| format!("{v:08x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_formats() {
        assert_eq!(format_utc_iso(0), "1970-01-01T00:00:00Z");
        // Two widely known time anchors
        assert_eq!(format_utc_iso(1_000_000_000), "2001-09-09T01:46:40Z");
        assert_eq!(format_utc_iso(2_000_000_000), "2033-05-18T03:33:20Z");
    }

    #[test]
    fn local_compact_shape() {
        // Shape-only assertion: the concrete value depends on the machine's time zone, so the
        // test anchors the format, not the clock
        let re = regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}$").unwrap();
        assert!(re.is_match(&format_local_compact(1_000_000_000)));
    }

    #[test]
    fn base64_roundtrip_and_strictness() {
        assert_eq!(base64_decode("bWVtb3J5X2xpc3Q=").unwrap(), b"memory_list");
        assert_eq!(base64_encode(b"memory_list"), "bWVtb3J5X2xpc3Q=");
        // The empty string is valid Base64 (RFC 4648)
        assert_eq!(base64_decode("").unwrap(), Vec::<u8>::new());
        assert_eq!(base64_decode("AAAA").unwrap(), vec![0, 0, 0]);
        assert_eq!(base64_decode("AAA=").unwrap(), vec![0, 0]);
        assert_eq!(base64_decode("AA==").unwrap(), vec![0]);
        // Padding misplaced / bad characters / bad length
        assert!(base64_decode("=AAA").is_none());
        assert!(base64_decode("A===").is_none());
        assert!(base64_decode("AAA").is_none());
        assert!(base64_decode("A@A=").is_none());
        // Round-trips across chunk boundaries (including non-ASCII UTF-8)
        for sample in ["", "a", "ab", "abc", "abcd", "项目记忆", &"x".repeat(1000)] {
            assert_eq!(
                base64_decode(&base64_encode(sample.as_bytes())).unwrap(),
                sample.as_bytes()
            );
        }
    }

    #[test]
    fn sha256_known_vectors() {
        // FIPS 180-4 standard test vectors
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        // Multi-block (>64 bytes) and length-alignment boundaries
        assert_eq!(
            sha256_hex(&[b'a'; 1000]),
            "41edece42d63e8d9bf515a9ba6932e1c20cbc9f5a5d134645adb5db1b9737ea3"
        );
    }

    #[test]
    fn percent_decode_strict_and_lenient() {
        // Shared core: escapes and multi-byte UTF-8 decode identically in both modes
        assert_eq!(percent_decode_strict("%E9%A1%B9%E7%9B%AE").unwrap(), "项目");
        assert_eq!(percent_decode_lenient("%E9%A1%B9%E7%9B%AE"), "项目");
        assert_eq!(percent_decode_lenient("plain"), "plain");
        assert_eq!(percent_decode_lenient("a%2Fb"), "a/b");
        // Strict mode rejects: malformed escape, truncated escape, non-UTF-8 bytes
        assert!(percent_decode_strict("a%ZZb").is_err());
        assert!(percent_decode_strict("a%2").is_err());
        assert!(percent_decode_strict("%FF").is_err());
        // Lenient mode keeps invalid escapes literally (per-escape, not whole-string fallback);
        // valid escapes still decode even in a mixed string, lossily when not valid UTF-8
        assert_eq!(percent_decode_lenient("a%ZZb"), "a%ZZb");
        assert_eq!(percent_decode_lenient("a%2"), "a%2");
        assert_eq!(percent_decode_lenient("%E9%A1%ZZ"), "\u{FFFD}%ZZ");
        // `+` is not a space in either mode (form encoding is not URI encoding)
        assert_eq!(percent_decode_strict("a+b").unwrap(), "a+b");
        assert_eq!(percent_decode_lenient("a+b"), "a+b");
    }

    #[test]
    fn leap_year_and_rollover() {
        // Leap day: 2024-02-29T00:00:00Z = 1709164800
        assert_eq!(format_utc_iso(1_709_164_800), "2024-02-29T00:00:00Z");
        // Year-end day boundary: 2023-12-31T23:59:59Z = 1704067199
        assert_eq!(format_utc_iso(1_704_067_199), "2023-12-31T23:59:59Z");
        assert_eq!(format_utc_iso(1_704_067_200), "2024-01-01T00:00:00Z");
    }
}
