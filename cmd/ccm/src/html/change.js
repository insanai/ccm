// Planning views: the development estimate, the change closure and acceptance plan, the
// comparison with a baseline, coverage notices, and the technical appendix.

const COMPONENT_COLOR = {
  "agent": "var(--accent)", "ai-implementation": "var(--accent)", "ai-checks": "var(--deep)",
  "tools": "var(--lilac)", "review": "var(--pink)", "human-checks": "var(--orange)",
  "human-implementation": "var(--muted)",
};

function renderEstimate(model) {
  const d = model.data;
  const ai = d.ai || {};
  const p = ai.assumptions || {};
  const e = d.economic || {};
  const a = e.assumptions || {};
  const section = h("section", { id: "estimate" });
  section.appendChild(sectionHead("Development estimate",
    "A planning scenario built from counted program lines and stated assumptions. "
    + "It is not measured spending, asset value, or a quote.",
    tag("Planning estimate", "warn")));
  const visible = d.budget.components.filter(([key, , value]) =>
    value > 0 || key === "ai-implementation" || key === "agent");
  const attempts = e.expected_attempts;
  const total = d.budget.total;
  const lines = ai.program_lines;
  section.appendChild(h("div", { class: "metrics" },
    metric("Total scenario", money(total), "USD, " + fixed(attempts, 2) + " expected attempts",
      "primary"),
    metric("Priced program lines", num(lines), ai.scope || ""),
    metric("Per program line", lines ? money(total / lines) : "-", "total / priced lines"),
    metric("AI per attempt", money(ai.implementation_usd_per_attempt + ai.checks_usd_per_attempt
      + ai.tools_usd_per_attempt), "implementation, checks, tools"),
    metric("Human hours per attempt", fixed(e.hours_per_attempt, 2),
      "at " + money(a.hourly_cost) + "/hour", "secondary")));
  if (d.budget.partial_scope) {
    section.appendChild(h("p", { class: "warning", text: "Partial scope: some implementation work "
      + "is not mapped to counted source files and is unpriced." }));
  }
  section.appendChild(h("h3", { text: "Where the total comes from" }));
  section.appendChild(stack(visible.map(([key, name, value]) =>
    [name, value, COMPONENT_COLOR[key] || "var(--accent)", money(value)]), { tall: true }));
  section.appendChild(componentTable(visible, total, ai, p, e, a));
  section.appendChild(sensitivity(ai, p, attempts, total));
  section.appendChild(languageWorkload(ai, p));
  section.appendChild(drivers(ai, p, e, a));
  return section;
}

function componentTable(rows, total, ai, p, e, a) {
  const units = e.pending_policy_units;
  const times = e.expected_attempts !== 1 ? " x " + fixed(e.expected_attempts, 3) + " attempts"
    : "";
  const formula = {
    "ai-implementation": num(Math.round(ai.input_tokens_inferred)) + " input tokens x $"
      + fixed(p.input_usd_per_million, 2) + "/M + " + num(Math.round(ai.output_tokens_inferred))
      + " output tokens x $" + fixed(p.output_usd_per_million, 2) + "/M" + times,
    "ai-checks": num(units) + " pending units x " + num(p.check_tokens_per_unit)
      + " tokens x " + fixed(p.workload_multiplier, 2) + " workload = "
      + num(Math.round(ai.check_tokens_inferred)) + " tokens, blended price" + times,
    "tools": num(ai.sessions_assumed) + " sessions x $" + fixed(p.usd_per_session, 2)
      + " (one per " + num(p.lines_per_session) + " lines)" + times,
    "review": num(e.review_artifacts) + " artifacts x " + fixed(a.review_hours_per_artifact, 2)
      + " h x " + money(a.hourly_cost) + "/h" + times,
    "human-checks": (units === null ? "unknown" : num(units)) + " pending units x "
      + fixed(a.hours_per_check_unit, 2) + " h x " + money(a.hourly_cost) + "/h" + times,
    "human-implementation": fixed(a.implementation_hours, 2) + " h x " + money(a.hourly_cost)
      + "/h" + times,
    "agent": money(a.agent_cost_per_attempt) + " manual AI and tool cost per attempt" + times,
  };
  const table = h("table", { "aria-label": "Budget components" },
    h("thead", null, h("tr", null, h("th", { class: "l", text: "Component" }),
      h("th", { text: "USD" }), h("th", { text: "Share" }),
      h("th", { class: "l hide-narrow", text: "How it is computed" }))));
  const body = h("tbody");
  for (const [key, name, value] of rows) {
    body.appendChild(h("tr", null,
      h("td", { class: "l" }, h("span", { class: "swatch",
        style: "background:" + (COMPONENT_COLOR[key] || "var(--accent)") }), " ", name),
      h("td", { text: money(value) }), h("td", { text: percent(value, total) }),
      h("td", { class: "formula hide-narrow", text: formula[key] || "" })));
  }
  body.appendChild(h("tr", { class: "total" }, h("td", { class: "l", text: "Total" }),
    h("td", { text: money(total) }), h("td", { text: "100%" }), h("td",
      { class: "hide-narrow" })));
  table.appendChild(body);
  return h("div", { class: "scroll" }, table);
}

