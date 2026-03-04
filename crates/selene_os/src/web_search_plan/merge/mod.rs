#![forbid(unsafe_code)]

pub mod boundary;
pub mod conflict;
pub mod delta;
pub mod internal_context;
pub mod merge_packet;

pub use conflict::{ConflictItem, ConflictReport};
pub use delta::{ChangeType, DeltaChange, ExternalFinding};
pub use internal_context::{InternalContext, InternalSourceType, InternalView};
pub use merge_packet::{
    append_merge_audit_metadata, build_merge_packet, run_internal_external_merge,
    MergeBuildError, MergeBuildOutput, MergePacket, MergeRequest, MERGE_ENGINE_ID,
    MERGE_SCHEMA_VERSION, MERGE_TEMPLATE_VERSION,
};

#[cfg(test)]
pub mod merge_tests;
