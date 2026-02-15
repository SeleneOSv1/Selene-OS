#![forbid(unsafe_code)]

use crate::ph1_voice_id::UserId;
use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};
use std::collections::BTreeMap;

pub const PH1LINK_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

// Simulation IDs (authoritative strings; must match docs/08_SIMULATION_CATALOG.md).
pub const LINK_INVITE_GENERATE_DRAFT: &str = "LINK_INVITE_GENERATE_DRAFT";
pub const LINK_INVITE_DRAFT_UPDATE_COMMIT: &str = "LINK_INVITE_DRAFT_UPDATE_COMMIT";
pub const LINK_INVITE_OPEN_ACTIVATE_COMMIT: &str = "LINK_INVITE_OPEN_ACTIVATE_COMMIT";
pub const LINK_INVITE_REVOKE_REVOKE: &str = "LINK_INVITE_REVOKE_REVOKE";
pub const LINK_INVITE_EXPIRED_RECOVERY_COMMIT: &str = "LINK_INVITE_EXPIRED_RECOVERY_COMMIT";
pub const LINK_INVITE_FORWARD_BLOCK_COMMIT: &str = "LINK_INVITE_FORWARD_BLOCK_COMMIT";
pub const LINK_ROLE_PROPOSE_DRAFT: &str = "LINK_ROLE_PROPOSE_DRAFT";
pub const LINK_INVITE_DUAL_ROLE_CONFLICT_ESCALATE_DRAFT: &str =
    "LINK_INVITE_DUAL_ROLE_CONFLICT_ESCALATE_DRAFT";

fn fnv1a64(bytes: &[u8]) -> u64 {
    // FNV-1a 64-bit (stable across platforms, deterministic).
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut h = OFFSET;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}

fn hash_hex_64(s: &str) -> String {
    let mut h = fnv1a64(s.as_bytes());
    if h == 0 {
        // Avoid a "zero hash" corner case so it can be treated as "present".
        h = 1;
    }
    format!("{:016x}", h)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimulationType {
    Draft,
    Commit,
    Revoke,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InviteeType {
    Company,
    Customer,
    Employee,
    FamilyMember,
    Friend,
    Associate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LinkStatus {
    DraftCreated,
    Sent,
    Opened,
    Activated,
    Consumed,
    Expired,
    Revoked,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DraftStatus {
    DraftCreated,
    DraftReady,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenId(String);

impl TokenId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        let v = Self(id);
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for TokenId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "token_id",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "token_id",
                reason: "must be <= 64 chars",
            });
        }
        if !self.0.is_ascii() {
            return Err(ContractViolation::InvalidValue {
                field: "token_id",
                reason: "must be ASCII",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DraftId(String);

impl DraftId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        let v = Self(id);
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for DraftId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "draft_id",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "draft_id",
                reason: "must be <= 64 chars",
            });
        }
        if !self.0.is_ascii() {
            return Err(ContractViolation::InvalidValue {
                field: "draft_id",
                reason: "must be ASCII",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrefilledContextRef(String);

impl PrefilledContextRef {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        let v = Self(id);
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for PrefilledContextRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "prefilled_context_ref",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "prefilled_context_ref",
                reason: "must be <= 128 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefilledContext {
    pub schema_version: SchemaVersion,
    // Keep prefilled onboarding data minimal and policy-safe:
    // - IDs/refs are allowed.
    // - Sensitive raw values (salary, ID numbers, etc.) must not be embedded here.
    pub tenant_id: Option<String>,
    pub company_id: Option<String>,
    pub position_id: Option<String>,
    pub location_id: Option<String>,
    pub start_date: Option<String>,
    pub working_hours: Option<String>,
    pub compensation_tier_ref: Option<String>,
    pub jurisdiction_tags: Vec<String>,
}

impl PrefilledContext {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        tenant_id: Option<String>,
        company_id: Option<String>,
        position_id: Option<String>,
        location_id: Option<String>,
        start_date: Option<String>,
        working_hours: Option<String>,
        compensation_tier_ref: Option<String>,
        jurisdiction_tags: Vec<String>,
    ) -> Result<Self, ContractViolation> {
        let p = Self {
            schema_version: PH1LINK_CONTRACT_VERSION,
            tenant_id,
            company_id,
            position_id,
            location_id,
            start_date,
            working_hours,
            compensation_tier_ref,
            jurisdiction_tags,
        };
        p.validate()?;
        Ok(p)
    }
}

fn validate_opt_id(
    field: &'static str,
    v: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(s) = v {
        if s.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "must not be empty when provided",
            });
        }
        if s.len() > max_len {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "too long",
            });
        }
        if !s.is_ascii() {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "must be ASCII when provided",
            });
        }
    }
    Ok(())
}

