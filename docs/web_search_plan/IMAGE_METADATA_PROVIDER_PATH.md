# Image Metadata Provider Path

## H388 Decision

H388 selects `Outcome C -- NO_APPROVED_PROVIDER_PATH`.

Current repo truth does not contain an approved live public image metadata provider/source path for displayable sourced image/photo cards. H388 therefore keeps sourced image/photo cards deferred and records the provider/source design requirements needed for a future implementation.

## H389 Decision

H389 selects `Outcome B -- APPROVED_BRAVE_IMAGE_METADATA_ONLY_PATH`.

PH1.E now has a narrow Brave-backed public image metadata path using the existing `brave_search_api_key` provider posture and the existing bounded HTTP client pattern. The path records endpoint label/hash, query hash, max query count, max result count, timeout, retry policy, provider outcome, support booleans, source-page binding, source-domain derivation, and safety proof metadata in Deep Research response metadata.

H389 does not display sourced image/photo cards. Brave image metadata remains metadata-only because the current provider proof does not establish complete display-safety and license/usage rights. The response therefore emits `WEB_IMAGE_METADATA_PROVIDER_PATH_METADATA_ONLY` and `WEB_IMAGE_SOURCE_CARD_DISPLAY_DEFERRED`, and must not emit `WEB_IMAGE_SOURCE_CARD_PASS`.

H389 also preserves the H388 safety boundary:

- no non-Brave provider is added
- no new provider dependency is added
- no provider secret value is printed or persisted
- image bytes are not downloaded
- source pages are not fetched to discover images
- raw private image queries are blocked or deferred
- query proof stores only a query hash or redacted query
- full provider request URLs with query strings are not persisted
- screenshot content remains layout guidance only, never evidence
- screenshot images remain non-provider images

## Candidate Matrix

| Candidate | Repo surface | Current capability | Display allowed | Blocker |
| --- | --- | --- | --- | --- |
| Existing Brave web/news result metadata | `crates/selene_engines/src/ph1e.rs`, `crates/selene_os/src/web_search_plan/web_provider/brave_adapter.rs` | Text web/news results with citation URLs, titles, and snippets | No | No source-page-bound image metadata is parsed or proven |
| Brave image metadata endpoint | `crates/selene_engines/src/ph1e.rs` | Metadata-only public image provider path with source-page binding where provider metadata supplies it | No | Display safety and license/usage proof remain incomplete |
| `web_search_plan/vision` | `crates/selene_os/src/web_search_plan/vision/**` | User-supplied/local/URL asset analysis with hash, locator, MIME, safe-mode, and budgets | No for web cards | Asset analysis is not a public web image-card provider |
| Page fetch/extraction | `crates/selene_os/src/web_search_plan/url/**` | Bounded URL fetch/text extraction surfaces | No | Page-image extraction lacks source, safety, and license policy for display |
| PH1.E provider abstraction | `crates/selene_engines/src/ph1e.rs` | Brave/OpenAI text web fallback path | No | No image endpoint/class in PH1.E provider config |
| Existing repo media/image provider | Search discovery | Not found | No | No approved public image metadata provider exists |
| Future provider path | Not implemented | Required design path | No | Requires approved provider selection before implementation |

## ImageProviderPathPacket Contract

The live Deep Research response metadata must be able to represent:

- `provider_path_id`
- `selected_outcome`
- `selected_candidate_id`
- `provider_name`
- `provider_kind`
- `secret_id`
- `endpoint_class`
- `endpoint_path_hash_or_label`
- `query_hash_or_redacted_query`
- `query_leakage_policy`
- `max_query_count`
- `max_result_count`
- `timeout_ms`
- `retry_policy`
- `candidate_matrix`
- `supports_image_url`
- `supports_thumbnail_url`
- `supports_source_page_url`
- `supports_source_domain`
- `supports_retrieved_at`
- `supports_display_safety`
- `supports_license_or_usage_note`
- `supports_image_source_verified`
- `supports_linked_claim_ids`
- `display_allowed`
- `display_deferred_reason`
- `blocker`
- `proof_id`
- `no_new_provider_dependency`
- `no_live_image_provider_call`
- `no_image_bytes_downloaded`
- `no_source_page_scrape`
- `query_hash_only`
- `screenshot_not_evidence`

## Future Image Evidence Contract

A future displayable `ImageEvidencePacket` or `ImageSourceCardPacket` must require:

- `image_id`
- `image_url`
- `source_page_url`
- `source_domain`
- `title_or_alt_text` when available
- `caption` when available
- `publisher` when available
- `retrieved_at`
- `image_type`
- `entity_or_topic`
- `license_or_usage_note` when available
- `display_safe`
- `linked_claim_ids` when applicable
- `provider`
- `proof_id`
- `image_source_verified`

## Safety Policy

- Image URL alone is insufficient.
- Thumbnail URL alone is insufficient.
- Missing `source_page_url`, `source_domain`, `retrieved_at`, `display_safe`, `provider`, or `proof_id` defers display.
- Unknown display safety defers display.
- Unknown rights/license defers display unless an approved repo policy explicitly allows display with an unknown-license note.
- Fixture image data must never be marked live.
- Generated images are allowed only through a separate user-requested image-generation flow.
- Screenshot facts are not evidence.
- Screenshot images are not provider images.
- Images support cited answers visually; they never replace text citations.

## Secret And Privacy Policy

- Provider packets may record a secret ID, never the secret value.
- Private image queries are blocked or deferred unless a future approved provider policy explicitly permits them.
- Image metadata cache/retention must default to metadata-only retention and must not create a hidden web image archive.
- H388 adds no new external provider dependency and introduces no new live image-provider call.
- H389 reuses the existing Brave secret ID and bounded HTTP posture.
- H389 stores query hash/redacted query proof only, not raw private queries.
- H389 does not download image bytes and does not scrape source pages.

## Future Implementation Step

Complete display-safety and license/usage verification for the Brave metadata-only path, or select an additional approved image metadata provider if Brave cannot provide those fields. Only after that proof may a separate sourced image/photo card UI/display build be authorized.
