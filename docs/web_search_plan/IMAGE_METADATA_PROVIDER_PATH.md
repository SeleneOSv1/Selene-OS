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

## H390 Decision

H390 selects `Outcome B -- DISPLAY_DEFERRED_LICENSE_OR_SAFETY_INCOMPLETE` for the live Brave metadata path.

PH1.E now emits an `ImageDisplayEligibilityPacket`-equivalent proof layer for Brave image metadata. The packet records provider identity, live/fixture separation, image URL presence, thumbnail URL presence, source-page URL presence, derived source domain, retrieved-at proof, source-page verification status/reason, canonical URL if safely available, `og:image` / `twitter:image` match booleans, page-title presence, explicit license/usage signal, robots `noindex` / `noimageindex`, display-safety posture, display eligibility, display-deferred or display-blocked reason, no-image-bytes-downloaded proof, no-raw-image-cache proof, no-source-page-scrape proof, query-hash-only proof, and the requirement that text citations remain separate.

H390 does not display sourced image/photo cards. It does not modify Desktop Swift/UI/chrome. It does not download image bytes, cache raw image bytes, create a hidden web image archive, fetch source pages broadly, run JavaScript, use cookies/auth headers, bypass paywalls, add providers, or infer license/usage rights from Brave provider presence.

The default live Brave outcome remains display deferred unless explicit source-page binding, display-safety, and license/usage proof are present. Image URL alone, thumbnail URL alone, source-page URL alone, Brave provider presence, screenshot content, screenshot images, fake metadata, generated placeholders, and fixture-only image data are not sufficient for display.

H390 proof passed with `cargo test -p selene_adapter h390 -- --test-threads=1 => 1 passed`, `cargo test -p selene_engines h390 -- --test-threads=1 => 5 passed; 1 ignored`, live `cargo test -p selene_engines h390_live_brave_image_display_eligibility_maps_real_metadata_without_display -- --ignored --test-threads=1 => 1 passed`, H389 live regression `cargo test -p selene_engines h389_live_brave_image_provider_approval_maps_real_metadata_without_secret_leak -- --ignored --test-threads=1 => 1 passed`, full adapter/OS/engine test suites, `git diff --check`, and macOS Desktop `xcodebuild => BUILD SUCCEEDED`. Direct runtime proof bound the adapter on `127.0.0.1:18080`, `/healthz` returned HTTP 200, and `/v1/voice/turn` remained fail-closed rather than fabricating display evidence.

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

## ImageDisplayEligibilityPacket Contract

The H390 display eligibility packet must represent:

- `proof_id`
- `selected_outcome`
- `provider`
- `live_or_fixture`
- `image_url_present`
- `thumbnail_url_present`
- `source_page_url_present`
- `source_domain`
- `retrieved_at_present`
- `source_page_verified`
- `source_page_verification_status`
- `source_page_verification_reason`
- `canonical_url` when safely extracted
- `og_image_matches_candidate`
- `twitter_image_matches_candidate`
- `page_title_present`
- `explicit_license_signal_present`
- `license_or_usage_note` when safely extracted
- `robots_noindex_or_noimageindex`
- `display_safe`
- `display_eligible`
- `display_deferred_reason`
- `display_blocked_reason`
- `no_image_bytes_downloaded`
- `no_raw_image_cache`
- `no_source_page_scrape`
- `query_hash_only`
- `text_citation_required`

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
- Source-page URL alone is insufficient for license or display-safety proof.
- Missing `source_page_url`, `source_domain`, `retrieved_at`, `display_safe`, `provider`, or `proof_id` defers display.
- Unknown display safety defers display.
- Unknown rights/license defers display unless an approved repo policy explicitly allows display with an unknown-license note.
- Fixture image data must never be marked live.
- Generated images are allowed only through a separate user-requested image-generation flow.
- Screenshot facts are not evidence.
- Screenshot images are not provider images.
- Images support cited answers visually; they never replace text citations.
- Minimal source-page verification may inspect only bounded public http/https HTML metadata. It must block localhost, loopback, private, link-local, multicast, metadata-service, and file URLs; enforce redirect, timeout, content-type, and response-size safety; avoid JavaScript, cookies, auth headers, secret leakage, paywall bypass, broad scraping, image byte download, and raw image caching.

## Secret And Privacy Policy

- Provider packets may record a secret ID, never the secret value.
- Private image queries are blocked or deferred unless a future approved provider policy explicitly permits them.
- Image metadata cache/retention must default to metadata-only retention and must not create a hidden web image archive.
- H388 adds no new external provider dependency and introduces no new live image-provider call.
- H389 reuses the existing Brave secret ID and bounded HTTP posture.
- H389 stores query hash/redacted query proof only, not raw private queries.
- H389 does not download image bytes and does not scrape source pages.

## Future Implementation Step

Complete an approved license/display-safety policy for Brave image metadata, or select an additional approved image metadata provider if Brave cannot provide explicit license/usage and display-safety fields. Only after that proof may a separate sourced image/photo card UI/display build be authorized.
