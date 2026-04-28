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

## H391 Decision

H391 selects `Outcome B -- BRAVE_IMAGE_DISPLAY_POLICY_METADATA_AND_SOURCE_LINK_ONLY`.

Brave remains the currently approved provider implementation, but H391 keeps the architecture provider-agnostic. Runtime packets and adapter metadata use generic image provider path, display eligibility, and provider display policy fields. Brave-specific logic is limited to the Brave metadata request/parser and the H391 Brave policy mapper/tests; a future provider should be added by implementing a provider adapter, mapping its metadata into the same generic packets, and adding provider-specific policy tests without changing Desktop UI or the core response shape.

H391 does not implement H392. It does not display image/photo cards, render thumbnails, render source-link visual cards, modify Desktop Swift/UI/chrome, download image bytes, create a raw image cache, or emit `WEB_IMAGE_SOURCE_CARD_PASS`.

Official Brave policy/docs reviewed on `2026-04-28`:

| URL | Page title | Policy fields found | Policy fields not found |
| --- | --- | --- | --- |
| `https://api-dashboard.search.brave.com/documentation/services/image-search` | `Image search` | Image Search endpoint posture, safe search, thumbnail/proxy behavior, source page URL, image URL/thumbnail metadata, publisher metadata, warning to be aware of copyright/licensing | No explicit publisher-rights grant, no explicit thumbnail display permission, no explicit full-image display permission |
| `https://api-dashboard.search.brave.com/api-reference/images/image_search` | `Images` | `GET /v1/images/search`, subscription-token auth, query/count/safesearch parameters, result list schema surface, default JSON response, cached content header posture | No explicit thumbnail display rights, no explicit full-image display rights, no publisher-rights transfer |
| `https://api-dashboard.search.brave.com/documentation/resources/terms-of-service` | `Terms of service` | Search Results may include third-party content, customer may use Search Results with applications within the agreement, attribution language for provider marks, transient-only storage restriction, third-party IP responsibility | No explicit permission that Brave-returned image thumbnails or full image URLs may be displayed as sourced image/photo cards without publisher/license proof |

H391 policy result:

- Metadata-only use is allowed for bounded research metadata.
- Source-page link use is allowed when source-page URL/domain are present and text citations remain separate.
- Thumbnail display is deferred/blocked because official policy proof does not explicitly prove thumbnail display rights.
- Full image display is blocked because official policy proof does not explicitly prove full-image display rights.
- Sourced image/photo cards remain deferred.
- Provider attribution is recorded as required before any future visual display.
- Publisher rights remain required and unverified.
- Unknown license keeps visual display deferred or blocked.
- Storage remains transient metadata-only; raw image bytes and raw image caches remain blocked.
- H392 handoff: design either a source-link-only visual citation card or perform separate legal/provider-rights review before any thumbnail/image UI.

## H392 Decision

H392 implements the H391-approved source-link-only visual citation-card handoff.

Selene now exposes provider-agnostic `deep_research.source_link_citation_cards` metadata for source links that pass H391 policy, public source-page URL gating, and text-only display rules. The packet is carried through existing Deep Research citation metadata so PH1.E field-count and field-size contracts remain intact, and the Desktop bridge/shell render only native text source chips/cards in the existing authoritative provenance surface.

H392 does not display thumbnails, full images, image/photo cards, image strips, OpenGraph/Twitter previews, page previews, WebViews, or provider media. Source-link cards use only `source_page_url`; `image_url` and `thumbnail_url` are never card targets and never displayed as media. No image bytes are downloaded, no raw image cache is created, no source-page media is fetched, and `WEB_IMAGE_SOURCE_CARD_PASS` is not emitted.

The attached screenshot was used only as a layout reference for spacing, hierarchy, and grey source-chip feel. Screenshot content is not evidence, screenshot images are not provider images, and screenshot facts are not reused without separate EvidencePacket/CitationPacket backing.

H392 proof passed with targeted `h392` adapter/engine tests, all required H379-H392 regressions, full adapter/OS/engine test suites, live H389/H390/H391 Brave regression tests, `git diff --check`, and macOS Desktop `xcodebuild => BUILD SUCCEEDED`. Direct runtime proof bound the adapter on `127.0.0.1:18080`, `/healthz` returned HTTP 200, and `/v1/voice/turn` failed closed on governance sync rather than fabricating image/card evidence.

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

## ImageProviderDisplayPolicy Contract

The H391 provider display policy packet must remain provider-agnostic and represent:

- `provider`
- `policy_version`
- `policy_outcome`
- `metadata_use_allowed`
- `source_link_use_allowed`
- `thumbnail_display_allowed`
- `full_image_display_allowed`
- `sourced_image_card_allowed`
- `ui_display_implemented`
- `attribution_required`
- `attribution_text_or_code`
- `provider_terms_reviewed`
- `official_docs_reviewed`
- `official_docs_unavailable`
- `thumbnail_display_rights_explicit`
- `full_image_display_rights_explicit`
- `attribution_requirements_explicit`
- `storage_cache_limits_explicit`
- `publisher_rights_required`
- `publisher_rights_verified`
- `license_required_for_display`
- `license_unknown_behavior`
- `storage_allowed`
- `transient_storage_only`
- `raw_image_cache_allowed`
- `image_bytes_download_allowed`
- `text_citation_still_required`
- `display_deferred_reason`
- `display_blocked_reason`
- `proof_id`
- `h392_handoff_recommendation`

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
- H391 records official policy proof but does not infer rights from API availability, provider presence, image URL, thumbnail URL, source-page URL, `og:image`, or `twitter:image`.
- H391 keeps generic packets provider-neutral; Brave may be replaced by adding a provider adapter, provider policy mapper, and tests.

## H392 Source-Link-Only Citation Cards

H392 implements text-only, source-page-link-only visual citation cards for source links allowed by the H391 provider display policy.

- Card metadata is provider-agnostic and exposed as `deep_research.source_link_citation_cards`.
- Runtime packet proof is carried inside the existing citation-card packet to preserve the PH1.E `DeepResearch` extracted-field contract limit.
- Desktop rendering is native Swift text/card/chip UI only.
- Link opening is user-click-only through a safe public `http` / `https` source-page URL.
- Source cards show source title, source domain, provider, and required provider attribution when present.
- Source cards do not replace text citations.
- TTS does not read source-card URLs or metadata.
- Screenshot layout is used only as visual layout reference; screenshot facts are not evidence and screenshot images are not provider images.
- No image strip, thumbnail rendering, full image rendering, image/photo card rendering, WebView, OpenGraph preview, Twitter card preview, page preview, source-page media fetch, image byte download, or raw image cache is introduced.
- `image_url` and `thumbnail_url` are never used as card targets and are never displayed as media.
- Brave remains isolated as provider-specific policy/proof logic; the card packet and Desktop rendering are provider-agnostic.

H392 result posture:

- `H392_SOURCE_LINK_ONLY_CITATION_CARD_PASS`
- `WEB_IMAGE_SOURCE_LINK_ONLY_CARD_PASS`
- `WEB_IMAGE_SOURCE_LINK_CARD_TEXT_ONLY_PASS`
- `WEB_IMAGE_SOURCE_LINK_CARD_NO_THUMBNAIL_PASS`
- `WEB_IMAGE_SOURCE_LINK_CARD_NO_FULL_IMAGE_PASS`
- `WEB_IMAGE_SOURCE_LINK_CARD_NO_IMAGE_PREVIEW_PASS`
- `WEB_IMAGE_SOURCE_LINK_CARD_NO_WEBVIEW_PASS`
- `WEB_IMAGE_SOURCE_LINK_CARD_SAFE_URL_PASS`
- `WEB_IMAGE_SOURCE_LINK_CARD_USER_CLICK_ONLY_PASS`
- `WEB_IMAGE_SOURCE_LINK_CARD_PROVIDER_AGNOSTIC_PASS`
- `WEB_IMAGE_SOURCE_LINK_CARD_BRAVE_ISOLATED_PASS`
- `WEB_IMAGE_SOURCE_LINK_CARD_ATTRIBUTION_PASS`
- `WEB_IMAGE_SOURCE_LINK_CARD_TTS_EXCLUDED_PASS`
- `WEB_IMAGE_SOURCE_LINK_CARD_SCREENSHOT_LAYOUT_REFERENCE_PASS`
- `SCREENSHOT_NOT_USED_AS_EVIDENCE_PASS`
- `SCREENSHOT_IMAGES_NOT_PROVIDER_IMAGES_PASS`
- `WEB_IMAGE_SOURCE_CARD_DISPLAY_DEFERRED`
- `WEB_IMAGE_SOURCE_CARD_PASS_BLOCKED`

## Future Implementation Step

Complete an approved license/display-safety policy for Brave image metadata, or select an additional approved image metadata provider if Brave cannot provide explicit license/usage and display-safety fields. Only after that proof may a separate sourced image/photo card UI/display build be authorized.
