// Inventory views: overview, languages, the directory explorer, and the file table.

const KIND_LABEL = { source: "Program source", documentation: "Documentation",
  data: "Data and configuration" };
const KIND_ORDER = ["source", "documentation", "data"];

function renderOverview(model, view) {
  const d = model.data;
  const t = d.totals;
  const section = h("section", { id: "overview" });
  const status = d.complete ? tag("Complete selection", "ok") : tag("Incomplete selection",
    "warn");
  section.append(sectionHead("Overview",
    "What was counted. Program lines are code lines of hand-written source files; "
    + "generated files, documentation and data are listed but kept out of that figure.", status));
  const cards = h("div", { class: "metrics" },
    metric("Selected files", num(model.files.length), plural(model.languageRows.length,
      "language"), "primary"),
    metric("Program lines", num(d.lines.program), d.lines.generated_source
      ? "+" + num(d.lines.generated_source) + " generated" : "hand-written source", "secondary"),
    metric("Total lines", num(t.lines), "code + comments + blank"),
    metric("Code", num(t.code), percent(t.code, t.lines) + " of lines"),
    metric("Comments", num(t.comments), percent(t.comments, t.lines) + " of lines"),
    metric("Blank", num(t.blank), percent(t.blank, t.lines) + " of lines"),
    metric("Bytes", bytes(t.bytes), num(t.bytes) + " bytes"));
  if (d.budget) cards.appendChild(metric("Budget scenario", money(d.budget.total),
    "planning estimate, USD"));
  section.appendChild(cards);
  const grid = h("div", { class: "grid2" });
  grid.appendChild(h("div", null, h("h3", { text: "Line composition" }), stack([
    ["Code", t.code, "var(--c-code)"], ["Comments", t.comments, "var(--c-comments)"],
    ["Blank", t.blank, "var(--c-blank)"]])));
  grid.appendChild(h("div", null, h("h3", { text: "Nonblank content by purpose" }), stack([
    ["Program", d.lines.program, "var(--c-code)"],
    ["Generated source", d.lines.generated_source, "var(--deep)"],
    ["Documentation", d.lines.documentation, "var(--c-comments)"],
    ["Data", d.lines.data, "var(--lilac)"]])));
  section.appendChild(grid);
  section.appendChild(overviewNotes(model, view));
  return section;
}

function overviewNotes(model, view) {
  const d = model.data;
  const list = h("div", { class: "chips" });
  const skipped = d.skipped;
  const add = (text, tone, target) => list.appendChild(target
    ? h("button", { class: "chip " + (tone || ""), type: "button", text, onclick: () =>
      view.go(target) })
    : h("span", { class: "chip " + (tone || ""), text }));
  add(plural(d.notices.length, "notice"), d.notices.length ? "warn" : "muted", "notices");
  add(plural(d.unsupported_files, "unsupported file"), d.unsupported_files ? "warn" : "muted",
    "notices");
  if (skipped.binary) add(plural(skipped.binary, "binary file") + " skipped", "muted", "notices");
  if (skipped.oversized) add(plural(skipped.oversized, "oversized file") + " skipped", "warn",
    "notices");
  if (skipped.generated) add(plural(skipped.generated, "generated file") + " skipped", "muted",
    "notices");
  if (skipped.symlinks) add(plural(skipped.symlinks, "symlink") + " skipped", "muted", "notices");
  if (skipped.ignored_entries) add(plural(skipped.ignored_entries, "ignored entry",
    "ignored entries"),
    "muted", "notices");
  add("Count policy " + d.count_policy, "muted", "appendix");
  const wrap = h("div", null, h("h3", { text: "Coverage" }), list);
  if (d.lines.generated_source) {
    wrap.appendChild(h("p", { class: "note", text: num(d.lines.generated_source)
      + " generated source lines are listed but excluded from program lines." }));
  }
  if (!d.complete) {
    wrap.appendChild(h("p", { class: "warning",
      text: "Incomplete: some source files were not counted. "
      + "The notices list each gap." }));
  }
  return wrap;
}

// ---------------------------------------------------------------------------------------

