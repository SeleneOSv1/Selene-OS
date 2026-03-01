#![forbid(unsafe_code)]

use std::cmp::Ordering;
use std::collections::BTreeMap;

use selene_kernel_contracts::ph1simcat::SimulationStatus;
use selene_kernel_contracts::ph1simfinder::{
    reason_codes, CatalogCheckKind, CatalogCheckTraceEntry, ClarifyOnExceedPolicy, ClarifyPacket,
    FinderFallbackPolicy, FinderRiskTier, FinderTerminalPacket, MissingSimulationPacket,
    RefusePacket, SimulationMatchPacket,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, Validate};

const WORD_INTENT_CONFIDENCE: u32 = 35;
const WORD_REQUIRED_FIELD_COVERAGE: u32 = 20;
const WORD_EVIDENCE_COVERAGE: u32 = 10;
const WORD_CATALOG_STATUS: u32 = 10;
const WORD_CONTEXT_ALIGNMENT: u32 = 10;
const WORD_OCR_ALIGNMENT: u32 = 5;
const WORD_LLM_ASSIST_ALIGNMENT: u32 = 5;
const WORD_GOLD_BONUS: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinderRuntimeConfig {
    pub match_direct_min_bp: u16,
    pub match_with_clarify_min_bp: u16,
    pub tie_margin_min_bp: u16,
    pub max_clarify_attempts: u8,
}

