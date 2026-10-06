//! RFC 6238 TOTP Engine and Vault Storage Implementation (REQ-37).
//!
//! Provides HMAC-SHA1 TOTP token generation, Base32 decoding/sanitization,
//! `otpauth://` URI parsing, and zeroization of secret keys in memory.

use crate::error::{CatermError, ValidationError};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use zeroize::Zeroize;

type HmacSha1 = Hmac<Sha1>;

/// Parses a Base32 string (RFC 4648 standard alphabet), ignoring spaces, hyphens, and padding ('='),
/// in a case-insensitive manner.
pub fn decode_base32(input: &str) -> Result<Vec<u8>, CatermError> {
    let mut clean_chars = Vec::with_capacity(input.len());
    for c in input.chars() {
        if c.is_whitespace() || c == '-' || c == '=' {
            continue;
        }
        let upper = c.to_ascii_uppercase();
        if ('A'..='Z').contains(&upper) || ('2'..='7').contains(&upper) {
            clean_chars.push(upper);
        } else {
            return Err(CatermError::Validation(ValidationError::Generic(format!(
                "Karakter Base32 tidak valid: {c}"
            ))));
        }
    }

    if clean_chars.is_empty() {
        return Err(CatermError::Validation(ValidationError::Generic(
            "Secret Base32 kosong".to_string(),
        )));
    }

    let mut buffer: u32 = 0;
    let mut bits_left: u8 = 0;
    let mut output: Vec<u8> = Vec::new();

    for &c in &clean_chars {
        let val: u8 = match c {
            'A'..='Z' => (c as u8) - b'A',
            '2'..='7' => (c as u8) - b'2' + 26,
            _ => unreachable!(),
        };

        buffer = (buffer << 5) | (val as u32);
        bits_left += 5;

        if bits_left >= 8 {
            bits_left -= 8;
            output.push((buffer >> bits_left) as u8);
            buffer &= (1 << bits_left) - 1;
        }
    }

    Ok(output)
}

/// Parsed metadata from `otpauth://totp/...` URI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtpAuthUri {
    pub label: String,
    pub issuer: Option<String>,
    pub secret: String,
    pub algorithm: String,
    pub digits: u32,
    pub period: u64,
}

/// Parses standard `otpauth://totp/...` URI.
pub fn parse_otpauth_uri(uri: &str) -> Result<OtpAuthUri, CatermError> {
    let parsed_url = url::Url::parse(uri).map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!("URI tidak valid: {e}")))
    })?;

    if parsed_url.scheme() != "otpauth" {
        return Err(CatermError::Validation(ValidationError::Generic(
            "Skema URI harus 'otpauth'".to_string(),
        )));
    }

    if parsed_url.host_str() != Some("totp") {
        return Err(CatermError::Validation(ValidationError::Generic(
            "Hanya tipe TOTP yang didukung".to_string(),
        )));
    }

    let raw_path = parsed_url.path().trim_start_matches('/');
    let label = percent_encoding::percent_decode_str(raw_path)
        .decode_utf8()
        .map_err(|e| {
            CatermError::Validation(ValidationError::Generic(format!(
                "Label URI tidak valid: {e}"
            )))
        })?
        .to_string();

    let mut secret: Option<String> = None;
    let mut issuer: Option<String> = None;
    let mut algorithm = "SHA1".to_string();
    let mut digits: u32 = 6;
    let mut period: u64 = 30;

    for (key, value) in parsed_url.query_pairs() {
        match key.as_ref() {
            "secret" => secret = Some(value.to_string()),
            "issuer" => issuer = Some(value.to_string()),
            "algorithm" => algorithm = value.to_uppercase(),
            "digits" => {
                if let Ok(d) = value.parse::<u32>() {
                    digits = d;
                }
            }
            "period" => {
                if let Ok(p) = value.parse::<u64>() {
                    period = p;
                }
            }
            _ => {}
        }
    }

    let secret = secret.ok_or_else(|| {
        CatermError::Validation(ValidationError::Generic(
            "Parameter 'secret' wajib ada dalam otpauth URI".to_string(),
        ))
    })?;

    // If issuer is missing from query param, check if label contains "Issuer:Account"
    if issuer.is_none() && label.contains(':') {
        if let Some((iss, _)) = label.split_once(':') {
            let iss = iss.trim();
            if !iss.is_empty() {
                issuer = Some(iss.to_string());
            }
        }
    }

    Ok(OtpAuthUri {
        label,
        issuer,
        secret,
        algorithm,
        digits,
        period,
    })
}

/// Generates a 6-digit TOTP token using HMAC-SHA1 according to RFC 6238.
///
/// Returns `(token, remaining_seconds_in_step)`.
/// Memory containing decoded raw secret bytes is guaranteed to be zeroized upon completion.
pub fn generate_totp_at_timestamp(
    secret: &str,
    timestamp_sec: u64,
    period_sec: u64,
    digits: u32,
) -> Result<(String, u64), CatermError> {
    if period_sec == 0 {
        return Err(CatermError::Validation(ValidationError::Generic(
            "Period TOTP tidak boleh 0".to_string(),
        )));
    }

    // Step 1: Decode Base32 secret bytes
    let mut key_bytes = decode_base32(secret)?;

    // Step 2: Calculate TOTP counter step
    let time_step = timestamp_sec / period_sec;
    let remaining_sec = period_sec - (timestamp_sec % period_sec);

    let step_bytes = time_step.to_be_bytes();

    // Step 3: Compute HMAC-SHA1
    let mut mac = HmacSha1::new_from_slice(&key_bytes).map_err(|e| {
        key_bytes.zeroize();
        CatermError::Validation(ValidationError::Generic(format!("HMAC key error: {e}")))
    })?;

    // Zeroize key immediately after initialization into HMAC state
    key_bytes.zeroize();

    mac.update(&step_bytes);
    let result = mac.finalize();
    let hmac_result = result.into_bytes();

    // Step 4: Truncate dynamically (RFC 4226 / RFC 6238)
    let offset = match hmac_result.last() {
        Some(b) => (b & 0x0f) as usize,
        None => {
            return Err(CatermError::Validation(ValidationError::Generic(
                "HMAC output kosong".to_string(),
            )));
        }
    };

    if offset + 4 > hmac_result.len() {
        return Err(CatermError::Validation(ValidationError::Generic(
            "HMAC output terlalu pendek untuk pemotongan".to_string(),
        )));
    }

    let binary = ((hmac_result[offset] as u32 & 0x7f) << 24)
        | ((hmac_result[offset + 1] as u32 & 0xff) << 16)
        | ((hmac_result[offset + 2] as u32 & 0xff) << 8)
        | (hmac_result[offset + 3] as u32 & 0xff);

    let divisor = 10_u32.pow(digits);
    let otp = binary % divisor;
    let otp_str = format!("{:0width$}", otp, width = digits as usize);

    Ok((otp_str, remaining_sec))
}

/// Generates TOTP for current time adjusted by `time_offset_sec`.
pub fn generate_totp(secret: &str, time_offset_sec: i64) -> Result<(String, u64), CatermError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| {
            CatermError::Validation(ValidationError::Generic(format!(
                "Waktu sistem tidak valid: {e}"
            )))
        })?
        .as_secs() as i64;

    let target_time = (now + time_offset_sec).max(0) as u64;
    generate_totp_at_timestamp(secret, target_time, 30, 6)
}