impl Validate for PrefilledContext {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LINK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "prefilled_context.schema_version",
                reason: "must match PH1LINK_CONTRACT_VERSION",
            });
        }
        validate_opt_id("prefilled_context.tenant_id", &self.tenant_id, 64)?;
        validate_opt_id("prefilled_context.company_id", &self.company_id, 64)?;
        validate_opt_id("prefilled_context.position_id", &self.position_id, 64)?;
        validate_opt_id("prefilled_context.location_id", &self.location_id, 64)?;
        validate_opt_id("prefilled_context.start_date", &self.start_date, 32)?;
        validate_opt_id("prefilled_context.working_hours", &self.working_hours, 64)?;
        validate_opt_id(
            "prefilled_context.compensation_tier_ref",
            &self.compensation_tier_ref,
            64,
        )?;

        if self.jurisdiction_tags.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "prefilled_context.jurisdiction_tags",
                reason: "must be <= 8 entries",
            });
        }
        for t in &self.jurisdiction_tags {
            if t.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "prefilled_context.jurisdiction_tags[]",
                    reason: "must not contain empty strings",
                });
            }
            if t.len() > 32 {
                return Err(ContractViolation::InvalidValue {
                    field: "prefilled_context.jurisdiction_tags[]",
                    reason: "must be <= 32 chars",
                });
            }
            if !t.is_ascii() {
                return Err(ContractViolation::InvalidValue {
                    field: "prefilled_context.jurisdiction_tags[]",
                    reason: "must be ASCII",
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkRecord {
    pub schema_version: SchemaVersion,
    pub token_id: TokenId,
    pub draft_id: DraftId,
    pub payload_hash: String,
    pub schema_version_id: Option<String>,
    pub missing_required_fields: Vec<String>,
    pub status: LinkStatus,
    pub created_at: MonotonicTimeNs,
    pub expires_at: MonotonicTimeNs,
    pub inviter_user_id: UserId,
    pub invitee_type: InviteeType,
    pub expiration_policy_id: Option<String>,
    pub prefilled_context: Option<PrefilledContext>,
    pub bound_device_fingerprint_hash: Option<String>,
    pub revoked_reason: Option<String>,
}

impl LinkRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        token_id: TokenId,
        draft_id: DraftId,
        payload_hash: String,
        schema_version_id: Option<String>,
        missing_required_fields: Vec<String>,
        status: LinkStatus,
        created_at: MonotonicTimeNs,
        expires_at: MonotonicTimeNs,
        inviter_user_id: UserId,
        invitee_type: InviteeType,
        expiration_policy_id: Option<String>,
        prefilled_context: Option<PrefilledContext>,
        bound_device_fingerprint_hash: Option<String>,
        revoked_reason: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1LINK_CONTRACT_VERSION,
            token_id,
            draft_id,
            payload_hash,
            schema_version_id,
            missing_required_fields,
            status,
            created_at,
            expires_at,
            inviter_user_id,
            invitee_type,
            expiration_policy_id,
            prefilled_context,
            bound_device_fingerprint_hash,
            revoked_reason,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for LinkRecord {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LINK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "link_record.schema_version",
                reason: "must match PH1LINK_CONTRACT_VERSION",
            });
        }
        self.token_id.validate()?;
        if self.payload_hash.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "link_record.payload_hash",
                reason: "must not be empty",
            });
        }
        if self.payload_hash.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "link_record.payload_hash",
                reason: "must be <= 128 chars",
            });
        }
        self.draft_id.validate()?;
        validate_opt_id("link_record.schema_version_id", &self.schema_version_id, 64)?;
        if self.missing_required_fields.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "link_record.missing_required_fields",
                reason: "must be <= 64 entries",
            });
        }
        for f in &self.missing_required_fields {
            if f.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "link_record.missing_required_fields[]",
                    reason: "must not contain empty entries",
                });
            }
            if f.len() > 64 || !f.is_ascii() {
                return Err(ContractViolation::InvalidValue {
                    field: "link_record.missing_required_fields[]",
                    reason: "must be ASCII and <= 64 chars",
                });
            }
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "link_record.created_at",
                reason: "must be > 0",
            });
        }
        if self.expires_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "link_record.expires_at",
                reason: "must be > 0",
            });
        }
        if self.expires_at.0 < self.created_at.0 {
            return Err(ContractViolation::InvalidValue {
                field: "link_record.expires_at",
                reason: "must be >= created_at",
            });
        }
        if self.inviter_user_id.as_str().trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "link_record.inviter_user_id",
                reason: "must not be empty",
            });
        }
        validate_opt_id(
            "link_record.expiration_policy_id",
            &self.expiration_policy_id,
            64,
        )?;
        if let Some(p) = &self.prefilled_context {
            p.validate()?;
        }
        if let Some(h) = &self.bound_device_fingerprint_hash {
            if h.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "link_record.bound_device_fingerprint_hash",
                    reason: "must not be empty when provided",
                });
            }
            if h.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "link_record.bound_device_fingerprint_hash",
                    reason: "must be <= 128 chars",
                });
            }
            if !h.is_ascii() {
                return Err(ContractViolation::InvalidValue {
                    field: "link_record.bound_device_fingerprint_hash",
                    reason: "must be ASCII",
                });
            }
        }
        if let Some(r) = &self.revoked_reason {
            if r.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "link_record.revoked_reason",
                    reason: "must not be empty when provided",
                });
            }
            if r.len() > 256 {
                return Err(ContractViolation::InvalidValue {
                    field: "link_record.revoked_reason",
                    reason: "must be <= 256 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkGenerateResult {
    pub schema_version: SchemaVersion,
    pub draft_id: DraftId,
    pub token_id: TokenId,
    pub link_url: String,
    pub missing_required_fields: Vec<String>,
    pub payload_hash: String,
    pub expires_at: MonotonicTimeNs,
    pub status: LinkStatus,
    pub prefilled_context_ref: Option<PrefilledContextRef>,
}

impl LinkGenerateResult {
    pub fn v1(
        draft_id: DraftId,
        token_id: TokenId,
        link_url: String,
        missing_required_fields: Vec<String>,
        payload_hash: String,
        expires_at: MonotonicTimeNs,
        status: LinkStatus,
        prefilled_context_ref: Option<PrefilledContextRef>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1LINK_CONTRACT_VERSION,
            draft_id,
            token_id,
            link_url,
            missing_required_fields,
            payload_hash,
            expires_at,
            status,
            prefilled_context_ref,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for LinkGenerateResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LINK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "link_generate_result.schema_version",
                reason: "must match PH1LINK_CONTRACT_VERSION",
            });
        }
        self.draft_id.validate()?;
        self.token_id.validate()?;
        if self.link_url.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "link_generate_result.link_url",
                reason: "must not be empty",
            });
        }
        if self.link_url.len() > 1024 {
            return Err(ContractViolation::InvalidValue {
                field: "link_generate_result.link_url",
                reason: "must be <= 1024 chars",
            });
        }
        if self.payload_hash.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "link_generate_result.payload_hash",
                reason: "must not be empty",
            });
        }
        if self.payload_hash.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "link_generate_result.payload_hash",
                reason: "must be <= 128 chars",
            });
        }
        if self.missing_required_fields.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "link_generate_result.missing_required_fields",
                reason: "must be <= 64 entries",
            });
        }
        for f in &self.missing_required_fields {
            if f.trim().is_empty() || f.len() > 64 || !f.is_ascii() {
                return Err(ContractViolation::InvalidValue {
                    field: "link_generate_result.missing_required_fields[]",
                    reason: "must be ASCII, non-empty, <= 64 chars",
                });
            }
        }
        if self.expires_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "link_generate_result.expires_at",
                reason: "must be > 0",
            });
        }
        if let Some(r) = &self.prefilled_context_ref {
            r.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkDraftUpdateResult {
    pub schema_version: SchemaVersion,
    pub draft_id: DraftId,
    pub draft_status: DraftStatus,
    pub missing_required_fields: Vec<String>,
}

impl LinkDraftUpdateResult {
    pub fn v1(
        draft_id: DraftId,
        draft_status: DraftStatus,
        missing_required_fields: Vec<String>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1LINK_CONTRACT_VERSION,
            draft_id,
            draft_status,
            missing_required_fields,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for LinkDraftUpdateResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LINK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "link_draft_update_result.schema_version",
                reason: "must match PH1LINK_CONTRACT_VERSION",
            });
        }
        self.draft_id.validate()?;
        if self.missing_required_fields.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "link_draft_update_result.missing_required_fields",
                reason: "must be <= 64 entries",
            });
        }
        for f in &self.missing_required_fields {
            if f.trim().is_empty() || f.len() > 64 || !f.is_ascii() {
                return Err(ContractViolation::InvalidValue {
                    field: "link_draft_update_result.missing_required_fields[]",
                    reason: "must be ASCII, non-empty, <= 64 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkActivationResult {
    pub schema_version: SchemaVersion,
    pub token_id: TokenId,
    pub draft_id: DraftId,
    pub activation_status: LinkStatus,
    pub missing_required_fields: Vec<String>,
    pub conflict_reason: Option<String>,
    pub bound_device_fingerprint_hash: Option<String>,
    pub prefilled_context_ref: Option<PrefilledContextRef>,
}

impl LinkActivationResult {
    pub fn v1(
        token_id: TokenId,
        draft_id: DraftId,
        activation_status: LinkStatus,
        missing_required_fields: Vec<String>,
        conflict_reason: Option<String>,
        bound_device_fingerprint_hash: Option<String>,
        prefilled_context_ref: Option<PrefilledContextRef>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1LINK_CONTRACT_VERSION,
            token_id,
            draft_id,
            activation_status,
            missing_required_fields,
            conflict_reason,
            bound_device_fingerprint_hash,
            prefilled_context_ref,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for LinkActivationResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LINK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "link_activation_result.schema_version",
                reason: "must match PH1LINK_CONTRACT_VERSION",
            });
        }
        self.token_id.validate()?;
        self.draft_id.validate()?;
        if self.missing_required_fields.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "link_activation_result.missing_required_fields",
                reason: "must be <= 64 entries",
            });
        }
        for f in &self.missing_required_fields {
            if f.trim().is_empty() || f.len() > 64 || !f.is_ascii() {
                return Err(ContractViolation::InvalidValue {
                    field: "link_activation_result.missing_required_fields[]",
                    reason: "must be ASCII, non-empty, <= 64 chars",
                });
            }
        }
        if let Some(r) = &self.conflict_reason {
            if r.trim().is_empty() || r.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "link_activation_result.conflict_reason",
                    reason: "must be non-empty and <= 128 chars when provided",
                });
            }
        }
        if let Some(h) = &self.bound_device_fingerprint_hash {
            if h.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "link_activation_result.bound_device_fingerprint_hash",
                    reason: "must not be empty when provided",
                });
            }
            if h.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "link_activation_result.bound_device_fingerprint_hash",
                    reason: "must be <= 128 chars",
                });
            }
        }
        if let Some(r) = &self.prefilled_context_ref {
            r.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkRevokeResult {
    pub schema_version: SchemaVersion,
    pub status: LinkStatus,
}

impl LinkRevokeResult {
    pub fn v1(status: LinkStatus) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1LINK_CONTRACT_VERSION,
            status,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for LinkRevokeResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LINK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "link_revoke_result.schema_version",
                reason: "must match PH1LINK_CONTRACT_VERSION",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkExpiredRecoveryResult {
    pub schema_version: SchemaVersion,
    pub new_token_id: TokenId,
    pub draft_id: DraftId,
    pub new_link_url: String,
    pub missing_required_fields: Vec<String>,
}

impl LinkExpiredRecoveryResult {
    pub fn v1(
        new_token_id: TokenId,
        draft_id: DraftId,
        new_link_url: String,
        missing_required_fields: Vec<String>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1LINK_CONTRACT_VERSION,
            new_token_id,
            draft_id,
            new_link_url,
            missing_required_fields,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for LinkExpiredRecoveryResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LINK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "link_expired_recovery_result.schema_version",
                reason: "must match PH1LINK_CONTRACT_VERSION",
            });
        }
        self.new_token_id.validate()?;
        self.draft_id.validate()?;
        if self.new_link_url.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "link_expired_recovery_result.new_link_url",
                reason: "must not be empty",
            });
        }
        if self.new_link_url.len() > 1024 {
            return Err(ContractViolation::InvalidValue {
                field: "link_expired_recovery_result.new_link_url",
                reason: "must be <= 1024 chars",
            });
        }
        if self.missing_required_fields.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "link_expired_recovery_result.missing_required_fields",
                reason: "must be <= 64 entries",
            });
        }
        for f in &self.missing_required_fields {
            if f.trim().is_empty() || f.len() > 64 || !f.is_ascii() {
                return Err(ContractViolation::InvalidValue {
                    field: "link_expired_recovery_result.missing_required_fields[]",
                    reason: "must be ASCII, non-empty, <= 64 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RoleProposalStatus {
    PendingApApproval,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleProposalResult {
    pub schema_version: SchemaVersion,
    pub role_proposal_id: String,
    pub status: RoleProposalStatus,
}

impl RoleProposalResult {
    pub fn v1(
        role_proposal_id: String,
        status: RoleProposalStatus,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1LINK_CONTRACT_VERSION,
            role_proposal_id,
            status,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for RoleProposalResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LINK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "role_proposal_result.schema_version",
                reason: "must match PH1LINK_CONTRACT_VERSION",
            });
        }
        if self.role_proposal_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "role_proposal_result.role_proposal_id",
                reason: "must not be empty",
            });
        }
        if self.role_proposal_id.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "role_proposal_result.role_proposal_id",
                reason: "must be <= 64 chars",
            });
        }
        if !self.role_proposal_id.is_ascii() {
            return Err(ContractViolation::InvalidValue {
                field: "role_proposal_result.role_proposal_id",
                reason: "must be ASCII",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EscalationStatus {
    Escalated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DualRoleConflictEscalationResult {
    pub schema_version: SchemaVersion,
    pub escalation_case_id: String,
    pub status: EscalationStatus,
}

impl DualRoleConflictEscalationResult {
    pub fn v1(
        escalation_case_id: String,
        status: EscalationStatus,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1LINK_CONTRACT_VERSION,
            escalation_case_id,
            status,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for DualRoleConflictEscalationResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LINK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "dual_role_conflict_escalation_result.schema_version",
                reason: "must match PH1LINK_CONTRACT_VERSION",
            });
        }
        if self.escalation_case_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "dual_role_conflict_escalation_result.escalation_case_id",
                reason: "must not be empty",
            });
        }
        if self.escalation_case_id.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "dual_role_conflict_escalation_result.escalation_case_id",
                reason: "must be <= 64 chars",
            });
        }
        if !self.escalation_case_id.is_ascii() {
            return Err(ContractViolation::InvalidValue {
                field: "dual_role_conflict_escalation_result.escalation_case_id",
                reason: "must be ASCII",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1LinkOk {
    pub schema_version: SchemaVersion,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub link_generate_result: Option<LinkGenerateResult>,
    pub link_draft_update_result: Option<LinkDraftUpdateResult>,
    pub link_activation_result: Option<LinkActivationResult>,
    pub link_revoke_result: Option<LinkRevokeResult>,
    pub link_expired_recovery_result: Option<LinkExpiredRecoveryResult>,
    pub role_proposal_result: Option<RoleProposalResult>,
    pub dual_role_conflict_escalation_result: Option<DualRoleConflictEscalationResult>,
}

impl Ph1LinkOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        simulation_id: String,
        reason_code: ReasonCodeId,
        link_generate_result: Option<LinkGenerateResult>,
        link_activation_result: Option<LinkActivationResult>,
        link_revoke_result: Option<LinkRevokeResult>,
        link_expired_recovery_result: Option<LinkExpiredRecoveryResult>,
        role_proposal_result: Option<RoleProposalResult>,
        dual_role_conflict_escalation_result: Option<DualRoleConflictEscalationResult>,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1LINK_CONTRACT_VERSION,
            simulation_id,
            reason_code,
            link_generate_result,
            link_draft_update_result: None,
            link_activation_result,
            link_revoke_result,
            link_expired_recovery_result,
            role_proposal_result,
            dual_role_conflict_escalation_result,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for Ph1LinkOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LINK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1link_ok.schema_version",
                reason: "must match PH1LINK_CONTRACT_VERSION",
            });
        }
        if self.simulation_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1link_ok.simulation_id",
                reason: "must not be empty",
            });
        }
        if self.simulation_id.len() > 96 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1link_ok.simulation_id",
                reason: "must be <= 96 chars",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1link_ok.reason_code",
                reason: "must be > 0",
            });
        }
        let mut count = 0u8;
        if let Some(r) = &self.link_generate_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.link_draft_update_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.link_activation_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.link_revoke_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.link_expired_recovery_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.role_proposal_result {
            r.validate()?;
            count += 1;
        }
        if let Some(r) = &self.dual_role_conflict_escalation_result {
            r.validate()?;
            count += 1;
        }
        if count != 1 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1link_ok",
                reason: "must contain exactly one result kind",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1LinkRefuse {
    pub schema_version: SchemaVersion,
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl Ph1LinkRefuse {
    pub fn v1(
        simulation_id: String,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1LINK_CONTRACT_VERSION,
            simulation_id,
            reason_code,
            message,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1LinkRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LINK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1link_refuse.schema_version",
                reason: "must match PH1LINK_CONTRACT_VERSION",
            });
        }
        if self.simulation_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1link_refuse.simulation_id",
                reason: "must not be empty",
            });
        }
        if self.simulation_id.len() > 96 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1link_refuse.simulation_id",
                reason: "must be <= 96 chars",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1link_refuse.reason_code",
                reason: "must be > 0",
            });
        }
        if self.message.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1link_refuse.message",
                reason: "must not be empty",
            });
        }
        if self.message.len() > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1link_refuse.message",
                reason: "must be <= 512 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1LinkResponse {
    Ok(Ph1LinkOk),
    Refuse(Ph1LinkRefuse),
}

