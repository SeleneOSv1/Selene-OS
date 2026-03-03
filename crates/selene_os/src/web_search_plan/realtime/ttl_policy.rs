#![forbid(unsafe_code)]

use crate::web_search_plan::perf_cost::tiers::ImportanceTier;
use crate::web_search_plan::realtime::domains::RealtimeDomain;

pub const REALTIME_TTL_POLICY_VERSION: &str = "1.0.0";

pub fn ttl_ms(domain: RealtimeDomain, tier: ImportanceTier) -> u64 {
    match (domain, tier) {
        (RealtimeDomain::Weather, ImportanceTier::Low) => 60 * 60 * 1_000,
        (RealtimeDomain::Weather, ImportanceTier::Medium) => 30 * 60 * 1_000,
        (RealtimeDomain::Weather, ImportanceTier::High) => 15 * 60 * 1_000,

        (RealtimeDomain::Finance, ImportanceTier::Low) => 30 * 60 * 1_000,
        (RealtimeDomain::Finance, ImportanceTier::Medium) => 15 * 60 * 1_000,
        (RealtimeDomain::Finance, ImportanceTier::High) => 5 * 60 * 1_000,

        (RealtimeDomain::Flights, ImportanceTier::Low) => 30 * 60 * 1_000,
        (RealtimeDomain::Flights, ImportanceTier::Medium) => 15 * 60 * 1_000,
        (RealtimeDomain::Flights, ImportanceTier::High) => 5 * 60 * 1_000,

        (RealtimeDomain::GenericRealTime, ImportanceTier::Low) => 30 * 60 * 1_000,
        (RealtimeDomain::GenericRealTime, ImportanceTier::Medium) => 15 * 60 * 1_000,
        (RealtimeDomain::GenericRealTime, ImportanceTier::High) => 5 * 60 * 1_000,
    }
}
