use std::time::Instant;
use std::sync::atomic::{AtomicU64, Ordering};

static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);

pub fn init_telemetry() {
    eprintln!("cirbinius-api: telemetry initialized (minimal mode)");
}

pub fn startup_msg(msg: &str) {
    eprintln!("cirbinius-api: {msg}");
}

pub fn record_request() {
    REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn request_count() -> u64 {
    REQUEST_COUNT.load(Ordering::Relaxed)
}

pub struct TimingScope {
    name: &'static str,
    start: Instant,
}

impl TimingScope {
    pub fn start(name: &'static str) -> Self {
        Self { name, start: Instant::now() }
    }
}

impl Drop for TimingScope {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        if elapsed.as_millis() > 100 {
            eprintln!("cirbinius-api: timing [{}] took {}ms", self.name, elapsed.as_millis());
        }
    }
}
