#![forbid(unsafe_code)]

use crate::web_search_plan::structured::types::{
    StructuredAdapterOutput, StructuredAdapterRequest, StructuredConnectorError,
    StructuredErrorKind, StructuredRuntimeConfig,
};

pub const ADAPTER_ID: &str = "pricing_products";

pub fn execute(
    _request: &StructuredAdapterRequest,
    _config: &StructuredRuntimeConfig,
) -> Result<StructuredAdapterOutput, StructuredConnectorError> {
    Err(StructuredConnectorError::new(
        ADAPTER_ID,
        StructuredErrorKind::ProviderUnconfigured,
        None,
        "Not Implemented: no provider credentials configured",
        0,
    ))
}
