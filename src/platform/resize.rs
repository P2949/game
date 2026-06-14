use std::time::{Duration, Instant};

/// Debounce window after the most recent resize event before a swapchain
/// recreate is allowed. Resizing can generate hundreds of transient drawable
/// sizes, so wait for a stable size before doing the (final, exact-fit) recreate.
/// We now keep rendering with the current swapchain throughout the resize, so
/// this only governs the clean settle-recreate and can stay snappy.
pub const DEFAULT_RESIZE_DEBOUNCE: Duration = Duration::from_millis(200);
/// Minimum spacing between consecutive recreates, so a slow continuous drag (or
/// a driver that keeps reporting suboptimal) cannot trigger a recreate storm.
pub const DEFAULT_MIN_RECREATE_INTERVAL: Duration = Duration::from_millis(350);
/// Drawable sizes below this are usually drag-through transients. Let them
/// settle for longer before recreating, but still allow them eventually so a
/// deliberately tiny window is not frozen forever.
pub const MIN_STABLE_RECREATE_SIZE: (u32, u32) = (320, 180);
pub const TINY_RESIZE_SETTLE: Duration = Duration::from_millis(750);

/// Pure timing policy for when a debounced window resize should trigger a
/// swapchain recreate. Holds no Vulkan/SDL state, so it can be unit-tested by
/// feeding explicit [`Instant`]s.
///
/// Zero-size (minimized) windows are handled by the caller before this is
/// consulted. Tiny nonzero windows use a longer settle time, which filters out
/// resize-drag intermediates without permanently stranding a real small window.
#[derive(Debug, Clone)]
pub struct ResizePolicy {
    debounce: Duration,
    min_recreate_interval: Duration,
    last_resize_event: Option<Instant>,
    last_recreate: Option<Instant>,
}

impl ResizePolicy {
    pub fn new(debounce: Duration, min_recreate_interval: Duration) -> Self {
        Self {
            debounce,
            min_recreate_interval,
            last_resize_event: None,
            last_recreate: None,
        }
    }

    /// Records that the drawable size changed, restarting the debounce window.
    pub fn note_resize(&mut self, now: Instant) {
        self.last_resize_event = Some(now);
    }

    /// Records that a recreate just happened, starting the rate-limit window.
    pub fn note_recreate(&mut self, now: Instant) {
        self.last_recreate = Some(now);
    }

    /// Whether a recreate is allowed at `now`: the debounce window since the last
    /// resize event has elapsed and the rate-limit since the last recreate has
    /// elapsed. Tiny drawable sizes use a longer settle time. With no pending
    /// resize event yet, recreation is always allowed.
    pub fn recreate_ready(&self, now: Instant, drawable_size: (u32, u32)) -> bool {
        let Some(last_resize_event) = self.last_resize_event else {
            return true;
        };

        if now.duration_since(last_resize_event) < self.debounce_for_size(drawable_size) {
            return false;
        }

        if let Some(last_recreate) = self.last_recreate
            && now.duration_since(last_recreate) < self.min_recreate_interval
        {
            return false;
        }

        true
    }

    fn debounce_for_size(&self, drawable_size: (u32, u32)) -> Duration {
        if drawable_size.0 < MIN_STABLE_RECREATE_SIZE.0
            || drawable_size.1 < MIN_STABLE_RECREATE_SIZE.1
        {
            TINY_RESIZE_SETTLE
        } else {
            self.debounce
        }
    }
}

impl Default for ResizePolicy {
    fn default() -> Self {
        Self::new(DEFAULT_RESIZE_DEBOUNCE, DEFAULT_MIN_RECREATE_INTERVAL)
    }
}

#[cfg(test)]
mod tests {
    use super::{ResizePolicy, TINY_RESIZE_SETTLE};
    use std::time::{Duration, Instant};

    const NORMAL_SIZE: (u32, u32) = (800, 600);
    const TINY_SIZE: (u32, u32) = (160, 90);

    fn policy() -> ResizePolicy {
        ResizePolicy::new(Duration::from_millis(200), Duration::from_millis(350))
    }

    #[test]
    fn ready_when_no_resize_event_recorded() {
        assert!(policy().recreate_ready(Instant::now(), NORMAL_SIZE));
    }

    #[test]
    fn not_ready_within_debounce_window() {
        let start = Instant::now();
        let mut p = policy();
        p.note_resize(start);
        assert!(!p.recreate_ready(start + Duration::from_millis(100), NORMAL_SIZE));
    }

    #[test]
    fn ready_after_debounce_with_no_prior_recreate() {
        let start = Instant::now();
        let mut p = policy();
        p.note_resize(start);
        assert!(p.recreate_ready(start + Duration::from_millis(250), NORMAL_SIZE));
    }

    #[test]
    fn rate_limited_after_a_recent_recreate() {
        let start = Instant::now();
        let mut p = policy();
        p.note_resize(start);
        p.note_recreate(start + Duration::from_millis(250));
        // Debounce satisfied, but only 100ms since the last recreate (< 350ms).
        assert!(!p.recreate_ready(start + Duration::from_millis(350), NORMAL_SIZE));
        // Once the rate-limit window passes, it is ready again.
        assert!(p.recreate_ready(start + Duration::from_millis(650), NORMAL_SIZE));
    }

    #[test]
    fn rapid_resize_events_are_coalesced() {
        let start = Instant::now();
        let mut p = policy();
        p.note_resize(start);
        // A later resize event restarts the debounce window.
        p.note_resize(start + Duration::from_millis(150));
        assert!(!p.recreate_ready(start + Duration::from_millis(300), NORMAL_SIZE));
        assert!(p.recreate_ready(start + Duration::from_millis(360), NORMAL_SIZE));
    }

    #[test]
    fn tiny_resize_intermediates_wait_longer() {
        let start = Instant::now();
        let mut p = policy();
        p.note_resize(start);

        assert!(!p.recreate_ready(start + Duration::from_millis(500), TINY_SIZE));
        assert!(p.recreate_ready(start + TINY_RESIZE_SETTLE, TINY_SIZE));
    }
}
