#![forbid(unsafe_code)]

pub mod evidence_pack;

pub use evidence_pack::{
    generate_release_evidence_pack, GenerateReleaseEvidenceConfig, ReleaseEvidencePack,
};

#[cfg(test)]
pub mod release_tests;