impl FinderRuntimeConfig {
    pub fn mvp_v1() -> Self {
        Self {
            match_direct_min_bp: 9_000,
            match_with_clarify_min_bp: 7_000,
            tie_margin_min_bp: 800,
            max_clarify_attempts: 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinderFieldSpec {
    pub field_name: String,
    pub required: bool,
    pub detector_terms: Vec<String>,
    pub domain_cardinality_bp: u16,
    pub candidate_split_bp: u16,
    pub downstream_risk_bp: u16,
    pub clarify_question: String,
    pub allowed_answer_formats: Vec<String>,
}

impl FinderFieldSpec {
    pub fn required_v1(
        field_name: String,
        detector_terms: Vec<String>,
        domain_cardinality_bp: u16,
        candidate_split_bp: u16,
        downstream_risk_bp: u16,
        clarify_question: String,
        allowed_answer_formats: Vec<String>,
    ) -> Result<Self, ContractViolation> {
        let spec = Self {
            field_name,
            required: true,
            detector_terms,
            domain_cardinality_bp,
            candidate_split_bp,
            downstream_risk_bp,
            clarify_question,
            allowed_answer_formats,
        };
        spec.validate()?;
        Ok(spec)
    }
}

impl Validate for FinderFieldSpec {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_required_text("finder_field_spec.field_name", &self.field_name, 128)?;
        validate_terms(
            "finder_field_spec.detector_terms",
            &self.detector_terms,
            32,
            64,
        )?;
        validate_bp(
            "finder_field_spec.domain_cardinality_bp",
            self.domain_cardinality_bp,
        )?;
        validate_bp(
            "finder_field_spec.candidate_split_bp",
            self.candidate_split_bp,
        )?;
        validate_bp(
            "finder_field_spec.downstream_risk_bp",
            self.downstream_risk_bp,
        )?;
        validate_required_text(
            "finder_field_spec.clarify_question",
            &self.clarify_question,
            240,
        )?;
        if !(2..=3).contains(&self.allowed_answer_formats.len()) {
            return Err(ContractViolation::InvalidValue {
                field: "finder_field_spec.allowed_answer_formats",
                reason: "must contain 2-3 entries",
            });
        }
        validate_terms(
            "finder_field_spec.allowed_answer_formats",
            &self.allowed_answer_formats,
            3,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinderSimulationCatalogEntry {
    pub simulation_id: String,
    pub intent_family: String,
    pub status: SimulationStatus,
    pub simulation_priority: u16,
    pub required_access_actions: Vec<String>,
    pub risk_tier: FinderRiskTier,
    pub confirm_required: bool,
    pub fallback_if_inactive_or_missing: FinderFallbackPolicy,
    pub synonym_terms: Vec<String>,
    pub required_fields: Vec<FinderFieldSpec>,
    pub required_integrations: Vec<String>,
}

impl FinderSimulationCatalogEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        simulation_id: String,
        intent_family: String,
        status: SimulationStatus,
        simulation_priority: u16,
        required_access_actions: Vec<String>,
        risk_tier: FinderRiskTier,
        confirm_required: bool,
        fallback_if_inactive_or_missing: FinderFallbackPolicy,
        synonym_terms: Vec<String>,
        required_fields: Vec<FinderFieldSpec>,
        required_integrations: Vec<String>,
    ) -> Result<Self, ContractViolation> {
        let entry = Self {
            simulation_id,
            intent_family,
            status,
            simulation_priority,
            required_access_actions,
            risk_tier,
            confirm_required,
            fallback_if_inactive_or_missing,
            synonym_terms,
            required_fields,
            required_integrations,
        };
        entry.validate()?;
        Ok(entry)
    }
}

impl Validate for FinderSimulationCatalogEntry {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_required_text(
            "finder_simulation_catalog_entry.simulation_id",
            &self.simulation_id,
            128,
        )?;
        validate_required_text(
            "finder_simulation_catalog_entry.intent_family",
            &self.intent_family,
            128,
        )?;
        validate_terms(
            "finder_simulation_catalog_entry.required_access_actions",
            &self.required_access_actions,
            32,
            128,
        )?;
        validate_terms(
            "finder_simulation_catalog_entry.synonym_terms",
            &self.synonym_terms,
            64,
            128,
        )?;
        validate_terms(
            "finder_simulation_catalog_entry.required_integrations",
            &self.required_integrations,
            64,
            128,
        )?;
        for spec in &self.required_fields {
            spec.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinderGoldMapping {
    pub utterance_fingerprint: String,
    pub simulation_id: String,
    pub gold_match_bonus_bp: u16,
}

impl FinderGoldMapping {
    pub fn v1(
        utterance_fingerprint: String,
        simulation_id: String,
        gold_match_bonus_bp: u16,
    ) -> Result<Self, ContractViolation> {
        let mapping = Self {
            utterance_fingerprint,
            simulation_id,
            gold_match_bonus_bp,
        };
        mapping.validate()?;
        Ok(mapping)
    }
}

impl Validate for FinderGoldMapping {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_required_text(
            "finder_gold_mapping.utterance_fingerprint",
            &self.utterance_fingerprint,
            128,
        )?;
        validate_required_text(
            "finder_gold_mapping.simulation_id",
            &self.simulation_id,
            128,
        )?;
        validate_bp(
            "finder_gold_mapping.gold_match_bonus_bp",
            self.gold_match_bonus_bp,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinderRunRequest {
    pub tenant_id: String,
    pub user_id: String,
    pub correlation_id: u128,
    pub turn_id: u64,
    pub now: MonotonicTimeNs,
    pub transcript_text: String,
    pub simulation_catalog_snapshot_hash: String,
    pub simulation_catalog_snapshot_version: u64,
    pub simulation_catalog: Vec<FinderSimulationCatalogEntry>,
    pub gold_mappings: Vec<FinderGoldMapping>,
    pub context_alignment_bp: u16,
    pub ocr_alignment_bp: u16,
    pub llm_assist_alignment_bp: u16,
    pub ambiguity_penalty_bp: u16,
    pub contradictory_field_penalty_bp: u16,
    pub policy_mismatch_penalty_bp: u16,
    pub clarify_attempt_index: u8,
    pub default_category: String,
    pub estimated_frequency_score_bp: u16,
    pub estimated_value_score_bp: u16,
    pub estimated_roi_score_bp: u16,
    pub estimated_feasibility_score_bp: u16,
    pub estimated_risk_score_bp: u16,
    pub scope_class: String,
    pub required_fields_schema_json: String,
    pub acceptance_test_suggestion: Vec<String>,
}

impl FinderRunRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        tenant_id: String,
        user_id: String,
        correlation_id: u128,
        turn_id: u64,
        now: MonotonicTimeNs,
        transcript_text: String,
        simulation_catalog_snapshot_hash: String,
        simulation_catalog_snapshot_version: u64,
        simulation_catalog: Vec<FinderSimulationCatalogEntry>,
        gold_mappings: Vec<FinderGoldMapping>,
        context_alignment_bp: u16,
        ocr_alignment_bp: u16,
        llm_assist_alignment_bp: u16,
        ambiguity_penalty_bp: u16,
        contradictory_field_penalty_bp: u16,
        policy_mismatch_penalty_bp: u16,
        clarify_attempt_index: u8,
        default_category: String,
        estimated_frequency_score_bp: u16,
        estimated_value_score_bp: u16,
        estimated_roi_score_bp: u16,
        estimated_feasibility_score_bp: u16,
        estimated_risk_score_bp: u16,
        scope_class: String,
        required_fields_schema_json: String,
        acceptance_test_suggestion: Vec<String>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            tenant_id,
            user_id,
            correlation_id,
            turn_id,
            now,
            transcript_text,
            simulation_catalog_snapshot_hash,
            simulation_catalog_snapshot_version,
            simulation_catalog,
            gold_mappings,
            context_alignment_bp,
            ocr_alignment_bp,
            llm_assist_alignment_bp,
            ambiguity_penalty_bp,
            contradictory_field_penalty_bp,
            policy_mismatch_penalty_bp,
            clarify_attempt_index,
            default_category,
            estimated_frequency_score_bp,
            estimated_value_score_bp,
            estimated_roi_score_bp,
            estimated_feasibility_score_bp,
            estimated_risk_score_bp,
            scope_class,
            required_fields_schema_json,
            acceptance_test_suggestion,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for FinderRunRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_required_text("finder_run_request.tenant_id", &self.tenant_id, 128)?;
        validate_required_text("finder_run_request.user_id", &self.user_id, 128)?;
        if self.correlation_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "finder_run_request.correlation_id",
                reason: "must be > 0",
            });
        }
        if self.turn_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "finder_run_request.turn_id",
                reason: "must be > 0",
            });
        }
        if self.now.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "finder_run_request.now",
                reason: "must be > 0",
            });
        }
        validate_required_text(
            "finder_run_request.transcript_text",
            &self.transcript_text,
            32_768,
        )?;
        validate_required_text(
            "finder_run_request.simulation_catalog_snapshot_hash",
            &self.simulation_catalog_snapshot_hash,
            128,
        )?;
        if self.simulation_catalog_snapshot_version == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "finder_run_request.simulation_catalog_snapshot_version",
                reason: "must be > 0",
            });
        }
        validate_bp(
            "finder_run_request.context_alignment_bp",
            self.context_alignment_bp,
        )?;
        validate_bp("finder_run_request.ocr_alignment_bp", self.ocr_alignment_bp)?;
        validate_bp(
            "finder_run_request.llm_assist_alignment_bp",
            self.llm_assist_alignment_bp,
        )?;
        validate_bp(
            "finder_run_request.ambiguity_penalty_bp",
            self.ambiguity_penalty_bp,
        )?;
        validate_bp(
            "finder_run_request.contradictory_field_penalty_bp",
            self.contradictory_field_penalty_bp,
        )?;
        validate_bp(
            "finder_run_request.policy_mismatch_penalty_bp",
            self.policy_mismatch_penalty_bp,
        )?;
        if self.clarify_attempt_index == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "finder_run_request.clarify_attempt_index",
                reason: "must be > 0",
            });
        }
        validate_required_text(
            "finder_run_request.default_category",
            &self.default_category,
            128,
        )?;
        validate_bp(
            "finder_run_request.estimated_frequency_score_bp",
            self.estimated_frequency_score_bp,
        )?;
        validate_bp(
            "finder_run_request.estimated_value_score_bp",
            self.estimated_value_score_bp,
        )?;
        validate_bp(
            "finder_run_request.estimated_roi_score_bp",
            self.estimated_roi_score_bp,
        )?;
        validate_bp(
            "finder_run_request.estimated_feasibility_score_bp",
            self.estimated_feasibility_score_bp,
        )?;
        validate_bp(
            "finder_run_request.estimated_risk_score_bp",
            self.estimated_risk_score_bp,
        )?;
        validate_required_text("finder_run_request.scope_class", &self.scope_class, 64)?;
        validate_required_text(
            "finder_run_request.required_fields_schema_json",
            &self.required_fields_schema_json,
            65_536,
        )?;
        validate_terms(
            "finder_run_request.acceptance_test_suggestion",
            &self.acceptance_test_suggestion,
            64,
            256,
        )?;
        for entry in &self.simulation_catalog {
            entry.validate()?;
        }
        for mapping in &self.gold_mappings {
            mapping.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Ph1SimFinderRuntime {
    config: FinderRuntimeConfig,
}

impl Ph1SimFinderRuntime {
    pub fn new(config: FinderRuntimeConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &FinderRunRequest) -> Result<FinderTerminalPacket, ContractViolation> {
        req.validate()?;

        let transcript = normalize_text(&req.transcript_text);
        let utterance_fingerprint = stable_fingerprint(&transcript);
        let gold_bonus_index: BTreeMap<&str, u16> = req
            .gold_mappings
            .iter()
            .filter(|mapping| mapping.utterance_fingerprint == utterance_fingerprint)
            .map(|mapping| (mapping.simulation_id.as_str(), mapping.gold_match_bonus_bp))
            .collect();

        let mut matched_active = Vec::new();
        let mut matched_draft = Vec::new();
        for entry in &req.simulation_catalog {
            if !candidate_matches_transcript(entry, &transcript) {
                continue;
            }
            match entry.status {
                SimulationStatus::Active => matched_active.push(entry),
                SimulationStatus::Draft => matched_draft.push(entry),
                SimulationStatus::Deprecated | SimulationStatus::Disabled => {}
            }
        }

        if !matched_active.is_empty() {
            let mut ranked = matched_active
                .iter()
                .map(|entry| {
                    score_candidate(
                        req,
                        entry,
                        *gold_bonus_index
                            .get(entry.simulation_id.as_str())
                            .unwrap_or(&0),
                        &transcript,
                    )
                })
                .collect::<Vec<_>>();
            ranked.sort_by(compare_ranked_candidates);

            let top = ranked
                .first()
                .expect("matched_active guaranteed non-empty after guard");
            let second = ranked.get(1);
            let top_margin_bp = second
                .map(|other| {
                    top.confidence_score_bp
                        .saturating_sub(other.confidence_score_bp)
                })
                .unwrap_or(10_000);

            if top.confidence_score_bp >= self.config.match_direct_min_bp
                && top.required_fields_missing.is_empty()
            {
                let reason_code = if top.gold_match_bonus_bp > 0 {
                    reason_codes::SIM_FINDER_MATCH_OK_GOLD_BOOSTED
                } else {
                    reason_codes::SIM_FINDER_MATCH_OK_CATALOG_ACTIVE
                };

                let packet = SimulationMatchPacket::v1(
                    req.tenant_id.clone(),
                    req.user_id.clone(),
                    req.correlation_id,
                    req.turn_id,
                    top.intent_family.clone(),
                    top.simulation_id.clone(),
                    top.confidence_score_bp,
                    top.required_fields_present.clone(),
                    top.required_fields_missing.clone(),
                    top.evidence_spans.clone(),
                    top.risk_tier,
                    top.confirm_required,
                    top.access_actions_required.clone(),
                    format!(
                        "sim_match:{}:{}:{}:{}:{}:{}",
                        req.tenant_id,
                        req.user_id,
                        req.correlation_id,
                        req.turn_id,
                        top.simulation_id,
                        stable_fingerprint(&top.required_fields_present.join("|"))
                    ),
                    "sim_match:deterministic_v1".to_string(),
                    top.fallback_if_inactive_or_missing,
                    reason_code,
                )?;
                let terminal = FinderTerminalPacket::SimulationMatch(packet);
                terminal.validate()?;
                return Ok(terminal);
            }

            let should_clarify = !top.required_fields_missing.is_empty()
                || top.confidence_score_bp >= self.config.match_with_clarify_min_bp
                || top_margin_bp < self.config.tie_margin_min_bp;
            if should_clarify {
                let (missing_field, question, answer_formats, reason_code) =
                    if !top.required_fields_missing.is_empty() {
                        let selection = select_primary_missing_field(top);
                        (
                            selection.field_name,
                            selection.clarify_question,
                            selection.allowed_answer_formats,
                            reason_codes::SIM_FINDER_CLARIFY_MISSING_FIELD,
                        )
                    } else {
                        (
                            "intent_choice".to_string(),
                            "Should I use the first or second matching action?".to_string(),
                            vec![
                                "Use the first one".to_string(),
                                "Use the second one".to_string(),
                                "Neither; I will rephrase".to_string(),
                            ],
                            reason_codes::SIM_FINDER_CLARIFY_LOW_CONFIDENCE_TIE,
                        )
                    };

                let packet = ClarifyPacket::v1(
                    req.tenant_id.clone(),
                    req.user_id.clone(),
                    req.correlation_id,
                    req.turn_id,
                    question,
                    missing_field.clone(),
                    answer_formats,
                    req.clarify_attempt_index
                        .min(self.config.max_clarify_attempts),
                    self.config.max_clarify_attempts,
                    ClarifyOnExceedPolicy::MissingSimulation,
                    format!(
                        "cand_ctx:{}:{}:{}:{}",
                        req.simulation_catalog_snapshot_hash,
                        req.simulation_catalog_snapshot_version,
                        top.simulation_id,
                        stable_fingerprint(&transcript)
                    ),
                    format!(
                        "sim_clarify:{}:{}:{}:{}:{}:{}",
                        req.tenant_id,
                        req.user_id,
                        req.correlation_id,
                        req.turn_id,
                        missing_field,
                        req.clarify_attempt_index
                            .min(self.config.max_clarify_attempts)
                    ),
                    reason_code,
                )?;
                let terminal = FinderTerminalPacket::Clarify(packet);
                terminal.validate()?;
                return Ok(terminal);
            }
        }

        if let Some(draft_entry) = choose_top_draft_candidate(&matched_draft, &transcript) {
            let packet = RefusePacket::v1(
                req.tenant_id.clone(),
                req.user_id.clone(),
                req.correlation_id,
                req.turn_id,
                reason_codes::SIM_FINDER_SIMULATION_INACTIVE,
                "The capability exists as a draft and is not active yet.".to_string(),
                vec![
                    format!(
                        "catalog.active.check:{}:{}:{}",
                        req.simulation_catalog_snapshot_hash, req.correlation_id, req.turn_id
                    ),
                    format!(
                        "catalog.draft.hit:{}:{}:{}",
                        draft_entry.simulation_id, req.correlation_id, req.turn_id
                    ),
                ],
                Some(format!(
                    "draft:{}:{}",
                    draft_entry.simulation_id, req.simulation_catalog_snapshot_version
                )),
            )?;
            let terminal = FinderTerminalPacket::Refuse(packet);
            terminal.validate()?;
            return Ok(terminal);
        }

        let active_proof_ref = format!(
            "catalog.active.none:{}:{}:{}",
            req.simulation_catalog_snapshot_hash, req.correlation_id, req.turn_id
        );
        let draft_proof_ref = format!(
            "catalog.draft.none:{}:{}:{}",
            req.simulation_catalog_snapshot_hash, req.correlation_id, req.turn_id
        );
        let no_match_proof_ref = format!(
            "catalog.none:{}:{}:{}",
            req.simulation_catalog_snapshot_hash, req.correlation_id, req.turn_id
        );

        let trace = vec![
            CatalogCheckTraceEntry {
                kind: CatalogCheckKind::ActiveCheck,
                checked_at: req.now,
                proof_ref: active_proof_ref.clone(),
            },
            CatalogCheckTraceEntry {
                kind: CatalogCheckKind::DraftCheck,
                checked_at: MonotonicTimeNs(req.now.0.saturating_add(1)),
                proof_ref: draft_proof_ref.clone(),
            },
            CatalogCheckTraceEntry {
                kind: CatalogCheckKind::NoneFound,
                checked_at: MonotonicTimeNs(req.now.0.saturating_add(2)),
                proof_ref: no_match_proof_ref.clone(),
            },
        ];

        let worthiness_score_bp = worthiness_score_bp(
            req.estimated_frequency_score_bp,
            req.estimated_value_score_bp,
            req.estimated_roi_score_bp,
            req.estimated_feasibility_score_bp,
            scope_score_from_class(&req.scope_class),
            req.estimated_risk_score_bp,
        );

        let packet = MissingSimulationPacket::v1(
            req.tenant_id.clone(),
            req.user_id.clone(),
            req.correlation_id,
            req.turn_id,
            capability_name_from_transcript(&transcript),
            req.transcript_text.clone(),
            transcript,
            req.default_category.clone(),
            req.estimated_frequency_score_bp,
            req.estimated_value_score_bp,
            req.estimated_roi_score_bp,
            req.estimated_feasibility_score_bp,
            req.estimated_risk_score_bp,
            worthiness_score_bp,
            req.scope_class.clone(),
            vec!["UNSPECIFIED_PROVIDER".to_string()],
            "unmapped.simulation.family".to_string(),
            req.required_fields_schema_json.clone(),
            req.acceptance_test_suggestion.clone(),
            format!(
                "{}:{}",
                req.tenant_id,
                stable_fingerprint(&req.transcript_text)
            ),
            trace,
            active_proof_ref,
            draft_proof_ref,
            no_match_proof_ref,
            format!(
                "missing_sim:{}:{}:{}:{}:{}",
                req.tenant_id,
                req.user_id,
                stable_fingerprint(&req.transcript_text),
                req.correlation_id,
                req.turn_id
            ),
            reason_codes::SIM_FINDER_MISSING_SIMULATION,
        )?;
        let terminal = FinderTerminalPacket::MissingSimulation(packet);
        terminal.validate()?;
        Ok(terminal)
    }
}

#[derive(Debug, Clone)]
struct RankedCandidate {
    simulation_id: String,
    intent_family: String,
    simulation_priority: u16,
    risk_tier: FinderRiskTier,
    confirm_required: bool,
    fallback_if_inactive_or_missing: FinderFallbackPolicy,
    access_actions_required: Vec<String>,
    confidence_score_bp: u16,
    gold_match_bonus_bp: u16,
    required_fields_present: Vec<String>,
    required_fields_missing: Vec<String>,
    required_field_specs: BTreeMap<String, FinderFieldSpec>,
    evidence_spans: Vec<String>,
}

#[derive(Debug, Clone)]
struct MissingFieldSelection {
    field_name: String,
    clarify_question: String,
    allowed_answer_formats: Vec<String>,
    entropy_score_bp: u16,
    downstream_risk_bp: u16,
}

fn score_candidate(
    req: &FinderRunRequest,
    entry: &FinderSimulationCatalogEntry,
    gold_match_bonus_bp: u16,
    transcript: &str,
) -> RankedCandidate {
    let matched_terms = matched_candidate_terms(entry, transcript);
    let intent_confidence_bp = intent_confidence_bp(entry, transcript, &matched_terms);

    let mut required_fields_present = Vec::new();
    let mut required_fields_missing = Vec::new();
    let mut required_field_specs = BTreeMap::new();
    for spec in &entry.required_fields {
        required_field_specs.insert(spec.field_name.clone(), spec.clone());
        if !spec.required {
            continue;
        }
        if field_present(spec, transcript) {
            required_fields_present.push(spec.field_name.clone());
        } else {
            required_fields_missing.push(spec.field_name.clone());
        }
    }
    required_fields_present.sort();
    required_fields_missing.sort();

    let required_field_coverage_bp = if entry.required_fields.is_empty() {
        10_000
    } else {
        ((required_fields_present.len() as u32 * 10_000u32) / entry.required_fields.len() as u32)
            as u16
    };

    let evidence_coverage_bp = if entry.synonym_terms.is_empty() {
        5_000
    } else {
        ((matched_terms.len() as u32 * 10_000u32) / entry.synonym_terms.len() as u32).min(10_000)
            as u16
    };

    let catalog_status_bp = match entry.status {
        SimulationStatus::Active => 10_000,
        SimulationStatus::Draft => 4_000,
        SimulationStatus::Deprecated => 1_000,
        SimulationStatus::Disabled => 0,
    };

    let penalty_bp_total = req
        .ambiguity_penalty_bp
        .saturating_add(req.contradictory_field_penalty_bp)
        .saturating_add(req.policy_mismatch_penalty_bp);

    let raw_score_bp = ((WORD_INTENT_CONFIDENCE * intent_confidence_bp as u32)
        + (WORD_REQUIRED_FIELD_COVERAGE * required_field_coverage_bp as u32)
        + (WORD_EVIDENCE_COVERAGE * evidence_coverage_bp as u32)
        + (WORD_CATALOG_STATUS * catalog_status_bp as u32)
        + (WORD_CONTEXT_ALIGNMENT * req.context_alignment_bp as u32)
        + (WORD_OCR_ALIGNMENT * req.ocr_alignment_bp as u32)
        + (WORD_LLM_ASSIST_ALIGNMENT * req.llm_assist_alignment_bp as u32)
        + (WORD_GOLD_BONUS * gold_match_bonus_bp as u32))
        / 100;
    let confidence_score_bp = raw_score_bp
        .saturating_sub(penalty_bp_total as u32)
        .min(10_000) as u16;

    let mut evidence_spans = matched_terms
        .iter()
        .map(|term| format!("term:{term}"))
        .collect::<Vec<_>>();
    if evidence_spans.is_empty() {
        evidence_spans.push(format!("intent:{}", entry.intent_family));
    }

    RankedCandidate {
        simulation_id: entry.simulation_id.clone(),
        intent_family: entry.intent_family.clone(),
        simulation_priority: entry.simulation_priority,
        risk_tier: entry.risk_tier,
        confirm_required: entry.confirm_required,
        fallback_if_inactive_or_missing: entry.fallback_if_inactive_or_missing,
        access_actions_required: entry.required_access_actions.clone(),
        confidence_score_bp,
        gold_match_bonus_bp,
        required_fields_present,
        required_fields_missing,
        required_field_specs,
        evidence_spans,
    }
}

fn compare_ranked_candidates(left: &RankedCandidate, right: &RankedCandidate) -> Ordering {
    right
        .confidence_score_bp
        .cmp(&left.confidence_score_bp)
        .then_with(|| right.gold_match_bonus_bp.cmp(&left.gold_match_bonus_bp))
        .then_with(|| right.simulation_priority.cmp(&left.simulation_priority))
        .then_with(|| left.simulation_id.cmp(&right.simulation_id))
}

fn choose_top_draft_candidate<'a>(
    drafts: &'a [&'a FinderSimulationCatalogEntry],
    transcript: &str,
) -> Option<&'a FinderSimulationCatalogEntry> {
    let mut ranked = drafts.to_vec();
    ranked.sort_by(|left, right| {
        let left_hits = matched_candidate_terms(left, transcript).len();
        let right_hits = matched_candidate_terms(right, transcript).len();
        right_hits
            .cmp(&left_hits)
            .then_with(|| right.simulation_priority.cmp(&left.simulation_priority))
            .then_with(|| left.simulation_id.cmp(&right.simulation_id))
    });
    ranked.first().copied()
}

