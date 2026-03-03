#![forbid(unsafe_code)]

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1VISION_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VisionCapabilityId {
    EvidenceExtract,
    VisibleContentValidate,
    AnalyzeMedia,
}

impl VisionCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            VisionCapabilityId::EvidenceExtract => "VISION_EVIDENCE_EXTRACT",
            VisionCapabilityId::VisibleContentValidate => "VISION_VISIBLE_CONTENT_VALIDATE",
            VisionCapabilityId::AnalyzeMedia => "VISION_ANALYZE_MEDIA",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub opt_in_enabled: bool,
    pub max_evidence_items: u8,
}

impl VisionRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        opt_in_enabled: bool,
        max_evidence_items: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            opt_in_enabled,
            max_evidence_items,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for VisionRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_request_envelope.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_evidence_items == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_request_envelope.max_evidence_items",
                reason: "must be > 0",
            });
        }
        if self.max_evidence_items > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_request_envelope.max_evidence_items",
                reason: "must be <= 64",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VisualSourceKind {
    Image,
    Screenshot,
    Diagram,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VisualSourceId(String);

impl VisualSourceId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "visual_source_id",
                reason: "must not be empty",
            });
        }
        if id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "visual_source_id",
                reason: "must be <= 128 chars",
            });
        }
        if id.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "visual_source_id",
                reason: "must not contain control characters",
            });
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for VisualSourceId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "visual_source_id",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "visual_source_id",
                reason: "must be <= 128 chars",
            });
        }
        if self.0.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "visual_source_id",
                reason: "must not contain control characters",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VisualSourceRef {
    pub schema_version: SchemaVersion,
    pub source_id: VisualSourceId,
    pub source_kind: VisualSourceKind,
}

impl VisualSourceRef {
    pub fn v1(
        source_id: VisualSourceId,
        source_kind: VisualSourceKind,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            source_id,
            source_kind,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for VisualSourceRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "visual_source_ref.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        self.source_id.validate()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundingBoxPx {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl BoundingBoxPx {
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Result<Self, ContractViolation> {
        let b = Self { x, y, w, h };
        b.validate()?;
        Ok(b)
    }
}

impl Validate for BoundingBoxPx {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.w == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "bounding_box_px.w",
                reason: "must be > 0",
            });
        }
        if self.h == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "bounding_box_px.h",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VisualToken {
    pub schema_version: SchemaVersion,
    pub token: String,
    pub bbox: Option<BoundingBoxPx>,
}

impl VisualToken {
    pub fn v1(token: String, bbox: Option<BoundingBoxPx>) -> Result<Self, ContractViolation> {
        let t = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            token,
            bbox,
        };
        t.validate()?;
        Ok(t)
    }
}

