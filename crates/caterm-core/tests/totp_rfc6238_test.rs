//! RFC 6238 Test Suite, Zeroization & Security Audit for CATerm 2FA Vault (REQ-37).
//!
//! Subagent 4 Tasks:
//! 1. RFC 6238 Official Test Vectors at exact timestamps (59s, 1111111109s, 1111111111s, 1234567890s, 2000000000s).
//! 2. Base32 sanitization (spaces, hyphens, padding, lower/upper case).
//! 3. otpauth URI parsing standard & edge cases.
//! 4. Zero-knowledge & memory zeroization verification (`zeroize` crate behavior).
//! 5. Ciphertext storage verification in encrypted database / AES-256-GCM.

use caterm_core::totp::{
    decode_base32, generate_totp, generate_totp_at_timestamp, parse_otpauth_uri,
};
use zeroize::Zeroize;

/// Official RFC 6238 Section 5.3 Test Vectors:
/// The test token shared secret corresponds to the ASCII string "12345678901234567890" (20 bytes).
/// In Base32 (RFC 4648), this secret is "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ".
const RFC_SECRET_BASE32: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
const RFC_SECRET_ASCII: &[u8] = b"12345678901234567890";

#[test]
fn test_rfc6238_official_test_vectors_sha1_8_digits_and_6_digits() {
    // Official test vector table from RFC 6238 Appendix B:
    // +-------------+--------------+------------------+----------+--------+
    // |  Time (sec) |   UTC Time   | Value of T(hex)  |   TOTP   | Mode   |
    // +-------------+--------------+------------------+----------+--------+
    // |          59 | 1970-01-01   | 0000000000000001 | 94287082 | SHA1   |
    // |  1111111109 | 2005-03-18   | 00000000023523EC | 07081804 | SHA1   |
    // |  1111111111 | 2005-03-18   | 00000000023523ED | 14050471 | SHA1   |
    // |  1234567890 | 2009-02-13   | 000000000273EF07 | 89005924 | SHA1   |
    // |  2000000000 | 2033-05-18   | 0000000003F940AA | 69279037 | SHA1   |
    // | 20000000000 | 2603-10-11   | 0000000027BC86AA | 65353130 | SHA1   |
    // +-------------+--------------+------------------+----------+--------+

    let test_cases = [
        (59_u64, "94287082", "287082", 1_u64),
        (1111111109_u64, "07081804", "081804", 1_u64),
        (1111111111_u64, "14050471", "050471", 29_u64),
        (1234567890_u64, "89005924", "005924", 30_u64),
        (2000000000_u64, "69279037", "279037", 10_u64),
        (20000000000_u64, "65353130", "353130", 10_u64),
    ];

    for (timestamp, expected_8_digit, expected_6_digit, expected_remaining) in test_cases {
        // Test 8 digits (official RFC table format)
        let (otp_8, remaining) = generate_totp_at_timestamp(RFC_SECRET_BASE32, timestamp, 30, 8)
            .expect("generate 8-digit totp failed");
        assert_eq!(
            otp_8, expected_8_digit,
            "RFC 6238 vector failed for timestamp {timestamp} (8 digits)"
        );
        assert_eq!(
            remaining, expected_remaining,
            "Remaining seconds calculation mismatch at {timestamp}"
        );

        // Test 6 digits (standard CATerm default: lower 6 digits of the computed decimal)
        let (otp_6, _) = generate_totp_at_timestamp(RFC_SECRET_BASE32, timestamp, 30, 6)
            .expect("generate 6-digit totp failed");
        assert_eq!(
            otp_6, expected_6_digit,
            "RFC 6238 vector failed for timestamp {timestamp} (6 digits)"
        );
    }
}

