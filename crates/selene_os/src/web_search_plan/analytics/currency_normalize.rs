#![forbid(unsafe_code)]

use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CurrencyRateTable {
    rates: BTreeMap<(String, String), Decimal>,
}

impl CurrencyRateTable {
    pub fn from_evidence_packet(evidence_packet: &Value) -> Self {
        let mut rates = BTreeMap::new();
        let entries = evidence_packet
            .pointer("/trust_metadata/analytics/currency_rates")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        for entry in entries {
            let from = entry
                .get("from")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| value.to_ascii_uppercase());
            let to = entry
                .get("to")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| value.to_ascii_uppercase());
            let rate = entry
                .get("rate")
                .and_then(Value::as_str)
                .and_then(|raw| raw.parse::<Decimal>().ok());

            if let (Some(from), Some(to), Some(rate)) = (from, to, rate) {
                rates.insert((from, to), rate);
            }
        }

        Self { rates }
    }

    pub fn rate(&self, from: &str, to: &str) -> Option<Decimal> {
        if from.eq_ignore_ascii_case(to) {
            return Some(Decimal::ONE);
        }
        self.rates
            .get(&(from.to_ascii_uppercase(), to.to_ascii_uppercase()))
            .copied()
    }

    pub fn convert(&self, amount: Decimal, from: &str, to: &str) -> Option<Decimal> {
        self.rate(from, to).map(|rate| amount * rate)
    }
}
