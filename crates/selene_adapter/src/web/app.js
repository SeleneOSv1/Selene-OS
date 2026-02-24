"use strict";

const CHECK_ORDER = ["VOICE", "WAKE", "SYNC", "STT", "TTS", "DELIVERY", "BUILDER", "MEMORY"];
const SECTION_TITLE = {
  selene: "Selene Conversation",
  health: "Health Status",
  inbox: "Selene Conversation",
  "work-orders": "Selene Conversation",
  learning: "Selene Conversation",
  governance: "Selene Conversation",
  settings: "Selene Conversation",
};

const state = {
  selectedSection: "health",
  selectedCheckId: "SYNC",
  checks: [],
  detail: null,
  selectedIssueId: null,
  timelineNextCursor: null,
  timelineHasNext: false,
  timelineLoading: false,
  reportContextId: null,
  reportNextCursor: null,
  reportPrevCursor: null,
  lastSeleneWaveTimestamp: 0,
};

let waveTimer = null;
let detailFilterDebounce = null;

function hasElement(id) {
  return Boolean(document.getElementById(id));
}

function formatTimestampNs(ns) {
  if (!Number.isFinite(ns) || ns <= 0) {
    return "-";
  }
  if (ns > 1_000_000_000_000_000) {
    return new Date(ns / 1_000_000).toLocaleString();
  }
  return String(ns);
}

function statusClass(status) {
  if (status === "CRITICAL") return "status-critical";
  if (status === "AT_RISK") return "status-risk";
  return "status-healthy";
}

function setLastRefreshedLabel(label) {
  const element = document.getElementById("last-refreshed");
  if (element) {
    element.textContent = `Last refreshed: ${label}`;
  }
}

function setMainTitle(value) {
  const title = document.getElementById("main-title");
  if (title) {
    title.textContent = value;
  }
}

function renderNavigationAndSections() {
  const navItems = document.querySelectorAll(".nav-item");
  for (const item of navItems) {
    const section = item.getAttribute("data-section");
    item.classList.toggle("nav-item-active", section === state.selectedSection);
  }
  const showHealth = state.selectedSection === "health";
  const seleneSection = document.getElementById("section-selene");
  const healthSection = document.getElementById("section-health");
  if (seleneSection) seleneSection.classList.toggle("hidden", showHealth);
  if (healthSection) healthSection.classList.toggle("hidden", !showHealth);
  setMainTitle(SECTION_TITLE[state.selectedSection] ?? "Selene Conversation");
}

function bindNavigationHandlers() {
  const navItems = document.querySelectorAll(".nav-item");
  for (const item of navItems) {
    item.addEventListener("click", () => {
      const section = item.getAttribute("data-section") ?? "selene";
      state.selectedSection = section;
      renderNavigationAndSections();
    });
  }
}

function renderChecks() {
  const container = document.getElementById("checks-list");
  if (!container) return;
  container.innerHTML = "";

  const sorted = [...state.checks].sort(
    (a, b) => CHECK_ORDER.indexOf(a.check_id) - CHECK_ORDER.indexOf(b.check_id)
  );
  for (const row of sorted) {
    const button = document.createElement("button");
    button.className = "check-row";
    if (row.check_id === state.selectedCheckId) button.classList.add("check-row-active");
    button.addEventListener("click", () => {
      selectCheck(row.check_id);
    });
    button.innerHTML = `
      <div class="check-top">
        <strong>${row.label}</strong>
        <span class="status-pill ${statusClass(row.status)}">${row.status}</span>
      </div>
      <div class="check-meta">
        <span>Open: ${row.open_issue_count}</span>
        <span>Last: ${formatTimestampNs(row.last_event_at_ns)}</span>
      </div>
    `;
    container.appendChild(button);
  }
}

function readDateRangeNs() {
  const fromValue = document.getElementById("date-from")?.value ?? "";
  const toValue = document.getElementById("date-to")?.value ?? "";
  const fromNs = fromValue ? Date.parse(fromValue) * 1_000_000 : null;
  const toNs = toValue ? Date.parse(toValue) * 1_000_000 : null;
  return { fromNs, toNs };
}

function timelineAutoPageSize() {
  return window.matchMedia("(max-width: 900px)").matches ? 8 : 14;
}

