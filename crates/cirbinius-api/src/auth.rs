use sha2::{Digest, Sha256};

use crate::uuid::Uuid;
use crate::state::{create_api_key, get_api_key_by_prefix, SharedStore};

pub fn hash_api_key(key: &str) -> String {
    // Simple SHA-256 based key hashing (argon2 not available)
    let hash = Sha256::digest(key.as_bytes());
    // Iterate hashing for key stretching
    let mut current = hash;
    for _ in 0..10 {
        current = Sha256::digest(&current);
    }
    hex::encode(current)
}

pub fn verify_api_key(key: &str, hash: &str) -> bool {
    hash_api_key(key) == hash
}

pub fn generate_api_key_value() -> (String, String) {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).expect("rng");
    let key = hex::encode(bytes);
    let prefix = key[..8].to_string();
    (key, prefix)
}

pub fn generate_api_key(
    store: &SharedStore,
    name: &str,
    project_id: Option<Uuid>,
    permissions: &[String],
    expires_in_days: Option<i64>,
) -> (String, String, Uuid) {
    let (api_key, prefix) = generate_api_key_value();
    let hash = hash_api_key(&api_key);
    let expires_at = expires_in_days.map(|days| {
        let secs = days * 86400;
        let dur = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let total = dur.as_secs() + secs as u64;
        // Convert to ISO-like string
        let days_from_epoch = total / 86400;
        let time_secs = total % 86400;
        let hours = time_secs / 3600;
        let minutes = (time_secs % 3600) / 60;
        let seconds = time_secs % 60;
        let (year, month, day) = days_to_date(days_from_epoch as i64);
        format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, hours, minutes, seconds)
    });

    let key = create_api_key(store, &prefix, &hash, name, project_id, permissions, expires_at);
    (api_key, prefix, key.id)
}

fn days_to_date(mut days: i64) -> (i64, i64, i64) {
    days += 719468;
    let era = if days >= 0 { days } else { days - 146096 };
    let era_days = era.rem_euclid(146097);
    let year_era = (era_days - era_days / 1460 + era_days / 36524 - era_days / 146096) / 365;
    let y = year_era + era / 146097 * 400;
    let doy = era_days - (365 * year_era + year_era / 4 - year_era / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    let y = if y <= 0 { y - 1 } else { y };
    (y, m, d)
}

pub fn authenticate(store: &SharedStore, auth_header: Option<&str>) -> Result<(Uuid, String, Vec<String>), String> {
    let header = auth_header.ok_or_else(|| "missing authorization".to_string())?;

    // Try API key
    let api_key = header.strip_prefix("Bearer ")
        .or_else(|| Some(header)) // also try raw api key
        .ok_or_else(|| "invalid authorization format".to_string())?;

    if api_key.len() < 8 {
        return Err("invalid api key".to_string());
    }
    let prefix = &api_key[..8];

    let stored = get_api_key_by_prefix(store, prefix)
        .ok_or_else(|| "invalid api key".to_string())?;

    if !verify_api_key(api_key, &stored.key_hash) {
        return Err("invalid api key".to_string());
    }

    // Check expiration
    if let Some(ref expires) = stored.expires_at {
        // Simple string comparison on ISO dates
        let now = {
            let dur = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let days_from_epoch = dur.as_secs() / 86400;
            let time_secs = dur.as_secs() % 86400;
            let hours = time_secs / 3600;
            let minutes = (time_secs % 3600) / 60;
            let seconds = time_secs % 60;
            let (y, m, d) = days_to_date(days_from_epoch as i64);
            format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, hours, minutes, seconds)
        };
        if now > *expires {
            return Err("api key expired".to_string());
        }
    }

    Ok((stored.id, stored.key_prefix, stored.permissions))
}
