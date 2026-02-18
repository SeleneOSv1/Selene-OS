#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1ENDPOINT_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndpointCapabilityId {
    EndpointHintsBuild,
    EndpointBoundaryScore,
}

impl EndpointCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            EndpointCapabilityId::EndpointHintsBuild => "ENDPOINT_HINTS_BUILD",
            EndpointCapabilityId::EndpointBoundaryScore => "ENDPOINT_BOUNDARY_SCORE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndpointConfidenceBucket {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndpointValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_vad_windows: u8,
}

impl EndpointRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_vad_windows: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1ENDPOINT_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_vad_windows,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for EndpointRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ENDPOINT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_request_envelope.schema_version",
                reason: "must match PH1ENDPOINT_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_vad_windows == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_request_envelope.max_vad_windows",
                reason: "must be > 0",
            });
        }
        if self.max_vad_windows > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_request_envelope.max_vad_windows",
                reason: "must be <= 32",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointVadWindow {
    pub schema_version: SchemaVersion,
    pub window_id: String,
    pub t_start_ms: u32,
    pub t_end_ms: u32,
    pub vad_confidence_pct: u8,
    pub speech_likeness_pct: u8,
    pub trailing_silence_ms: u16,
}

impl EndpointVadWindow {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        window_id: String,
        t_start_ms: u32,
        t_end_ms: u32,
        vad_confidence_pct: u8,
        speech_likeness_pct: u8,
        trailing_silence_ms: u16,
    ) -> Result<Self, ContractViolation> {
        let window = Self {
            schema_version: PH1ENDPOINT_CONTRACT_VERSION,
            window_id,
            t_start_ms,
            t_end_ms,
            vad_confidence_pct,
            speech_likeness_pct,
            trailing_silence_ms,
        };
        window.validate()?;
        Ok(window)
    }
}

impl Validate for EndpointVadWindow {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ENDPOINT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_vad_window.schema_version",
                reason: "must match PH1ENDPOINT_CONTRACT_VERSION",
            });
        }
        validate_token("endpoint_vad_window.window_id", &self.window_id, 64)?;
        if self.t_start_ms >= self.t_end_ms {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_vad_window.t_start_ms",
                reason: "must be < t_end_ms",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointSegmentHint {
    pub schema_version: SchemaVersion,
    pub segment_id: String,
    pub window_id: String,
    pub t_start_ms: u32,
    pub t_end_ms: u32,
    pub endpoint_reason: String,
    pub endpoint_score_pct: u8,
    pub confidence_bucket: EndpointConfidenceBucket,
    pub should_finalize_turn: bool,
}

impl EndpointSegmentHint {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        segment_id: String,
        window_id: String,
        t_start_ms: u32,
        t_end_ms: u32,
        endpoint_reason: String,
        endpoint_score_pct: u8,
        confidence_bucket: EndpointConfidenceBucket,
        should_finalize_turn: bool,
    ) -> Result<Self, ContractViolation> {
        let hint = Self {
            schema_version: PH1ENDPOINT_CONTRACT_VERSION,
            segment_id,
            window_id,
            t_start_ms,
            t_end_ms,
            endpoint_reason,
            endpoint_score_pct,
            confidence_bucket,
            should_finalize_turn,
        };
        hint.validate()?;
        Ok(hint)
    }
}

impl Validate for EndpointSegmentHint {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ENDPOINT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_segment_hint.schema_version",
                reason: "must match PH1ENDPOINT_CONTRACT_VERSION",
            });
        }
        validate_token("endpoint_segment_hint.segment_id", &self.segment_id, 64)?;
        validate_token("endpoint_segment_hint.window_id", &self.window_id, 64)?;
        if self.t_start_ms >= self.t_end_ms {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_segment_hint.t_start_ms",
                reason: "must be < t_end_ms",
            });
        }
        validate_token(
            "endpoint_segment_hint.endpoint_reason",
            &self.endpoint_reason,
            64,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointHintsBuildRequest {
    pub schema_version: SchemaVersion,
    pub envelope: EndpointRequestEnvelope,
    pub vad_windows: Vec<EndpointVadWindow>,
    pub transcript_token_estimate: u16,
    pub tts_playback_active: bool,
}

impl EndpointHintsBuildRequest {
    pub fn v1(
        envelope: EndpointRequestEnvelope,
        vad_windows: Vec<EndpointVadWindow>,
        transcript_token_estimate: u16,
        tts_playback_active: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1ENDPOINT_CONTRACT_VERSION,
            envelope,
            vad_windows,
            transcript_token_estimate,
            tts_playback_active,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for EndpointHintsBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ENDPOINT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_hints_build_request.schema_version",
                reason: "must match PH1ENDPOINT_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        if self.vad_windows.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_hints_build_request.vad_windows",
                reason: "must not be empty",
            });
        }
        if self.vad_windows.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_hints_build_request.vad_windows",
                reason: "must be <= 32",
            });
        }

        let mut seen_window_ids: BTreeSet<&str> = BTreeSet::new();
        let mut previous_end: Option<u32> = None;
        for window in &self.vad_windows {
            window.validate()?;
            if !seen_window_ids.insert(window.window_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "endpoint_hints_build_request.vad_windows",
                    reason: "window_id must be unique",
                });
            }
            if let Some(prev_end) = previous_end {
                if window.t_start_ms < prev_end {
                    return Err(ContractViolation::InvalidValue {
                        field: "endpoint_hints_build_request.vad_windows",
                        reason: "windows must be non-overlapping and ordered by t_start_ms",
                    });
                }
            }
            previous_end = Some(window.t_end_ms);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointBoundaryScoreRequest {
    pub schema_version: SchemaVersion,
    pub envelope: EndpointRequestEnvelope,
    pub selected_segment_id: String,
    pub ordered_segment_hints: Vec<EndpointSegmentHint>,
    pub previous_selected_segment_id: Option<String>,
}