impl Validate for VisualToken {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "visual_token.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.token.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "visual_token.token",
                reason: "must not be empty",
            });
        }
        if self.token.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "visual_token.token",
                reason: "must be <= 256 chars",
            });
        }
        if self.token.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "visual_token.token",
                reason: "must not contain control characters",
            });
        }
        if let Some(b) = &self.bbox {
            b.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionEvidenceExtractRequest {
    pub schema_version: SchemaVersion,
    pub envelope: VisionRequestEnvelope,
    pub source_ref: VisualSourceRef,
    /// Input token list is the strict visible-content plane. No inference beyond these tokens.
    pub visible_tokens: Vec<VisualToken>,
}

impl VisionEvidenceExtractRequest {
    pub fn v1(
        envelope: VisionRequestEnvelope,
        source_ref: VisualSourceRef,
        visible_tokens: Vec<VisualToken>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            envelope,
            source_ref,
            visible_tokens,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for VisionEvidenceExtractRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_extract_request.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.source_ref.validate()?;
        if self.visible_tokens.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_extract_request.visible_tokens",
                reason: "must not be empty",
            });
        }
        if self.visible_tokens.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_extract_request.visible_tokens",
                reason: "must be <= 256 items",
            });
        }
        for token in &self.visible_tokens {
            token.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionVisibleContentValidateRequest {
    pub schema_version: SchemaVersion,
    pub envelope: VisionRequestEnvelope,
    pub source_ref: VisualSourceRef,
    pub visible_tokens: Vec<VisualToken>,
    pub evidence_items: Vec<VisionEvidenceItem>,
}

impl VisionVisibleContentValidateRequest {
    pub fn v1(
        envelope: VisionRequestEnvelope,
        source_ref: VisualSourceRef,
        visible_tokens: Vec<VisualToken>,
        evidence_items: Vec<VisionEvidenceItem>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            envelope,
            source_ref,
            visible_tokens,
            evidence_items,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for VisionVisibleContentValidateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_request.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.source_ref.validate()?;

        if self.visible_tokens.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_request.visible_tokens",
                reason: "must not be empty",
            });
        }
        if self.visible_tokens.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_request.visible_tokens",
                reason: "must be <= 256 items",
            });
        }
        for token in &self.visible_tokens {
            token.validate()?;
        }

        if self.evidence_items.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_request.evidence_items",
                reason: "must not be empty",
            });
        }
        if self.evidence_items.len() > self.envelope.max_evidence_items as usize {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_request.evidence_items",
                reason: "must be <= envelope.max_evidence_items",
            });
        }
        for item in &self.evidence_items {
            item.validate()?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VisionMediaMode {
    ImageOcr,
    ImageObjects,
    ImageAnalyze,
    VideoTranscribe,
    VideoKeyframes,
    VideoAnalyze,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionAssetRef {
    pub schema_version: SchemaVersion,
    pub asset_hash: String,
    pub locator: String,
    pub mime_type: String,
    pub size_bytes: u64,
}

impl VisionAssetRef {
    pub fn v1(
        asset_hash: String,
        locator: String,
        mime_type: String,
        size_bytes: u64,
    ) -> Result<Self, ContractViolation> {
        let asset = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            asset_hash,
            locator,
            mime_type,
            size_bytes,
        };
        asset.validate()?;
        Ok(asset)
    }
}

impl Validate for VisionAssetRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_asset_ref.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.asset_hash.len() != 64 || !self.asset_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ContractViolation::InvalidValue {
                field: "vision_asset_ref.asset_hash",
                reason: "must be 64-char hex sha256 digest",
            });
        }
        if self.locator.trim().is_empty() || self.locator.len() > 2048 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_asset_ref.locator",
                reason: "must be non-empty and <= 2048 chars",
            });
        }
        if self.locator.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "vision_asset_ref.locator",
                reason: "must not contain control characters",
            });
        }
        if self.mime_type.trim().is_empty() || self.mime_type.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_asset_ref.mime_type",
                reason: "must be non-empty and <= 128 chars",
            });
        }
        if self.size_bytes == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_asset_ref.size_bytes",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionMediaOptions {
    pub schema_version: SchemaVersion,
    pub language_hint: Option<String>,
    pub max_frames: Option<u32>,
    pub frame_stride_ms: Option<u32>,
    pub safe_mode: bool,
}

impl VisionMediaOptions {
    pub fn v1(
        language_hint: Option<String>,
        max_frames: Option<u32>,
        frame_stride_ms: Option<u32>,
        safe_mode: bool,
    ) -> Result<Self, ContractViolation> {
        let options = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            language_hint,
            max_frames,
            frame_stride_ms,
            safe_mode,
        };
        options.validate()?;
        Ok(options)
    }
}

