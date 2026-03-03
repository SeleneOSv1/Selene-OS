#![forbid(unsafe_code)]

pub mod calibration;
pub mod confidence;
pub mod factors;
pub mod guardrails;
pub mod risk_packet;
pub mod scoring;

pub use calibration::{risk_model_version, FactorCalibration, RiskThresholds, RISK_MODEL_VERSION};
pub use confidence::{build_confidence, ConfidenceOutput};
pub use factors::{extract_factor_scores, FactorId, FactorScore, FactorExtractionResult};
pub use guardrails::{enforce_non_advice_guardrails, RISK_DISCLAIMER_TEXT};
pub use risk_packet::{build_risk_packet, RiskBuildError, RiskPacket, RiskRequest};
pub use scoring::{classify_risk_level, compute_composite_risk, CompositeRiskScore, RiskLevel};

#[cfg(test)]
pub mod risk_tests;
