#![forbid(unsafe_code)]

pub mod debug_packet;
pub mod error_taxonomy;
pub mod redaction;
pub mod state_trace;

pub use debug_packet::{
    DebugPacket, DebugPacketContext, DebugStatus, HealthStatusBeforeFallback,
};
pub use state_trace::{
    default_degraded_transitions, default_failed_transitions, TurnStateTransition,
};

pub fn build_debug_packet(context: DebugPacketContext<'_>) -> DebugPacket {
    debug_packet::try_build_debug_packet(context)
        .expect("debug packet construction failed (fail-closed)")
}

pub fn try_build_debug_packet(context: DebugPacketContext<'_>) -> Result<DebugPacket, String> {
    debug_packet::try_build_debug_packet(context)
}

#[cfg(test)]
pub mod diag_tests;
