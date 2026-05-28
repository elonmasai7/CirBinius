use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    inner: Mutex<Inner>,
}

struct Inner {
    requests: HashMap<String, Vec<Instant>>,
    max_per_minute: usize,
    max_per_hour: usize,
}

impl RateLimiter {
    pub fn new(max_per_minute: usize, max_per_hour: usize) -> Self {
        Self {
            inner: Mutex::new(Inner {
                requests: HashMap::new(),
                max_per_minute,
                max_per_hour,
            }),
        }
    }

    pub fn check(&self, key: &str) -> Result<(), &'static str> {
        let now = Instant::now();
        let mut inner = self.inner.lock().unwrap();
        let max_min = inner.max_per_minute;
        let max_hour = inner.max_per_hour;
        let entries = inner.requests.entry(key.to_string()).or_default();

        entries.retain(|t| now.duration_since(*t) < Duration::from_secs(60));
        if entries.len() >= max_min {
            return Err("rate limit exceeded: per minute");
        }

        let hour_ago = now - Duration::from_secs(3600);
        let hour_count = entries.iter().filter(|t| **t > hour_ago).count();
        if hour_count >= max_hour {
            return Err("rate limit exceeded: per hour");
        }

        entries.push(now);
        Ok(())
    }
}