function buildDetailQuery(timelineCursor) {
  const { fromNs, toNs } = readDateRangeNs();
  const params = new URLSearchParams();
  const issueQuery = (document.getElementById("issue-search")?.value ?? "").trim();
  const engineOwner = (document.getElementById("issue-engine")?.value ?? "").trim();
  const openOnly = Boolean(document.getElementById("filter-open-only")?.checked);
  const criticalOnly = Boolean(document.getElementById("filter-critical-only")?.checked);
  const escalatedOnly = Boolean(document.getElementById("filter-escalated-only")?.checked);
  if (issueQuery) params.set("issue_query", issueQuery);
  if (engineOwner) params.set("engine_owner", engineOwner);
  if (openOnly) params.set("open_only", "true");
  if (criticalOnly) params.set("critical_only", "true");
  if (escalatedOnly) params.set("escalated_only", "true");
  if (Number.isFinite(fromNs) && fromNs > 0) params.set("from_utc_ns", String(fromNs));
  if (Number.isFinite(toNs) && toNs > 0) params.set("to_utc_ns", String(toNs));
  if (state.selectedIssueId) params.set("selected_issue_id", state.selectedIssueId);
  if (timelineCursor) params.set("timeline_cursor", timelineCursor);
  params.set("timeline_page_size", String(timelineAutoPageSize()));
  return params.toString();
}

function scheduleDetailRefresh() {
  if (detailFilterDebounce) clearTimeout(detailFilterDebounce);
  detailFilterDebounce = setTimeout(() => {
    selectCheck(state.selectedCheckId || "SYNC").catch((error) => {
      console.error(error);
    });
  }, 180);
}

function renderSummary(summary) {
  if (!hasElement("summary-open")) return;
  document.getElementById("summary-open").textContent = String(summary.open_issues ?? 0);
  document.getElementById("summary-critical").textContent = String(summary.critical_open_count ?? 0);
  document.getElementById("summary-auto-resolved").textContent = String(summary.auto_resolved_24h_count ?? 0);
  document.getElementById("summary-escalated").textContent = String(summary.escalated_24h_count ?? 0);
  document.getElementById("summary-mttr").textContent =
    Number.isFinite(summary.mttr_ms) && summary.mttr_ms > 0 ? `${summary.mttr_ms}ms` : "-";
}

function renderIssues() {
  const body = document.getElementById("issues-body");
  const empty = document.getElementById("issues-empty");
  if (!body || !state.detail) return;

  const visibleIssues = state.detail.issues ?? [];
  body.innerHTML = "";

  if (visibleIssues.length === 0) {
    empty.classList.remove("hidden");
    return;
  }
  empty.classList.add("hidden");

  for (const issue of visibleIssues) {
    const row = document.createElement("tr");
    row.className = "issue-row";
    if (issue.issue_id === state.selectedIssueId) {
      row.classList.add("issue-row-active");
    }
    row.addEventListener("click", async () => {
      state.selectedIssueId = issue.issue_id;
      await selectCheck(state.selectedCheckId, { timelineCursor: null });
    });
    row.innerHTML = `
      <td>${issue.severity}</td>
      <td>${issue.issue_type}</td>
      <td>${issue.engine_owner}</td>
      <td>${formatTimestampNs(issue.first_seen_at_ns)}</td>
      <td>${formatTimestampNs(issue.last_update_at_ns)}</td>
      <td>${issue.status}</td>
      <td>${issue.resolution_state}</td>
    `;
    body.appendChild(row);
  }
}

function renderTimeline() {
  const list = document.getElementById("timeline-list");
  const header = document.getElementById("detail-header");
  const empty = document.getElementById("timeline-empty");
  const meta = document.getElementById("timeline-meta");
  const loadMoreButton = document.getElementById("timeline-load-more");
  if (!list || !header || !empty || !meta || !loadMoreButton) return;

  list.innerHTML = "";
  const detail = state.detail;
  if (!detail) {
    empty.classList.remove("hidden");
    meta.textContent = "Showing 0 of 0 events";
    loadMoreButton.classList.add("hidden");
    return;
  }

  const issue =
    detail.issues.find((entry) => entry.issue_id === state.selectedIssueId) ??
    detail.issues.find((entry) => entry.issue_id === detail.active_issue_id) ??
    detail.issues[0] ??
    null;
  if (!issue) {
    header.textContent = "Select an issue from the center table.";
    empty.classList.remove("hidden");
    meta.textContent = "Showing 0 of 0 events";
    loadMoreButton.classList.add("hidden");
    return;
  }

  state.selectedIssueId = issue.issue_id;
  header.textContent = `${issue.issue_id} - ${issue.issue_type} (${issue.severity})`;
  const entries = detail.timeline ?? [];
  if (entries.length === 0) {
    empty.classList.remove("hidden");
    meta.textContent = `Showing 0 of ${detail.timeline_paging?.total_entries ?? 0} events`;
    loadMoreButton.classList.toggle("hidden", !state.timelineHasNext);
    return;
  }
  empty.classList.add("hidden");

  for (const entry of entries) {
    const node = document.createElement("li");
    node.className = "timeline-item";
    node.innerHTML = `
      <div class="timeline-top">
        <strong>${entry.action_id}</strong>
        <span>${formatTimestampNs(entry.at_ns)}</span>
      </div>
      <div class="timeline-result">Result: ${entry.result}</div>
      <div class="timeline-result">Reason: ${entry.reason_code}</div>
    `;
    list.appendChild(node);
  }

  const total = detail.timeline_paging?.total_entries ?? entries.length;
  meta.textContent = `Showing ${entries.length} of ${total} events`;
  loadMoreButton.classList.toggle("hidden", !state.timelineHasNext);
}