function renderLanguages(model, view) {
  const section = h("section", { id: "languages" });
  const total = model.data.totals;
  let grouped = true;
  let table = null;
  section.appendChild(sectionHead("Languages",
    "Counts per language policy. Select a language to list its files."));
  const toggle = h("label", { class: "check" }, h("input", { type: "checkbox", checked: true,
    onchange: (event) => {
      grouped = event.target.checked;
      const old = table.el;
      old.replaceWith(build().el);
    } }), " Group by purpose");
  section.appendChild(h("div", { class: "toolbar" }, toggle));
  const columns = [
    { key: "name", label: "Language", value: (r) => r.name, cell: (r) => r.group ? r.name
      : h("button", { class: "link", type: "button", text: r.name,
        onclick: () => view.files({ language: r.name }) }) },
    { key: "files", label: "Files", num: true, value: (r) => r.files },
    { key: "code", label: "Code", num: true, value: (r) => r.code },
    { key: "comments", label: "Comments", num: true, value: (r) => r.comments, narrow: true },
    { key: "blank", label: "Blank", num: true, value: (r) => r.blank, narrow: true },
    { key: "lines", label: "Lines", num: true, value: (r) => r.lines, narrow: true },
    { key: "bytes", label: "Bytes", num: true, value: (r) => r.bytes, cell: (r) => bytes(r.bytes),
      narrow: true },
    { key: "share", label: "Share of code", bar: true, cell: (r) =>
      h("span", { class: "share" }, bar(r.code, total.code, r.kind === "source" ? "" : "pink"),
        h("small", { text: percent(r.code, total.code) })) },
  ];
  const groups = (rows) => KIND_ORDER.map((kind) => {
    const members = rows.filter((r) => r.kind === kind);
    const sum = members.reduce((acc, r) => {
      for (const key of ["files", "code", "comments", "blank", "lines",
        "bytes"]) acc[key] += r[key];
      return acc;
    }, Object.assign(emptyTotals(), { name: KIND_LABEL[kind] + " (" + members.length + ")", kind,
      group: true }));
    return { total: sum, rows: members };
  }).filter((g) => g.rows.length);
  const totalRow = (rows) => rows.reduce((acc, r) => {
    for (const key of ["files", "code", "comments", "blank", "lines", "bytes"]) acc[key] += r[key];
    return acc;
  }, Object.assign(emptyTotals(), { name: "All languages", kind: "source", group: true }));
  const build = () => {
    table = dataTable({ columns, rows: model.languageRows, sort: "code", dir: "desc",
      groups: grouped ? groups : null, total: totalRow, label: "Languages",
      onRow: (r) => view.files({ language: r.name }), rowClass: () => "" });
    return table;
  };
  build();
  section.appendChild(table.el);
  return section;
}

// ---------------------------------------------------------------------------------------
// The directory explorer: a breadcrumb for the focused directory and a collapsible tree
// below it. Children are rendered only when their parent is expanded.

const TREE_CHILD_LIMIT = 200;
const TREE_ROW_LIMIT = 3000;