impl Validate for Ph1LinkResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1LinkResponse::Ok(o) => o.validate(),
            Ph1LinkResponse::Refuse(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkRequest {
    InviteGenerateDraft(InviteGenerateDraftRequest),
    InviteDraftUpdateCommit(InviteDraftUpdateCommitRequest),
    InviteOpenActivateCommit(InviteOpenActivateCommitRequest),
    InviteRevokeRevoke(InviteRevokeRevokeRequest),
    InviteExpiredRecoveryCommit(InviteExpiredRecoveryCommitRequest),
    InviteForwardBlockCommit(InviteForwardBlockCommitRequest),
    RoleProposeDraft(RoleProposeDraftRequest),
    DualRoleConflictEscalateDraft(DualRoleConflictEscalateDraftRequest),
}

impl LinkRequest {
    pub fn simulation_id(&self) -> &'static str {
        match self {
            LinkRequest::InviteGenerateDraft(_) => LINK_INVITE_GENERATE_DRAFT,
            LinkRequest::InviteDraftUpdateCommit(_) => LINK_INVITE_DRAFT_UPDATE_COMMIT,
            LinkRequest::InviteOpenActivateCommit(_) => LINK_INVITE_OPEN_ACTIVATE_COMMIT,
            LinkRequest::InviteRevokeRevoke(_) => LINK_INVITE_REVOKE_REVOKE,
            LinkRequest::InviteExpiredRecoveryCommit(_) => LINK_INVITE_EXPIRED_RECOVERY_COMMIT,
            LinkRequest::InviteForwardBlockCommit(_) => LINK_INVITE_FORWARD_BLOCK_COMMIT,
            LinkRequest::RoleProposeDraft(_) => LINK_ROLE_PROPOSE_DRAFT,
            LinkRequest::DualRoleConflictEscalateDraft(_) => {
                LINK_INVITE_DUAL_ROLE_CONFLICT_ESCALATE_DRAFT
            }
        }
    }

    pub fn simulation_type(&self) -> SimulationType {
        match self {
            LinkRequest::InviteGenerateDraft(_) => SimulationType::Draft,
            LinkRequest::RoleProposeDraft(_) => SimulationType::Draft,
            LinkRequest::DualRoleConflictEscalateDraft(_) => SimulationType::Draft,
            LinkRequest::InviteDraftUpdateCommit(_)
            | LinkRequest::InviteOpenActivateCommit(_)
            | LinkRequest::InviteExpiredRecoveryCommit(_)
            | LinkRequest::InviteForwardBlockCommit(_) => SimulationType::Commit,
            LinkRequest::InviteRevokeRevoke(_) => SimulationType::Revoke,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InviteGenerateDraftRequest {
    pub inviter_user_id: UserId,
    pub invitee_type: InviteeType,
    pub tenant_id: Option<String>,
    pub schema_version_id: Option<String>,
    pub prefilled_context: Option<PrefilledContext>,
    pub expiration_policy_id: Option<String>,
}

impl Validate for InviteGenerateDraftRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.inviter_user_id.as_str().trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "invite_generate_draft_request.inviter_user_id",
                reason: "must not be empty",
            });
        }
        validate_opt_id(
            "invite_generate_draft_request.tenant_id",
            &self.tenant_id,
            64,
        )?;
        validate_opt_id(
            "invite_generate_draft_request.schema_version_id",
            &self.schema_version_id,
            64,
        )?;
        validate_opt_id(
            "invite_generate_draft_request.expiration_policy_id",
            &self.expiration_policy_id,
            64,
        )?;
        if let Some(p) = &self.prefilled_context {
            p.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InviteDraftUpdateCommitRequest {
    pub draft_id: DraftId,
    pub creator_update_fields: BTreeMap<String, String>,
    pub idempotency_key: String,
}

impl Validate for InviteDraftUpdateCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.draft_id.validate()?;
        if self.creator_update_fields.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "invite_draft_update_commit_request.creator_update_fields",
                reason: "must include at least one update field",
            });
        }
        if self.creator_update_fields.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "invite_draft_update_commit_request.creator_update_fields",
                reason: "must include <= 64 entries",
            });
        }
        for (k, v) in &self.creator_update_fields {
            if k.trim().is_empty() || k.len() > 64 || !k.is_ascii() {
                return Err(ContractViolation::InvalidValue {
                    field: "invite_draft_update_commit_request.creator_update_fields.key",
                    reason: "must be ASCII, non-empty, <= 64 chars",
                });
            }
            if v.trim().is_empty() || v.len() > 1024 {
                return Err(ContractViolation::InvalidValue {
                    field: "invite_draft_update_commit_request.creator_update_fields.value",
                    reason: "must be non-empty and <= 1024 chars",
                });
            }
        }
        if self.idempotency_key.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "invite_draft_update_commit_request.idempotency_key",
                reason: "must not be empty",
            });
        }
        if self.idempotency_key.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "invite_draft_update_commit_request.idempotency_key",
                reason: "must be <= 128 chars",
            });
        }
        if !self.idempotency_key.is_ascii() {
            return Err(ContractViolation::InvalidValue {
                field: "invite_draft_update_commit_request.idempotency_key",
                reason: "must be ASCII",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InviteOpenActivateCommitRequest {
    pub token_id: TokenId,
    pub device_fingerprint: String,
    pub idempotency_key: String,
}

impl Validate for InviteOpenActivateCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.token_id.validate()?;
        if self.device_fingerprint.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "invite_open_activate_commit_request.device_fingerprint",
                reason: "must not be empty",
            });
        }
        if self.device_fingerprint.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "invite_open_activate_commit_request.device_fingerprint",
                reason: "must be <= 256 chars",
            });
        }
        if self.idempotency_key.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "invite_open_activate_commit_request.idempotency_key",
                reason: "must not be empty",
            });
        }
        if self.idempotency_key.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "invite_open_activate_commit_request.idempotency_key",
                reason: "must be <= 128 chars",
            });
        }
        if !self.idempotency_key.is_ascii() {
            return Err(ContractViolation::InvalidValue {
                field: "invite_open_activate_commit_request.idempotency_key",
                reason: "must be ASCII",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InviteRevokeRevokeRequest {
    pub token_id: TokenId,
    pub reason: String,
}

impl Validate for InviteRevokeRevokeRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.token_id.validate()?;
        if self.reason.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "invite_revoke_revoke_request.reason",
                reason: "must not be empty",
            });
        }
        if self.reason.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "invite_revoke_revoke_request.reason",
                reason: "must be <= 256 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InviteExpiredRecoveryCommitRequest {
    pub token_id: TokenId,
    pub idempotency_key: Option<String>,
}

impl Validate for InviteExpiredRecoveryCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.token_id.validate()?;
        if let Some(k) = &self.idempotency_key {
            if k.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "invite_expired_recovery_commit_request.idempotency_key",
                    reason: "must not be empty when provided",
                });
            }
            if k.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "invite_expired_recovery_commit_request.idempotency_key",
                    reason: "must be <= 128 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InviteForwardBlockCommitRequest {
    pub token_id: TokenId,
    pub device_fingerprint: String,
}

impl Validate for InviteForwardBlockCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.token_id.validate()?;
        if self.device_fingerprint.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "invite_forward_block_commit_request.device_fingerprint",
                reason: "must not be empty",
            });
        }
        if self.device_fingerprint.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "invite_forward_block_commit_request.device_fingerprint",
                reason: "must be <= 256 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleProposeDraftRequest {
    pub tenant_id: Option<String>,
    pub proposal_text: String,
}

impl Validate for RoleProposeDraftRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_opt_id("role_propose_draft_request.tenant_id", &self.tenant_id, 64)?;
        if self.proposal_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "role_propose_draft_request.proposal_text",
                reason: "must not be empty",
            });
        }
        if self.proposal_text.len() > 1024 {
            return Err(ContractViolation::InvalidValue {
                field: "role_propose_draft_request.proposal_text",
                reason: "must be <= 1024 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DualRoleConflictEscalateDraftRequest {
    pub tenant_id: Option<String>,
    pub token_id: Option<TokenId>,
    pub note: String,
}

impl Validate for DualRoleConflictEscalateDraftRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_opt_id(
            "dual_role_conflict_escalate_draft_request.tenant_id",
            &self.tenant_id,
            64,
        )?;
        if let Some(id) = &self.token_id {
            id.validate()?;
        }
        if self.note.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "dual_role_conflict_escalate_draft_request.note",
                reason: "must not be empty",
            });
        }
        if self.note.len() > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "dual_role_conflict_escalate_draft_request.note",
                reason: "must be <= 512 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1LinkRequest {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub now: MonotonicTimeNs,
    pub simulation_id: String,
    pub simulation_type: SimulationType,
    pub request: LinkRequest,
}

