#![forbid(unsafe_code)]

use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UnitConversionTable {
    factors: BTreeMap<(String, String), Decimal>,
}

impl UnitConversionTable {
    pub fn from_evidence_packet(evidence_packet: &Value) -> Self {
        let mut factors = BTreeMap::new();
        let conversions = evidence_packet
            .pointer("/trust_metadata/analytics/unit_conversions")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        for entry in conversions {
            let from = entry
                .get("from")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| value.to_ascii_lowercase());
            let to = entry
                .get("to")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| value.to_ascii_lowercase());
            let factor = entry
                .get("factor")
                .and_then(Value::as_str)
                .and_then(|raw| raw.parse::<Decimal>().ok());

            if let (Some(from), Some(to), Some(factor)) = (from, to, factor) {
                factors.insert((from, to), factor);
            }
        }

        Self { factors }
    }

    pub fn factor(&self, from: &str, to: &str) -> Option<Decimal> {
        if from.eq_ignore_ascii_case(to) {
            return Some(Decimal::ONE);
        }
        self.factors
            .get(&(from.to_ascii_lowercase(), to.to_ascii_lowercase()))
            .copied()
    }

    pub fn convert(&self, value: Decimal, from: &str, to: &str) -> Option<Decimal> {
        self.factor(from, to).map(|factor| value * factor)
    }
}
