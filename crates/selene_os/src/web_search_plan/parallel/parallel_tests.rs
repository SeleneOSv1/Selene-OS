#![forbid(unsafe_code)]

use crate::web_search_plan::parallel::join::join_in_planned_order;
use crate::web_search_plan::parallel::limiter::{ConcurrencyLimiter, LimiterConfig, SubmissionResult};
use crate::web_search_plan::parallel::merge_order::{merge_completed_by_plan, merge_lead_then_fallback, MergeItem};
use crate::web_search_plan::parallel::scheduler::{schedule_deterministically, RetrievalTask};
use std::collections::BTreeMap;

#[test]
fn test_t6_scheduler_enforces_concurrency_cap() {
    let mut limiter = ConcurrencyLimiter::new(LimiterConfig {
        max_concurrent_fetches: 2,
        max_queue_len: 4,
    })
    .expect("limiter config should be valid");

    assert_eq!(limiter.submit("task-a".to_string()), Ok(SubmissionResult::Started));
    assert_eq!(limiter.submit("task-b".to_string()), Ok(SubmissionResult::Started));
    assert_eq!(limiter.submit("task-c".to_string()), Ok(SubmissionResult::Queued));

    assert_eq!(limiter.in_flight_len(), 2);
    assert_eq!(limiter.queue_snapshot(), vec!["task-c".to_string()]);

    let resumed = limiter.complete("task-a");
    assert_eq!(resumed.as_deref(), Some("task-c"));
    assert_eq!(limiter.in_flight_len(), 2);
    assert_eq!(limiter.queue_len(), 0);
}

#[test]
fn test_t7_queue_cap_enforced_deterministically() {
    let mut limiter = ConcurrencyLimiter::new(LimiterConfig {
        max_concurrent_fetches: 1,
        max_queue_len: 1,
    })
    .expect("limiter config should be valid");

    assert_eq!(limiter.submit("task-a".to_string()), Ok(SubmissionResult::Started));
    assert_eq!(limiter.submit("task-b".to_string()), Ok(SubmissionResult::Queued));
    assert_eq!(limiter.submit("task-c".to_string()), Err("quota_exceeded"));
    assert_eq!(limiter.queue_snapshot(), vec!["task-b".to_string()]);
}

#[test]
fn test_t8_merge_order_stable_regardless_of_completion_timing() {
    let tasks = vec![
        RetrievalTask {
            task_id: "b".to_string(),
            priority: 2,
            canonical_url: "https://b.example.com".to_string(),
            provider_id: "brave_web_search".to_string(),
            task_type: "web".to_string(),
        },
        RetrievalTask {
            task_id: "a".to_string(),
            priority: 1,
            canonical_url: "https://a.example.com".to_string(),
            provider_id: "brave_web_search".to_string(),
            task_type: "web".to_string(),
        },
        RetrievalTask {
            task_id: "c".to_string(),
            priority: 2,
            canonical_url: "https://c.example.com".to_string(),
            provider_id: "openai_web_search".to_string(),
            task_type: "web".to_string(),
        },
    ];

    let planned = schedule_deterministically(tasks)
        .into_iter()
        .map(|task| task.task_id)
        .collect::<Vec<String>>();
    assert_eq!(planned, vec!["a", "b", "c"]);

    let mut completed = BTreeMap::new();
    completed.insert("c".to_string(), 3);
    completed.insert("a".to_string(), 1);
    completed.insert("b".to_string(), 2);

    let joined = join_in_planned_order(&planned, &completed);
    assert_eq!(joined, vec![1, 2, 3]);
}

#[test]
fn test_t9_cache_hit_vs_network_hit_does_not_change_final_ordering() {
    let lead = vec![
        MergeItem {
            canonical_url: "https://a.example.com".to_string(),
            payload: "lead-a".to_string(),
        },
        MergeItem {
            canonical_url: "https://b.example.com".to_string(),
            payload: "lead-b".to_string(),
        },
    ];

    let fallback_network = vec![
        MergeItem {
            canonical_url: "https://b.example.com".to_string(),
            payload: "fallback-b".to_string(),
        },
        MergeItem {
            canonical_url: "https://c.example.com".to_string(),
            payload: "fallback-c".to_string(),
        },
    ];

    let fallback_cache = vec![
        MergeItem {
            canonical_url: "https://c.example.com".to_string(),
            payload: "fallback-c-cached".to_string(),
        },
        MergeItem {
            canonical_url: "https://b.example.com".to_string(),
            payload: "fallback-b-cached".to_string(),
        },
    ];

    let merged_network = merge_lead_then_fallback(&lead, &fallback_network)
        .into_iter()
        .map(|item| item.canonical_url)
        .collect::<Vec<String>>();

    let merged_cache = merge_lead_then_fallback(&lead, &fallback_cache)
        .into_iter()
        .map(|item| item.canonical_url)
        .collect::<Vec<String>>();

    assert_eq!(merged_network, vec![
        "https://a.example.com".to_string(),
        "https://b.example.com".to_string(),
        "https://c.example.com".to_string(),
    ]);
    assert_eq!(merged_cache, merged_network);

    let planned = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let mut completed = BTreeMap::new();
    completed.insert(
        "c".to_string(),
        MergeItem {
            canonical_url: "https://c.example.com".to_string(),
            payload: "cache-hit".to_string(),
        },
    );
    completed.insert(
        "a".to_string(),
        MergeItem {
            canonical_url: "https://a.example.com".to_string(),
            payload: "network-hit".to_string(),
        },
    );
    completed.insert(
        "b".to_string(),
        MergeItem {
            canonical_url: "https://b.example.com".to_string(),
            payload: "network-hit".to_string(),
        },
    );

    let merged = merge_completed_by_plan(&planned, &completed)
        .into_iter()
        .map(|item| item.canonical_url)
        .collect::<Vec<String>>();
    assert_eq!(merged, vec![
        "https://a.example.com".to_string(),
        "https://b.example.com".to_string(),
        "https://c.example.com".to_string(),
    ]);
}