function sensitivity(ai, p, attempts, total) {
  const band = p.implementation_sensitivity_usd;
  const wrap = h("div", null, h("h3", { text: "Calibration sensitivity" }));
  if (!Array.isArray(band)) {
    wrap.appendChild(h("p", { class: "note", text: "No sensitivity band: a custom cost profile "
      + "replaces the default calibration." }));
    return wrap;
  }
  const [low, high] = band;
  const point = ai.implementation_usd_per_attempt;
  const top = Math.max(high, point) || 1;
  const at = (value) => (100 * value) / top + "%";
  wrap.append(
    h("p", { class: "note", text: "AI implementation per attempt under the low and high "
      + "published per-line costs of the calibration study." }),
    h("div", { class: "range", role: "img", "aria-label": "Range " + money(low) + " to "
      + money(high) + ", chosen " + money(point) },
      h("span", { class: "track" }),
      h("span", { class: "band", style: "left:" + at(low) + ";width:calc(" + at(high) + " - "
        + at(low) + ")" }),
      h("span", { class: "point", style: "left:" + at(point) })),
    keyValues([
      ["Low", money(low)], ["Chosen midpoint", money(point)], ["High", money(high)],
      ["Total at low", money(total + (low - point) * attempts)],
      ["Total at high", money(total + (high - point) * attempts)],
    ]));
  return wrap;
}

function languageWorkload(ai, p) {
  const rows = (ai.language_workload || []).map((r) => Object.assign({}, r, {
    usd: (r.lines * p.input_per_line * p.input_usd_per_million + r.lines * r.output_factor
      * p.output_per_line * p.output_usd_per_million) * p.workload_multiplier / 1e6,
  }));
  const wrap = h("div", null, h("h3", { text: "Priced lines by language" }));
  const sum = rows.reduce((acc, r) => acc + r.usd, 0);
  wrap.appendChild(dataTable({ label: "Priced lines by language", rows, sort: "usd",
    empty: "No program lines are priced.",
    columns: [
      { key: "language", label: "Language", value: (r) => r.language },
      { key: "lines", label: "Program lines", num: true, value: (r) => r.lines },
      { key: "factor", label: "Output factor", num: true, value: (r) => r.output_factor,
        cell: (r) => "x" + fixed(r.output_factor, 2) },
      { key: "usd", label: "Implementation", num: true, value: (r) => r.usd,
        cell: (r) => money(r.usd) },
      { key: "share", label: "Share", bar: true, cell: (r) => bar(r.usd, sum) },
    ] }).el);
  return wrap;
}

