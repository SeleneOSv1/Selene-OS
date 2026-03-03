#![forbid(unsafe_code)]

pub mod asof;
pub mod change_classify;
pub mod diff;
pub mod temporal_packet;
pub mod timeline;

pub use asof::{
    filter_rows_for_window, resolve_asof_windows, AsOfResolutionError, AsOfWindow, AsOfWindowInput,
    MissingTimestampPolicy, WindowFilterResult, DEFAULT_WINDOW_MS,
};
pub use diff::{build_changes, ChangeItem, DiffBuildResult};
pub use temporal_packet::{
    append_temporal_audit_metadata, build_temporal_comparison_packet, TemporalBuildError,
    TemporalBuildOutput, TemporalComparisonPacket, TemporalRequest, TEMPORAL_ENGINE_ID,
    TEMPORAL_SCHEMA_VERSION,
};
pub use timeline::{build_timeline_events, TemporalValue, TimelineEvent};

#[cfg(test)]
pub mod temporal_tests;