impl Validate for VisionMediaOptions {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_media_options.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if let Some(language_hint) = &self.language_hint {
            if language_hint.trim().is_empty() || language_hint.len() > 32 {
                return Err(ContractViolation::InvalidValue {
                    field: "vision_media_options.language_hint",
                    reason: "must be non-empty and <= 32 chars when provided",
                });
            }
        }
        if let Some(max_frames) = self.max_frames {
            if max_frames == 0 || max_frames > 1024 {
                return Err(ContractViolation::InvalidValue {
                    field: "vision_media_options.max_frames",
                    reason: "must be in 1..=1024 when provided",
                });
            }
        }
        if let Some(stride) = self.frame_stride_ms {
            if stride == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "vision_media_options.frame_stride_ms",
                    reason: "must be > 0 when provided",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionMediaBudgets {
    pub schema_version: SchemaVersion,
    pub timeout_ms: u64,
    pub max_bytes: u64,
}

impl VisionMediaBudgets {
    pub fn v1(timeout_ms: u64, max_bytes: u64) -> Result<Self, ContractViolation> {
        let budgets = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            timeout_ms,
            max_bytes,
        };
        budgets.validate()?;
        Ok(budgets)
    }
}

impl Validate for VisionMediaBudgets {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_media_budgets.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.timeout_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_media_budgets.timeout_ms",
                reason: "must be > 0",
            });
        }
        if self.max_bytes == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_media_budgets.max_bytes",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionAnalyzeMediaRequest {
    pub schema_version: SchemaVersion,
    pub envelope: VisionRequestEnvelope,
    pub mode: VisionMediaMode,
    pub asset_ref: VisionAssetRef,
    pub options: VisionMediaOptions,
    pub budgets: VisionMediaBudgets,
    pub policy_snapshot_id: String,
}

impl VisionAnalyzeMediaRequest {
    pub fn v1(
        envelope: VisionRequestEnvelope,
        mode: VisionMediaMode,
        asset_ref: VisionAssetRef,
        options: VisionMediaOptions,
        budgets: VisionMediaBudgets,
        policy_snapshot_id: String,
    ) -> Result<Self, ContractViolation> {
        let request = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            envelope,
            mode,
            asset_ref,
            options,
            budgets,
            policy_snapshot_id,
        };
        request.validate()?;
        Ok(request)
    }
}

impl Validate for VisionAnalyzeMediaRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_analyze_media_request.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.asset_ref.validate()?;
        self.options.validate()?;
        self.budgets.validate()?;
        if self.policy_snapshot_id.trim().is_empty() || self.policy_snapshot_id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_analyze_media_request.policy_snapshot_id",
                reason: "must be non-empty and <= 128 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisionBoundingBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl VisionBoundingBox {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Result<Self, ContractViolation> {
        let bbox = Self { x, y, w, h };
        bbox.validate()?;
        Ok(bbox)
    }
}

impl Validate for VisionBoundingBox {
    fn validate(&self) -> Result<(), ContractViolation> {
        for (field_name, value) in [
            ("vision_bounding_box.x", self.x),
            ("vision_bounding_box.y", self.y),
            ("vision_bounding_box.w", self.w),
            ("vision_bounding_box.h", self.h),
        ] {
            if !value.is_finite() {
                return Err(ContractViolation::NotFinite { field: field_name });
            }
        }
        if self.w <= 0.0 || self.h <= 0.0 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_bounding_box",
                reason: "w and h must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisionOcrTextBlock {
    pub schema_version: SchemaVersion,
    pub bbox: VisionBoundingBox,
    pub text: String,
    pub confidence: f64,
    pub pii_suspected: Option<bool>,
}

impl VisionOcrTextBlock {
    pub fn v1(
        bbox: VisionBoundingBox,
        text: String,
        confidence: f64,
        pii_suspected: Option<bool>,
    ) -> Result<Self, ContractViolation> {
        let block = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            bbox,
            text,
            confidence,
            pii_suspected,
        };
        block.validate()?;
        Ok(block)
    }
}