function drivers(ai, p, e, a) {
  const grid = h("div", { class: "grid2" },
    h("div", null, h("h4", { text: "AI price card and workload" }), keyValues([
      ["Price card", p.model], ["Profile", ai.profile + " (" + ai.year + ")"],
      ["Price date", ai.price_date], ["Input price", "$" + fixed(p.input_usd_per_million, 2)
        + "/M"],
      ["Output price", "$" + fixed(p.output_usd_per_million, 2) + "/M"],
      ["Input tokens per line", fixed(p.input_per_line, 2)],
      ["Output tokens per line", fixed(p.output_per_line, 2)],
      ["Workload multiplier", "x" + fixed(p.workload_multiplier, 2)],
      ["Workload driver", p.workload_driver], ["Check tokens per unit",
        num(p.check_tokens_per_unit)],
      ["Session price", money(p.usd_per_session)], ["Lines per session", num(p.lines_per_session)],
      ["Manual override", yesNo(ai.manual_override)], ["Status", ai.status],
    ])),
    h("div", null, h("h4", { text: "Human work and repeats" }), keyValues([
      ["Hourly cost", money(a.hourly_cost)], ["Review artifacts", num(e.review_artifacts)],
      ["Review hours per artifact", fixed(a.review_hours_per_artifact, 2)],
      ["Pending policy units", e.pending_policy_units === null ? "unknown (no model)"
        : num(e.pending_policy_units)],
      ["Hours per check unit", fixed(a.hours_per_check_unit, 2)],
      ["Implementation hours", fixed(a.implementation_hours, 2)],
      ["Agent cost per attempt", money(a.agent_cost_per_attempt)],
      ["Rework probability", fixed(a.rework_probability, 3)],
      ["Expected attempts", fixed(e.expected_attempts, 3) + " = 1 / (1 - p)"],
      ["Hours per attempt", fixed(e.hours_per_attempt, 2)],
      ["Human and override cost", money(e.human_and_override_expected_cost)],
      ["Acceptance scope known", yesNo(p.acceptance_scope_known)],
      ["Scenario", e.model + ", " + e.status],
    ])));
  const sources = h("ul", { class: "plain" });
  for (const url of ai.sources || []) sources.appendChild(h("li", { class: "mono", text: url }));
  return block("Drivers, assumptions and sources", [grid, h("h4", { text: "Research sources" }),
    sources, h("p", { class: "note", text: "Default token conversion and language factors are "
      + "inferred from published studies, not measured for this project." })]);
}

// ---------------------------------------------------------------------------------------
// The change closure and the acceptance plan.

function changeIndex(m) {
  const affected = new Map();
  m.affected.forEach(([id, previous]) => affected.set(id, previous));
  const depth = new Map();
  const level = (id) => {
    if (depth.has(id)) return depth.get(id);
    depth.set(id, 0);
    const previous = affected.get(id);
    const value = previous === null || previous === undefined ? 0 : level(previous) + 1;
    depth.set(id, value);
    return value;
  };
  for (const id of affected.keys()) level(id);
  return { affected, depth, seeds: new Set(m.seeds) };
}

function checkStatus(check) {
  const [, , active, pending, failed] = check;
  if (!active) return ["Not triggered", "muted"];
  if (failed) return ["Failed", "bad"];
  if (pending) return ["Needs evidence", "warn"];
  return ["Satisfied", "ok"];
}

function renderChange(model, view) {
  const m = model.data.model;
  const index = changeIndex(m);
  const section = h("section", { id: "change" });
  const active = m.checks.filter((c) => c[2]);
  const pending = m.checks.filter((c) => c[2] && c[3]);
  const invalid = m.evidence.filter((e) => e[3] === "invalidated");
  section.appendChild(sectionHead("Change and acceptance", "Exact within the author-declared "
    + "model: the changed artifacts, everything that depends on them, the checks they "
    + "trigger and the evidence that still holds.", tag(pending.length ? "Pending checks"
    : "No pending checks", pending.length ? "warn" : "ok")));
  section.appendChild(h("p", { class: "callout info" }, h("strong", { text: "Context " }),
    shown(m.context)));
  section.appendChild(h("div", { class: "metrics" },
    metric("Seeds", num(m.seeds.length), "changed artifacts", "secondary"),
    metric("Affected", num(m.affected.length), "of " + plural(m.artifacts.length, "artifact"),
      "primary"),
    metric("Active checks", num(active.length), "of " + num(m.checks.length) + " declared"),
    metric("Pending checks", num(pending.length), num(m.pending_units) + " of "
      + num(m.required_units) + " policy units"),
    metric("Evidence invalidated", num(invalid.length), "of " + num(m.evidence.length)
      + " items")));
  section.appendChild(h("h3", { text: "Closure graph" }));
  section.appendChild(closureGraph(m, index, view));
  section.appendChild(closureTable(model, m, index, view));
  section.appendChild(checksTable(m, index));
  section.appendChild(evidenceTable(m, index));
  if (model.data.behavior) section.appendChild(behavior(model.data.behavior, m));
  if (model.data.interaction) section.appendChild(interaction(model.data.interaction, m));
  return section;
}

const GRAPH_LIMIT = 160;

