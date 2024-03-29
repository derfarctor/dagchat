use bigdecimal::BigDecimal;
use std::str::FromStr;

pub fn get_32_bytes(string: &str) -> [u8; 32] {
    let bytes = hex::decode(string).unwrap();
    let mut array = [0u8; 32];
    array.copy_from_slice(bytes.as_slice());
    array
}

// Also validates if the amount submitted was possible
pub fn whole_to_raw(whole: String, multiplier: &str) -> Option<u128> {
    let amount = BigDecimal::from_str(whole.trim());
    if amount.is_err() {
        return None;
    }
    let multi = BigDecimal::from_str(multiplier).unwrap();
    let amount_raw = amount.unwrap() * multi;
    if amount_raw.is_integer() {
        let raw_string = amount_raw.with_scale(0).to_string();
        let raw: u128 = raw_string.parse().unwrap();
        if raw == 0 {
            None
        } else {
            Some(raw)
        }
    } else {
        None
    }
}

pub fn raw_to_whole(raw: &str, multiplier: &str) -> String {
    let amount = BigDecimal::from_str(raw).unwrap();
    let multiplier = BigDecimal::from_str(multiplier).unwrap();
    let whole = amount / multiplier;
    whole.to_string()
}

pub fn display_to_dp(raw: u128, dp: usize, multiplier: &str, ticker: &str) -> String {
    let raw_string = raw.to_string();
    let raw = BigDecimal::from_str(&raw_string).unwrap();
    let multi = BigDecimal::from_str(multiplier).unwrap();
    let raw_threshold = &multi / BigDecimal::from(10u32.pow(dp as u32));
    if raw < raw_threshold {
        format!("{} RAW", raw)
    } else {
        let adjusted = raw / multi;
        let s = adjusted.to_string();

        // If decimal part, trim to dp
        if s.contains('.') {
            let mut parts: Vec<&str> = s.split('.').collect();
            let real_dp = parts[1].len();
            if real_dp > dp {
                parts[1] = parts[1].get(0..dp).unwrap();
            }
            format!("{}.{}{}", parts[0], parts[1], ticker)
        } else {
            format!("{}{}", s, ticker)
        }
    }
}
