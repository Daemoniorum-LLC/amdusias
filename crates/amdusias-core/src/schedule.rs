//! Sample-accurate event scheduling for automation and MIDI.

use alloc::{collections::BTreeMap, vec::Vec};
use core::sync::atomic::{AtomicU64, Ordering};

/// Sample position in the timeline (absolute).
pub type SamplePosition = u64;

/// A scheduled event with associated data.
#[derive(Debug, Clone)]
pub struct ScheduledEvent<T> {
    /// The sample position when this event should trigger.
    pub position: SamplePosition,
    /// The event data.
    pub data: T,
}

impl<T> ScheduledEvent<T> {
    /// Creates a new scheduled event.
    #[must_use]
    pub const fn new(position: SamplePosition, data: T) -> Self {
        Self { position, data }
    }
}

/// Sample-accurate event scheduler.
///
/// Events are stored in a sorted structure and can be queried by time range.
/// The scheduler is designed to be updated from a non-audio thread and
/// queried from the audio thread.
///
/// # Design
///
/// - Events are stored in a `BTreeMap` for efficient range queries
/// - The current position is tracked atomically
/// - Events in the past are automatically skipped
pub struct Scheduler<T> {
    /// Scheduled events, sorted by position.
    events: BTreeMap<SamplePosition, Vec<T>>,
    /// Current playback position.
    current_position: AtomicU64,
}

impl<T> Default for Scheduler<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Scheduler<T> {
    /// Creates a new scheduler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            events: BTreeMap::new(),
            current_position: AtomicU64::new(0),
        }
    }

    /// Returns the current playback position.
    #[inline]
    #[must_use]
    pub fn position(&self) -> SamplePosition {
        self.current_position.load(Ordering::Relaxed)
    }

    /// Sets the current playback position.
    #[inline]
    pub fn set_position(&self, position: SamplePosition) {
        self.current_position.store(position, Ordering::Relaxed);
    }

    /// Advances the position by the given number of samples.
    #[inline]
    pub fn advance(&self, samples: u64) {
        self.current_position.fetch_add(samples, Ordering::Relaxed);
    }

    /// Schedules an event at the given position.
    ///
    /// Multiple events can be scheduled at the same position.
    pub fn schedule(&mut self, position: SamplePosition, event: T) {
        self.events.entry(position).or_default().push(event);
    }

    /// Schedules an event relative to the current position.
    pub fn schedule_relative(&mut self, offset: u64, event: T) {
        let position = self.position() + offset;
        self.schedule(position, event);
    }

    /// Returns events in the given range [start, end).
    ///
    /// This is the primary query method for the audio thread.
    pub fn events_in_range(
        &self,
        start: SamplePosition,
        end: SamplePosition,
    ) -> impl Iterator<Item = (SamplePosition, &T)> {
        self.events
            .range(start..end)
            .flat_map(|(&pos, events)| events.iter().map(move |e| (pos, e)))
    }

    /// Removes and returns all events before the given position.
    ///
    /// Call this periodically to clean up processed events.
    pub fn drain_before(&mut self, position: SamplePosition) -> Vec<(SamplePosition, T)> {
        let mut result = Vec::new();

        // Collect keys to remove
        let keys_to_remove: Vec<_> = self
            .events
            .range(..position)
            .map(|(&k, _)| k)
            .collect();

        for key in keys_to_remove {
            if let Some(events) = self.events.remove(&key) {
                for event in events {
                    result.push((key, event));
                }
            }
        }

        result
    }

    /// Clears all scheduled events.
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Returns the number of scheduled events.
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.values().map(Vec::len).sum()
    }

    /// Returns true if there are no scheduled events.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Automation point for parameter changes.
#[derive(Debug, Clone, Copy)]
pub struct AutomationPoint {
    /// Target value.
    pub value: f32,
    /// Interpolation curve type.
    pub curve: AutomationCurve,
}