// Lays artifacts out in columns by closure depth; unaffected artifacts form a last column.
function closureGraph(m, index, view) {
  const columns = new Map();
  const place = (id, column) => {
    if (!columns.has(column)) columns.set(column, []);
    columns.get(column).push(id);
  };
  const keep = new Set(index.affected.keys());
  for (const [from, to] of m.edges) {
    if (keep.has(from) || keep.has(to)) {
      keep.add(from);
      keep.add(to);
    }
  }
  m.artifacts.forEach((_, id) => {
    if (m.artifacts.length <= 40) keep.add(id);
  });
  if (keep.size > GRAPH_LIMIT) {
    return h("p", { class: "note", text: "The graph has " + num(keep.size) + " artifacts, too "
      + "many to draw legibly. The closure table below lists every affected artifact." });
  }
  const maxDepth = Math.max(0, ...index.depth.values());
  for (const id of keep) {
    place(id, index.affected.has(id) ? index.depth.get(id) : maxDepth + 1);
  }
  const width = 190;
  const gap = 70;
  const rowHeight = 44;
  const order = Array.from(columns.keys()).sort((a, b) => a - b);
  const height = 40 + rowHeight * Math.max(1, ...order.map((c) => columns.get(c).length));
  const position = new Map();
  order.forEach((column, x) => {
    columns.get(column).sort((a, b) => m.artifacts[a].localeCompare(m.artifacts[b]))
      .forEach((id, y) => position.set(id, [16 + x * (width + gap), 34 + y * rowHeight]));
  });
  const total = 32 + order.length * (width + gap) - gap;
  const root = svg("svg", { viewBox: "0 0 " + total + " " + height, width: total, height,
    role: "img", "aria-label": "Dependency graph of the change closure" });
  order.forEach((column, x) => root.appendChild(svg("text", { class: "col-label",
    x: 16 + x * (width + gap), y: 18,
    text: column > maxDepth ? "NOT AFFECTED" : column === 0 ? "SEEDS" : "DEPTH " + column })));
  for (const [from, to] of m.edges) {
    const a = position.get(from);
    const b = position.get(to);
    if (!a || !b) continue;
    const hot = index.affected.has(from) && index.affected.has(to);
    const [x1, y1, x2, y2] = [a[0] + width, a[1] + 15, b[0], b[1] + 15];
    const bend = Math.max(30, Math.abs(x2 - x1) / 2);
    root.appendChild(svg("path", { class: "edge" + (hot ? " hot" : ""),
      "marker-end": "url(#arrow)",
      d: "M" + x1 + "," + y1 + " C" + (x1 + bend) + "," + y1 + " " + (x2 - bend) + "," + y2
        + " " + x2 + "," + y2 }));
  }
  root.insertBefore(svg("defs", null, svg("marker", { id: "arrow", viewBox: "0 0 8 8", refX: 7,
    refY: 4, markerWidth: 7, markerHeight: 7, orient: "auto" },
  svg("path", { d: "M0,0 L8,4 L0,8 z", fill: "currentColor", class: "arrow" }))), root.firstChild);
  for (const [id, [x, y]] of position) {
    const role = index.seeds.has(id) ? "seed" : index.affected.has(id) ? "affected" : "dim";
    const label = shown(m.artifacts[id]);
    const node = svg("g", { class: "node " + role, tabindex: 0, role: "button",
      "aria-label": label + " (" + role + ")",
      onclick: () => view.artifact(id), onkeydown: (event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          view.artifact(id);
        }
      } },
    svg("title", { text: label }),
    svg("rect", { x, y, width, height: 30, rx: 8 }),
    svg("text", { x: x + 12, y: y + 19, text: label.length > 26 ? label.slice(0, 25) + "..."
      : label }));
    root.appendChild(node);
  }
  return h("div", { class: "graph" }, root,
    h("ul", { class: "legend", style: "padding:0 12px" },
      h("li", null, h("span", { class: "swatch", style: "background:var(--pink)" }), "Seed"),
      h("li", null, h("span", { class: "swatch", style: "background:var(--accent)" }),
        "Affected"),
      h("li", null, h("span", { class: "swatch", style: "background:var(--line-strong)" }),
        "Not affected")));
}

function witness(m, index, id) {
  const chain = [];
  for (let at = id; at !== null && at !== undefined && chain.length < 1000;
    at = index.affected.get(at)) {
    chain.unshift(m.artifacts[at]);
  }
  return chain.map(shown).join(" -> ");
}