impl EndpointBoundaryScoreRequest {
    pub fn v1(
        envelope: EndpointRequestEnvelope,
        selected_segment_id: String,
        ordered_segment_hints: Vec<EndpointSegmentHint>,
        previous_selected_segment_id: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1ENDPOINT_CONTRACT_VERSION,
            envelope,
            selected_segment_id,
            ordered_segment_hints,
            previous_selected_segment_id,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for EndpointBoundaryScoreRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ENDPOINT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_boundary_score_request.schema_version",
                reason: "must match PH1ENDPOINT_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "endpoint_boundary_score_request.selected_segment_id",
            &self.selected_segment_id,
            64,
        )?;
        if self.ordered_segment_hints.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_boundary_score_request.ordered_segment_hints",
                reason: "must not be empty",
            });
        }
        if self.ordered_segment_hints.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_boundary_score_request.ordered_segment_hints",
                reason: "must be <= 32",
            });
        }
        let mut seen_segment_ids: BTreeSet<&str> = BTreeSet::new();
        for hint in &self.ordered_segment_hints {
            hint.validate()?;
            if !seen_segment_ids.insert(hint.segment_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "endpoint_boundary_score_request.ordered_segment_hints",
                    reason: "segment_id must be unique",
                });
            }
        }
        if let Some(previous) = &self.previous_selected_segment_id {
            validate_token(
                "endpoint_boundary_score_request.previous_selected_segment_id",
                previous,
                64,
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1EndpointRequest {
    EndpointHintsBuild(EndpointHintsBuildRequest),
    EndpointBoundaryScore(EndpointBoundaryScoreRequest),
}

impl Validate for Ph1EndpointRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1EndpointRequest::EndpointHintsBuild(req) => req.validate(),
            Ph1EndpointRequest::EndpointBoundaryScore(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointHintsBuildOk {
    pub schema_version: SchemaVersion,
    pub capability_id: EndpointCapabilityId,
    pub reason_code: ReasonCodeId,
    pub selected_segment_id: String,
    pub ordered_segment_hints: Vec<EndpointSegmentHint>,
    pub no_semantic_mutation: bool,
    pub no_execution_authority: bool,
}

impl EndpointHintsBuildOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        selected_segment_id: String,
        ordered_segment_hints: Vec<EndpointSegmentHint>,
        no_semantic_mutation: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1ENDPOINT_CONTRACT_VERSION,
            capability_id: EndpointCapabilityId::EndpointHintsBuild,
            reason_code,
            selected_segment_id,
            ordered_segment_hints,
            no_semantic_mutation,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for EndpointHintsBuildOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ENDPOINT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_hints_build_ok.schema_version",
                reason: "must match PH1ENDPOINT_CONTRACT_VERSION",
            });
        }
        if self.capability_id != EndpointCapabilityId::EndpointHintsBuild {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_hints_build_ok.capability_id",
                reason: "must be ENDPOINT_HINTS_BUILD",
            });
        }
        validate_token(
            "endpoint_hints_build_ok.selected_segment_id",
            &self.selected_segment_id,
            64,
        )?;
        if self.ordered_segment_hints.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_hints_build_ok.ordered_segment_hints",
                reason: "must not be empty",
            });
        }
        if self.ordered_segment_hints.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_hints_build_ok.ordered_segment_hints",
                reason: "must be <= 32",
            });
        }
        let mut seen_segment_ids: BTreeSet<&str> = BTreeSet::new();
        for hint in &self.ordered_segment_hints {
            hint.validate()?;
            if !seen_segment_ids.insert(hint.segment_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "endpoint_hints_build_ok.ordered_segment_hints",
                    reason: "segment_id must be unique",
                });
            }
        }
        if !self
            .ordered_segment_hints
            .iter()
            .any(|hint| hint.segment_id == self.selected_segment_id)
        {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_hints_build_ok.selected_segment_id",
                reason: "must exist in ordered_segment_hints",
            });
        }
        if !self.no_semantic_mutation {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_hints_build_ok.no_semantic_mutation",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_hints_build_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointBoundaryScoreOk {
    pub schema_version: SchemaVersion,
    pub capability_id: EndpointCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: EndpointValidationStatus,
    pub diagnostics: Vec<String>,
    pub no_semantic_mutation: bool,
    pub no_execution_authority: bool,
}

impl EndpointBoundaryScoreOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: EndpointValidationStatus,
        diagnostics: Vec<String>,
        no_semantic_mutation: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1ENDPOINT_CONTRACT_VERSION,
            capability_id: EndpointCapabilityId::EndpointBoundaryScore,
            reason_code,
            validation_status,
            diagnostics,
            no_semantic_mutation,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for EndpointBoundaryScoreOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ENDPOINT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_boundary_score_ok.schema_version",
                reason: "must match PH1ENDPOINT_CONTRACT_VERSION",
            });
        }
        if self.capability_id != EndpointCapabilityId::EndpointBoundaryScore {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_boundary_score_ok.capability_id",
                reason: "must be ENDPOINT_BOUNDARY_SCORE",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_boundary_score_ok.diagnostics",
                reason: "must be <= 16 entries",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_token("endpoint_boundary_score_ok.diagnostics", diagnostic, 96)?;
        }
        if self.validation_status == EndpointValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_boundary_score_ok.diagnostics",
                reason: "must include diagnostics when validation_status=FAIL",
            });
        }
        if !self.no_semantic_mutation {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_boundary_score_ok.no_semantic_mutation",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_boundary_score_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: EndpointCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl EndpointRefuse {
    pub fn v1(
        capability_id: EndpointCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1ENDPOINT_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for EndpointRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ENDPOINT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_refuse.schema_version",
                reason: "must match PH1ENDPOINT_CONTRACT_VERSION",
            });
        }
        validate_token("endpoint_refuse.message", &self.message, 256)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1EndpointResponse {
    EndpointHintsBuildOk(EndpointHintsBuildOk),
    EndpointBoundaryScoreOk(EndpointBoundaryScoreOk),
    Refuse(EndpointRefuse),
}

impl Validate for Ph1EndpointResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1EndpointResponse::EndpointHintsBuildOk(out) => out.validate(),
            Ph1EndpointResponse::EndpointBoundaryScoreOk(out) => out.validate(),
            Ph1EndpointResponse::Refuse(out) => out.validate(),
        }
    }
}

fn validate_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if value.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(max_vad_windows: u8) -> EndpointRequestEnvelope {
        EndpointRequestEnvelope::v1(CorrelationId(1), TurnId(1), max_vad_windows).unwrap()
    }

    fn window() -> EndpointVadWindow {
        EndpointVadWindow::v1("window_1".to_string(), 0, 420, 90, 87, 320).unwrap()
    }

    fn hint() -> EndpointSegmentHint {
        EndpointSegmentHint::v1(
            "segment_1".to_string(),
            "window_1".to_string(),
            0,
            420,
            "silence_window".to_string(),
            92,
            EndpointConfidenceBucket::High,
            true,
        )
        .unwrap()
    }

    #[test]
    fn endpoint_hints_build_request_rejects_empty_windows() {
        let req = EndpointHintsBuildRequest::v1(envelope(8), vec![], 20, false);
        assert!(req.is_err());
    }

    #[test]
    fn endpoint_hints_build_ok_requires_selected_segment_to_exist() {
        let out = EndpointHintsBuildOk::v1(
            ReasonCodeId(1),
            "segment_2".to_string(),
            vec![hint()],
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn endpoint_boundary_score_ok_requires_diagnostics_when_status_fail() {
        let out = EndpointBoundaryScoreOk::v1(
            ReasonCodeId(2),
            EndpointValidationStatus::Fail,
            vec![],
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn endpoint_boundary_score_request_rejects_duplicate_segment_ids() {
        let req = EndpointBoundaryScoreRequest::v1(
            envelope(8),
            "segment_1".to_string(),
            vec![hint(), hint()],
            None,
        );
        assert!(req.is_err());
    }

    #[test]
    fn endpoint_hints_build_request_rejects_overlapping_vad_windows() {
        let req = EndpointHintsBuildRequest::v1(
            envelope(8),
            vec![
                EndpointVadWindow::v1("window_1".to_string(), 0, 300, 90, 90, 250).unwrap(),
                EndpointVadWindow::v1("window_2".to_string(), 280, 500, 85, 82, 120).unwrap(),
            ],
            20,
            false,
        );
        assert!(req.is_err());
    }

    #[test]
    fn endpoint_hints_build_request_accepts_non_overlapping_vad_windows() {
        let req = EndpointHintsBuildRequest::v1(envelope(8), vec![window()], 20, false);
        assert!(req.is_ok());
    }
}