fn select_primary_missing_field(candidate: &RankedCandidate) -> MissingFieldSelection {
    let mut selections = candidate
        .required_fields_missing
        .iter()
        .filter_map(|name| {
            candidate.required_field_specs.get(name).map(|spec| {
                let entropy_score_bp = (((50u32 * spec.domain_cardinality_bp as u32)
                    + (30u32 * spec.candidate_split_bp as u32)
                    + (20u32 * spec.downstream_risk_bp as u32))
                    / 100) as u16;
                MissingFieldSelection {
                    field_name: spec.field_name.clone(),
                    clarify_question: spec.clarify_question.clone(),
                    allowed_answer_formats: spec.allowed_answer_formats.clone(),
                    entropy_score_bp,
                    downstream_risk_bp: spec.downstream_risk_bp,
                }
            })
        })
        .collect::<Vec<_>>();

    selections.sort_by(|left, right| {
        right
            .entropy_score_bp
            .cmp(&left.entropy_score_bp)
            .then_with(|| right.downstream_risk_bp.cmp(&left.downstream_risk_bp))
            .then_with(|| left.field_name.cmp(&right.field_name))
    });
    selections
        .into_iter()
        .next()
        .expect("select_primary_missing_field requires at least one missing field")
}

fn candidate_matches_transcript(entry: &FinderSimulationCatalogEntry, transcript: &str) -> bool {
    if matched_candidate_terms(entry, transcript).is_empty() {
        return false;
    }
    true
}