impl Ph1LinkRequest {
    pub fn invite_generate_draft_v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        now: MonotonicTimeNs,
        inviter_user_id: UserId,
        invitee_type: InviteeType,
        tenant_id: Option<String>,
        schema_version_id: Option<String>,
        prefilled_context: Option<PrefilledContext>,
        expiration_policy_id: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let req = InviteGenerateDraftRequest {
            inviter_user_id,
            invitee_type,
            tenant_id,
            schema_version_id,
            prefilled_context,
            expiration_policy_id,
        };
        let r = Self {
            schema_version: PH1LINK_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: LINK_INVITE_GENERATE_DRAFT.to_string(),
            simulation_type: SimulationType::Draft,
            request: LinkRequest::InviteGenerateDraft(req),
        };
        r.validate()?;
        Ok(r)
    }

    pub fn invite_open_activate_commit_v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        now: MonotonicTimeNs,
        token_id: TokenId,
        device_fingerprint: String,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let req = InviteOpenActivateCommitRequest {
            token_id,
            device_fingerprint,
            idempotency_key,
        };
        let r = Self {
            schema_version: PH1LINK_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: LINK_INVITE_OPEN_ACTIVATE_COMMIT.to_string(),
            simulation_type: SimulationType::Commit,
            request: LinkRequest::InviteOpenActivateCommit(req),
        };
        r.validate()?;
        Ok(r)
    }

    pub fn invite_draft_update_commit_v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        now: MonotonicTimeNs,
        draft_id: DraftId,
        creator_update_fields: BTreeMap<String, String>,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let req = InviteDraftUpdateCommitRequest {
            draft_id,
            creator_update_fields,
            idempotency_key,
        };
        let r = Self {
            schema_version: PH1LINK_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: LINK_INVITE_DRAFT_UPDATE_COMMIT.to_string(),
            simulation_type: SimulationType::Commit,
            request: LinkRequest::InviteDraftUpdateCommit(req),
        };
        r.validate()?;
        Ok(r)
    }

    pub fn invite_revoke_revoke_v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        now: MonotonicTimeNs,
        token_id: TokenId,
        reason: String,
    ) -> Result<Self, ContractViolation> {
        let req = InviteRevokeRevokeRequest { token_id, reason };
        let r = Self {
            schema_version: PH1LINK_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            simulation_id: LINK_INVITE_REVOKE_REVOKE.to_string(),
            simulation_type: SimulationType::Revoke,
            request: LinkRequest::InviteRevokeRevoke(req),
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1LinkRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LINK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1link_request.schema_version",
                reason: "must match PH1LINK_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.now.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1link_request.now",
                reason: "must be > 0",
            });
        }
        if self.simulation_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1link_request.simulation_id",
                reason: "must not be empty",
            });
        }
        if self.simulation_id != self.request.simulation_id() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1link_request.simulation_id",
                reason: "must match the request variant's simulation_id",
            });
        }
        if self.simulation_type != self.request.simulation_type() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1link_request.simulation_type",
                reason: "must match the request variant's simulation_type",
            });
        }

        match &self.request {
            LinkRequest::InviteGenerateDraft(r) => r.validate(),
            LinkRequest::InviteDraftUpdateCommit(r) => r.validate(),
            LinkRequest::InviteOpenActivateCommit(r) => r.validate(),
            LinkRequest::InviteRevokeRevoke(r) => r.validate(),
            LinkRequest::InviteExpiredRecoveryCommit(r) => r.validate(),
            LinkRequest::InviteForwardBlockCommit(r) => r.validate(),
            LinkRequest::RoleProposeDraft(r) => r.validate(),
            LinkRequest::DualRoleConflictEscalateDraft(r) => r.validate(),
        }
    }
}