/// Automation curve types.
#[derive(Debug, Clone, Copy, Default)]
pub enum AutomationCurve {
    /// Instant jump to value.
    #[default]
    Step,
    /// Linear interpolation.
    Linear,
    /// Exponential curve (good for volume/frequency).
    Exponential,
    /// S-curve (smooth transitions).
    SCurve,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_and_query() {
        let mut scheduler = Scheduler::new();

        scheduler.schedule(100, "event1");
        scheduler.schedule(200, "event2");
        scheduler.schedule(150, "event3");

        let events: Vec<_> = scheduler.events_in_range(0, 175).collect();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], (100, &"event1"));
        assert_eq!(events[1], (150, &"event3"));
    }

    #[test]
    fn test_drain_before() {
        let mut scheduler = Scheduler::new();

        scheduler.schedule(100, 1);
        scheduler.schedule(200, 2);
        scheduler.schedule(300, 3);

        let drained = scheduler.drain_before(250);
        assert_eq!(drained.len(), 2);
        assert_eq!(scheduler.len(), 1);
    }

    #[test]
    fn test_sample_accurate_timing() {
        let mut scheduler = Scheduler::new();

        // Schedule events at exact sample positions
        scheduler.schedule(0, "sample_0");
        scheduler.schedule(1, "sample_1");
        scheduler.schedule(255, "sample_255");
        scheduler.schedule(256, "sample_256");

        // Query exactly one sample
        let events: Vec<_> = scheduler.events_in_range(0, 1).collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, 0);

        // Query sample 256 buffer (0..256)
        let events: Vec<_> = scheduler.events_in_range(0, 256).collect();
        assert_eq!(events.len(), 3, "Should include samples 0, 1, 255 but not 256");

        // Query next buffer (256..512)
        let events: Vec<_> = scheduler.events_in_range(256, 512).collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, 256);
    }

    #[test]
    fn test_multiple_events_same_position() {
        let mut scheduler = Scheduler::new();

        // Schedule multiple events at the same sample
        scheduler.schedule(100, "note_on_c4");
        scheduler.schedule(100, "note_on_e4");
        scheduler.schedule(100, "note_on_g4");

        let events: Vec<_> = scheduler.events_in_range(100, 101).collect();
        assert_eq!(events.len(), 3);

        // All should be at position 100
        for (pos, _) in &events {
            assert_eq!(*pos, 100);
        }
    }

    #[test]
    fn test_ordering_preserved() {
        let mut scheduler = Scheduler::new();

        // Events at same position should maintain insertion order
        scheduler.schedule(100, "first");
        scheduler.schedule(100, "second");
        scheduler.schedule(100, "third");

        let events: Vec<_> = scheduler.events_in_range(100, 101).collect();
        assert_eq!(events[0].1, &"first");
        assert_eq!(events[1].1, &"second");
        assert_eq!(events[2].1, &"third");
    }

    #[test]
    fn test_position_tracking() {
        let scheduler = Scheduler::<()>::new();

        assert_eq!(scheduler.position(), 0);

        scheduler.set_position(1000);
        assert_eq!(scheduler.position(), 1000);

        scheduler.advance(256);
        assert_eq!(scheduler.position(), 1256);

        scheduler.advance(256);
        assert_eq!(scheduler.position(), 1512);
    }

    #[test]
    fn test_schedule_relative() {
        let mut scheduler = Scheduler::new();

        scheduler.set_position(1000);
        scheduler.schedule_relative(100, "event");

        let events: Vec<_> = scheduler.events_in_range(1100, 1101).collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, 1100);
    }

    #[test]
    fn test_empty_range_query() {
        let mut scheduler = Scheduler::new();

        scheduler.schedule(100, "event");

        // Query empty range
        let events: Vec<_> = scheduler.events_in_range(200, 300).collect();
        assert!(events.is_empty());

        // Query range before any events
        let events: Vec<_> = scheduler.events_in_range(0, 50).collect();
        assert!(events.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut scheduler = Scheduler::new();

        scheduler.schedule(100, 1);
        scheduler.schedule(200, 2);
        scheduler.schedule(300, 3);

        assert_eq!(scheduler.len(), 3);
        assert!(!scheduler.is_empty());

        scheduler.clear();

        assert_eq!(scheduler.len(), 0);
        assert!(scheduler.is_empty());
    }

    #[test]
    fn test_scheduled_event_struct() {
        let event = ScheduledEvent::new(12345, "data");
        assert_eq!(event.position, 12345);
        assert_eq!(event.data, "data");
    }

    #[test]
    fn test_automation_point() {
        let point = AutomationPoint {
            value: 0.75,
            curve: AutomationCurve::Linear,
        };
        assert_eq!(point.value, 0.75);
    }

    #[test]
    fn test_large_timeline() {
        let mut scheduler = Scheduler::new();

        // Schedule events across a large timeline (simulating hours of audio)
        // 48000 samples/sec * 3600 sec = 172,800,000 samples per hour
        let hour_in_samples: u64 = 48000 * 3600;

        scheduler.schedule(0, "start");
        scheduler.schedule(hour_in_samples, "one_hour");
        scheduler.schedule(hour_in_samples * 2, "two_hours");

        // Query around the one hour mark
        let events: Vec<_> = scheduler
            .events_in_range(hour_in_samples - 1, hour_in_samples + 1)
            .collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].1, &"one_hour");
    }

    #[test]
    fn test_drain_maintains_remaining() {
        let mut scheduler = Scheduler::new();

        scheduler.schedule(100, "a");
        scheduler.schedule(200, "b");
        scheduler.schedule(300, "c");
        scheduler.schedule(400, "d");
        scheduler.schedule(500, "e");

        // Drain first two
        let drained = scheduler.drain_before(250);
        assert_eq!(drained.len(), 2);

        // Remaining should still be queryable
        let events: Vec<_> = scheduler.events_in_range(0, 1000).collect();
        assert_eq!(events.len(), 3);

        // Drain one more
        let drained = scheduler.drain_before(350);
        assert_eq!(drained.len(), 1);
        assert_eq!(scheduler.len(), 2);
    }
}