function closureTable(model, m, index, view) {
  const rows = m.artifacts.map((name, id) => ({ id, name,
    role: index.seeds.has(id) ? "Seed" : index.affected.has(id) ? "Affected" : "Not affected",
    depth: index.affected.has(id) ? index.depth.get(id) : null,
    file: model.byPath.get(name) || null }));
  const table = dataTable({ label: "Artifacts", rows, sort: "depth", dir: "asc",
    pageSize: rows.length > 100 ? 50 : 0, onRow: (r) => view.artifact(r.id),
    columns: [
      { key: "name", label: "Artifact", value: (r) => r.name, cell: (r) => shown(r.name) },
      { key: "role", label: "Role", value: (r) => r.role, cell: (r) => tag(r.role,
        r.role === "Seed" ? "seed" : r.role === "Affected" ? "affected" : "") },
      { key: "depth", label: "Depth", num: true, value: (r) => r.depth },
      { key: "witness", label: "Dependency witness", narrow: true, cell: (r) =>
        r.depth === null ? "-" : witness(m, index, r.id) },
      { key: "file", label: "Source file", cell: (r) => r.file ? h("button", { class: "link path",
        type: "button", text: shown(r.file.path), onclick: () => view.file(r.file) }) : "-" },
    ] });
  return block("Artifacts and dependency witnesses (" + num(m.artifacts.length) + ")", table.el,
    true);
}

function triggerText(m, triggers) {
  if (!triggers.length) return "never";
  return triggers.map((t) => t.length ? t.map((id) => shown(m.artifacts[id])).join(" and ")
    : "always (standing)").join(" or ");
}

function checksTable(m) {
  const discharging = (index) => m.evidence.filter((e) => e[6].includes(index));
  const rows = m.checks.map((c, index) => ({ index, name: c[0], units: c[1],
    status: checkStatus(c), triggers: triggerText(m, c[5]), evidence: discharging(index) }));
  const table = dataTable({ label: "Checks", rows, sort: "status", dir: "asc",
    columns: [
      { key: "name", label: "Check", value: (r) => r.name, cell: (r) => shown(r.name) },
      { key: "units", label: "Units", num: true, value: (r) => r.units },
      { key: "status", label: "Status", value: (r) => r.status[0],
        cell: (r) => tag(r.status[0], r.status[1]) },
      { key: "triggers", label: "Triggered by", narrow: true, cell: (r) => r.triggers },
      { key: "evidence", label: "Evidence", cell: (r) => r.evidence.length
        ? r.evidence.map((e) => h("div", null, shown(e[0]), " ", tag(e[3], e[3] === "surviving"
          ? "ok" : e[3] === "invalidated" ? "bad" : ""))) : "-" },
    ] });
  return block("Checks (" + num(m.checks.length) + ")", table.el, true);
}

function evidenceTable(m, index) {
  const rows = m.evidence.map((e, id) => ({ id, name: e[0], cls: e[1], accepted: e[2],
    status: e[3], conclusion: e[4], support: e[5], discharges: e[6] }));
  const table = dataTable({ label: "Evidence", rows, sort: "status", dir: "asc",
    columns: [
      { key: "name", label: "Evidence", value: (r) => r.name, cell: (r) => shown(r.name) },
      { key: "class", label: "Class", value: (r) => r.cls, cell: (r) => shown(r.cls) },
      { key: "accepted", label: "Accepted", value: (r) => String(r.accepted),
        cell: (r) => yesNo(r.accepted) },
      { key: "conclusion", label: "Conclusion", value: (r) => r.conclusion, narrow: true },
      { key: "status", label: "Status", value: (r) => r.status, cell: (r) => tag(r.status,
        r.status === "surviving" ? "ok" : r.status === "invalidated" ? "bad" : "") },
      { key: "support", label: "Rests on", narrow: true, cell: (r) => r.support.length
        ? r.support.map((id) => h("div", null, shown(m.artifacts[id]),
          index.affected.has(id) ? tag("changed", "bad") : null)) : "-" },
      { key: "discharges", label: "Addresses", narrow: true, cell: (r) => r.discharges
        .map((c) => m.checks[c] ? shown(m.checks[c][0]) : "#" + c).join(", ") || "-" },
    ] });
  return block("Evidence (" + num(m.evidence.length) + ")", [table.el, h("p", { class: "note",
    text: "Evidence is invalidated when an artifact it rests on is in the closure. Acceptance "
      + "and source binding are declared by the model author." })], true);
}