impl Validate for VisionOcrTextBlock {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_ocr_text_block.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        self.bbox.validate()?;
        if self.text.trim().is_empty() || self.text.len() > 4096 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_ocr_text_block.text",
                reason: "must be non-empty and <= 4096 chars",
            });
        }
        if !self.confidence.is_finite() {
            return Err(ContractViolation::NotFinite {
                field: "vision_ocr_text_block.confidence",
            });
        }
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(ContractViolation::InvalidRange {
                field: "vision_ocr_text_block.confidence",
                min: 0.0,
                max: 1.0,
                got: self.confidence,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisionOcrResult {
    pub schema_version: SchemaVersion,
    pub page_or_frame_index: u32,
    pub timestamp_ms: Option<u64>,
    pub ocr_engine_id: String,
    pub language: String,
    pub text_blocks: Vec<VisionOcrTextBlock>,
    pub full_text: String,
}

impl Validate for VisionOcrResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_ocr_result.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.ocr_engine_id.trim().is_empty() || self.ocr_engine_id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_ocr_result.ocr_engine_id",
                reason: "must be non-empty and <= 128 chars",
            });
        }
        if self.language.trim().is_empty() || self.language.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_ocr_result.language",
                reason: "must be non-empty and <= 32 chars",
            });
        }
        if self.text_blocks.len() > 4096 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_ocr_result.text_blocks",
                reason: "must be <= 4096 blocks",
            });
        }
        for block in &self.text_blocks {
            block.validate()?;
        }
        if self.full_text.len() > 262_144 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_ocr_result.full_text",
                reason: "must be <= 262144 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisionDetectedObject {
    pub schema_version: SchemaVersion,
    pub label: String,
    pub bbox: VisionBoundingBox,
    pub confidence: f64,
}

impl Validate for VisionDetectedObject {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_detected_object.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.label.trim().is_empty() || self.label.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_detected_object.label",
                reason: "must be non-empty and <= 128 chars",
            });
        }
        self.bbox.validate()?;
        if !self.confidence.is_finite() {
            return Err(ContractViolation::NotFinite {
                field: "vision_detected_object.confidence",
            });
        }
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(ContractViolation::InvalidRange {
                field: "vision_detected_object.confidence",
                min: 0.0,
                max: 1.0,
                got: self.confidence,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisionObjectDetectionResult {
    pub schema_version: SchemaVersion,
    pub frame_index: Option<u32>,
    pub timestamp_ms: Option<u64>,
    pub model_id: String,
    pub objects: Vec<VisionDetectedObject>,
}

impl Validate for VisionObjectDetectionResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_object_detection_result.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.model_id.trim().is_empty() || self.model_id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_object_detection_result.model_id",
                reason: "must be non-empty and <= 128 chars",
            });
        }
        if self.objects.len() > 4096 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_object_detection_result.objects",
                reason: "must be <= 4096 items",
            });
        }
        for object in &self.objects {
            object.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionTranscriptSegment {
    pub schema_version: SchemaVersion,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
    pub confidence: u32,
}

impl Validate for VisionTranscriptSegment {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_transcript_segment.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.end_ms < self.start_ms {
            return Err(ContractViolation::InvalidValue {
                field: "vision_transcript_segment",
                reason: "end_ms must be >= start_ms",
            });
        }
        if self.text.trim().is_empty() || self.text.len() > 4096 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_transcript_segment.text",
                reason: "must be non-empty and <= 4096 chars",
            });
        }
        if self.confidence > 100 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_transcript_segment.confidence",
                reason: "must be <= 100",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionVideoTranscriptResult {
    pub schema_version: SchemaVersion,
    pub stt_provider_id: String,
    pub language: String,
    pub segments: Vec<VisionTranscriptSegment>,
    pub full_transcript: String,
}

impl Validate for VisionVideoTranscriptResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_video_transcript_result.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.stt_provider_id.trim().is_empty() || self.stt_provider_id.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_video_transcript_result.stt_provider_id",
                reason: "must be non-empty and <= 64 chars",
            });
        }
        if self.language.trim().is_empty() || self.language.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_video_transcript_result.language",
                reason: "must be non-empty and <= 32 chars",
            });
        }
        if self.segments.len() > 4096 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_video_transcript_result.segments",
                reason: "must be <= 4096 items",
            });
        }
        for segment in &self.segments {
            segment.validate()?;
        }
        if self.full_transcript.len() > 262_144 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_video_transcript_result.full_transcript",
                reason: "must be <= 262144 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionKeyframeEntry {
    pub schema_version: SchemaVersion,
    pub timestamp_ms: u64,
    pub frame_index: u32,
    pub frame_hash: String,
}

