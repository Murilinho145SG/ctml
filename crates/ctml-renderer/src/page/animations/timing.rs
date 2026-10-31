use std::time::{Duration, Instant};

use crate::style::TransitionEasing;

use super::OpacityTransition;

pub(super) fn start_transition(
    from: f32,
    to: f32,
    enabled: bool,
    duration_ms: f32,
    delay_ms: f32,
    easing: TransitionEasing,
) -> Option<OpacityTransition> {
    if !enabled || duration_ms <= 0.0 || (from - to).abs() <= f32::EPSILON {
        return None;
    }
    let duration = duration_from_millis(duration_ms);
    let delay = duration_from_millis(delay_ms.abs());
    let now = Instant::now();
    let starts_at = if delay_ms < 0.0 {
        now.checked_sub(delay).unwrap_or(now)
    } else {
        now.checked_add(delay).unwrap_or(now)
    };
    let transition = OpacityTransition {
        from,
        to,
        starts_at,
        duration,
        easing,
    };
    (now < starts_at + duration).then_some(transition)
}

fn duration_from_millis(milliseconds: f32) -> Duration {
    let seconds = f64::from(milliseconds.max(0.0)) / 1000.0;
    let duration = Duration::from_secs_f64(seconds.min(315_360_000.0));
    if milliseconds > 0.0 && duration.is_zero() {
        Duration::from_nanos(1)
    } else {
        duration
    }
}
