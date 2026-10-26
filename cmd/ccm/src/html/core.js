"use strict";
// Core helpers of the CCM report page: element building, number formatting, the data
// model derived from the embedded JSON, a sortable and paginated table, and a dialog.
// Every untrusted string reaches the page through text nodes, never through markup.

const NUMBER = new Intl.NumberFormat("en-US");
const MONEY = new Intl.NumberFormat("en-US", {
  style: "currency", currency: "USD", minimumFractionDigits: 2, maximumFractionDigits: 2,
});

function h(tag, attrs, ...children) {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs || {})) {
    if (value === undefined || value === null || value === false) continue;
    if (key === "class") node.className = value;
    else if (key === "text") node.textContent = value;
    else if (key.startsWith("on")) node.addEventListener(key.slice(2), value);
    else node.setAttribute(key, value === true ? "" : String(value));
  }
  append(node, children);
  return node;
}

function append(node, children) {
  for (const child of children) {
    if (child === undefined || child === null || child === false) continue;
    if (Array.isArray(child)) append(node, child);
    else if (child instanceof Node) node.appendChild(child);
    else node.appendChild(document.createTextNode(String(child)));
  }
  return node;
}

function svg(tag, attrs, ...children) {
  const node = document.createElementNS("http://www.w3.org/2000/svg", tag);
  for (const [key, value] of Object.entries(attrs || {})) {
    if (value === undefined || value === null) continue;
    if (key === "text") node.textContent = value;
    else if (key.startsWith("on")) node.addEventListener(key.slice(2), value);
    else node.setAttribute(key, String(value));
  }
  append(node, children);
  return node;
}

// Shows control characters and bidirectional overrides as visible escapes, as the
// terminal report does, so a crafted path cannot hide or reorder text.
function shown(value) {
  return String(value).replace(/[\u0000-\u001f\u007f-\u009f‪-‮⁦-⁩]/g,
    (c) => "\\u{" + c.codePointAt(0).toString(16) + "}");
}

function num(value) {
  return typeof value === "number" && isFinite(value) ? NUMBER.format(value) : "-";
}

function money(value) {
  if (typeof value !== "number" || !isFinite(value)) return "-";
  return MONEY.format(value + 0).replace("-$0.00", "$0.00");
}

function fixed(value, digits) {
  if (typeof value !== "number" || !isFinite(value)) return "-";
  return value.toLocaleString("en-US", {
    minimumFractionDigits: digits, maximumFractionDigits: digits,
  });
}

function percent(part, whole) {
  if (!whole) return "-";
  const value = (100 * part) / whole;
  return (value > 0 && value < 0.1 ? "<0.1" : fixed(value, value >= 10 ? 0 : 1)) + "%";
}

function bytes(value) {
  if (typeof value !== "number" || !isFinite(value)) return "-";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let index = 0;
  let size = value;
  while (size >= 1024 && index < units.length - 1) {
    size /= 1024;
    index += 1;
  }
  return index === 0 ? num(value) + " B" : fixed(size, size >= 100 ? 0 : 1) + " " + units[index];
}

function plural(count, word, many) {
  return num(count) + " " + (count === 1 ? word : many || word + "s");
}

function yesNo(value) {
  if (value === null || value === undefined) return "-";
  return value ? "yes" : "no";
}

function bar(part, whole, tone) {
  const width = whole > 0 ? Math.max(0, Math.min(100, (100 * part) / whole)) : 0;
  return h("span", { class: "bar" + (tone ? " " + tone : ""), role: "presentation" },
    h("span", { style: "width:" + width.toFixed(2) + "%" }));
}