function renderDetail() {
  if (!state.detail) return;
  renderSummary(state.detail.summary);
  renderIssues();
  renderTimeline();
}

function startVoiceWave(durationMs) {
  const wave = document.getElementById("voice-wave");
  if (!wave) return;
  const bars = wave.querySelectorAll(".wave-bar");
  wave.classList.add("wave-active");
  if (waveTimer) clearInterval(waveTimer);
  const stopAt = Date.now() + durationMs;
  waveTimer = setInterval(() => {
    const now = Date.now();
    for (const bar of bars) {
      const height = 6 + Math.floor(Math.random() * 22);
      bar.style.height = `${height}px`;
    }
    if (now >= stopAt) {
      clearInterval(waveTimer);
      waveTimer = null;
      wave.classList.remove("wave-active");
      for (const bar of bars) {
        bar.style.height = "6px";
      }
    }
  }, 85);
}

function renderTranscript(response) {
  const list = document.getElementById("transcript-list");
  const empty = document.getElementById("transcript-empty");
  if (!list || !empty) return;
  list.innerHTML = "";
  if (!response || !Array.isArray(response.messages) || response.messages.length === 0) {
    empty.classList.remove("hidden");
    return;
  }
  empty.classList.add("hidden");

  for (const message of response.messages) {
    const row = document.createElement("li");
    row.className = "transcript-row";
    const finality = message.finalized ? "FINAL" : "PARTIAL";
    row.innerHTML = `
      <div class="transcript-role">${message.role} · ${message.source} · ${finality} · ${formatTimestampNs(message.timestamp_ns)}</div>
      <div>${message.text}</div>
    `;
    list.appendChild(row);

    if (
      message.role === "SELENE" &&
      message.finalized &&
      Number.isFinite(message.timestamp_ns) &&
      message.timestamp_ns > state.lastSeleneWaveTimestamp
    ) {
      state.lastSeleneWaveTimestamp = message.timestamp_ns;
      startVoiceWave(1500);
    }
  }
}

async function loadChecks() {
  const response = await fetch("/v1/ui/health/checks", { cache: "no-store" });
  if (!response.ok) throw new Error(`failed to load checks: ${response.status}`);
  const payload = await response.json();
  state.checks = payload.checks ?? [];
  renderChecks();
  setLastRefreshedLabel(formatTimestampNs(payload.generated_at_ns));
}

async function selectCheck(checkId, options = {}) {
  const { timelineCursor = null } = options;
  state.selectedCheckId = checkId;
  renderChecks();
  const query = buildDetailQuery(timelineCursor);
  const response = await fetch(
    `/v1/ui/health/detail/${encodeURIComponent(checkId)}${query ? `?${query}` : ""}`,
    { cache: "no-store" }
  );
  if (!response.ok) throw new Error(`failed to load detail: ${response.status}`);
  const payload = await response.json();
  if (timelineCursor && state.detail && state.detail.selected_check_id === payload.selected_check_id) {
    const seen = new Set();
    const merged = [];
    for (const entry of [...(state.detail.timeline ?? []), ...(payload.timeline ?? [])]) {
      const key = `${entry.issue_id}|${entry.at_ns ?? 0}|${entry.action_id}|${entry.reason_code}`;
      if (seen.has(key)) continue;
      seen.add(key);
      merged.push(entry);
    }
    payload.timeline = merged;
    payload.timeline_paging = {
      ...(payload.timeline_paging ?? {}),
      total_entries: payload.timeline_paging?.total_entries ?? merged.length,
      visible_entries: merged.length,
    };
  }
  state.detail = payload;
  state.selectedIssueId = payload.active_issue_id ?? payload.issues?.[0]?.issue_id ?? null;
  state.timelineNextCursor = payload.timeline_paging?.next_cursor ?? null;
  state.timelineHasNext = Boolean(payload.timeline_paging?.has_next);
  renderDetail();
}

