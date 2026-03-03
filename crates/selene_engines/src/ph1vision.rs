#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use crate::ph1vision_media;

use selene_kernel_contracts::ph1vision::{
    BoundingBoxPx, Ph1VisionRequest, Ph1VisionResponse, VisionAnalyzeMediaRequest,
    VisionCapabilityId, VisionEvidenceExtractOk, VisionEvidenceExtractRequest, VisionEvidenceItem,
    VisionRefuse, VisionValidationStatus, VisionVisibleContentValidateOk,
    VisionVisibleContentValidateRequest,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.VISION reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const PH1_VISION_OK_EVIDENCE_EXTRACT: ReasonCodeId = ReasonCodeId(0x5649_0101);
    pub const PH1_VISION_OK_VISIBLE_CONTENT_VALIDATE: ReasonCodeId = ReasonCodeId(0x5649_0102);
    pub const PH1_VISION_OK_ANALYZE_MEDIA: ReasonCodeId = ReasonCodeId(0x5649_0103);

    pub const PH1_VISION_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x5649_01F1);
    pub const PH1_VISION_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x5649_01F2);
    pub const PH1_VISION_OPT_IN_DISABLED: ReasonCodeId = ReasonCodeId(0x5649_01F3);
    pub const PH1_VISION_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x5649_01F4);
    pub const PH1_VISION_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5649_01F5);
    pub const PH1_VISION_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5649_01F6);
    pub const PH1_VISION_ANALYZE_MEDIA_DEGRADED: ReasonCodeId = ReasonCodeId(0x5649_01F7);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1VisionConfig {
    pub max_extract_items: u8,
    pub max_input_tokens: usize,
    pub max_diagnostics: u8,
}

impl Ph1VisionConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_extract_items: 32,
            max_input_tokens: 256,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1VisionRuntime {
    config: Ph1VisionConfig,
}