// A stacked bar of `[label, value, color]` parts, with an optional legend.
function stack(parts, options) {
  const total = parts.reduce((sum, part) => sum + Math.max(0, part[1]), 0);
  const label = parts.map((p) => p[0] + " " + percent(p[1], total)).join(", ");
  const strip = h("div", { class: "stack" + (options && options.tall ? " tall" : ""),
    role: "img", "aria-label": label });
  for (const [name, value, color] of parts) {
    if (!(value > 0) || !total) continue;
    strip.appendChild(h("span", { title: name + ": " + percent(value, total),
      style: "width:" + ((100 * value) / total).toFixed(3) + "%;background:" + color }));
  }
  if (options && options.legend === false) return strip;
  const legend = h("ul", { class: "legend" });
  for (const [name, value, color, text] of parts) {
    legend.appendChild(h("li", null, h("span", { class: "swatch", style: "background:" + color }),
      name + " " + (text || num(value)) + " (" + percent(value, total) + ")"));
  }
  return h("div", null, strip, legend);
}

function metric(label, value, note, tone) {
  return h("div", { class: "metric" + (tone ? " " + tone : "") },
    h("span", { text: label }), h("strong", { text: value }), note ? h("small", { text: note })
      : null);
}

function tag(text, tone) {
  return h("span", { class: "tag" + (tone ? " " + tone : ""), text });
}

function sectionHead(title, description, extra) {
  return h("div", { class: "section-head" }, h("h2", { text: title }), extra || null,
    description ? h("p", { text: description }) : null);
}

function keyValues(pairs) {
  const list = h("dl", { class: "kv" });
  for (const [key, value] of pairs) {
    if (value === undefined) continue;
    list.appendChild(h("dt", { text: key }));
    list.appendChild(h("dd", null, value instanceof Node ? value : shown(value === null ? "-"
      : value)));
  }
  return list;
}

function facts(pairs) {
  const list = h("dl", { class: "facts" });
  for (const [key, value] of pairs) {
    list.appendChild(h("div", null, h("dt", { text: key }),
      h("dd", null, value instanceof Node ? value : shown(value === null ? "-" : value))));
  }
  return list;
}

function block(summary, body, open) {
  return h("details", { class: "block", open: open || false },
    h("summary", null, summary), h("div", { class: "body" }, body));
}