impl Validate for VisionKeyframeEntry {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_keyframe_entry.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.frame_hash.len() != 64 || !self.frame_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ContractViolation::InvalidValue {
                field: "vision_keyframe_entry.frame_hash",
                reason: "must be 64-char hex sha256 digest",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionKeyframeIndexResult {
    pub schema_version: SchemaVersion,
    pub keyframes: Vec<VisionKeyframeEntry>,
}

impl Validate for VisionKeyframeIndexResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_keyframe_index_result.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.keyframes.len() > 4096 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_keyframe_index_result.keyframes",
                reason: "must be <= 4096 items",
            });
        }
        for frame in &self.keyframes {
            frame.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VisionMediaOutput {
    OCRResult(VisionOcrResult),
    ObjectDetectionResult(VisionObjectDetectionResult),
    VideoTranscriptResult(VisionVideoTranscriptResult),
    KeyframeIndexResult(VisionKeyframeIndexResult),
}

impl Validate for VisionMediaOutput {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Self::OCRResult(value) => value.validate(),
            Self::ObjectDetectionResult(value) => value.validate(),
            Self::VideoTranscriptResult(value) => value.validate(),
            Self::KeyframeIndexResult(value) => value.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionProviderErrorRecord {
    pub schema_version: SchemaVersion,
    pub error_kind: String,
    pub reason_code: String,
    pub message: String,
}

impl Validate for VisionProviderErrorRecord {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_provider_error_record.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.error_kind.trim().is_empty() || self.error_kind.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_provider_error_record.error_kind",
                reason: "must be non-empty and <= 64 chars",
            });
        }
        if self.reason_code.trim().is_empty() || self.reason_code.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_provider_error_record.reason_code",
                reason: "must be non-empty and <= 64 chars",
            });
        }
        if self.message.trim().is_empty() || self.message.len() > 1024 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_provider_error_record.message",
                reason: "must be non-empty and <= 1024 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionProviderRun {
    pub schema_version: SchemaVersion,
    pub provider_id: String,
    pub endpoint: String,
    pub latency_ms: u64,
    pub error: Option<VisionProviderErrorRecord>,
}