impl Ph1VisionRuntime {
    pub fn new(config: Ph1VisionConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1VisionRequest) -> Ph1VisionResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_VISION_INPUT_SCHEMA_INVALID,
                "vision request failed contract validation",
            );
        }

        match req {
            Ph1VisionRequest::EvidenceExtract(r) => self.run_extract(r),
            Ph1VisionRequest::VisibleContentValidate(r) => self.run_visible_content_validate(r),
            Ph1VisionRequest::AnalyzeMedia(r) => self.run_media_analyze(r),
        }
    }

    fn run_extract(&self, req: &VisionEvidenceExtractRequest) -> Ph1VisionResponse {
        if !req.envelope.opt_in_enabled {
            return self.refuse(
                VisionCapabilityId::EvidenceExtract,
                reason_codes::PH1_VISION_OPT_IN_DISABLED,
                "vision opt-in is disabled",
            );
        }

        if req.visible_tokens.is_empty() {
            return self.refuse(
                VisionCapabilityId::EvidenceExtract,
                reason_codes::PH1_VISION_UPSTREAM_INPUT_MISSING,
                "no upstream visible tokens were provided",
            );
        }

        if req.visible_tokens.len() > self.config.max_input_tokens {
            return self.refuse(
                VisionCapabilityId::EvidenceExtract,
                reason_codes::PH1_VISION_BUDGET_EXCEEDED,
                "visible token budget exceeded",
            );
        }

        let budget = min(
            req.envelope.max_evidence_items,
            self.config.max_extract_items,
        ) as usize;
        if budget == 0 {
            return self.refuse(
                VisionCapabilityId::EvidenceExtract,
                reason_codes::PH1_VISION_BUDGET_EXCEEDED,
                "evidence budget exceeded",
            );
        }

        let mut dedupe: BTreeSet<(String, Option<BoundingBoxPx>)> = BTreeSet::new();
        let mut out: Vec<VisionEvidenceItem> = Vec::new();

        for token in &req.visible_tokens {
            if out.len() >= budget {
                break;
            }

            let canonical = canonical_text(&token.token);
            if canonical.is_empty() {
                continue;
            }
            let key = (canonical, token.bbox);
            if dedupe.insert(key) {
                match VisionEvidenceItem::v1(token.token.clone(), token.bbox) {
                    Ok(item) => out.push(item),
                    Err(_) => {
                        return self.refuse(
                            VisionCapabilityId::EvidenceExtract,
                            reason_codes::PH1_VISION_INTERNAL_PIPELINE_ERROR,
                            "failed to build vision evidence item",
                        )
                    }
                }
            }
        }

        if out.is_empty() {
            return self.refuse(
                VisionCapabilityId::EvidenceExtract,
                reason_codes::PH1_VISION_UPSTREAM_INPUT_MISSING,
                "no visible-content evidence could be extracted",
            );
        }

        match VisionEvidenceExtractOk::v1(
            reason_codes::PH1_VISION_OK_EVIDENCE_EXTRACT,
            req.source_ref.clone(),
            out,
            true,
        ) {
            Ok(ok) => Ph1VisionResponse::EvidenceExtractOk(ok),
            Err(_) => self.refuse(
                VisionCapabilityId::EvidenceExtract,
                reason_codes::PH1_VISION_INTERNAL_PIPELINE_ERROR,
                "failed to construct extract output",
            ),
        }
    }

    fn run_visible_content_validate(
        &self,
        req: &VisionVisibleContentValidateRequest,
    ) -> Ph1VisionResponse {
        if !req.envelope.opt_in_enabled {
            return self.refuse(
                VisionCapabilityId::VisibleContentValidate,
                reason_codes::PH1_VISION_OPT_IN_DISABLED,
                "vision opt-in is disabled",
            );
        }

        if req.visible_tokens.is_empty() {
            return self.refuse(
                VisionCapabilityId::VisibleContentValidate,
                reason_codes::PH1_VISION_UPSTREAM_INPUT_MISSING,
                "no upstream visible tokens were provided",
            );
        }

        if req.evidence_items.len() > req.envelope.max_evidence_items as usize {
            return self.refuse(
                VisionCapabilityId::VisibleContentValidate,
                reason_codes::PH1_VISION_BUDGET_EXCEEDED,
                "evidence item budget exceeded",
            );
        }

        let mut visible_pairs: BTreeSet<(String, Option<BoundingBoxPx>)> = BTreeSet::new();
        let mut visible_texts: BTreeSet<String> = BTreeSet::new();

        for token in &req.visible_tokens {
            let canonical = canonical_text(&token.token);
            if canonical.is_empty() {
                continue;
            }
            visible_texts.insert(canonical.clone());
            visible_pairs.insert((canonical, token.bbox));
        }

        let mut diagnostics: Vec<String> = Vec::new();

        for (idx, item) in req.evidence_items.iter().enumerate() {
            let canonical = canonical_text(&item.text);
            let matched = if let Some(bbox) = item.bbox {
                visible_pairs.contains(&(canonical, Some(bbox)))
            } else {
                visible_texts.contains(&canonical)
            };

            if !matched {
                diagnostics.push(format!("evidence_index_{idx}_not_visible_content"));
                if diagnostics.len() >= self.config.max_diagnostics as usize {
                    break;
                }
            }
        }

        let (status, reason_code) = if diagnostics.is_empty() {
            (
                VisionValidationStatus::Ok,
                reason_codes::PH1_VISION_OK_VISIBLE_CONTENT_VALIDATE,
            )
        } else {
            (
                VisionValidationStatus::Fail,
                reason_codes::PH1_VISION_VALIDATION_FAILED,
            )
        };

        match VisionVisibleContentValidateOk::v1(
            reason_code,
            req.source_ref.clone(),
            status,
            diagnostics,
            true,
        ) {
            Ok(ok) => Ph1VisionResponse::VisibleContentValidateOk(ok),
            Err(_) => self.refuse(
                VisionCapabilityId::VisibleContentValidate,
                reason_codes::PH1_VISION_INTERNAL_PIPELINE_ERROR,
                "failed to construct visible-content validation output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: VisionCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1VisionResponse {
        let r = VisionRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("VisionRefuse::v1 must construct for static message");
        Ph1VisionResponse::Refuse(r)
    }

    fn run_media_analyze(&self, req: &VisionAnalyzeMediaRequest) -> Ph1VisionResponse {
        if !req.envelope.opt_in_enabled {
            return self.refuse(
                VisionCapabilityId::AnalyzeMedia,
                reason_codes::PH1_VISION_OPT_IN_DISABLED,
                "vision opt-in is disabled",
            );
        }

        let providers = ph1vision_media::ProductionVisionMediaProviders;
        match ph1vision_media::run_media_analyze_with_providers(
            req,
            &providers,
            reason_codes::PH1_VISION_OK_ANALYZE_MEDIA,
            reason_codes::PH1_VISION_ANALYZE_MEDIA_DEGRADED,
        ) {
            Ok(ok) => Ph1VisionResponse::AnalyzeMediaOk(ok),
            Err(_) => self.refuse(
                VisionCapabilityId::AnalyzeMedia,
                reason_codes::PH1_VISION_INTERNAL_PIPELINE_ERROR,
                "failed to construct media analysis output",
            ),
        }
    }
}

fn capability_from_request(req: &Ph1VisionRequest) -> VisionCapabilityId {
    match req {
        Ph1VisionRequest::EvidenceExtract(_) => VisionCapabilityId::EvidenceExtract,
        Ph1VisionRequest::VisibleContentValidate(_) => VisionCapabilityId::VisibleContentValidate,
        Ph1VisionRequest::AnalyzeMedia(_) => VisionCapabilityId::AnalyzeMedia,
    }
}

fn canonical_text(input: &str) -> String {
    let lowered = input.trim().to_ascii_lowercase();
    lowered
        .split_whitespace()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1vision::{
        VisionAnalyzeMediaRequest, VisionAssetRef, VisionBoundingBox, VisionDetectedObject,
        VisionKeyframeEntry, VisionKeyframeIndexResult, VisionMediaBudgets, VisionMediaMode,
        VisionMediaOptions, VisionMediaOutput, VisionObjectDetectionResult, VisionOcrResult,
        VisionOcrTextBlock, VisionRequestEnvelope, VisionTranscriptSegment,
        VisionVideoTranscriptResult, VisualSourceId, VisualSourceKind, VisualSourceRef,
        VisualToken,
    };
    use serde_json::Value;
    use std::fs;
    use std::path::PathBuf;

    fn runtime() -> Ph1VisionRuntime {
        Ph1VisionRuntime::new(Ph1VisionConfig::mvp_v1())
    }

    fn env(opt_in: bool, max: u8) -> VisionRequestEnvelope {
        VisionRequestEnvelope::v1(CorrelationId(123), TurnId(7), opt_in, max).unwrap()
    }

    fn source() -> VisualSourceRef {
        VisualSourceRef::v1(
            VisualSourceId::new("src_001").unwrap(),
            VisualSourceKind::Screenshot,
        )
        .unwrap()
    }

    fn token(s: &str) -> VisualToken {
        VisualToken::v1(s.to_string(), None).unwrap()
    }

    fn media_request(
        mode: VisionMediaMode,
        mime_type: &str,
        bytes: &[u8],
    ) -> VisionAnalyzeMediaRequest {
        let asset_hash = ph1vision_media::tests_support::sha256_for_tests(bytes);
        VisionAnalyzeMediaRequest::v1(
            env(true, 16),
            mode,
            VisionAssetRef::v1(
                asset_hash,
                "asset://fixtures/media".to_string(),
                mime_type.to_string(),
                bytes.len() as u64,
            )
            .unwrap(),
            VisionMediaOptions::v1(Some("en-US".to_string()), Some(3), Some(1000), true).unwrap(),
            VisionMediaBudgets::v1(1500, 2 * 1024 * 1024).unwrap(),
            "policy_media_v1".to_string(),
        )
        .unwrap()
    }

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/web_search_plan/vision_fixtures/expected")
            .join(name)
    }

    fn read_expected(name: &str) -> Value {
        let path = fixture_path(name);
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed reading fixture {}: {}", path.display(), err));
        serde_json::from_str(&text).expect("fixture json must parse")
    }

    #[test]
    fn at_vision_01_extract_output_is_schema_valid() {
        let req = Ph1VisionRequest::EvidenceExtract(
            VisionEvidenceExtractRequest::v1(
                env(true, 4),
                source(),
                vec![token("Revenue"), token("Cost")],
            )
            .unwrap(),
        );

        let resp = runtime().run(&req);
        assert!(resp.validate().is_ok());
        match resp {
            Ph1VisionResponse::EvidenceExtractOk(ok) => {
                assert_eq!(ok.capability_id, VisionCapabilityId::EvidenceExtract);
                assert!(ok.visible_content_only);
                assert_eq!(ok.evidence_items.len(), 2);
            }
            _ => panic!("expected EvidenceExtractOk"),
        }
    }

    #[test]
    fn at_vision_02_order_is_deterministic_and_preserved() {
        let req = Ph1VisionRequest::EvidenceExtract(
            VisionEvidenceExtractRequest::v1(
                env(true, 8),
                source(),
                vec![token("B"), token("A"), token("B"), token("C"), token("A")],
            )
            .unwrap(),
        );

        let runtime = runtime();
        let resp1 = runtime.run(&req);
        let resp2 = runtime.run(&req);

        let list1 = match resp1 {
            Ph1VisionResponse::EvidenceExtractOk(ok) => ok
                .evidence_items
                .into_iter()
                .map(|it| it.text)
                .collect::<Vec<_>>(),
            _ => panic!("expected EvidenceExtractOk"),
        };
        let list2 = match resp2 {
            Ph1VisionResponse::EvidenceExtractOk(ok) => ok
                .evidence_items
                .into_iter()
                .map(|it| it.text)
                .collect::<Vec<_>>(),
            _ => panic!("expected EvidenceExtractOk"),
        };

        assert_eq!(list1, vec!["B", "A", "C"]);
        assert_eq!(list1, list2);
    }

    #[test]
    fn at_vision_03_opt_in_disabled_skips_with_refuse() {
        let req = Ph1VisionRequest::EvidenceExtract(
            VisionEvidenceExtractRequest::v1(
                env(false, 4),
                source(),
                vec![token("should_not_run")],
            )
            .unwrap(),
        );

        let resp = runtime().run(&req);
        match resp {
            Ph1VisionResponse::Refuse(r) => {
                assert_eq!(r.reason_code, reason_codes::PH1_VISION_OPT_IN_DISABLED);
                assert_eq!(r.capability_id, VisionCapabilityId::EvidenceExtract);
            }
            _ => panic!("expected Refuse"),
        }
    }

    #[test]
    fn at_vision_04_validate_fails_for_non_visible_inferred_item() {
        let req = Ph1VisionRequest::VisibleContentValidate(
            VisionVisibleContentValidateRequest::v1(
                env(true, 4),
                source(),
                vec![token("visible_total"), token("visible_tax")],
                vec![
                    VisionEvidenceItem::v1("visible_total".to_string(), None).unwrap(),
                    VisionEvidenceItem::v1("inferred_profit".to_string(), None).unwrap(),
                ],
            )
            .unwrap(),
        );

        let resp = runtime().run(&req);
        match resp {
            Ph1VisionResponse::VisibleContentValidateOk(ok) => {
                assert_eq!(ok.validation_status, VisionValidationStatus::Fail);
                assert_eq!(ok.reason_code, reason_codes::PH1_VISION_VALIDATION_FAILED);
                assert!(ok
                    .diagnostics
                    .iter()
                    .any(|d| d == "evidence_index_1_not_visible_content"));
                assert!(ok.visible_content_only);
            }
            _ => panic!("expected VisibleContentValidateOk"),
        }
    }

    #[test]
    fn at_vision_media_parity_ocr_fixture_matches() {
        let request = media_request(VisionMediaMode::ImageOcr, "image/png", b"img-ocr");
        let providers = ph1vision_media::tests_support::FixedClockProviders {
            now_ms_value: 1000,
            load_result: Ok(ph1vision_media::LoadedAsset {
                bytes: b"img-ocr".to_vec(),
                mime_type: "image/png".to_string(),
                size_bytes: 7,
                redacted_locator: "asset://local/redacted".to_string(),
            }),
            ocr_result: Ok(VisionOcrResult {
                schema_version: request.schema_version,
                page_or_frame_index: 0,
                timestamp_ms: None,
                ocr_engine_id: "fixture_ocr".to_string(),
                language: "en-US".to_string(),
                text_blocks: vec![VisionOcrTextBlock {
                    schema_version: request.schema_version,
                    bbox: VisionBoundingBox::new(0.0, 0.0, 10.0, 4.0).unwrap(),
                    text: "HELLO WORLD".to_string(),
                    confidence: 0.97,
                    pii_suspected: Some(false),
                }],
                full_text: "HELLO WORLD".to_string(),
            }),
            objects_result: Err(ph1vision_media::ProviderFailure::new(
                "vision_objects",
                "objects",
                "provider_unconfigured",
                "provider_unconfigured",
                "unused",
                0,
            )),
            transcript_result: Err(ph1vision_media::ProviderFailure::new(
                "google_stt",
                "stt",
                "provider_unconfigured",
                "provider_unconfigured",
                "unused",
                0,
            )),
            keyframes_result: Err(ph1vision_media::ProviderFailure::new(
                "vision_keyframes",
                "keyframes",
                "provider_unconfigured",
                "provider_unconfigured",
                "unused",
                0,
            )),
        };

        let ok = ph1vision_media::run_media_analyze_with_providers(
            &request,
            &providers,
            reason_codes::PH1_VISION_OK_ANALYZE_MEDIA,
            reason_codes::PH1_VISION_ANALYZE_MEDIA_DEGRADED,
        )
        .expect("media analyze should succeed");

        assert_eq!(ok.outputs.len(), 1);
        let expected = read_expected("ocr_expected.json");
        match &ok.outputs[0] {
            VisionMediaOutput::OCRResult(result) => {
                assert_eq!(
                    expected.get("ocr_engine_id").and_then(Value::as_str),
                    Some(result.ocr_engine_id.as_str())
                );
                assert_eq!(
                    expected.get("language").and_then(Value::as_str),
                    Some(result.language.as_str())
                );
                assert_eq!(
                    expected
                        .get("text_blocks")
                        .and_then(Value::as_array)
                        .map(Vec::len),
                    Some(result.text_blocks.len())
                );
                assert_eq!(
                    expected.get("full_text").and_then(Value::as_str),
                    Some(result.full_text.as_str())
                );
            }
            _ => panic!("expected OCR output"),
        }
    }

    #[test]
    fn at_vision_media_parity_objects_fixture_matches() {
        let request = media_request(VisionMediaMode::ImageObjects, "image/jpeg", b"img-objects");
        let providers = ph1vision_media::tests_support::FixedClockProviders {
            now_ms_value: 1000,
            load_result: Ok(ph1vision_media::LoadedAsset {
                bytes: b"img-objects".to_vec(),
                mime_type: "image/jpeg".to_string(),
                size_bytes: 11,
                redacted_locator: "asset://local/redacted".to_string(),
            }),
            ocr_result: Err(ph1vision_media::ProviderFailure::new(
                "vision_ocr",
                "ocr",
                "provider_unconfigured",
                "provider_unconfigured",
                "unused",
                0,
            )),
            objects_result: Ok(VisionObjectDetectionResult {
                schema_version: request.schema_version,
                frame_index: None,
                timestamp_ms: None,
                model_id: "fixture_objects_model".to_string(),
                objects: vec![VisionDetectedObject {
                    schema_version: request.schema_version,
                    label: "person".to_string(),
                    bbox: VisionBoundingBox::new(5.0, 8.0, 20.0, 40.0).unwrap(),
                    confidence: 0.92,
                }],
            }),
            transcript_result: Err(ph1vision_media::ProviderFailure::new(
                "google_stt",
                "stt",
                "provider_unconfigured",
                "provider_unconfigured",
                "unused",
                0,
            )),
            keyframes_result: Err(ph1vision_media::ProviderFailure::new(
                "vision_keyframes",
                "keyframes",
                "provider_unconfigured",
                "provider_unconfigured",
                "unused",
                0,
            )),
        };

        let ok = ph1vision_media::run_media_analyze_with_providers(
            &request,
            &providers,
            reason_codes::PH1_VISION_OK_ANALYZE_MEDIA,
            reason_codes::PH1_VISION_ANALYZE_MEDIA_DEGRADED,
        )
        .expect("media analyze should succeed");

        let expected = read_expected("objects_expected.json");
        match &ok.outputs[0] {
            VisionMediaOutput::ObjectDetectionResult(result) => {
                assert_eq!(
                    expected.get("model_id").and_then(Value::as_str),
                    Some(result.model_id.as_str())
                );
                assert_eq!(
                    expected
                        .get("objects")
                        .and_then(Value::as_array)
                        .map(Vec::len),
                    Some(result.objects.len())
                );
                assert_eq!(result.objects[0].label, "person");
            }
            _ => panic!("expected object output"),
        }
    }

    #[test]
    fn at_vision_media_parity_transcript_fixture_matches() {
        let request = media_request(VisionMediaMode::VideoTranscribe, "video/mp4", b"vid-stt");
        let providers = ph1vision_media::tests_support::FixedClockProviders {
            now_ms_value: 1000,
            load_result: Ok(ph1vision_media::LoadedAsset {
                bytes: b"vid-stt".to_vec(),
                mime_type: "video/mp4".to_string(),
                size_bytes: 7,
                redacted_locator: "asset://local/redacted".to_string(),
            }),
            ocr_result: Err(ph1vision_media::ProviderFailure::new(
                "vision_ocr",
                "ocr",
                "provider_unconfigured",
                "provider_unconfigured",
                "unused",
                0,
            )),
            objects_result: Err(ph1vision_media::ProviderFailure::new(
                "vision_objects",
                "objects",
                "provider_unconfigured",
                "provider_unconfigured",
                "unused",
                0,
            )),
            transcript_result: Ok(VisionVideoTranscriptResult {
                schema_version: request.schema_version,
                stt_provider_id: "GOOGLE_STT".to_string(),
                language: "en-US".to_string(),
                segments: vec![
                    VisionTranscriptSegment {
                        schema_version: request.schema_version,
                        start_ms: 0,
                        end_ms: 1200,
                        text: "hello".to_string(),
                        confidence: 92,
                    },
                    VisionTranscriptSegment {
                        schema_version: request.schema_version,
                        start_ms: 1200,
                        end_ms: 2500,
                        text: "world".to_string(),
                        confidence: 89,
                    },
                ],
                full_transcript: "hello\nworld".to_string(),
            }),
            keyframes_result: Err(ph1vision_media::ProviderFailure::new(
                "vision_keyframes",
                "keyframes",
                "provider_unconfigured",
                "provider_unconfigured",
                "unused",
                0,
            )),
        };

        let ok = ph1vision_media::run_media_analyze_with_providers(
            &request,
            &providers,
            reason_codes::PH1_VISION_OK_ANALYZE_MEDIA,
            reason_codes::PH1_VISION_ANALYZE_MEDIA_DEGRADED,
        )
        .expect("media analyze should succeed");

        let expected = read_expected("transcript_expected.json");
        match &ok.outputs[0] {
            VisionMediaOutput::VideoTranscriptResult(result) => {
                assert_eq!(
                    expected.get("language").and_then(Value::as_str),
                    Some(result.language.as_str())
                );
                assert_eq!(
                    expected
                        .get("segments")
                        .and_then(Value::as_array)
                        .map(Vec::len),
                    Some(result.segments.len())
                );
                assert_eq!(result.full_transcript, "hello\nworld");
            }
            _ => panic!("expected transcript output"),
        }
    }

    #[test]
    fn at_vision_media_parity_keyframes_fixture_matches() {
        let request = media_request(VisionMediaMode::VideoKeyframes, "video/mp4", b"vid-kf");
        let providers = ph1vision_media::tests_support::FixedClockProviders {
            now_ms_value: 1000,
            load_result: Ok(ph1vision_media::LoadedAsset {
                bytes: b"vid-kf".to_vec(),
                mime_type: "video/mp4".to_string(),
                size_bytes: 6,
                redacted_locator: "asset://local/redacted".to_string(),
            }),
            ocr_result: Err(ph1vision_media::ProviderFailure::new(
                "vision_ocr",
                "ocr",
                "provider_unconfigured",
                "provider_unconfigured",
                "unused",
                0,
            )),
            objects_result: Err(ph1vision_media::ProviderFailure::new(
                "vision_objects",
                "objects",
                "provider_unconfigured",
                "provider_unconfigured",
                "unused",
                0,
            )),
            transcript_result: Err(ph1vision_media::ProviderFailure::new(
                "google_stt",
                "stt",
                "provider_unconfigured",
                "provider_unconfigured",
                "unused",
                0,
            )),
            keyframes_result: Ok(VisionKeyframeIndexResult {
                schema_version: request.schema_version,
                keyframes: vec![
                    VisionKeyframeEntry {
                        schema_version: request.schema_version,
                        timestamp_ms: 0,
                        frame_index: 0,
                        frame_hash:
                            "0000000000000000000000000000000000000000000000000000000000000000"
                                .to_string(),
                    },
                    VisionKeyframeEntry {
                        schema_version: request.schema_version,
                        timestamp_ms: 1000,
                        frame_index: 1,
                        frame_hash:
                            "1111111111111111111111111111111111111111111111111111111111111111"
                                .to_string(),
                    },
                ],
            }),
        };

        let ok = ph1vision_media::run_media_analyze_with_providers(
            &request,
            &providers,
            reason_codes::PH1_VISION_OK_ANALYZE_MEDIA,
            reason_codes::PH1_VISION_ANALYZE_MEDIA_DEGRADED,
        )
        .expect("media analyze should succeed");

        let expected = read_expected("keyframes_expected.json");
        match &ok.outputs[0] {
            VisionMediaOutput::KeyframeIndexResult(result) => {
                assert_eq!(
                    expected
                        .get("keyframes")
                        .and_then(Value::as_array)
                        .map(Vec::len),
                    Some(result.keyframes.len())
                );
                assert_eq!(result.keyframes[0].timestamp_ms, 0);
                assert_eq!(result.keyframes[1].timestamp_ms, 1000);
            }
            _ => panic!("expected keyframes output"),
        }
    }

    #[test]
    fn at_vision_media_threshold_and_redaction_are_deterministic() {
        let request = media_request(VisionMediaMode::ImageAnalyze, "image/png", b"img-threshold");
        let providers = ph1vision_media::tests_support::FixedClockProviders {
            now_ms_value: 1000,
            load_result: Ok(ph1vision_media::LoadedAsset {
                bytes: b"img-threshold".to_vec(),
                mime_type: "image/png".to_string(),
                size_bytes: 13,
                redacted_locator: "asset://local/redacted".to_string(),
            }),
            ocr_result: Ok(VisionOcrResult {
                schema_version: request.schema_version,
                page_or_frame_index: 0,
                timestamp_ms: None,
                ocr_engine_id: "fixture_ocr".to_string(),
                language: "en-US".to_string(),
                text_blocks: vec![VisionOcrTextBlock {
                    schema_version: request.schema_version,
                    bbox: VisionBoundingBox::new(0.0, 0.0, 2.0, 2.0).unwrap(),
                    text: "drop".to_string(),
                    confidence: 0.20,
                    pii_suspected: Some(false),
                }],
                full_text: "drop".to_string(),
            }),
            objects_result: Ok(VisionObjectDetectionResult {
                schema_version: request.schema_version,
                frame_index: None,
                timestamp_ms: None,
                model_id: "fixture".to_string(),
                objects: vec![VisionDetectedObject {
                    schema_version: request.schema_version,
                    label: "noise".to_string(),
                    bbox: VisionBoundingBox::new(0.0, 0.0, 2.0, 2.0).unwrap(),
                    confidence: 0.20,
                }],
            }),
            transcript_result: Err(ph1vision_media::ProviderFailure::new(
                "google_stt",
                "stt",
                "provider_unconfigured",
                "provider_unconfigured",
                "unused",
                0,
            )),
            keyframes_result: Err(ph1vision_media::ProviderFailure::new(
                "vision_keyframes",
                "keyframes",
                "provider_unconfigured",
                "provider_unconfigured",
                "unused",
                0,
            )),
        };

        let first = ph1vision_media::run_media_analyze_with_providers(
            &request,
            &providers,
            reason_codes::PH1_VISION_OK_ANALYZE_MEDIA,
            reason_codes::PH1_VISION_ANALYZE_MEDIA_DEGRADED,
        )
        .expect("first run should succeed");
        let second = ph1vision_media::run_media_analyze_with_providers(
            &request,
            &providers,
            reason_codes::PH1_VISION_OK_ANALYZE_MEDIA,
            reason_codes::PH1_VISION_ANALYZE_MEDIA_DEGRADED,
        )
        .expect("second run should succeed");

        assert_eq!(first.output_hash, second.output_hash);
        assert_eq!(first.outputs, second.outputs);
        assert!(first
            .reason_codes
            .iter()
            .any(|reason| reason == "insufficient_evidence"));

        let redacted = ph1vision_media::tests_support::redact_error_message_for_tests(
            "Authorization: Bearer sk-secret api_key=abc123",
        );
        assert!(!redacted.contains("sk-secret"));
        assert!(!redacted.contains("api_key"));
    }
}