// Deterministic hashing helpers used by PH1.LINK implementations.
pub fn deterministic_payload_hash_hex(
    inviter_user_id: &UserId,
    invitee_type: InviteeType,
    tenant_id: &Option<String>,
    schema_version_id: &Option<String>,
    expiration_policy_id: &Option<String>,
    prefilled_context: &Option<PrefilledContext>,
) -> String {
    // Note: This is not a security hash. It is a deterministic idempotency key / content hash.
    let mut b: Vec<u8> = Vec::new();
    b.extend_from_slice(inviter_user_id.as_str().as_bytes());
    b.push(0);
    b.extend_from_slice(format!("{invitee_type:?}").as_bytes());
    b.push(0);
    if let Some(t) = tenant_id {
        b.extend_from_slice(t.as_bytes());
    }
    b.push(0);
    if let Some(s) = schema_version_id {
        b.extend_from_slice(s.as_bytes());
    }
    b.push(0);
    if let Some(p) = expiration_policy_id {
        b.extend_from_slice(p.as_bytes());
    }
    b.push(0);
    if let Some(ctx) = prefilled_context {
        // Deterministic, stable string representation (bounded fields only).
        if let Some(x) = &ctx.tenant_id {
            b.extend_from_slice(x.as_bytes());
        }
        b.push(0);
        if let Some(x) = &ctx.company_id {
            b.extend_from_slice(x.as_bytes());
        }
        b.push(0);
        if let Some(x) = &ctx.position_id {
            b.extend_from_slice(x.as_bytes());
        }
        b.push(0);
        if let Some(x) = &ctx.location_id {
            b.extend_from_slice(x.as_bytes());
        }
        b.push(0);
        if let Some(x) = &ctx.start_date {
            b.extend_from_slice(x.as_bytes());
        }
        b.push(0);
        if let Some(x) = &ctx.working_hours {
            b.extend_from_slice(x.as_bytes());
        }
        b.push(0);
        if let Some(x) = &ctx.compensation_tier_ref {
            b.extend_from_slice(x.as_bytes());
        }
        b.push(0);
        for t in &ctx.jurisdiction_tags {
            b.extend_from_slice(t.as_bytes());
            b.push(0);
        }
    }

    let mut h = fnv1a64(&b);
    if h == 0 {
        h = 1;
    }
    format!("{:016x}", h)
}

pub fn deterministic_contact_hash_hex(contact: &str) -> String {
    hash_hex_64(contact)
}

pub fn deterministic_device_fingerprint_hash_hex(device_fingerprint: &str) -> String {
    hash_hex_64(device_fingerprint)
}
