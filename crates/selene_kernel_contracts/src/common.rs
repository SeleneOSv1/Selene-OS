#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemaVersion(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonotonicTimeNs(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReasonCodeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionState {
    Closed,
    Open,
    Active,
    SoftClosed,
    Suspended,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContractViolation {
    InvalidValue {
        field: &'static str,
        reason: &'static str,
    },
    InvalidRange {
        field: &'static str,
        min: f64,
        max: f64,
        got: f64,
    },
    NotFinite {
        field: &'static str,
    },
}

pub trait Validate {
    fn validate(&self) -> Result<(), ContractViolation>;
}