function download(name, type, text) {
  const url = URL.createObjectURL(new Blob([text], { type }));
  const link = h("a", { href: url, download: name });
  document.body.appendChild(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

function csvField(value) {
  const text = value === null || value === undefined ? "" : String(value);
  return /[",\r\n]/.test(text) || /^[=+\-@]/.test(text) ? "\"" + text.replace(/"/g, "\"\"")
    + "\"" : text;
}

function debounce(action, delay) {
  let timer = 0;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => action(...args), delay);
  };
}

// ---------------------------------------------------------------------------------------
// The data model.

function loadModel(data) {
  const languages = data.languages.map(([name, kind, policy]) => ({ name, kind, policy }));
  const files = data.files.map((row, index) => {
    const language = languages[row[2]] || { name: "?", kind: "data", policy: "?" };
    const dir = data.dirs[row[0]] || "";
    const file = {
      index, dir, name: row[1], path: dir ? dir + "/" + row[1] : row[1],
      language: language.name, kind: language.kind, policy: language.policy,
      generated: row[3] === 1, code: row[4], comments: row[5], blank: row[6], bytes: row[7],
      tokens: row[8],
    };
    file.lines = file.code + file.comments + file.blank;
    file.program = file.kind === "source" && !file.generated;
    return file;
  });
  const byPath = new Map(files.map((file) => [file.path, file]));
  return { data, languages, files, byPath, tree: buildTree(files),
    languageRows: languageRows(files) };
}

function emptyTotals() {
  return { files: 0, code: 0, comments: 0, blank: 0, lines: 0, bytes: 0, program: 0 };
}

function addTotals(totals, file) {
  totals.files += 1;
  totals.code += file.code;
  totals.comments += file.comments;
  totals.blank += file.blank;
  totals.lines += file.lines;
  totals.bytes += file.bytes;
  if (file.program) totals.program += file.code;
}

function languageRows(files) {
  const rows = new Map();
  for (const file of files) {
    let row = rows.get(file.language);
    if (!row) {
      row = Object.assign(emptyTotals(), { name: file.language, kind: file.kind,
        policy: file.policy, generated: 0 });
      rows.set(file.language, row);
    }
    addTotals(row, file);
    if (file.generated) row.generated += 1;
  }
  return Array.from(rows.values());
}

// Builds the directory tree. Every directory holds the totals of all files below it.
function buildTree(files) {
  const root = { name: "", path: "", parent: null, dirs: new Map(), files: [],
    totals: emptyTotals() };
  for (const file of files) {
    let node = root;
    addTotals(node.totals, file);
    for (const part of file.dir.split("/")) {
      if (!part) continue;
      let next = node.dirs.get(part);
      if (!next) {
        next = { name: part, path: node.path ? node.path + "/" + part : part, parent: node,
          dirs: new Map(), files: [], totals: emptyTotals() };
        node.dirs.set(part, next);
      }
      node = next;
      addTotals(node.totals, file);
    }
    node.files.push(file);
  }
  return root;
}

function findDir(tree, path) {
  let node = tree;
  for (const part of path.split("/")) {
    if (!part) continue;
    node = node.dirs.get(part);
    if (!node) return null;
  }
  return node;
}

// ---------------------------------------------------------------------------------------
// A sortable, paginated table. Columns: { key, label, num, value(row), cell(row), narrow }.
// Options: rows, sort, dir, pageSize, onRow(row), rowClass(row), groups, empty, label.

function dataTable(options) {
  const state = {
    rows: options.rows, sort: options.sort, dir: options.dir || "desc", page: 0,
    pageSize: options.pageSize || 0,
  };
  const table = h("table", options.label ? { "aria-label": options.label } : null);
  const head = h("tr");
  const body = h("tbody");
  table.append(h("thead", null, head), body);
  const pager = h("div", { class: "pager" });
  const root = h("div", null, h("div", { class: "scroll" }, table), pager);

  for (const column of options.columns) {
    const cell = h("th", { scope: "col", class: (column.num ? "" : "l") + (column.narrow
      ? " hide-narrow" : "") });
    if (column.value) {
      cell.appendChild(h("button", { type: "button", text: column.label, onclick: () => {
        state.dir = state.sort === column.key && state.dir === "desc" ? "asc"
          : state.sort === column.key ? "desc" : column.num ? "desc" : "asc";
        state.sort = column.key;
        state.page = 0;
        render();
      } }));
    } else {
      cell.textContent = column.label;
    }
    column.header = cell;
    head.appendChild(cell);
  }

  function sorted(rows) {
    const column = options.columns.find((c) => c.key === state.sort);
    if (!column || !column.value) return rows;
    const sign = state.dir === "asc" ? 1 : -1;
    const value = column.value;
    return rows.slice().sort((a, b) => {
      const x = value(a);
      const y = value(b);
      if (x === y) return 0;
      if (x === null || x === undefined) return 1;
      if (y === null || y === undefined) return -1;
      return (typeof x === "string" ? x.localeCompare(y) : x - y) * sign;
    });
  }

  function row(item, extraClass) {
    const tr = h("tr", { class: [extraClass || "", options.rowClass ? options.rowClass(item) : "",
      options.onRow ? "clickable" : ""].join(" ").trim() || null });
    for (const column of options.columns) {
      const content = column.cell ? column.cell(item) : column.value(item);
      tr.appendChild(h("td", { class: (column.num ? "" : "l") + (column.narrow ? " hide-narrow"
        : "")
        + (column.bar ? " bar-cell" : "") }, typeof content === "number" ? num(content)
          : content));
    }
    if (options.onRow && !extraClass) {
      tr.tabIndex = 0;
      tr.addEventListener("click", (event) => {
        if (event.target.closest("button,a,input,select")) return;
        options.onRow(item);
      });
      tr.addEventListener("keydown", (event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          options.onRow(item);
        }
      });
    }
    return tr;
  }

  function render() {
    for (const column of options.columns) {
      if (column.key === state.sort) column.header.setAttribute("aria-sort", state.dir === "asc"
        ? "ascending" : "descending");
      else column.header.removeAttribute("aria-sort");
    }
    body.textContent = "";
    if (!state.rows.length) {
      body.appendChild(h("tr", null, h("td", { colspan: options.columns.length, class: "empty",
        text: options.empty || "Nothing to show." })));
    }
    if (options.groups) {
      for (const group of options.groups(state.rows)) {
        body.appendChild(row(group.total, "group"));
        for (const item of sorted(group.rows)) body.appendChild(row(item));
      }
      if (options.total && state.rows.length) body.appendChild(row(options.total(state.rows),
        "total"));
      pager.textContent = "";
      return;
    }
    const rows = sorted(state.rows);
    const size = state.pageSize || rows.length || 1;
    const pages = Math.max(1, Math.ceil(rows.length / size));
    state.page = Math.min(state.page, pages - 1);
    const start = state.page * size;
    const fragment = document.createDocumentFragment();
    for (const item of rows.slice(start, start + size)) fragment.appendChild(row(item));
    body.appendChild(fragment);
    if (options.total && rows.length) body.appendChild(row(options.total(rows), "total"));
    renderPager(rows.length, start, size, pages);
  }

  function renderPager(count, start, size, pages) {
    pager.textContent = "";
    if (!state.pageSize || count <= Math.min(...pageSizes())) return;
    const go = (page) => () => {
      state.page = Math.max(0, Math.min(pages - 1, page));
      render();
      if (table.getBoundingClientRect().top < 64) {
        table.scrollIntoView({ block: "start" });
      }
    };
    const select = h("select", { "aria-label": "Rows per page", onchange: (event) => {
      state.pageSize = Number(event.target.value);
      state.page = 0;
      render();
    } });
    for (const choice of pageSizes()) {
      select.appendChild(h("option", { value: choice, selected: choice === state.pageSize,
        text: choice + " rows" }));
    }
    pager.append(
      h("span", { text: "Showing " + num(start + 1) + "-" + num(Math.min(start + size, count))
        + " of " + num(count) }),
      h("div", { class: "buttons" }, select,
        h("button", { class: "btn", type: "button", disabled: state.page === 0, onclick: go(0),
          text: "First" }),
        h("button", { class: "btn", type: "button", disabled: state.page === 0,
          onclick: go(state.page - 1),
          text: "Previous" }),
        h("span", { text: "Page " + num(state.page + 1) + " of " + num(pages) }),
        h("button", { class: "btn", type: "button", disabled: state.page >= pages - 1,
          onclick: go(state.page + 1), text: "Next" }),
        h("button", { class: "btn", type: "button", disabled: state.page >= pages - 1,
          onclick: go(pages - 1),
          text: "Last" })));
  }

  function pageSizes() {
    return [25, 50, 100, 250, 1000];
  }

  render();
  return {
    el: root,
    setRows(rows) {
      state.rows = rows;
      state.page = 0;
      render();
    },
    rows: () => sorted(state.rows),
  };
}

