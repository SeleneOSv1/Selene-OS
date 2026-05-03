#![forbid(unsafe_code)]

use crate::{ContractViolation, MonotonicTimeNs, SchemaVersion, Validate};

pub const CONSENT_STATE_PACKET_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProviderSecretId {
    BraveSearchApiKey,
    OpenAIApiKey,
    GoogleSttApiKey,
    GoogleTtsApiKey,
    GoogleTimeZoneApiKey,
    TimeZoneDbApiKey,
    AzureSpeechKey,
    DeepgramApiKey,
    ElevenLabsApiKey,
    AnthropicApiKey,
    WeatherApiKey,
    TomorrowIoApiKey,
}

impl ProviderSecretId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BraveSearchApiKey => "brave_search_api_key",
            Self::OpenAIApiKey => "openai_api_key",
            Self::GoogleSttApiKey => "google_stt_api_key",
            Self::GoogleTtsApiKey => "google_tts_api_key",
            Self::GoogleTimeZoneApiKey => "google_time_zone_api_key",
            Self::TimeZoneDbApiKey => "timezonedb_api_key",
            Self::AzureSpeechKey => "azure_speech_key",
            Self::DeepgramApiKey => "deepgram_api_key",
            Self::ElevenLabsApiKey => "elevenlabs_api_key",
            Self::AnthropicApiKey => "anthropic_api_key",
            Self::WeatherApiKey => "weather_api_key",
            Self::TomorrowIoApiKey => "tomorrow_io_api_key",
        }
    }

    pub const fn all() -> &'static [Self] {
        &[
            Self::BraveSearchApiKey,
            Self::OpenAIApiKey,
            Self::GoogleSttApiKey,
            Self::GoogleTtsApiKey,
            Self::GoogleTimeZoneApiKey,
            Self::TimeZoneDbApiKey,
            Self::AzureSpeechKey,
            Self::DeepgramApiKey,
            Self::ElevenLabsApiKey,
            Self::AnthropicApiKey,
            Self::WeatherApiKey,
            Self::TomorrowIoApiKey,
        ]
    }

    pub fn parse(raw: &str) -> Option<Self> {
        let normalized = raw.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "brave_search_api_key" => Some(Self::BraveSearchApiKey),
            "openai_api_key" => Some(Self::OpenAIApiKey),
            "google_stt_api_key" => Some(Self::GoogleSttApiKey),
            "google_tts_api_key" => Some(Self::GoogleTtsApiKey),
            "google_time_zone_api_key" | "google_timezone_api_key" => {
                Some(Self::GoogleTimeZoneApiKey)
            }
            "timezonedb_api_key" | "timezone_db_api_key" => Some(Self::TimeZoneDbApiKey),
            "azure_speech_key" => Some(Self::AzureSpeechKey),
            "deepgram_api_key" => Some(Self::DeepgramApiKey),
            "elevenlabs_api_key" => Some(Self::ElevenLabsApiKey),
            "anthropic_api_key" => Some(Self::AnthropicApiKey),
            "weather_api_key" => Some(Self::WeatherApiKey),
            "tomorrow_io_api_key" => Some(Self::TomorrowIoApiKey),
            _ => None,
        }
    }

    pub fn allowed_key_names() -> Vec<&'static str> {
        Self::all().iter().map(|id| id.as_str()).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ConsentScope {
    WakeTraining,
    VoiceIdEnrollment,
    VoiceIdMatching,
    RecordMode,
    MemoryCapture,
    MemoryRecall,
    ProviderCapableVoiceProcessing,
}

impl ConsentScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WakeTraining => "WAKE_TRAINING",
            Self::VoiceIdEnrollment => "VOICE_ID_ENROLLMENT",
            Self::VoiceIdMatching => "VOICE_ID_MATCHING",
            Self::RecordMode => "RECORD_MODE",
            Self::MemoryCapture => "MEMORY_CAPTURE",
            Self::MemoryRecall => "MEMORY_RECALL",
            Self::ProviderCapableVoiceProcessing => "PROVIDER_CAPABLE_VOICE_PROCESSING",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsentDecisionState {
    Granted,
    Denied,
    Revoked,
}

impl ConsentDecisionState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Granted => "GRANTED",
            Self::Denied => "DENIED",
            Self::Revoked => "REVOKED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsentStatePacket {
    pub schema_version: SchemaVersion,
    pub consent_state_id: String,
    pub subject_user_ref: String,
    pub tenant_id: Option<String>,
    pub workspace_id: Option<String>,
    pub scope: ConsentScope,
    pub state: ConsentDecisionState,
    pub policy_ref: String,
    pub evidence_ref: Option<String>,
    pub created_at: MonotonicTimeNs,
    pub updated_at: MonotonicTimeNs,
    pub revoked_at: Option<MonotonicTimeNs>,
}