function behavior(b, m) {
  const parts = [keyValues([["Consistent", yesNo(b.consistent)],
    ["Compatible hypotheses", num(b.hypotheses)], ["Distinct observations", num(b.observations)],
    ["Distinction bits", b.distinction_bits === null ? "unavailable" : fixed(b.distinction_bits,
      3)],
    ["Fidelity", b.fidelity]])];
  if (m.cases.length) {
    const used = m.filters.map(([e]) => e);
    const head = h("tr", null, h("th", { text: "Case" }), h("th", { class: "l",
      text: "Observation" }),
      used.map((e) => h("th", { text: m.evidence[e] ? shown(m.evidence[e][0]) : "#" + e })));
    const body = h("tbody");
    m.cases.forEach(([name, observation], c) => {
      body.appendChild(h("tr", null, h("td", { text: shown(name) }),
        h("td", { class: "l", text: shown(observation) }),
        m.filters.map(([, compatible]) => compatible.includes(c)
          ? h("td", { class: "yes", text: "yes" }) : h("td", { class: "no", text: "no" }))));
    });
    parts.push(h("h4", { text: "Cases allowed by each evidence item" }), h("div",
      { class: "scroll" },
      h("table", { class: "matrix", "aria-label": "Behavioral cases" }, h("thead", null, head),
        body)));
  }
  parts.push(h("p", { class: "note", text: "Counts are exact for the declared hypothesis "
    + "universe. They are cardinalities, not probabilities." }));
  return block("Behavioral ambiguity", parts, true);
}

function interaction(i, m) {
  const names = (list) => list.length ? list.map((c) => m.checks[c] ? shown(m.checks[c][0])
    : "#" + c).join(", ") : "none";
  return block("Batch interaction", [keyValues([["Split", "first seed versus remaining seeds"],
    ["Shared checks", names(i.shared)], ["Shared units", num(i.shared_units)],
    ["Joint checks", names(i.joint)], ["Joint units", num(i.joint_units)],
    ["Net units", (i.net_units > 0 ? "+" : "") + num(i.net_units)]]),
  h("p", { class: "note", text: "Shared checks are triggered by both batches. Joint checks "
    + "appear only when the batches change together." })], true);
}

function artifactDetail(model, view, id) {
  const m = model.data.model;
  const index = changeIndex(m);
  const name = m.artifacts[id];
  const into = m.edges.filter(([, to]) => to === id).map(([from]) => shown(m.artifacts[from]));
  const out = m.edges.filter(([from]) => from === id).map(([, to]) => shown(m.artifacts[to]));
  const checks = m.checks.filter((c) => c[5].some((t) => t.includes(id)));
  const evidence = m.evidence.filter((e) => e[5].includes(id));
  const file = model.byPath.get(name);
  const body = h("div", null, facts([
    ["Role", index.seeds.has(id) ? "Seed" : index.affected.has(id) ? "Affected" : "Not affected"],
    ["Depth", index.affected.has(id) ? num(index.depth.get(id)) : "-"],
    ["Depends on", into.join(", ") || "-"], ["Dependents", out.join(", ") || "-"],
    ["Witness", index.affected.has(id) ? witness(m, index, id) : "-"],
    ["Triggers checks", checks.map((c) => shown(c[0])).join(", ") || "-"],
    ["Supports evidence", evidence.map((e) => shown(e[0])).join(", ") || "-"],
  ]));
  if (file) {
    body.appendChild(h("div", { class: "actions" }, h("button", { class: "btn", type: "button",
      text: "Open source file", onclick: () => view.file(file) })));
  }
  showDetail(name, "Artifact", body);
}

// ---------------------------------------------------------------------------------------