// ---------------------------------------------------------------------------------------
// A modal detail panel shared by every drill-down.

let dialog = null;

function showDetail(title, subtitle, body) {
  if (!dialog) {
    dialog = h("dialog", { class: "panel", "aria-labelledby": "panel-title" });
    dialog.addEventListener("close", () => {
      if (/^#(file|artifact)=/.test(location.hash)) {
        try {
          history.replaceState(null, "", location.pathname + location.search);
        } catch (error) {
          // Ignored: see remember().
        }
      }
    });
    dialog.addEventListener("click", (event) => {
      if (event.target === dialog) dialog.close();
    });
    document.body.appendChild(dialog);
  }
  dialog.textContent = "";
  dialog.append(
    h("div", { class: "panel-head" },
      h("div", null, subtitle ? h("span", { class: "eyebrow", text: subtitle }) : null,
        h("h2", { id: "panel-title", text: shown(title) })),
      h("button", { class: "btn", type: "button", text: "Close", onclick: () => dialog.close() })),
    h("div", { class: "panel-body" }, body));
  if (typeof dialog.showModal === "function") {
    if (!dialog.open) dialog.showModal();
  } else {
    dialog.setAttribute("open", "");
  }
}

function closeDetail() {
  if (dialog && dialog.open) dialog.close();
}