fn matched_candidate_terms(entry: &FinderSimulationCatalogEntry, transcript: &str) -> Vec<String> {
    let mut terms = entry
        .synonym_terms
        .iter()
        .filter_map(|term| {
            if transcript_contains_term(transcript, term) {
                Some(term.to_ascii_lowercase())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if transcript_contains_term(transcript, &entry.intent_family) {
        terms.push(entry.intent_family.to_ascii_lowercase());
    }
    terms.sort();
    terms.dedup();
    terms
}

fn intent_confidence_bp(
    entry: &FinderSimulationCatalogEntry,
    transcript: &str,
    matched_terms: &[String],
) -> u16 {
    let required_hits = matched_terms.len() as u32;
    if required_hits == 0 {
        return 0;
    }
    let synonym_denominator = entry.synonym_terms.len().max(1) as u32;
    let term_ratio_bp = (required_hits * 10_000u32 / synonym_denominator).min(10_000);
    let contains_intent_family = transcript_contains_term(transcript, &entry.intent_family);
    let family_bonus_bp = if contains_intent_family { 1_000u32 } else { 0 };
    (term_ratio_bp.saturating_add(family_bonus_bp).min(10_000)) as u16
}

fn field_present(spec: &FinderFieldSpec, transcript: &str) -> bool {
    spec.detector_terms
        .iter()
        .any(|term| transcript_contains_term(transcript, term))
}

fn transcript_contains_term(transcript: &str, term: &str) -> bool {
    let t = term.to_ascii_lowercase();
    let hay = transcript.to_ascii_lowercase();
    if t.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace())
    {
        contains_word(&hay, &t)
    } else {
        hay.contains(&t)
    }
}

fn contains_word(haystack: &str, needle: &str) -> bool {
    let trimmed = needle.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.contains(' ') {
        return haystack.contains(trimmed);
    }
    haystack
        .split(|c: char| !c.is_ascii_alphanumeric())
        .any(|w| w == trimmed)
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_ascii_lowercase()
}

fn capability_name_from_transcript(transcript: &str) -> String {
    let normalized = transcript
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    let compact = normalized
        .split('_')
        .filter(|part| !part.is_empty())
        .take(8)
        .collect::<Vec<_>>()
        .join("_");
    if compact.is_empty() {
        "unknown_capability".to_string()
    } else {
        compact
    }
}

fn scope_score_from_class(scope_class: &str) -> u16 {
    if scope_class.eq_ignore_ascii_case("cross_tenant") {
        9_000
    } else {
        5_000
    }
}

fn worthiness_score_bp(
    frequency_bp: u16,
    value_bp: u16,
    estimated_roi_score_bp: u16,
    feasibility_bp: u16,
    scope_bp: u16,
    risk_bp: u16,
) -> u16 {
    let raw = ((25u32 * frequency_bp as u32)
        + (25u32 * value_bp as u32)
        + (15u32 * estimated_roi_score_bp as u32)
        + (20u32 * feasibility_bp as u32)
        + (15u32 * scope_bp as u32))
        / 100;
    let risk_penalty_bp = risk_bp as u32 / 2;
    raw.saturating_sub(risk_penalty_bp).min(10_000) as u16
}

fn stable_fingerprint(input: &str) -> String {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{hash:016x}")
}

fn validate_required_text(
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
    Ok(())
}

fn validate_terms(
    field: &'static str,
    values: &[String],
    max_items: usize,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if values.is_empty() || values.len() > max_items {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain a bounded non-empty list",
        });
    }
    for value in values {
        validate_required_text(field, value, max_len)?;
    }
    Ok(())
}

