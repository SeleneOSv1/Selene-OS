#![forbid(unsafe_code)]

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenBudgetPolicy {
    pub max_urls_opened_per_query: usize,
    pub per_domain_cap: usize,
    pub max_total_extracted_chars: usize,
    pub max_chunks_total: usize,
}

impl Default for OpenBudgetPolicy {
    fn default() -> Self {
        Self {
            max_urls_opened_per_query: 3,
            per_domain_cap: 2,
            max_total_extracted_chars: 120_000,
            max_chunks_total: 192,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetControl {
    policy: OpenBudgetPolicy,
    pub successful_opens: usize,
    pub selected_per_domain: BTreeMap<String, usize>,
    pub total_extracted_chars: usize,
    pub total_chunks: usize,
}

impl BudgetControl {
    pub fn new(policy: OpenBudgetPolicy) -> Self {
        Self {
            policy,
            successful_opens: 0,
            selected_per_domain: BTreeMap::new(),
            total_extracted_chars: 0,
            total_chunks: 0,
        }
    }

    pub fn policy(&self) -> &OpenBudgetPolicy {
        &self.policy
    }

    pub fn can_select_domain(&self, domain: &str) -> bool {
        self.selected_per_domain.get(domain).copied().unwrap_or(0) < self.policy.per_domain_cap
    }

    pub fn record_selection(&mut self, domain: &str) {
        let counter = self
            .selected_per_domain
            .entry(domain.to_string())
            .or_insert(0);
        *counter = counter.saturating_add(1);
    }

    pub fn can_accept_success(&self, extracted_chars: usize, chunk_count: usize) -> bool {
        self.total_extracted_chars.saturating_add(extracted_chars)
            <= self.policy.max_total_extracted_chars
            && self.total_chunks.saturating_add(chunk_count) <= self.policy.max_chunks_total
    }

    pub fn record_success(&mut self, extracted_chars: usize, chunk_count: usize) {
        self.successful_opens = self.successful_opens.saturating_add(1);
        self.total_extracted_chars = self.total_extracted_chars.saturating_add(extracted_chars);
        self.total_chunks = self.total_chunks.saturating_add(chunk_count);
    }

    pub fn reached_open_limit(&self) -> bool {
        self.successful_opens >= self.policy.max_urls_opened_per_query
    }

    pub fn exhausted_structural_budget(&self) -> bool {
        self.total_extracted_chars >= self.policy.max_total_extracted_chars
            || self.total_chunks >= self.policy.max_chunks_total
    }
}
