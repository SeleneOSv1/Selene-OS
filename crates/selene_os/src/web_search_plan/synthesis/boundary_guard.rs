#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvidenceBoundaryContext {
    pub external_lookup_requested: bool,
    pub memory_access_requested: bool,
    pub hidden_global_access_requested: bool,
    pub policy_mutation_requested: bool,
}

impl EvidenceBoundaryContext {
    pub const fn from_external_lookup_requested(external_lookup_requested: bool) -> Self {
        Self {
            external_lookup_requested,
            memory_access_requested: false,
            hidden_global_access_requested: false,
            policy_mutation_requested: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceBoundaryViolation {
    ExternalLookupForbidden,
    MemoryAccessForbidden,
    HiddenGlobalAccessForbidden,
    PolicyMutationForbidden,
}

impl EvidenceBoundaryViolation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExternalLookupForbidden => "external lookup is forbidden",
            Self::MemoryAccessForbidden => "memory access outside EvidencePacket is forbidden",
            Self::HiddenGlobalAccessForbidden => "hidden global state is forbidden",
            Self::PolicyMutationForbidden => "policy mutation from synthesis is forbidden",
        }
    }
}

pub fn assert_evidence_boundary(
    context: EvidenceBoundaryContext,
) -> Result<(), EvidenceBoundaryViolation> {
    if context.external_lookup_requested {
        return Err(EvidenceBoundaryViolation::ExternalLookupForbidden);
    }
    if context.memory_access_requested {
        return Err(EvidenceBoundaryViolation::MemoryAccessForbidden);
    }
    if context.hidden_global_access_requested {
        return Err(EvidenceBoundaryViolation::HiddenGlobalAccessForbidden);
    }
    if context.policy_mutation_requested {
        return Err(EvidenceBoundaryViolation::PolicyMutationForbidden);
    }

    Ok(())
}
