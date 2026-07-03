//! Temporary, zero-dependency instrumentation, enabled with `KRACKO_PROF=1`.

use std::sync::OnceLock;
use std::time::Instant;

pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("KRACKO_PROF").is_some())
}

fn epoch() -> Instant {
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    *EPOCH.get_or_init(Instant::now)
}

/// Logs `label` with a session-relative timestamp and the duration of the
/// measured section.
pub struct Span {
    label: String,
    started: Option<Instant>,
}

impl Span {
    pub fn start(label: impl Into<String>) -> Self {
        let started = enabled().then(Instant::now);
        Self {
            label: label.into(),
            started,
        }
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        if let Some(started) = self.started {
            eprintln!(
                "[prof {:>10.3}ms] {} took {:.3}ms",
                epoch().elapsed().as_secs_f64() * 1000.0,
                self.label,
                started.elapsed().as_secs_f64() * 1000.0,
            );
        }
    }
}

/// Logs a one-off event with a session-relative timestamp.
pub fn mark(label: impl AsRef<str>) {
    if enabled() {
        eprintln!(
            "[prof {:>10.3}ms] {}",
            epoch().elapsed().as_secs_f64() * 1000.0,
            label.as_ref(),
        );
    }
}