function renderComparison(model, view) {
  const c = model.data.comparison;
  const before = c.baseline_totals;
  const after = model.data.totals;
  const section = h("section", { id: "comparison" });
  section.appendChild(sectionHead("Comparison", "Files whose bytes differ from the baseline. "
    + "This is a file-level byte comparison; no semantic equivalence is inferred."));
  const kinds = new Map();
  for (const row of c.files) kinds.set(row[1], (kinds.get(row[1]) || 0) + 1);
  const delta = (x, y) => (y - x > 0 ? "+" : "") + num(y - x);
  section.appendChild(h("div", { class: "metrics" },
    metric("Changed files", num(c.files.length), Array.from(kinds).map(([k, n]) => num(n) + " "
      + k).join(", ") || "none", "primary"),
    metric("Files", num(model.files.length), delta(c.baseline_files, model.files.length)
      + " from " + num(c.baseline_files)),
    metric("Program lines", num(model.data.lines.program), delta(c.baseline_program_lines,
      model.data.lines.program)),
    metric("Code", num(after.code), delta(before.code, after.code)),
    metric("Bytes", bytes(after.bytes), delta(before.bytes, after.bytes) + " bytes")));
  const totals = h("tbody");
  for (const key of ["lines", "code", "comments", "blank", "bytes"]) {
    totals.appendChild(h("tr", null, h("td", { text: key[0].toUpperCase() + key.slice(1) }),
      h("td", { text: num(before[key]) }), h("td", { text: num(after[key]) }),
      h("td", { text: delta(before[key], after[key]) })));
  }
  section.appendChild(block("Totals before and after", h("table", null, h("thead", null,
    h("tr", null, h("th", { text: "Measure" }), h("th", { text: "Baseline" }),
      h("th", { text: "Current" }), h("th", { text: "Change" }))), totals)));
  const rows = c.files.map(([path, kind, b, a]) => ({ path, kind, before: b, after: a,
    code: (a ? a.code : 0) - (b ? b.code : 0), lines: (a ? a.lines : 0) - (b ? b.lines : 0),
    bytes: (a ? a.bytes : 0) - (b ? b.bytes : 0) }));
  const signed = (v) => (v > 0 ? "+" : "") + num(v);
  const table = dataTable({ label: "Changed files", rows, sort: "path", dir: "asc",
    pageSize: 50, onRow: (r) => changeDetail(model, view, r), empty: "No files changed.",
    columns: [
      { key: "path", label: "Path", value: (r) => r.path, cell: (r) => h("span",
        { class: "path" }, shown(r.path)) },
      { key: "kind", label: "Change", value: (r) => r.kind, cell: (r) => tag(r.kind,
        r.kind === "added" ? "ok" : r.kind === "removed" ? "bad" : "affected") },
      { key: "before", label: "Code before", num: true, narrow: true,
        value: (r) => (r.before ? r.before.code : null) },
      { key: "after", label: "Code after", num: true, narrow: true,
        value: (r) => (r.after ? r.after.code : null) },
      { key: "code", label: "Code change", num: true, value: (r) => r.code,
        cell: (r) => signed(r.code) },
      { key: "bytes", label: "Byte change", num: true, value: (r) => r.bytes,
        cell: (r) => signed(r.bytes), narrow: true },
    ] });
  section.appendChild(table.el);
  return section;
}

function changeDetail(model, view, row) {
  const keys = ["lines", "code", "comments", "blank", "bytes"];
  const body = h("tbody");
  for (const key of keys) {
    const b = row.before ? row.before[key] : null;
    const a = row.after ? row.after[key] : null;
    body.appendChild(h("tr", null, h("td", { text: key }), h("td", { text: num(b) }),
      h("td", { text: num(a) }), h("td", { text: (a || 0) - (b || 0) > 0 ? "+"
        + num((a || 0) - (b || 0)) : num((a || 0) - (b || 0)) })));
  }
  const file = model.byPath.get(row.path);
  const content = h("div", null, h("p", null, tag(row.kind, "affected")),
    h("table", null, h("thead", null, h("tr", null, h("th", { text: "Measure" }),
      h("th", { text: "Baseline" }), h("th", { text: "Current" }), h("th", { text: "Change" }))),
    body));
  if (file) {
    content.appendChild(h("div", { class: "actions" }, h("button", { class: "btn", type: "button",
      text: "Open current file", onclick: () => view.file(file) })));
  }
  showDetail(row.path, "Changed file", content);
}

// ---------------------------------------------------------------------------------------

