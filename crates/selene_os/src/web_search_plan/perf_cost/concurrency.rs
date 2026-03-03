#![forbid(unsafe_code)]

use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmitDecision {
    Started,
    Queued,
    Rejected { reason_code: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcurrencyController {
    max_concurrent_fetches: usize,
    max_queue_len: usize,
    active: Vec<String>,
    queue: VecDeque<String>,
    concurrency_peak: usize,
}

impl ConcurrencyController {
    pub fn new(max_concurrent_fetches: usize, max_queue_len: usize) -> Result<Self, String> {
        if max_concurrent_fetches == 0 {
            return Err("max_concurrent_fetches must be >= 1".to_string());
        }

        Ok(Self {
            max_concurrent_fetches,
            max_queue_len,
            active: Vec::new(),
            queue: VecDeque::new(),
            concurrency_peak: 0,
        })
    }

    pub fn submit(&mut self, task_id: &str) -> SubmitDecision {
        let trimmed = task_id.trim();
        if trimmed.is_empty() {
            return SubmitDecision::Rejected {
                reason_code: "quota_exceeded",
            };
        }

        if self.active.iter().any(|id| id == trimmed) {
            return SubmitDecision::Started;
        }
        if self.queue.iter().any(|id| id == trimmed) {
            return SubmitDecision::Queued;
        }

        if self.active.len() < self.max_concurrent_fetches {
            self.active.push(trimmed.to_string());
            self.concurrency_peak = self.concurrency_peak.max(self.active.len());
            return SubmitDecision::Started;
        }

        if self.queue.len() >= self.max_queue_len {
            return SubmitDecision::Rejected {
                reason_code: "quota_exceeded",
            };
        }

        self.queue.push_back(trimmed.to_string());
        SubmitDecision::Queued
    }

    pub fn complete(&mut self, task_id: &str) -> Option<String> {
        let trimmed = task_id.trim();
        if trimmed.is_empty() {
            return None;
        }

        if let Some(position) = self.active.iter().position(|value| value == trimmed) {
            self.active.remove(position);
        } else {
            return None;
        }

        let next = self.queue.pop_front();
        if let Some(next_task) = next.clone() {
            self.active.push(next_task);
            self.concurrency_peak = self.concurrency_peak.max(self.active.len());
        }
        next
    }

    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    pub const fn concurrency_peak(&self) -> usize {
        self.concurrency_peak
    }
}
