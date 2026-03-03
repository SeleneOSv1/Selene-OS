#![forbid(unsafe_code)]

use crate::web_search_plan::analytics::types::NumericValue;
use crate::web_search_plan::structured::types::StructuredValue;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};

pub const DECIMAL_SCALE: u32 = 6;
pub const ROUNDING_STRATEGY: RoundingStrategy = RoundingStrategy::MidpointAwayFromZero;
pub const MAD_OUTLIER_THRESHOLD: i64 = 3;

pub fn structured_value_to_decimal(value: &StructuredValue) -> Option<Decimal> {
    match value {
        StructuredValue::Int { value } => Some(Decimal::from(*value)),
        StructuredValue::Float { value } => Decimal::from_f64_retain(*value),
        StructuredValue::Currency { amount, .. } => Decimal::from_f64_retain(*amount),
        StructuredValue::Percent { value } => Decimal::from_f64_retain(*value),
        _ => None,
    }
}

pub fn round_decimal(value: Decimal) -> Decimal {
    value.round_dp_with_strategy(DECIMAL_SCALE, ROUNDING_STRATEGY)
}

pub fn decimal_to_string(value: Decimal) -> String {
    let rounded = round_decimal(value).normalize();
    let mut text = rounded.to_string();
    if text == "-0" {
        text = "0".to_string();
    }
    text
}

pub fn decimal_to_numeric_value(value: Decimal) -> NumericValue {
    let rounded = round_decimal(value);
    if rounded.fract().is_zero() {
        if let Some(as_i64) = rounded.to_i64() {
            return NumericValue::Int { value: as_i64 };
        }
    }
    NumericValue::Decimal {
        value: decimal_to_string(rounded),
    }
}

pub fn mean(values: &[Decimal]) -> Decimal {
    if values.is_empty() {
        return Decimal::ZERO;
    }
    let sum = values.iter().copied().sum::<Decimal>();
    round_decimal(sum / Decimal::from(values.len() as u64))
}

pub fn weighted_mean(values: &[(Decimal, Decimal)]) -> Decimal {
    if values.is_empty() {
        return Decimal::ZERO;
    }
    let numerator = values.iter().fold(Decimal::ZERO, |acc, (value, weight)| {
        acc + (*value * *weight)
    });
    let denominator = values
        .iter()
        .fold(Decimal::ZERO, |acc, (_, weight)| acc + *weight);
    if denominator.is_zero() {
        Decimal::ZERO
    } else {
        round_decimal(numerator / denominator)
    }
}

pub fn min(values: &[Decimal]) -> Decimal {
    values.iter().copied().min().unwrap_or(Decimal::ZERO)
}

pub fn max(values: &[Decimal]) -> Decimal {
    values.iter().copied().max().unwrap_or(Decimal::ZERO)
}

pub fn median(sorted_values: &[Decimal]) -> Decimal {
    percentile(sorted_values, 50)
}

pub fn percentile(sorted_values: &[Decimal], p: u32) -> Decimal {
    if sorted_values.is_empty() {
        return Decimal::ZERO;
    }
    if sorted_values.len() == 1 {
        return sorted_values[0];
    }

    let n_minus_one = (sorted_values.len() - 1) as u32;
    let numer = p.saturating_mul(n_minus_one);
    let lower_index = (numer / 100) as usize;
    let upper_index = (lower_index + 1).min(sorted_values.len() - 1);
    let remainder = Decimal::from((numer % 100) as i64) / Decimal::from(100);

    let lower = sorted_values[lower_index];
    let upper = sorted_values[upper_index];
    let interpolated = lower + (upper - lower) * remainder;
    round_decimal(interpolated)
}

pub fn trimmed_mean(sorted_values: &[Decimal]) -> Decimal {
    if sorted_values.is_empty() {
        return Decimal::ZERO;
    }
    let trim_each_side = sorted_values.len() / 10;
    if trim_each_side == 0 || trim_each_side * 2 >= sorted_values.len() {
        return mean(sorted_values);
    }
    mean(&sorted_values[trim_each_side..(sorted_values.len() - trim_each_side)])
}

pub fn stddev(values: &[Decimal]) -> Decimal {
    if values.len() <= 1 {
        return Decimal::ZERO;
    }
    let avg = mean(values);
    let variance_sum = values.iter().fold(Decimal::ZERO, |acc, value| {
        let delta = *value - avg;
        acc + delta * delta
    });
    let variance = variance_sum / Decimal::from(values.len() as u64);
    sqrt_decimal(variance)
}

pub fn mad(values: &[Decimal]) -> Decimal {
    if values.is_empty() {
        return Decimal::ZERO;
    }
    let mut sorted = values.to_vec();
    sorted.sort();
    let med = median(&sorted);

    let mut deviations = values
        .iter()
        .map(|value| (*value - med).abs())
        .collect::<Vec<Decimal>>();
    deviations.sort();
    median(&deviations)
}

pub fn is_outlier(value: Decimal, center: Decimal, mad_value: Decimal) -> bool {
    if mad_value.is_zero() {
        return false;
    }
    (value - center).abs() > mad_value * Decimal::from(MAD_OUTLIER_THRESHOLD)
}

pub fn sqrt_decimal(value: Decimal) -> Decimal {
    if value <= Decimal::ZERO {
        return Decimal::ZERO;
    }
    let two = Decimal::from(2);
    let mut estimate = if value > Decimal::ONE {
        value / two
    } else {
        Decimal::ONE
    };

    for _ in 0..32 {
        estimate = round_decimal((estimate + value / estimate) / two);
    }
    round_decimal(estimate)
}