function renderNotices(model) {
  const d = model.data;
  const section = h("section", { id: "notices" });
  section.appendChild(sectionHead("Coverage and notices", "What the selection left out, and "
    + "why. An incomplete selection makes every total a lower bound."));
  const s = d.skipped;
  section.appendChild(h("div", { class: "metrics" },
    metric("Notices", num(d.notices.length), "coverage warnings", d.notices.length ? "secondary"
      : ""),
    metric("Unsupported files", num(d.unsupported_files), "no language policy"),
    metric("Binary skipped", num(s.binary)), metric("Oversized skipped", num(s.oversized)),
    metric("Generated skipped", num(s.generated)), metric("Symlinks skipped", num(s.symlinks)),
    metric("Ignored entries", num(s.ignored_entries))));
  const groups = new Map();
  for (const [path, message] of d.notices) {
    if (!groups.has(message)) groups.set(message, []);
    groups.get(message).push(path);
  }
  if (!groups.size) section.appendChild(h("p", { class: "empty", text: "No coverage notices." }));
  for (const [message, paths] of groups) {
    const list = h("ul", { class: "plain" });
    const show = (limit) => {
      list.textContent = "";
      for (const path of paths.slice(0, limit)) list.appendChild(h("li", { class: "path",
        text: shown(path) }));
      if (paths.length > limit) {
        list.appendChild(h("li", null, h("button", { class: "link", type: "button",
          text: "Show all " + num(paths.length), onclick: () => show(paths.length) })));
      }
    };
    show(200);
    section.appendChild(block([h("span", { text: shown(message) }), tag(num(paths.length),
      "warn")], list, groups.size === 1));
  }
  if (d.unknown_extensions.length) {
    section.appendChild(block("Unsupported extensions (" + num(d.unknown_extensions.length) + ")",
      dataTable({ label: "Unsupported extensions", rows: d.unknown_extensions, sort: "count",
        pageSize: 50, columns: [
          { key: "ext", label: "Extension", value: (r) => r[0], cell: (r) => shown(r[0]
            || "(none)") },
          { key: "count", label: "Files", num: true, value: (r) => r[1] },
        ] }).el));
  }
  return section;
}

function list(values) {
  return values && values.length ? values.map(shown).join(", ") : "none";
}

function renderAppendix(model) {
  const d = model.data;
  const s = d.selection;
  const section = h("section", { id: "appendix" });
  section.appendChild(sectionHead("Technical appendix", "Options, limits and policies behind "
    + "every number in this report."));
  section.appendChild(h("div", { class: "grid2" },
    h("div", null, h("h3", { text: "Selection" }), keyValues([
      ["Paths", list(s.paths)], ["Symlinks", s.symlinks], ["Ignore files", yesNo(s.gitignore)],
      ["Default exclusions", yesNo(s.default_exclusions)], ["Exclusions", list(s.exclusions)],
      ["Include extensions", list(s.include_extensions)],
      ["Exclude extensions", list(s.exclude_extensions)],
      ["Count as", s.count_as.length ? s.count_as.map(([e, l]) => shown(e) + " as " + l)
        .join(", ") : "none"],
      ["Skip generated", yesNo(s.skip_generated)], ["Work tokens", yesNo(s.collect_work)],
      ["Baseline", s.compare === null ? "none" : shown(s.compare)],
      ["Model", s.model === null ? "none" : shown(s.model)], ["Changed", list(s.changed)],
    ])),
    h("div", null, h("h3", { text: "Limits" }), keyValues([
      ["Max files", num(s.max_files)], ["Max file bytes", num(s.max_file_bytes)],
      ["Max total bytes", num(s.max_total_bytes)], ["Max depth", num(s.max_depth)],
      ["Max report bytes", num(s.max_report_bytes)],
    ]), h("h3", { text: "Report" }), keyValues([
      ["Tool", "ccm " + d.version], ["Data schema", d.schema], ["Count policy", d.count_policy],
      ["Complete", yesNo(d.complete)], ["Languages", num(model.languages.length)],
      ["Directories", num(countDirs(model.tree))],
      ["Model", d.model ? d.model.schema + ", " + d.model.fidelity + ", source binding "
        + d.model.source_binding : "none"],
      ["Cost", d.no_cost ? "disabled (--no-cost)" : d.economic ? d.economic.model : "none"],
    ]))));
  section.appendChild(h("h3", { text: "How to read the numbers" }));
  section.appendChild(h("ul", null, [
    "Mixed lines, with code and a comment, count as code. Symlinks are skipped.",
    "Selection follows the ignore rules, default exclusions and explicit exclusions above.",
    "Source counting does not establish semantic coverage. Task outcomes require declared cases.",
    "Change results are exact within the author-declared model; evidence acceptance and source "
      + "binding are supplied by the model author.",
    "Estimates are planning scenarios with stated assumptions, not measured spending.",
  ].map((text) => h("li", { text }))));
  section.appendChild(h("div", { class: "toolbar" },
    h("button", { class: "btn", type: "button", text: "Download report data (JSON)",
      onclick: () => download("ccm-report-data.json", "application/json",
        JSON.stringify(d, null, 1)) }),
    h("button", { class: "btn", type: "button", text: "Download all files (CSV)",
      onclick: () => download("ccm-files.csv", "text/csv", filesCsv(model.files)) })));
  return section;
}