impl Validate for VisionProviderRun {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_provider_run.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.provider_id.trim().is_empty() || self.provider_id.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_provider_run.provider_id",
                reason: "must be non-empty and <= 64 chars",
            });
        }
        if self.endpoint.trim().is_empty() || self.endpoint.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_provider_run.endpoint",
                reason: "must be non-empty and <= 32 chars",
            });
        }
        if let Some(error) = &self.error {
            error.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisionConfidenceSummary {
    pub schema_version: SchemaVersion,
    pub mean_confidence: f64,
    pub ocr_blocks_retained: u64,
    pub objects_retained: u64,
    pub transcript_segments_retained: u64,
    pub output_count: u64,
}

impl Validate for VisionConfidenceSummary {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_confidence_summary.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if !self.mean_confidence.is_finite() {
            return Err(ContractViolation::NotFinite {
                field: "vision_confidence_summary.mean_confidence",
            });
        }
        if !(0.0..=1.0).contains(&self.mean_confidence) {
            return Err(ContractViolation::InvalidRange {
                field: "vision_confidence_summary.mean_confidence",
                min: 0.0,
                max: 1.0,
                got: self.mean_confidence,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionPacketHashes {
    pub schema_version: SchemaVersion,
    pub asset_hash: String,
    pub provider_runs_hash: String,
    pub outputs_hash: String,
}

impl Validate for VisionPacketHashes {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_packet_hashes.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        for (field_name, value) in [
            ("vision_packet_hashes.asset_hash", &self.asset_hash),
            (
                "vision_packet_hashes.provider_runs_hash",
                &self.provider_runs_hash,
            ),
            ("vision_packet_hashes.outputs_hash", &self.outputs_hash),
        ] {
            if value.len() != 64 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(ContractViolation::InvalidValue {
                    field: field_name,
                    reason: "must be 64-char hex sha256 digest",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisionAnalyzeMediaOk {
    pub schema_version: SchemaVersion,
    pub capability_id: VisionCapabilityId,
    pub reason_code: ReasonCodeId,
    pub asset_ref: VisionAssetRef,
    pub provider_runs: Vec<VisionProviderRun>,
    pub outputs: Vec<VisionMediaOutput>,
    pub confidence_summary: VisionConfidenceSummary,
    pub reason_codes: Vec<String>,
    pub packet_hashes: VisionPacketHashes,
    pub output_hash: String,
}

impl VisionAnalyzeMediaOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        asset_ref: VisionAssetRef,
        provider_runs: Vec<VisionProviderRun>,
        outputs: Vec<VisionMediaOutput>,
        confidence_summary: VisionConfidenceSummary,
        reason_codes: Vec<String>,
        packet_hashes: VisionPacketHashes,
        output_hash: String,
    ) -> Result<Self, ContractViolation> {
        let ok = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            capability_id: VisionCapabilityId::AnalyzeMedia,
            reason_code,
            asset_ref,
            provider_runs,
            outputs,
            confidence_summary,
            reason_codes,
            packet_hashes,
            output_hash,
        };
        ok.validate()?;
        Ok(ok)
    }
}

impl Validate for VisionAnalyzeMediaOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_analyze_media_ok.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.capability_id != VisionCapabilityId::AnalyzeMedia {
            return Err(ContractViolation::InvalidValue {
                field: "vision_analyze_media_ok.capability_id",
                reason: "must be VISION_ANALYZE_MEDIA",
            });
        }
        self.asset_ref.validate()?;
        if self.provider_runs.is_empty() || self.provider_runs.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_analyze_media_ok.provider_runs",
                reason: "must be in 1..=128",
            });
        }
        for run in &self.provider_runs {
            run.validate()?;
        }
        if self.outputs.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_analyze_media_ok.outputs",
                reason: "must be <= 128",
            });
        }
        for output in &self.outputs {
            output.validate()?;
        }
        self.confidence_summary.validate()?;
        if self.reason_codes.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_analyze_media_ok.reason_codes",
                reason: "must be <= 64",
            });
        }
        for reason_code in &self.reason_codes {
            if reason_code.trim().is_empty() || reason_code.len() > 64 {
                return Err(ContractViolation::InvalidValue {
                    field: "vision_analyze_media_ok.reason_codes",
                    reason: "entries must be non-empty and <= 64 chars",
                });
            }
        }
        self.packet_hashes.validate()?;
        if self.output_hash.len() != 64 || !self.output_hash.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Err(ContractViolation::InvalidValue {
                field: "vision_analyze_media_ok.output_hash",
                reason: "must be 64-char hex sha256 digest",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1VisionRequest {
    EvidenceExtract(VisionEvidenceExtractRequest),
    VisibleContentValidate(VisionVisibleContentValidateRequest),
    AnalyzeMedia(VisionAnalyzeMediaRequest),
}

impl Validate for Ph1VisionRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1VisionRequest::EvidenceExtract(r) => r.validate(),
            Ph1VisionRequest::VisibleContentValidate(r) => r.validate(),
            Ph1VisionRequest::AnalyzeMedia(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VisionEvidenceItem {
    pub schema_version: SchemaVersion,
    pub text: String,
    pub bbox: Option<BoundingBoxPx>,
}

impl VisionEvidenceItem {
    pub fn v1(text: String, bbox: Option<BoundingBoxPx>) -> Result<Self, ContractViolation> {
        let i = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            text,
            bbox,
        };
        i.validate()?;
        Ok(i)
    }
}

impl Validate for VisionEvidenceItem {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_item.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_item.text",
                reason: "must not be empty",
            });
        }
        if self.text.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_item.text",
                reason: "must be <= 256 chars",
            });
        }
        if self.text.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_item.text",
                reason: "must not contain control characters",
            });
        }
        if let Some(b) = &self.bbox {
            b.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VisionValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionEvidenceExtractOk {
    pub schema_version: SchemaVersion,
    pub capability_id: VisionCapabilityId,
    pub reason_code: ReasonCodeId,
    pub source_ref: VisualSourceRef,
    pub evidence_items: Vec<VisionEvidenceItem>,
    pub visible_content_only: bool,
}

impl VisionEvidenceExtractOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        source_ref: VisualSourceRef,
        evidence_items: Vec<VisionEvidenceItem>,
        visible_content_only: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            capability_id: VisionCapabilityId::EvidenceExtract,
            reason_code,
            source_ref,
            evidence_items,
            visible_content_only,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for VisionEvidenceExtractOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_extract_ok.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.capability_id != VisionCapabilityId::EvidenceExtract {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_extract_ok.capability_id",
                reason: "must be VISION_EVIDENCE_EXTRACT",
            });
        }
        self.source_ref.validate()?;

        if self.evidence_items.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_extract_ok.evidence_items",
                reason: "must not be empty",
            });
        }
        if self.evidence_items.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_extract_ok.evidence_items",
                reason: "must be <= 64 items",
            });
        }
        for item in &self.evidence_items {
            item.validate()?;
        }

        if !self.visible_content_only {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_extract_ok.visible_content_only",
                reason: "must be true",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionVisibleContentValidateOk {
    pub schema_version: SchemaVersion,
    pub capability_id: VisionCapabilityId,
    pub reason_code: ReasonCodeId,
    pub source_ref: VisualSourceRef,
    pub validation_status: VisionValidationStatus,
    pub diagnostics: Vec<String>,
    pub visible_content_only: bool,
}

impl VisionVisibleContentValidateOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        source_ref: VisualSourceRef,
        validation_status: VisionValidationStatus,
        diagnostics: Vec<String>,
        visible_content_only: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            capability_id: VisionCapabilityId::VisibleContentValidate,
            reason_code,
            source_ref,
            validation_status,
            diagnostics,
            visible_content_only,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for VisionVisibleContentValidateOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_ok.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.capability_id != VisionCapabilityId::VisibleContentValidate {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_ok.capability_id",
                reason: "must be VISION_VISIBLE_CONTENT_VALIDATE",
            });
        }
        self.source_ref.validate()?;

        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_ok.diagnostics",
                reason: "must be <= 16 items",
            });
        }
        for d in &self.diagnostics {
            if d.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "vision_visible_content_validate_ok.diagnostics",
                    reason: "entries must not be empty",
                });
            }
            if d.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "vision_visible_content_validate_ok.diagnostics",
                    reason: "entry must be <= 128 chars",
                });
            }
        }
        if self.validation_status == VisionValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_ok.diagnostics",
                reason: "must include at least one diagnostic when validation_status=FAIL",
            });
        }

        if !self.visible_content_only {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_ok.visible_content_only",
                reason: "must be true",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: VisionCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl VisionRefuse {
    pub fn v1(
        capability_id: VisionCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for VisionRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_refuse.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.message.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "vision_refuse.message",
                reason: "must not be empty",
            });
        }
        if self.message.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_refuse.message",
                reason: "must be <= 256 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ph1VisionResponse {
    EvidenceExtractOk(VisionEvidenceExtractOk),
    VisibleContentValidateOk(VisionVisibleContentValidateOk),
    AnalyzeMediaOk(VisionAnalyzeMediaOk),
    Refuse(VisionRefuse),
}

