// Starts the report: reads the data island, renders every section, and connects the
// drill-down links between them.

function boot() {
  const island = document.getElementById("ccm-data");
  const app = document.getElementById("app");
  let data;
  try {
    data = JSON.parse(island.textContent);
  } catch (error) {
    return;
  }
  if (!data || data.schema !== "ccm.html-data.v1") return;
  const model = loadModel(data);
  const view = {
    go(id) {
      const target = document.getElementById(id);
      if (target) target.scrollIntoView({ block: "start" });
    },
    files(filter) {
      closeDetail();
      if (view.setFileFilter) view.setFileFilter(filter || {});
      view.go("files");
    },
    file(file) {
      fileDetail(model, view, file);
      remember("file=" + encodeURIComponent(file.path));
    },
    dir(path) {
      if (view.focusDir) view.focusDir(path);
      view.go("directories");
    },
    artifact(id) {
      artifactDetail(model, view, id);
      remember("artifact=" + encodeURIComponent(data.model.artifacts[id]));
    },
    fileContext(file) {
      return fileContext(model, view, file);
    },
  };
  const sections = [
    ["overview", "Overview", null, renderOverview],
    ["languages", "Languages", model.languageRows.length, renderLanguages],
    ["directories", "Directories", countDirs(model.tree), renderDirectories],
    ["files", "Files", model.files.length, renderFiles],
  ];
  if (data.budget) sections.push(["estimate", "Estimate", null, renderEstimate]);
  if (data.model) sections.push(["change", "Change", data.model.affected.length, renderChange]);
  if (data.comparison) {
    sections.push(["comparison", "Comparison", data.comparison.files.length, renderComparison]);
  }
  sections.push(["notices", "Coverage", data.notices.length, renderNotices]);
  sections.push(["appendix", "Appendix", null, renderAppendix]);
  app.textContent = "";
  for (const [, , , render] of sections) app.appendChild(render(model, view));
  renderNav(sections);
  themeToggle();
  follow(model, view);
}

// Records the open drill-down in the address, so a link can reopen it.
function remember(fragment) {
  try {
    history.replaceState(null, "", "#" + fragment);
  } catch (error) {
    // Some local-file contexts forbid history changes; links then stay unchanged.
  }
}

// Opens the drill-down named by the address: #file=PATH, #dir=PATH or #artifact=NAME.
function follow(model, view) {
  const match = /^#(file|dir|artifact)=(.*)$/.exec(location.hash);
  if (!match) {
    // Sections are rendered after load, so the browser could not scroll to them itself.
    if (/^#[a-z]+$/.test(location.hash)) view.go(location.hash.slice(1));
    return;
  }
  let value = "";
  try {
    value = decodeURIComponent(match[2]);
  } catch (error) {
    return;
  }
  if (match[1] === "file" && model.byPath.has(value)) view.file(model.byPath.get(value));
  if (match[1] === "dir") view.dir(value);
  const m = model.data.model;
  if (match[1] === "artifact" && m && m.artifacts.includes(value)) {
    view.artifact(m.artifacts.indexOf(value));
  }
}

// Extra context for a file panel: its comparison row and its role in the change model.
function fileContext(model, view, file) {
  const parts = [];
  const c = model.data.comparison;
  const change = c ? c.files.find((row) => row[0] === file.path) : null;
  if (change) {
    parts.push(["Comparison", change[1] + ", code " + num(change[2] ? change[2].code : 0)
      + " -> " + num(change[3] ? change[3].code : 0)]);
  } else if (c) {
    parts.push(["Comparison", "unchanged"]);
  }
  const m = model.data.model;
  const id = m ? m.artifacts.indexOf(file.path) : -1;
  if (m && id >= 0) {
    const index = changeIndex(m);
    parts.push(["Model artifact", index.seeds.has(id) ? "seed" : index.affected.has(id)
      ? "affected" : "not affected"]);
  }
  if (!parts.length) return null;
  const wrap = h("div", null, h("h3", { text: "In this change" }), facts(parts));
  if (id >= 0) {
    wrap.appendChild(h("div", { class: "actions" }, h("button", { class: "btn", type: "button",
      text: "Open artifact", onclick: () => view.artifact(id) })));
  }
  return wrap;
}

function renderNav(sections) {
  const nav = document.getElementById("nav");
  const list = h("ul");
  const links = new Map();
  for (const [id, label, count] of sections) {
    const link = h("a", { href: "#" + id }, label,
      count === null ? null : h("span", { class: "count", text: num(count) }));
    links.set(id, link);
    list.appendChild(h("li", null, link));
  }
  nav.textContent = "";
  nav.appendChild(list);
  if (!("IntersectionObserver" in window)) return;
  const observer = new IntersectionObserver((entries) => {
    for (const entry of entries) {
      if (!entry.isIntersecting) continue;
      for (const link of links.values()) link.removeAttribute("aria-current");
      const link = links.get(entry.target.id);
      if (link) {
        link.setAttribute("aria-current", "true");
        if (list.scrollWidth > list.clientWidth) {
          list.scrollLeft = link.offsetLeft - (list.clientWidth - link.offsetWidth) / 2;
        }
      }
    }
  }, { rootMargin: "-20% 0px -70% 0px" });
  for (const [id] of sections) {
    const target = document.getElementById(id);
    if (target) observer.observe(target);
  }
}

// Light by default; dark under the system preference unless the reader chose a theme.
// The choice is shared with the CCM website through one storage key.
function themeToggle() {
  const header = document.querySelector(".masthead .brand");
  if (!header) return;
  const root = document.documentElement;
  const dark = () => root.dataset.theme === "dark" || (root.dataset.theme !== "light"
    && window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches);
  const button = h("button", { class: "btn theme no-print", type: "button" });
  const label = () => {
    button.textContent = dark() ? "Light theme" : "Dark theme";
    button.setAttribute("aria-pressed", String(dark()));
  };
  button.addEventListener("click", () => {
    const next = dark() ? "light" : "dark";
    root.dataset.theme = next;
    try {
      localStorage.setItem("guidedog-theme", next);
    } catch (error) {
      // Storage may be unavailable for local files; the choice then lasts for this view.
    }
    label();
  });
  label();
  header.appendChild(button);
}

boot();