function renderDirectories(model, view) {
  const section = h("section", { id: "directories" });
  const state = { focus: model.tree, open: new Set(), more: new Map(), metric: "lines" };
  section.appendChild(sectionHead("Directories",
    "Totals aggregated from file paths. Expand a directory in place, or open it to focus on it."));
  const crumbs = h("ol", { class: "crumbs", "aria-label": "Directory path" });
  const metricSelect = h("select", { "aria-label": "Measure", onchange: (event) => {
    state.metric = event.target.value;
    render();
  } }, ["lines", "code", "comments", "blank", "bytes", "files"].map((key) =>
    h("option", { value: key, text: "Measure: " + key })));
  const summary = h("p", { class: "note" });
  const toolbar = h("div", { class: "toolbar" }, metricSelect,
    h("button", { class: "btn", type: "button", text: "Expand all", onclick: () => {
      expandAll(state.focus, state.open, 400);
      render();
    } }),
    h("button", { class: "btn", type: "button", text: "Collapse all", onclick: () => {
      state.open.clear();
      render();
    } }),
    h("button", { class: "btn", type: "button", text: "List these files", onclick: () =>
      view.files({ dir: state.focus.path, dirLabel: state.focus.path || rootLabel(model) }) }));
  const body = h("tbody");
  const table = h("table", { "aria-label": "Directory tree" },
    h("thead", null, h("tr", null, h("th", { class: "l", text: "Name" }), h("th",
      { text: "Files" }),
      h("th", { text: "Code" }), h("th", { class: "hide-narrow", text: "Comments" }),
      h("th", { class: "hide-narrow", text: "Blank" }), h("th", { class: "hide-narrow",
        text: "Lines" }),
      h("th", { class: "hide-narrow", text: "Bytes" }), h("th", { class: "l", text: "Share" }))),
    body);
  section.append(crumbs, toolbar, summary, h("div", { class: "scroll" }, table));

  function focus(node) {
    state.focus = node;
    state.open.clear();
    render();
  }

  function render() {
    crumbs.textContent = "";
    const chain = [];
    for (let node = state.focus; node; node = node.parent) chain.unshift(node);
    chain.forEach((node, index) => {
      const label = node.parent ? shown(node.name) : rootLabel(model);
      crumbs.appendChild(h("li", null, index === chain.length - 1
        ? h("span", { class: "current", "aria-current": "location", text: label })
        : h("button", { class: "link", type: "button", text: label, onclick: () =>
          focus(node) })));
    });
    const t = state.focus.totals;
    summary.textContent = plural(t.files, "file") + " in " + plural(countDirs(state.focus),
      "directory",
      "directories") + "; " + num(t.code) + " code lines, " + num(t.program) + " program lines, "
      + bytes(t.bytes) + ".";
    body.textContent = "";
    const rows = [];
    collect(state.focus, 0, rows);
    const fragment = document.createDocumentFragment();
    for (const row of rows) fragment.appendChild(row);
    body.appendChild(fragment);
  }

  function children(node) {
    const key = state.metric;
    const value = (item) => (item.totals ? item.totals[key] : key === "files" ? 1 : item[key]);
    const dirs = Array.from(node.dirs.values()).sort((a, b) => value(b) - value(a)
      || a.name.localeCompare(b.name));
    const files = node.files.slice().sort((a, b) => value(b) - value(a)
      || a.name.localeCompare(b.name));
    return dirs.concat(files);
  }

  function collect(node, depth, rows) {
    const items = children(node);
    const limit = state.more.get(node.path) || TREE_CHILD_LIMIT;
    for (const item of items.slice(0, limit)) {
      if (rows.length >= TREE_ROW_LIMIT) {
        rows.push(noteRow("Row limit reached. Open a subdirectory to see more."));
        return;
      }
      rows.push(item.totals ? dirRow(item, depth) : fileRow(item, depth));
      if (item.totals && state.open.has(item.path)) collect(item, depth + 1, rows);
    }
    if (items.length > limit) {
      const rest = items.length - limit;
      rows.push(h("tr", null, h("td", { colspan: 8, style: "padding-left:" + (12 + depth * 18)
        + "px" },
        h("button", { class: "link", type: "button", text: "Show " + num(Math.min(rest,
          TREE_CHILD_LIMIT))
          + " more of " + num(rest) + " remaining", onclick: () => {
          state.more.set(node.path, limit + TREE_CHILD_LIMIT);
          render();
        } }))));
    }
  }

  function share(value) {
    const whole = state.metric === "files" ? state.focus.totals.files
      : state.focus.totals[state.metric];
    return h("span", { class: "share" }, bar(value, whole), h("small", { text: percent(value,
      whole) }));
  }

  function dirRow(node, depth) {
    const open = state.open.has(node.path);
    const t = node.totals;
    const twisty = h("button", { class: "twisty", type: "button", "aria-expanded": String(open),
      "aria-label": (open ? "Collapse " : "Expand ") + node.name, onclick: () => {
        if (open) state.open.delete(node.path);
        else state.open.add(node.path);
        render();
      } });
    const name = h("div", { class: "tree-name", style: "padding-left:" + depth * 18 + "px" },
      twisty,
      h("span", { class: "icon-dir", "aria-hidden": "true" }),
      h("button", { class: "link", type: "button", text: shown(node.name) + "/", title: "Open "
        + node.path,
        onclick: () => focus(node) }));
    return h("tr", { class: "dir-row" }, h("td", { class: "l" }, name), h("td",
      { text: num(t.files) }),
      h("td", { text: num(t.code) }), h("td", { class: "hide-narrow", text: num(t.comments) }),
      h("td", { class: "hide-narrow", text: num(t.blank) }), h("td", { class: "hide-narrow",
        text: num(t.lines) }),
      h("td", { class: "hide-narrow", text: bytes(t.bytes) }),
      h("td", { class: "l bar-cell" }, share(state.metric === "files" ? t.files
        : t[state.metric])));
  }

  function fileRow(file, depth) {
    const name = h("div", { class: "tree-name", style: "padding-left:" + depth * 18 + "px" },
      h("span", { class: "twisty leaf", "aria-hidden": "true" }), h("span", { class: "icon-file",
        "aria-hidden": "true" }),
      h("button", { class: "link path", type: "button", text: shown(file.name), onclick: () =>
        view.file(file) }),
      file.generated ? tag("generated", "gen") : null, tag(file.language, "hide-narrow"));
    return h("tr", null, h("td", { class: "l" }, name), h("td", { text: "1" }), h("td",
      { text: num(file.code) }),
      h("td", { class: "hide-narrow", text: num(file.comments) }), h("td", { class: "hide-narrow",
        text: num(file.blank) }),
      h("td", { class: "hide-narrow", text: num(file.lines) }), h("td", { class: "hide-narrow",
        text: bytes(file.bytes) }),
      h("td", { class: "l bar-cell" }, share(state.metric === "files" ? 1 : file[state.metric])));
  }

  render();
  view.focusDir = (path) => {
    const node = findDir(model.tree, path);
    if (node) focus(node);
  };
  return section;
}