impl Validate for Ph1VisionResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1VisionResponse::EvidenceExtractOk(o) => o.validate(),
            Ph1VisionResponse::VisibleContentValidateOk(o) => o.validate(),
            Ph1VisionResponse::AnalyzeMediaOk(o) => o.validate(),
            Ph1VisionResponse::Refuse(r) => r.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(max: u8, opt_in: bool) -> VisionRequestEnvelope {
        VisionRequestEnvelope::v1(CorrelationId(1), TurnId(1), opt_in, max).unwrap()
    }

    fn source() -> VisualSourceRef {
        VisualSourceRef::v1(
            VisualSourceId::new("img_001").unwrap(),
            VisualSourceKind::Image,
        )
        .unwrap()
    }

    fn token(s: &str) -> VisualToken {
        VisualToken::v1(s.to_string(), None).unwrap()
    }

    #[test]
    fn envelope_rejects_zero_max_evidence_items() {
        let env = VisionRequestEnvelope::v1(CorrelationId(1), TurnId(1), true, 0);
        assert!(env.is_err());
    }

    #[test]
    fn extract_request_requires_visible_tokens() {
        let req = VisionEvidenceExtractRequest::v1(envelope(4, true), source(), vec![]);
        assert!(req.is_err());
    }

    #[test]
    fn validate_request_rejects_more_evidence_than_envelope_budget() {
        let req = VisionVisibleContentValidateRequest::v1(
            envelope(1, true),
            source(),
            vec![token("hello")],
            vec![
                VisionEvidenceItem::v1("hello".to_string(), None).unwrap(),
                VisionEvidenceItem::v1("world".to_string(), None).unwrap(),
            ],
        );
        assert!(req.is_err());
    }

    #[test]
    fn evidence_ok_requires_visible_content_only_true() {
        let out = VisionEvidenceExtractOk::v1(
            ReasonCodeId(1),
            source(),
            vec![VisionEvidenceItem::v1("invoice_total".to_string(), None).unwrap()],
            false,
        );
        assert!(out.is_err());
    }

    #[test]
    fn validate_ok_requires_diagnostic_when_status_fail() {
        let out = VisionVisibleContentValidateOk::v1(
            ReasonCodeId(1),
            source(),
            VisionValidationStatus::Fail,
            vec![],
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn analyze_media_request_requires_policy_snapshot() {
        let req = VisionAnalyzeMediaRequest::v1(
            envelope(4, true),
            VisionMediaMode::ImageAnalyze,
            VisionAssetRef::v1(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                "asset://fixture".to_string(),
                "image/png".to_string(),
                3,
            )
            .unwrap(),
            VisionMediaOptions::v1(Some("en-US".to_string()), Some(3), Some(1000), true).unwrap(),
            VisionMediaBudgets::v1(2000, 1024).unwrap(),
            "".to_string(),
        );
        assert!(req.is_err());
    }

    #[test]
    fn analyze_media_response_validates() {
        let ok = VisionAnalyzeMediaOk::v1(
            ReasonCodeId(11),
            VisionAssetRef::v1(
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
                "asset://fixture".to_string(),
                "video/mp4".to_string(),
                9,
            )
            .unwrap(),
            vec![VisionProviderRun {
                schema_version: PH1VISION_CONTRACT_VERSION,
                provider_id: "vision_download".to_string(),
                endpoint: "download".to_string(),
                latency_ms: 1,
                error: None,
            }],
            vec![VisionMediaOutput::KeyframeIndexResult(
                VisionKeyframeIndexResult {
                    schema_version: PH1VISION_CONTRACT_VERSION,
                    keyframes: vec![VisionKeyframeEntry {
                        schema_version: PH1VISION_CONTRACT_VERSION,
                        timestamp_ms: 0,
                        frame_index: 0,
                        frame_hash:
                            "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                                .to_string(),
                    }],
                },
            )],
            VisionConfidenceSummary {
                schema_version: PH1VISION_CONTRACT_VERSION,
                mean_confidence: 1.0,
                ocr_blocks_retained: 0,
                objects_retained: 0,
                transcript_segments_retained: 0,
                output_count: 1,
            },
            vec!["insufficient_evidence".to_string()],
            VisionPacketHashes {
                schema_version: PH1VISION_CONTRACT_VERSION,
                asset_hash: "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
                    .to_string(),
                provider_runs_hash:
                    "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string(),
                outputs_hash: "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                    .to_string(),
            },
            "1111111111111111111111111111111111111111111111111111111111111111".to_string(),
        )
        .unwrap();

        assert!(ok.validate().is_ok());
        let wrapped = Ph1VisionResponse::AnalyzeMediaOk(ok);
        assert!(wrapped.validate().is_ok());
    }
}
