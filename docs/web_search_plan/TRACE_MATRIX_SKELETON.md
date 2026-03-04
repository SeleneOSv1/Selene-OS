# Trace Matrix (Runs 1-30)

Format:
`RUN=<n> | acceptance_tests=<reference> | ci_script=<reference> | proof_commands=<reference>`

RUN=1 | acceptance_tests=crates/selene_os/src/web_search_plan/tests.rs::test_valid_fixtures_pass | ci_script=scripts/web_search_plan/check_contracts.sh | proof_commands=scripts/web_search_plan/check_contracts.sh ; scripts/web_search_plan/check_reason_codes.sh ; scripts/web_search_plan/check_idempotency.sh ; scripts/web_search_plan/check_turn_state_machine.sh ; scripts/web_search_plan/check_handoff_ownership.sh
RUN=2 | acceptance_tests=crates/selene_os/src/web_search_plan/proxy/proxy_tests.rs | ci_script=scripts/web_search_plan/check_proxy_universal.sh | proof_commands=scripts/web_search_plan/check_proxy_universal.sh
RUN=3 | acceptance_tests=crates/selene_os/src/web_search_plan/url/url_tests.rs | ci_script=scripts/web_search_plan/check_url_fetch_core.sh | proof_commands=scripts/web_search_plan/check_url_fetch_core.sh
RUN=4 | acceptance_tests=crates/selene_os/src/web_search_plan/chunk/chunk_tests.rs | ci_script=scripts/web_search_plan/check_chunk_hash_core.sh | proof_commands=scripts/web_search_plan/check_chunk_hash_core.sh
RUN=5 | acceptance_tests=crates/selene_os/src/web_search_plan/synthesis/synthesis_tests.rs | ci_script=scripts/web_search_plan/check_synthesis_core.sh | proof_commands=scripts/web_search_plan/check_synthesis_core.sh
RUN=6 | acceptance_tests=crates/selene_os/src/web_search_plan/write/write_tests.rs | ci_script=scripts/web_search_plan/check_write_core.sh | proof_commands=scripts/web_search_plan/check_write_core.sh
RUN=7 | acceptance_tests=crates/selene_os/src/web_search_plan/web_provider/web_provider_tests.rs | ci_script=scripts/web_search_plan/check_web_provider_ladder.sh | proof_commands=scripts/web_search_plan/check_web_provider_ladder.sh
RUN=8 | acceptance_tests=crates/selene_os/src/web_search_plan/news_provider/news_tests.rs | ci_script=scripts/web_search_plan/check_news_provider_ladder.sh | proof_commands=scripts/web_search_plan/check_news_provider_ladder.sh
RUN=9 | acceptance_tests=crates/selene_os/src/web_search_plan/planning/planning_tests.rs | ci_script=scripts/web_search_plan/check_search_topk_pipeline.sh | proof_commands=scripts/web_search_plan/check_search_topk_pipeline.sh
RUN=10 | acceptance_tests=crates/selene_os/src/web_search_plan/news/news_tests.rs | ci_script=scripts/web_search_plan/check_news_provider_ladder.sh | proof_commands=scripts/web_search_plan/check_news_provider_ladder.sh
RUN=11 | acceptance_tests=crates/selene_os/src/web_search_plan/learn/learn_tests.rs | ci_script=scripts/web_search_plan/check_learning_layer.sh | proof_commands=scripts/web_search_plan/check_learning_layer.sh
RUN=12 | acceptance_tests=crates/selene_os/src/web_search_plan/synthesis/synthesis_tests.rs | ci_script=scripts/web_search_plan/check_synthesis_core.sh | proof_commands=scripts/web_search_plan/check_synthesis_core.sh
RUN=13 | acceptance_tests=crates/selene_os/src/web_search_plan/write/write_tests.rs | ci_script=scripts/web_search_plan/check_write_core.sh | proof_commands=scripts/web_search_plan/check_write_core.sh
RUN=14 | acceptance_tests=crates/selene_os/src/web_search_plan/diag/diag_tests.rs | ci_script=scripts/web_search_plan/check_debug_packet.sh | proof_commands=scripts/web_search_plan/check_debug_packet.sh
RUN=15 | acceptance_tests=crates/selene_os/src/web_search_plan/perf_cost/perf_cost_tests.rs | ci_script=scripts/web_search_plan/check_perf_cost_tiers.sh | proof_commands=scripts/web_search_plan/check_perf_cost_tiers.sh
RUN=16 | acceptance_tests=crates/selene_os/src/web_search_plan/cache/cache_tests.rs ; crates/selene_os/src/web_search_plan/parallel/parallel_tests.rs | ci_script=scripts/web_search_plan/check_cache_parallel.sh | proof_commands=scripts/web_search_plan/check_cache_parallel.sh
RUN=17 | acceptance_tests=crates/selene_os/src/web_search_plan/replay/replay_tests.rs | ci_script=scripts/web_search_plan/check_replay_harness.sh ; scripts/web_search_plan/check_quality_gates.sh | proof_commands=scripts/web_search_plan/check_replay_harness.sh ; scripts/web_search_plan/check_quality_gates.sh
RUN=18 | acceptance_tests=crates/selene_os/src/web_search_plan/learn/learn_tests.rs | ci_script=scripts/web_search_plan/check_learning_layer.sh | proof_commands=scripts/web_search_plan/check_learning_layer.sh
RUN=19 | acceptance_tests=crates/selene_os/src/web_search_plan/structured/structured_tests.rs | ci_script=scripts/web_search_plan/check_structured_connectors.sh | proof_commands=scripts/web_search_plan/check_structured_connectors.sh
RUN=20 | acceptance_tests=crates/selene_os/src/web_search_plan/document/document_tests.rs | ci_script=scripts/web_search_plan/check_document_parsing.sh | proof_commands=scripts/web_search_plan/check_document_parsing.sh
RUN=21 | acceptance_tests=crates/selene_os/src/web_search_plan/analytics/analytics_tests.rs | ci_script=scripts/web_search_plan/check_analytics_numeric_consensus.sh | proof_commands=scripts/web_search_plan/check_analytics_numeric_consensus.sh
RUN=22 | acceptance_tests=crates/selene_os/src/web_search_plan/competitive/competitive_tests.rs | ci_script=scripts/web_search_plan/check_competitive_intel.sh | proof_commands=scripts/web_search_plan/check_competitive_intel.sh
RUN=23 | acceptance_tests=crates/selene_os/src/web_search_plan/realtime/realtime_tests.rs | ci_script=scripts/web_search_plan/check_realtime_api_mode.sh | proof_commands=scripts/web_search_plan/check_realtime_api_mode.sh
RUN=24 | acceptance_tests=crates/selene_os/src/web_search_plan/regulatory/regulatory_tests.rs | ci_script=scripts/web_search_plan/check_regulatory_mode.sh | proof_commands=scripts/web_search_plan/check_regulatory_mode.sh
RUN=25 | acceptance_tests=crates/selene_os/src/web_search_plan/trust/trust_tests.rs | ci_script=scripts/web_search_plan/check_trust_model.sh | proof_commands=scripts/web_search_plan/check_trust_model.sh
RUN=26 | acceptance_tests=crates/selene_os/src/web_search_plan/multihop/multihop_tests.rs | ci_script=scripts/web_search_plan/check_multihop_research.sh | proof_commands=scripts/web_search_plan/check_multihop_research.sh
RUN=27 | acceptance_tests=crates/selene_os/src/web_search_plan/temporal/temporal_tests.rs | ci_script=scripts/web_search_plan/check_temporal_mode.sh | proof_commands=scripts/web_search_plan/check_temporal_mode.sh
RUN=28 | acceptance_tests=crates/selene_os/src/web_search_plan/risk/risk_tests.rs | ci_script=scripts/web_search_plan/check_risk_mode.sh | proof_commands=scripts/web_search_plan/check_risk_mode.sh
RUN=29 | acceptance_tests=crates/selene_os/src/web_search_plan/merge/merge_tests.rs | ci_script=scripts/web_search_plan/check_merge_mode.sh | proof_commands=scripts/web_search_plan/check_merge_mode.sh
RUN=30 | acceptance_tests=crates/selene_os/src/web_search_plan/release/release_tests.rs | ci_script=scripts/web_search_plan/check_release_lock.sh ; scripts/web_search_plan/check_trace_matrix.sh ; scripts/web_search_plan/check_slo_lock.sh ; scripts/web_search_plan/generate_release_evidence_pack.sh | proof_commands=scripts/web_search_plan/check_release_lock.sh ; scripts/web_search_plan/check_trace_matrix.sh ; scripts/web_search_plan/check_slo_lock.sh ; scripts/web_search_plan/generate_release_evidence_pack.sh

Supplemental:
RUN=12A | acceptance_tests=crates/selene_os/src/web_search_plan/vision/vision_tests.rs | ci_script=scripts/web_search_plan/check_vision_engine.sh | proof_commands=scripts/web_search_plan/check_vision_engine.sh
RUN=21A | acceptance_tests=crates/selene_os/src/web_search_plan/vision/vision_tests.rs | ci_script=scripts/web_search_plan/check_vision_engine.sh | proof_commands=scripts/web_search_plan/check_vision_engine.sh
