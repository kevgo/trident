use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const JITTER_MIN_MS: u64 = 10;
const JITTER_MAX_MS: u64 = 1000;

/// whether to wait in `jitter`
static ENABLED: AtomicBool = AtomicBool::new(true);

/// disables jittering, for example when scenarios run one by one
pub(crate) fn disable() {
    ENABLED.store(false, Ordering::Relaxed);
}

/// waits a random duration
///
/// to be used before steps that cause lots of file system activity
/// to avoid overwhelming the file system with too many concurrent operations
pub(crate) async fn jitter() {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    let duration = Duration::from_millis(rand::random_range(JITTER_MIN_MS..JITTER_MAX_MS));
    tokio::time::sleep(duration).await;
}