impl ConsentStatePacket {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        consent_state_id: String,
        subject_user_ref: String,
        tenant_id: Option<String>,
        workspace_id: Option<String>,
        scope: ConsentScope,
        state: ConsentDecisionState,
        policy_ref: String,
        evidence_ref: Option<String>,
        created_at: MonotonicTimeNs,
        updated_at: MonotonicTimeNs,
        revoked_at: Option<MonotonicTimeNs>,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            schema_version: CONSENT_STATE_PACKET_VERSION,
            consent_state_id,
            subject_user_ref,
            tenant_id,
            workspace_id,
            scope,
            state,
            policy_ref,
            evidence_ref,
            created_at,
            updated_at,
            revoked_at,
        };
        packet.validate()?;
        Ok(packet)
    }

    pub fn is_active_grant_for_scope(&self, scope: ConsentScope) -> bool {
        self.scope == scope
            && self.state == ConsentDecisionState::Granted
            && self.revoked_at.is_none()
    }
}

impl Validate for ConsentStatePacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != CONSENT_STATE_PACKET_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "consent_state_packet.schema_version",
                reason: "must match CONSENT_STATE_PACKET_VERSION",
            });
        }
        validate_ascii_text(
            "consent_state_packet.consent_state_id",
            &self.consent_state_id,
            128,
        )?;
        validate_ascii_text(
            "consent_state_packet.subject_user_ref",
            &self.subject_user_ref,
            128,
        )?;
        if let Some(tenant_id) = &self.tenant_id {
            validate_ascii_text("consent_state_packet.tenant_id", tenant_id, 96)?;
        }
        if let Some(workspace_id) = &self.workspace_id {
            validate_ascii_text("consent_state_packet.workspace_id", workspace_id, 96)?;
        }
        validate_ascii_text("consent_state_packet.policy_ref", &self.policy_ref, 128)?;
        if let Some(evidence_ref) = &self.evidence_ref {
            validate_ascii_text("consent_state_packet.evidence_ref", evidence_ref, 160)?;
        }

        if self.created_at.0 == 0 || self.updated_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "consent_state_packet.timestamps",
                reason: "created_at and updated_at must be > 0",
            });
        }
        if self.updated_at.0 < self.created_at.0 {
            return Err(ContractViolation::InvalidValue {
                field: "consent_state_packet.updated_at",
                reason: "must be >= created_at",
            });
        }
        match self.state {
            ConsentDecisionState::Revoked => {
                let revoked_at = self.revoked_at.ok_or(ContractViolation::InvalidValue {
                    field: "consent_state_packet.revoked_at",
                    reason: "must be present when state=REVOKED",
                })?;
                if revoked_at.0 < self.created_at.0 || revoked_at.0 > self.updated_at.0 {
                    return Err(ContractViolation::InvalidValue {
                        field: "consent_state_packet.revoked_at",
                        reason: "must be within created_at..=updated_at",
                    });
                }
            }
            ConsentDecisionState::Granted | ConsentDecisionState::Denied => {
                if self.revoked_at.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "consent_state_packet.revoked_at",
                        reason: "must be absent unless state=REVOKED",
                    });
                }
            }
        }

        Ok(())
    }
}

fn validate_ascii_text(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.trim().is_empty() || value.len() > max_len || !value.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty ASCII text within the length limit",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ConsentDecisionState, ConsentScope, ConsentStatePacket, ProviderSecretId};
    use crate::{MonotonicTimeNs, Validate};

    #[test]
    fn provider_secret_ids_are_roundtrippable() {
        for secret in ProviderSecretId::all() {
            let parsed = ProviderSecretId::parse(secret.as_str());
            assert_eq!(parsed, Some(*secret));
        }
    }

    #[test]
    fn stage3a_consent_state_packet_validates_revocation_and_active_grant() {
        let granted = ConsentStatePacket::v1(
            "consent_stage3a_voice_1".to_string(),
            "user:stage3a".to_string(),
            Some("tenant_stage3a".to_string()),
            Some("workspace_stage3a".to_string()),
            ConsentScope::ProviderCapableVoiceProcessing,
            ConsentDecisionState::Granted,
            "policy:voice-provider:v1".to_string(),
            Some("audit:evidence:1".to_string()),
            MonotonicTimeNs(10),
            MonotonicTimeNs(11),
            None,
        )
        .expect("granted consent packet should validate");
        assert!(granted.is_active_grant_for_scope(ConsentScope::ProviderCapableVoiceProcessing));
        assert!(!granted.is_active_grant_for_scope(ConsentScope::RecordMode));

        let revoked = ConsentStatePacket::v1(
            "consent_stage3a_voice_2".to_string(),
            "user:stage3a".to_string(),
            Some("tenant_stage3a".to_string()),
            None,
            ConsentScope::ProviderCapableVoiceProcessing,
            ConsentDecisionState::Revoked,
            "policy:voice-provider:v1".to_string(),
            Some("audit:revocation:1".to_string()),
            MonotonicTimeNs(10),
            MonotonicTimeNs(20),
            Some(MonotonicTimeNs(20)),
        )
        .expect("revoked consent packet should validate");
        assert!(!revoked.is_active_grant_for_scope(ConsentScope::ProviderCapableVoiceProcessing));

        let bad_revoked = ConsentStatePacket {
            revoked_at: None,
            ..revoked
        };
        assert!(bad_revoked.validate().is_err());
    }
}
