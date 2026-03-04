#![forbid(unsafe_code)]

pub mod consistency;
pub mod enterprise_pipeline;
pub mod enterprise_request;
pub mod mode_router;
pub mod provenance;

pub use consistency::{validate_cross_mode_consistency, ConsistencyError};
pub use enterprise_pipeline::{
    run_enterprise_pipeline, EnterprisePipelineError, EnterprisePipelineOutput,
};
pub use enterprise_request::{EnterpriseConstraints, EnterpriseRequest};
pub use mode_router::{parse_mode, route_mode, EnterpriseMode};
pub use provenance::{build_enterprise_provenance, EnterpriseProvenance};

#[cfg(test)]
pub mod enterprise_tests;