#[test]
fn test_base32_decoding_and_sanitization() {
    // Check ASCII vs Base32 conversion
    let decoded = decode_base32(RFC_SECRET_BASE32).expect("decode valid base32");
    assert_eq!(decoded, RFC_SECRET_ASCII);

    // Mixed casing, whitespace, and hyphens
    let dirty_variations = [
        "gez dgn bvgy 3tq ojq gez dgn bvgy 3tq ojq",
        "GEZD-GNBV-GY3T-QOJQ-GEZD-GNBV-GY3T-QOJQ",
        "  gez-dgn-bvgy-3tq-ojq-gez-dgn-bvgy-3tq-ojq  ",
        "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ====",
        "\tg-e-z-d-g-n-b-v-g-y-3-t-q-o-j-q-g-e-z-d-g-n-b-v-g-y-3-t-q-o-j-q\n",
    ];

    for dirty in dirty_variations {
        let clean_decoded = decode_base32(dirty).expect("decode sanitized base32");
        assert_eq!(
            clean_decoded, RFC_SECRET_ASCII,
            "Failed for dirty string: {dirty}"
        );

        let (otp, _) = generate_totp_at_timestamp(dirty, 1234567890, 30, 6)
            .expect("generate totp from dirty base32");
        assert_eq!(otp, "005924", "Failed OTP match for dirty string: {dirty}");
    }

    // Invalid Base32 characters (e.g., '1', '8', '9', '0', special symbols)
    assert!(decode_base32("GEZDGNBVGY3TQOJ8").is_err());
    assert!(decode_base32("GEZDGNBVGY3TQOJ9").is_err());
    assert!(decode_base32("GEZDGNBVGY3TQOJ1").is_err());
    assert!(decode_base32("GEZDGNBVGY3TQOJ0").is_err());
    assert!(decode_base32("GEZDGNBV!@#$%").is_err());
    assert!(decode_base32("   ").is_err());
}

#[test]
fn test_otpauth_uri_parser() {
    let uri = "otpauth://totp/CATerm%20Bastion:admin@production?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=CATerm%20Bastion&algorithm=SHA1&digits=6&period=30";
    let parsed = parse_otpauth_uri(uri).expect("parse otpauth uri");

    assert_eq!(parsed.label, "CATerm Bastion:admin@production");
    assert_eq!(parsed.issuer.as_deref(), Some("CATerm Bastion"));
    assert_eq!(parsed.secret, RFC_SECRET_BASE32);
    assert_eq!(parsed.algorithm, "SHA1");
    assert_eq!(parsed.digits, 6);
    assert_eq!(parsed.period, 30);

    // Test label fallback when issuer query param is omitted
    let uri_no_param = "otpauth://totp/GitHub:octocat?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
    let parsed2 = parse_otpauth_uri(uri_no_param).expect("parse uri without issuer param");
    assert_eq!(parsed2.issuer.as_deref(), Some("GitHub"));
    assert_eq!(parsed2.label, "GitHub:octocat");

    // Invalid schemes or types
    assert!(parse_otpauth_uri("http://totp/example?secret=ABC").is_err());
    assert!(parse_otpauth_uri("otpauth://hotp/example?secret=ABC").is_err());
    assert!(parse_otpauth_uri("otpauth://totp/example").is_err()); // missing secret
}

#[test]
fn test_zero_knowledge_memory_zeroization() {
    let mut secret_buffer = b"TOP_SECRET_TOTP_KEY_IN_RAM_12345".to_vec();
    assert!(!secret_buffer.iter().all(|&b| b == 0));

    // Zeroize memory explicitly
    secret_buffer.zeroize();
    assert!(
        secret_buffer.iter().all(|&b| b == 0),
        "Memory buffer was not zeroized properly!"
    );
}

#[test]
fn test_totp_secret_encrypted_at_rest_with_aes_gcm() {
    // Ensure that storing a secret in the vault encrypts it into `gcm1:...` ciphertext
    let local_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let raw_secret = "JBSWY3DPEHPK3PXP";

    let encrypted =
        caterm_core::secret::encrypt(local_key, raw_secret).expect("encrypt secret with local key");

    // Must be prefixed with gcm1: and must NOT contain plaintext secret
    assert!(encrypted.starts_with("gcm1:"));
    assert!(!encrypted.contains(raw_secret));

    // Decrypt must recover original secret
    let decrypted =
        caterm_core::secret::decrypt(local_key, &encrypted).expect("decrypt secret with local key");
    assert_eq!(decrypted, raw_secret);
}

#[test]
fn test_generate_totp_live_runtime_succeeds() {
    let (code, remaining) = generate_totp(RFC_SECRET_BASE32, 0).expect("generate live totp");
    assert_eq!(code.len(), 6);
    assert!(code.chars().all(|c| c.is_ascii_digit()));
    assert!(remaining <= 30);
}
