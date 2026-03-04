#![forbid(unsafe_code)]

pub mod orchestrator;

pub use orchestrator::{
    execute_web_search_turn, AuditPacket, EvidencePacket, ReasonCodeId, SearchAssistPacket,
    SynthesisPacket, ToolRequestPacket, TurnInputPacket, WritePacket,
};

#[cfg(test)]
pub mod runtime_tests;