async function loadTranscript() {
  const response = await fetch("/v1/ui/chat/transcript", { cache: "no-store" });
  if (!response.ok) throw new Error(`failed to load transcript: ${response.status}`);
  const payload = await response.json();
  renderTranscript(payload);
}

function reportRequest(pageAction, pageCursor) {
  const fromRaw = document.getElementById("report-from")?.value ?? "";
  const toRaw = document.getElementById("report-to")?.value ?? "";
  const owner = (document.getElementById("report-owner")?.value ?? "").trim();
  const companyIds = (document.getElementById("report-company-ids")?.value ?? "")
    .split(",")
    .map((v) => v.trim())
    .filter(Boolean);
  const countryCodes = (document.getElementById("report-country-codes")?.value ?? "")
    .split(",")
    .map((v) => v.trim())
    .filter(Boolean);
  const displayTarget = (document.getElementById("report-display-target")?.value ?? "").trim();
  const pageSize = Number.parseInt(document.getElementById("report-page-size")?.value ?? "20", 10);

  return {
    tenant_id: "tenant_a",
    viewer_user_id: "viewer_01",
    report_kind: (document.getElementById("report-kind")?.value ?? "UNRESOLVED_ESCALATED").trim(),
    from_utc_ns: fromRaw ? Date.parse(fromRaw) * 1_000_000 : undefined,
    to_utc_ns: toRaw ? Date.parse(toRaw) * 1_000_000 : undefined,
    engine_owner_filter: owner || undefined,
    company_scope: companyIds.length > 0 ? "CROSS_TENANT_TENANT_ROWS" : "TENANT_ONLY",
    company_ids: companyIds,
    country_codes: countryCodes,
    escalated_only: Boolean(document.getElementById("report-escalated-only")?.checked),
    unresolved_only: Boolean(document.getElementById("report-unresolved-only")?.checked),
    display_target: displayTarget || undefined,
    page_action: pageAction,
    page_cursor: pageCursor || undefined,
    report_context_id: state.reportContextId || undefined,
    page_size: Number.isFinite(pageSize) ? pageSize : 20,
  };
}

function renderReport(response) {
  const rowsNode = document.getElementById("report-rows");
  const empty = document.getElementById("report-empty");
  const status = document.getElementById("report-status");
  const context = document.getElementById("report-context");
  const normalized = document.getElementById("report-normalized");
  const prev = document.getElementById("report-prev");
  const next = document.getElementById("report-next");
  if (!rowsNode || !empty || !status || !context || !normalized || !prev || !next) return;

  status.textContent = `Status: ${response.status} (${response.reason_code ?? "-"})`;
  context.textContent = `Context: ${response.report_context_id ?? "-"}`;
  normalized.textContent = `Query: ${response.normalized_query ?? "-"}`;

  if (response.requires_clarification) {
    status.textContent = `Status: clarify needed (${response.requires_clarification})`;
  }

  rowsNode.innerHTML = "";
  if (!Array.isArray(response.rows) || response.rows.length === 0) {
    empty.classList.remove("hidden");
  } else {
    empty.classList.add("hidden");
    for (const row of response.rows) {
      const tr = document.createElement("tr");
      tr.innerHTML = `
        <td>${row.tenant_id ?? "-"}</td>
        <td>${row.issue_id ?? "-"}</td>
        <td>${row.owner_engine_id ?? "-"}</td>
        <td>${row.severity ?? "-"}</td>
        <td>${row.status ?? "-"}</td>
        <td>${row.latest_reason_code ?? "-"}</td>
        <td>${row.bcast_id ?? "-"}</td>
        <td>${row.ack_state ?? "-"}</td>
        <td>${row.impact_summary ?? row.unresolved_reason_exact ?? "-"}</td>
      `;
      rowsNode.appendChild(tr);
    }
  }

  state.reportContextId = response.report_context_id ?? null;
  state.reportNextCursor = response.paging?.next_cursor ?? null;
  state.reportPrevCursor = response.paging?.prev_cursor ?? null;
  prev.disabled = !Boolean(response.paging?.has_prev);
  next.disabled = !Boolean(response.paging?.has_next);

  if (response.display_target_applied && hasElement("report-display-target")) {
    document.getElementById("report-display-target").value = response.display_target_applied;
  } else if (response.remembered_display_target && hasElement("report-display-target")) {
    document.getElementById("report-display-target").value = response.remembered_display_target;
  }
}