function noteRow(text) {
  return h("tr", null, h("td", { colspan: 8, class: "empty", text }));
}

function countDirs(node) {
  let count = 0;
  for (const child of node.dirs.values()) count += 1 + countDirs(child);
  return count;
}

function expandAll(node, open, budget) {
  const queue = [node];
  while (queue.length && open.size < budget) {
    const next = queue.shift();
    for (const child of next.dirs.values()) {
      open.add(child.path);
      queue.push(child);
    }
  }
}

function rootLabel(model) {
  const paths = model.data.selection.paths || [];
  return "Root (" + (paths.length ? paths.map(shown).join(", ") : ".") + ")";
}

// ---------------------------------------------------------------------------------------
// The file table: search, filters, sorting and pages over all selected files.

function renderFiles(model, view) {
  const section = h("section", { id: "files" });
  const filter = { query: "", language: "", kind: "", generated: "", dir: null, dirLabel: "" };
  const anyTokens = model.files.some((f) => f.tokens !== null);
  section.appendChild(sectionHead("Files",
    "Every selected file. Search by path, filter, sort by any column, "
    + "and select a row for its details."));
  const search = h("input", { type: "search", placeholder: "Search paths, e.g. src/ parser .rs",
    "aria-label": "Search file paths", oninput: debounce((event) => {
      filter.query = event.target.value;
      apply();
    }, 120) });
  const language = h("select", { "aria-label": "Language", onchange: (event) => {
    filter.language = event.target.value;
    apply();
  } }, h("option", { value: "", text: "All languages" }),
  model.languageRows.slice().sort((a, b) => a.name.localeCompare(b.name))
    .map((r) => h("option", { value: r.name, text: r.name + " (" + num(r.files) + ")" })));
  const kind = h("select", { "aria-label": "Purpose", onchange: (event) => {
    filter.kind = event.target.value;
    apply();
  } }, [["", "All purposes"], ["program", "Program files"], ["source", "Source (with generated)"],
    ["documentation", "Documentation"], ["data", "Data and configuration"]]
    .map(([value, text]) => h("option", { value, text })));
  const generated = h("select", { "aria-label": "Generated", onchange: (event) => {
    filter.generated = event.target.value;
    apply();
  } }, [["", "Generated and hand-written"], ["no", "Hand-written only"], ["yes", "Generated only"]]
    .map(([value, text]) => h("option", { value, text })));
  const chips = h("div", { class: "chips" });
  const summary = h("p", { class: "note", "aria-live": "polite" });
  section.append(h("div", { class: "toolbar" },
    h("label", { class: "grow" }, "Search", search), h("label", null, "Language", language),
    h("label", null, "Purpose", kind), h("label", null, "Generated", generated),
    h("label", null, "Export", h("button", { class: "btn", type: "button", text: "CSV of matches",
      onclick: () => download("ccm-files.csv", "text/csv", filesCsv(table.rows())) }))), chips,
        summary);
  const columns = [
    { key: "path", label: "Path", value: (f) => f.path, cell: (f) => h("span", { class: "path" },
      f.dir ? h("span", { class: "dir", text: shown(f.dir) + "/" }) : null, shown(f.name),
      f.generated ? tag("generated", "gen") : null) },
    { key: "language", label: "Language", value: (f) => f.language, narrow: true },
    { key: "code", label: "Code", num: true, value: (f) => f.code },
    { key: "comments", label: "Comments", num: true, value: (f) => f.comments, narrow: true },
    { key: "blank", label: "Blank", num: true, value: (f) => f.blank, narrow: true },
    { key: "lines", label: "Lines", num: true, value: (f) => f.lines },
    { key: "bytes", label: "Bytes", num: true, value: (f) => f.bytes, cell: (f) => bytes(f.bytes),
      narrow: true },
  ];
  if (anyTokens) columns.push({ key: "tokens", label: "Work tokens", num: true, value: (f) =>
    f.tokens, narrow: true });
  const table = dataTable({ columns, rows: model.files, sort: "lines", dir: "desc", pageSize: 50,
    onRow: (f) => view.file(f), label: "Files", empty: "No file matches these filters." });
  section.appendChild(table.el);

  function matches(file, terms) {
    if (filter.language && file.language !== filter.language) return false;
    if (filter.kind === "program" ? !file.program : filter.kind
      && file.kind !== filter.kind) return false;
    if (filter.generated && file.generated !== (filter.generated === "yes")) return false;
    if (filter.dir !== null && filter.dir !== "" && file.dir !== filter.dir
      && !file.dir.startsWith(filter.dir + "/")) {
      return false;
    }
    const path = file.path.toLowerCase();
    return terms.every((term) => path.includes(term));
  }

  function apply() {
    const terms = filter.query.toLowerCase().split(/\s+/).filter(Boolean);
    const rows = model.files.filter((file) => matches(file, terms));
    table.setRows(rows);
    const code = rows.reduce((sum, f) => sum + f.code, 0);
    summary.textContent = num(rows.length) + " of " + plural(model.files.length, "file")
      + " match; "
      + num(code) + " code lines (" + percent(code, model.data.totals.code) + " of all code).";
    chips.textContent = "";
    if (filter.dir !== null) {
      chips.appendChild(h("button", { class: "chip", type: "button",
        "aria-label": "Remove directory filter",
        text: "In " + (filter.dirLabel || filter.dir) + "/  x", onclick: () => {
          filter.dir = null;
          apply();
        } }));
    }
  }

  view.setFileFilter = (next) => {
    filter.language = next.language || "";
    filter.kind = next.kind || "";
    filter.generated = next.generated || "";
    filter.dir = next.dir === undefined ? null : next.dir;
    filter.dirLabel = next.dirLabel || "";
    filter.query = next.query || "";
    search.value = filter.query;
    language.value = filter.language;
    kind.value = filter.kind;
    generated.value = filter.generated;
    apply();
  };
  apply();
  return section;
}

