#![forbid(unsafe_code)]

use std::collections::{BTreeSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LimiterConfig {
    pub max_concurrent_fetches: usize,
    pub max_queue_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmissionResult {
    Started,
    Queued,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcurrencyLimiter {
    config: LimiterConfig,
    in_flight: BTreeSet<String>,
    queue: VecDeque<String>,
}

impl ConcurrencyLimiter {
    pub fn new(config: LimiterConfig) -> Result<Self, String> {
        if config.max_concurrent_fetches == 0 {
            return Err("max_concurrent_fetches must be >= 1".to_string());
        }
        Ok(Self {
            config,
            in_flight: BTreeSet::new(),
            queue: VecDeque::new(),
        })
    }

    pub fn submit(&mut self, task_id: String) -> Result<SubmissionResult, &'static str> {
        if self.in_flight.contains(task_id.as_str()) || self.queue.iter().any(|id| id == &task_id) {
            return Ok(SubmissionResult::Queued);
        }

        if self.in_flight.len() < self.config.max_concurrent_fetches {
            self.in_flight.insert(task_id);
            return Ok(SubmissionResult::Started);
        }

        if self.queue.len() >= self.config.max_queue_len {
            return Err("quota_exceeded");
        }

        self.queue.push_back(task_id);
        Ok(SubmissionResult::Queued)
    }

    pub fn complete(&mut self, task_id: &str) -> Option<String> {
        if !self.in_flight.remove(task_id) {
            return None;
        }

        let next = self.queue.pop_front()?;
        self.in_flight.insert(next.clone());
        Some(next)
    }

    pub fn in_flight_len(&self) -> usize {
        self.in_flight.len()
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    pub fn queue_snapshot(&self) -> Vec<String> {
        self.queue.iter().cloned().collect()
    }
}