async function queryReport(pageAction, pageCursor) {
  const status = document.getElementById("report-status");
  if (status) status.textContent = "Status: loading...";
  const requestBody = reportRequest(pageAction, pageCursor);
  const response = await fetch("/v1/ui/health/report/query", {
    method: "POST",
    cache: "no-store",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(requestBody),
  });
  const payload = await response.json();
  renderReport(payload);
}

function bindFilterHandlers() {
  const issueSearch = document.getElementById("issue-search");
  const issueEngine = document.getElementById("issue-engine");
  const dateFrom = document.getElementById("date-from");
  const dateTo = document.getElementById("date-to");
  const openOnly = document.getElementById("filter-open-only");
  const criticalOnly = document.getElementById("filter-critical-only");
  const escalatedOnly = document.getElementById("filter-escalated-only");
  const timelineLoadMore = document.getElementById("timeline-load-more");
  const timelineList = document.getElementById("timeline-list");
  const refreshBtn = document.getElementById("refresh-btn");
  const presetButtons = document.querySelectorAll("[data-preset]");

  if (issueSearch) issueSearch.addEventListener("input", scheduleDetailRefresh);
  if (issueEngine) issueEngine.addEventListener("input", scheduleDetailRefresh);
  if (dateFrom) dateFrom.addEventListener("change", scheduleDetailRefresh);
  if (dateTo) dateTo.addEventListener("change", scheduleDetailRefresh);
  if (openOnly) openOnly.addEventListener("change", scheduleDetailRefresh);
  if (criticalOnly) criticalOnly.addEventListener("change", scheduleDetailRefresh);
  if (escalatedOnly) escalatedOnly.addEventListener("change", scheduleDetailRefresh);
  if (refreshBtn) refreshBtn.addEventListener("click", refreshAll);

  for (const button of presetButtons) {
    button.addEventListener("click", () => {
      const now = new Date();
      const from = new Date(now);
      const preset = button.getAttribute("data-preset");
      if (preset === "24h") from.setHours(now.getHours() - 24);
      if (preset === "7d") from.setDate(now.getDate() - 7);
      if (preset === "30d") from.setDate(now.getDate() - 30);
      if (dateFrom) dateFrom.value = from.toISOString().slice(0, 16);
      if (dateTo) dateTo.value = now.toISOString().slice(0, 16);
      scheduleDetailRefresh();
    });
  }

  if (timelineLoadMore) {
    timelineLoadMore.addEventListener("click", async () => {
      if (!state.timelineHasNext || !state.timelineNextCursor || state.timelineLoading) return;
      state.timelineLoading = true;
      try {
        await selectCheck(state.selectedCheckId, { timelineCursor: state.timelineNextCursor });
      } finally {
        state.timelineLoading = false;
      }
    });
  }

  if (timelineList) {
    timelineList.addEventListener("scroll", async () => {
      if (!state.timelineHasNext || !state.timelineNextCursor || state.timelineLoading) return;
      const threshold = 28;
      const nearBottom =
        timelineList.scrollTop + timelineList.clientHeight >= timelineList.scrollHeight - threshold;
      if (!nearBottom) return;
      state.timelineLoading = true;
      try {
        await selectCheck(state.selectedCheckId, { timelineCursor: state.timelineNextCursor });
      } finally {
        state.timelineLoading = false;
      }
    });
  }

  const reportRun = document.getElementById("report-run-btn");
  const reportPrev = document.getElementById("report-prev");
  const reportNext = document.getElementById("report-next");
  if (reportRun) {
    reportRun.addEventListener("click", async () => {
      await queryReport("FIRST", null);
    });
  }
  if (reportPrev) {
    reportPrev.addEventListener("click", async () => {
      await queryReport("PREV", state.reportPrevCursor);
    });
  }
  if (reportNext) {
    reportNext.addEventListener("click", async () => {
      await queryReport("NEXT", state.reportNextCursor);
    });
  }
}

async function refreshAll() {
  try {
    if (hasElement("checks-list")) {
      await loadChecks();
      await selectCheck(state.selectedCheckId || "SYNC");
    }
    if (hasElement("transcript-list")) {
      await loadTranscript();
    }
    setLastRefreshedLabel(new Date().toLocaleString());
  } catch (error) {
    console.error(error);
    setLastRefreshedLabel(`error (${error.message})`);
  }
}

document.addEventListener("DOMContentLoaded", async () => {
  if (document.querySelector(".nav-item")) {
    bindNavigationHandlers();
    renderNavigationAndSections();
  }
  if (hasElement("issue-search")) {
    bindFilterHandlers();
  }
  await refreshAll();
  if (hasElement("report-rows")) {
    await queryReport("FIRST", null);
  }
  setInterval(refreshAll, 15_000);
});