function filesCsv(rows) {
  const lines = ["path,language,kind,policy,generated,code,comments,blank,lines,bytes,"
    + "source_work_tokens"];
  for (const f of rows) {
    lines.push([f.path, f.language, f.kind, f.policy, f.generated, f.code, f.comments, f.blank,
      f.lines,
      f.bytes, f.tokens].map(csvField).join(","));
  }
  return lines.join("\n") + "\n";
}

// The detail panel of one file: every recorded field and its context in the report.
function fileDetail(model, view, file) {
  const t = model.data.totals;
  const body = h("div", null, facts([
    ["Language", file.language], ["Purpose", KIND_LABEL[file.kind] || file.kind],
    ["Language policy", file.policy], ["Generated", yesNo(file.generated)],
    ["Counts as program", yesNo(file.program)], ["Code lines", num(file.code)],
    ["Comment lines", num(file.comments)], ["Blank lines", num(file.blank)],
    ["Total lines", num(file.lines)], ["Bytes", num(file.bytes) + " (" + bytes(file.bytes) + ")"],
    ["Source work tokens", file.tokens === null ? "not collected" : num(file.tokens)],
    ["Comment density", percent(file.comments, file.code + file.comments)],
    ["Share of all code", percent(file.code, t.code)],
    ["Share of program lines", file.program ? percent(file.code, model.data.lines.program) : "-"],
  ]));
  body.appendChild(stack([["Code", file.code, "var(--c-code)"], ["Comments", file.comments,
    "var(--c-comments)"],
    ["Blank", file.blank, "var(--c-blank)"]]));
  const extra = view.fileContext ? view.fileContext(file) : null;
  if (extra) body.appendChild(extra);
  body.appendChild(h("div", { class: "actions" },
    file.dir ? h("button", { class: "btn", type: "button", text: "Open directory "
      + shown(file.dir),
      onclick: () => {
        closeDetail();
        view.dir(file.dir);
      } }) : null,
    h("button", { class: "btn", type: "button", text: "All " + file.language + " files",
      onclick: () => {
      closeDetail();
      view.files({ language: file.language });
    } })));
  showDetail(file.path, "File", body);
}
