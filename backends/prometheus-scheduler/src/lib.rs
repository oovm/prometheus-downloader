//! Download task scheduler.
//!
//! Persistence is not implemented yet.

#![deny(missing_docs)]

use prometheus_types::Result;

/// Task states used by future queue implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Queued but not started.
    Pending,
    /// Actively downloading.
    Running,
    /// Finished successfully.
    Completed,
    /// Failed.
    Failed,
}

/// No-op scheduler reserved for future queue wiring.
#[derive(Debug, Default)]
pub struct Scheduler;

impl Scheduler {
    /// Create an empty in-memory scheduler.
    pub fn new() -> Self {
        Self
    }

    /// Current task count (always zero until persistence lands).
    pub fn len(&self) -> usize {
        0
    }

    /// Whether the scheduler has no tasks.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reserved hook so callers can depend on a fallible API shape.
    pub fn ping(&self) -> Result<()> {
        Ok(())
    }
}