fn validate_bp(field: &'static str, value: u16) -> Result<(), ContractViolation> {
    if value > 10_000 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 10000",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_request(
        simulation_catalog: Vec<FinderSimulationCatalogEntry>,
    ) -> Result<FinderRunRequest, ContractViolation> {
        FinderRunRequest::v1(
            "tenant_1".to_string(),
            "user_1".to_string(),
            111,
            222,
            MonotonicTimeNs(1_000),
            "selene schedule dinner tomorrow at 7".to_string(),
            "simcat_hash_v1".to_string(),
            1,
            simulation_catalog,
            Vec::new(),
            10_000,
            10_000,
            10_000,
            0,
            0,
            0,
            1,
            "scheduling".to_string(),
            6_500,
            7_000,
            6_000,
            6_500,
            3_000,
            "tenant_only".to_string(),
            "{\"required\":[\"when\"]}".to_string(),
            vec!["AT-SIM-FINDER-M2-03".to_string()],
        )
    }

    fn field_when() -> FinderFieldSpec {
        FinderFieldSpec::required_v1(
            "when".to_string(),
            vec!["tomorrow".to_string(), "at".to_string()],
            7_000,
            6_500,
            4_000,
            "What exact time should I use?".to_string(),
            vec![
                "Tomorrow at 7 PM".to_string(),
                "2026-03-02T19:00".to_string(),
            ],
        )
        .expect("field spec should build")
    }

    fn field_where() -> FinderFieldSpec {
        FinderFieldSpec::required_v1(
            "where".to_string(),
            vec!["in".to_string(), "at home".to_string()],
            9_000,
            9_000,
            9_000,
            "Where should this happen?".to_string(),
            vec![
                "At home".to_string(),
                "At a restaurant in Pudong".to_string(),
            ],
        )
        .expect("field spec should build")
    }

    fn active_entry_with_priority(
        simulation_id: &str,
        simulation_priority: u16,
    ) -> FinderSimulationCatalogEntry {
        FinderSimulationCatalogEntry::v1(
            simulation_id.to_string(),
            "schedule".to_string(),
            SimulationStatus::Active,
            simulation_priority,
            vec!["REMINDER_SCHEDULE".to_string()],
            FinderRiskTier::Low,
            false,
            FinderFallbackPolicy::Clarify,
            vec![
                "schedule".to_string(),
                "dinner".to_string(),
                "tomorrow".to_string(),
            ],
            vec![field_when()],
            vec!["calendar".to_string()],
        )
        .expect("entry should build")
    }

    #[test]
    fn at_sim_finder_m2_01_deterministic_top1_selection() {
        let runtime = Ph1SimFinderRuntime::new(FinderRuntimeConfig::mvp_v1());
        let req = base_request(vec![
            active_entry_with_priority("PH1.REM.100", 10),
            active_entry_with_priority("PH1.REM.200", 100),
        ])
        .expect("request should build");

        let out = runtime.run(&req).expect("finder run should succeed");
        match out {
            FinderTerminalPacket::SimulationMatch(packet) => {
                assert_eq!(packet.simulation_id, "PH1.REM.200");
                assert_eq!(packet.candidate_rank, 1);
            }
            other => panic!("expected SimulationMatch, got {other:?}"),
        }
    }

    #[test]
    fn at_sim_finder_m2_02_active_draft_none_rule_with_proofs() {
        let runtime = Ph1SimFinderRuntime::new(FinderRuntimeConfig::mvp_v1());

        let req_active = base_request(vec![
            FinderSimulationCatalogEntry::v1(
                "PH1.REM.DRAFT".to_string(),
                "schedule".to_string(),
                SimulationStatus::Draft,
                10,
                vec!["REMINDER_SCHEDULE".to_string()],
                FinderRiskTier::Low,
                false,
                FinderFallbackPolicy::Clarify,
                vec!["schedule".to_string(), "dinner".to_string()],
                vec![field_when()],
                vec!["calendar".to_string()],
            )
            .unwrap(),
            active_entry_with_priority("PH1.REM.ACTIVE", 90),
        ])
        .expect("request should build");
        match runtime.run(&req_active).expect("run should succeed") {
            FinderTerminalPacket::SimulationMatch(packet) => {
                assert_eq!(packet.simulation_id, "PH1.REM.ACTIVE");
            }
            other => panic!("expected match when active exists, got {other:?}"),
        }

        let req_draft_only = base_request(vec![FinderSimulationCatalogEntry::v1(
            "PH1.REM.DRAFT_ONLY".to_string(),
            "schedule".to_string(),
            SimulationStatus::Draft,
            50,
            vec!["REMINDER_SCHEDULE".to_string()],
            FinderRiskTier::Low,
            false,
            FinderFallbackPolicy::Clarify,
            vec!["schedule".to_string(), "dinner".to_string()],
            vec![field_when()],
            vec!["calendar".to_string()],
        )
        .unwrap()])
        .expect("request should build");
        match runtime.run(&req_draft_only).expect("run should succeed") {
            FinderTerminalPacket::Refuse(packet) => {
                assert_eq!(
                    packet.reason_code,
                    reason_codes::SIM_FINDER_SIMULATION_INACTIVE
                );
                assert!(packet.existing_draft_ref.is_some());
            }
            other => panic!("expected refuse for draft-only candidate, got {other:?}"),
        }

        let req_none = base_request(vec![]).expect("request should build");
        match runtime.run(&req_none).expect("run should succeed") {
            FinderTerminalPacket::MissingSimulation(packet) => {
                assert!(!packet.active_check_proof_ref.is_empty());
                assert!(!packet.draft_check_proof_ref.is_empty());
                assert!(!packet.no_match_proof_ref.is_empty());
                assert_eq!(packet.catalog_check_trace.len(), 3);
                assert_eq!(
                    packet.catalog_check_trace[0].kind,
                    CatalogCheckKind::ActiveCheck
                );
                assert_eq!(
                    packet.catalog_check_trace[1].kind,
                    CatalogCheckKind::DraftCheck
                );
                assert_eq!(
                    packet.catalog_check_trace[2].kind,
                    CatalogCheckKind::NoneFound
                );
            }
            other => panic!("expected missing simulation packet, got {other:?}"),
        }
    }

    #[test]
    fn at_sim_finder_m2_03_clarify_selects_one_missing_field_deterministically() {
        let runtime = Ph1SimFinderRuntime::new(FinderRuntimeConfig::mvp_v1());
        let entry = FinderSimulationCatalogEntry::v1(
            "PH1.REM.300".to_string(),
            "schedule".to_string(),
            SimulationStatus::Active,
            10,
            vec!["REMINDER_SCHEDULE".to_string()],
            FinderRiskTier::Low,
            false,
            FinderFallbackPolicy::Clarify,
            vec!["schedule".to_string(), "dinner".to_string()],
            vec![field_when(), field_where()],
            vec!["calendar".to_string()],
        )
        .expect("entry should build");

        let req = FinderRunRequest::v1(
            "tenant_1".to_string(),
            "user_1".to_string(),
            111,
            222,
            MonotonicTimeNs(1_000),
            "selene schedule dinner".to_string(),
            "simcat_hash_v1".to_string(),
            1,
            vec![entry],
            Vec::new(),
            8_000,
            6_000,
            5_000,
            0,
            0,
            0,
            1,
            "scheduling".to_string(),
            6_500,
            7_000,
            6_000,
            6_500,
            3_000,
            "tenant_only".to_string(),
            "{\"required\":[\"where\"]}".to_string(),
            vec!["AT-SIM-FINDER-M2-04".to_string()],
        )
        .expect("request should build");

        let out = runtime.run(&req).expect("run should succeed");
        match out {
            FinderTerminalPacket::Clarify(packet) => {
                assert_eq!(packet.missing_field, "where");
                assert_eq!(packet.attempt_index, 1);
                assert_eq!(packet.allowed_answer_formats.len(), 2);
            }
            other => panic!("expected clarify output, got {other:?}"),
        }
    }
}
