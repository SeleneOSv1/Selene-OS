#![forbid(unsafe_code)]

use crate::web_search_plan::risk::factors::FactorId;
use rust_decimal::Decimal;

pub const RISK_MODEL_VERSION: &str = "1.0.0";
pub const ROUNDING_SCALE: u32 = 6;
pub const CONFIDENCE_LOW_THRESHOLD_BPS: u32 = 4_500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FactorCalibration {
    pub weight_bps: u32,
    pub min_evidence_refs: usize,
    pub saturation_points: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RiskThresholds {
    pub low_max_bps: u32,
    pub medium_max_bps: u32,
}

pub fn risk_model_version() -> &'static str {
    RISK_MODEL_VERSION
}

pub fn factor_calibration(factor_id: FactorId) -> FactorCalibration {
    match factor_id {
        FactorId::FinancialStress => FactorCalibration {
            weight_bps: 2_800,
            min_evidence_refs: 1,
            saturation_points: 6,
        },
        FactorId::LegalEvents => FactorCalibration {
            weight_bps: 2_000,
            min_evidence_refs: 1,
            saturation_points: 5,
        },
        FactorId::RegulatoryEvents => FactorCalibration {
            weight_bps: 2_000,
            min_evidence_refs: 1,
            saturation_points: 5,
        },
        FactorId::NegativeNewsCluster => FactorCalibration {
            weight_bps: 1_600,
            min_evidence_refs: 2,
            saturation_points: 8,
        },
        FactorId::OperationalReliability => FactorCalibration {
            weight_bps: 1_600,
            min_evidence_refs: 1,
            saturation_points: 6,
        },
    }
}

pub fn risk_thresholds() -> RiskThresholds {
    RiskThresholds {
        low_max_bps: 3_300,
        medium_max_bps: 6_600,
    }
}

pub fn bps_to_decimal(value_bps: u32) -> Decimal {
    Decimal::from(value_bps) / Decimal::from(10_000u32)
}
