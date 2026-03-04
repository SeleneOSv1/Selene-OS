#![forbid(unsafe_code)]

pub mod analytics;
pub mod cache;
pub mod chunk;
pub mod competitive;
pub mod contract_hash;
pub mod diag;
pub mod document;
pub mod eval;
pub mod idempotency_validator;
pub mod learn;
pub mod learning;
pub mod merge;
pub mod multihop;
pub mod news;
pub mod news_provider;
pub mod packet_validator;
pub mod parallel;
pub mod perf_cost;
pub mod planning;
pub mod proxy;
pub mod realtime;
pub mod reason_code_validator;
pub mod regulatory;
pub mod release;
pub mod registry_loader;
pub mod replay;
pub mod risk;
pub mod structured;
pub mod synthesis;
pub mod temporal;
pub mod trust;
pub mod turn_state_machine_validator;
pub mod url;
pub mod vision;
pub mod web_provider;
pub mod write;

#[cfg(test)]
mod tests;
