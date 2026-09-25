import { provide as Je, inject as Ke, ref as h, computed as re, onMounted as ie, watch as fe, onUnmounted as We, defineComponent as Z, openBlock as y, createElementBlock as z, createBlock as D, unref as e, withCtx as o, createVNode as a, createElementVNode as U, toDisplayString as i, createTextVNode as v, Fragment as Y, renderList as ee, createCommentVNode as Q, withKeys as Qe, isRef as X, withDirectives as te, reactive as Ge, renderSlot as Xe, vShow as me } from "vue";
import { createI18n as Ye } from "vue-i18n";
import { ElDialog as Se, ElForm as Te, ElFormItem as Ee, ElInput as ge, ElRadioGroup as ve, ElRadioButton as Ue, ElSelect as Me, ElOption as Ve, ElButton as de, ElDrawer as Ze, ElTag as _e, ElDivider as et, ElAlert as $e, ElTable as xe, ElTableColumn as Pe, ElLoadingDirective as ze, ElPagination as tt, ElRadio as at, ElRow as lt, ElCol as ot, ElCard as nt, ElStatistic as st, ElDescriptions as rt, ElDescriptionsItem as it, ElEmpty as dt, ElContainer as ut, ElAside as ct, ElIcon as mt, ElMenu as pt, ElMenuItem as ft, ElText as gt, ElMain as vt, ElTabs as _t, ElTabPane as yt } from "element-plus/es";
import { Plus as De, Search as Ie, Refresh as bt, Download as ht, UploadFilled as wt, Collection as kt, Notebook as Ct, PriceTag as St, Odometer as Tt } from "@element-plus/icons-vue";
import { ElMessage as B, ElMessageBox as Le } from "element-plus";
import { Marked as Et } from "marked";
import Ne from "dompurify";
const Ut = {
  nav: {
    memories: "记忆管理",
    tags: "标签管理",
    ops: "运维"
  },
  common: {
    detail: "详情",
    edit: "编辑",
    delete: "删除",
    cancel: "取消",
    save: "保存"
  },
  admin: {
    hint: "自托管 · 多 agent 共享"
  },
  memories: {
    title: "记忆管理",
    subtitle: "多 agent 共享的记忆库，正文支持 Markdown",
    create: "新建记忆",
    searchPlaceholder: "关键词搜索（空格分隔、全部命中；中文按子串匹配）",
    tagFilter: "按标签过滤",
    sortUpdated: "按更新时间",
    sortCreated: "按创建时间",
    orderDesc: "倒序",
    orderAsc: "正序",
    colId: "ID",
    colSummary: "摘要",
    colTags: "标签",
    colScore: "评分",
    colCreatedAt: "创建时间",
    colUpdatedAt: "更新时间",
    colActions: "操作",
    deleteTitle: "删除确认",
    deleteConfirm: "确定永久删除记忆 {id}？",
    deleted: "已删除"
  },
  tags: {
    title: "标签管理",
    subtitle: "标签是 agent 自主维护的分类体系；改名会同步更新所有引用它的记忆",
    create: "新建标签",
    colName: "名称",
    colDescription: "描述",
    colMemoryCount: "记忆数",
    colLastUsed: "最近使用",
    editTitle: "编辑标签",
    createTitle: "新建标签",
    nameLabel: "名称（唯一，≤100 字符）",
    namePlaceholder: "如 rust、项目、工作流",
    renameLabel: "改为新名称（留空表示不改名）",
    renamePlaceholder: "仅大小写改名也可用于合并拼写偏差",
    descLabel: "描述（可选，≤500 字符）",
    descPlaceholder: "这个标签用来组织什么内容",
    deleteTitle: "删除标签",
    deleteBefore: "标签",
    deleteMiddle: "当前被",
    deleteAfter: "条记忆使用。请选择删除方式：",
    detach: "仅摘除引用（保留全部记忆）",
    purge: "连带删除记忆（不可恢复）",
    purgeConfirmTitle: "高危操作确认",
    purgeConfirm: "将永久删除标签「{name}」及其关联的 {count} 条记忆，且不可恢复！",
    purgeButton: "永久删除",
    saved: "已保存",
    created: "已创建",
    deleted: "已删除"
  },
  ops: {
    title: "运维",
    subtitle: "数据库概况、备份导入导出与数据体检",
    refresh: "刷新概况",
    statMemories: "记忆条数",
    statTags: "标签数",
    statSize: "数据库大小",
    statNextId: "下一个记忆 ID",
    dbCard: "数据库",
    path: "文件路径",
    lastUpdate: "最近更新",
    lastUpdateValue: "{id}（{time}）",
    crossPlatform: "跨平台迁移",
    crossPlatformNote: "SQLite 文件格式平台无关，停服后可直接复制 .db 文件到其他机器；运行中请改用「导出备份」。",
    version: "版本",
    export: "导出备份（JSON）",
    import: "导入备份",
    importHint: "导入仅支持空数据库（导入是恢复而非合并）；当前库已有数据时请换一个空的 --db 路径再导入。",
    doctorCard: "数据体检（doctor）",
    runDoctor: "运行体检",
    doctorOk: "体检通过：未发现孤儿引用、大小写冲突或空字段",
    doctorFail: "发现 {count} 个问题",
    doctorEmpty: "点击「运行体检」检查孤儿标签引用、大小写冲突标签组、空摘要/正文",
    imported: "已导入 {memories} 条记忆、{tags} 个标签"
  },
  drawer: {
    title: "记忆 {id}",
    rendered: "渲染",
    source: "源码"
  },
  editor: {
    editTitle: "编辑记忆",
    createTitle: "新建记忆",
    summaryLabel: "摘要（列表与搜索展示的一行简介）",
    summaryPlaceholder: "精确、自洽的一句话",
    contentLabel: "正文（Markdown）",
    tabEdit: "编辑",
    tabPreview: "预览",
    contentPlaceholder: "支持 Markdown：标题、列表、代码块、表格……",
    tagsLabel: "标签（回车添加，可新建）",
    tagsPlaceholder: "选择或输入标签",
    updated: "已更新",
    created: "已创建"
  },
  errors: {
    http: "请求失败 (HTTP {status})",
    invalidBackup: "备份文件不是有效的 JSON"
  },
  shell: {
    theme: "主题",
    themeLight: "浅色",
    themeDark: "深色",
    themeSystem: "跟随系统",
    language: "语言"
  }
}, Mt = {
  nav: {
    memories: "Memories",
    tags: "Tags",
    ops: "Operations"
  },
  common: {
    detail: "Details",
    edit: "Edit",
    delete: "Delete",
    cancel: "Cancel",
    save: "Save"
  },
  admin: {
    hint: "Self-hosted · shared by multiple agents"
  },
  memories: {
    title: "Memories",
    subtitle: "A memory store shared by multiple agents; content supports Markdown",
    create: "New Memory",
    searchPlaceholder: "Keyword search (space-separated, all must match; Chinese matches by substring)",
    tagFilter: "Filter by tag",
    sortUpdated: "By updated time",
    sortCreated: "By created time",
    orderDesc: "Descending",
    orderAsc: "Ascending",
    colId: "ID",
    colSummary: "Summary",
    colTags: "Tags",
    colScore: "Score",
    colCreatedAt: "Created",
    colUpdatedAt: "Updated",
    colActions: "Actions",
    deleteTitle: "Confirm deletion",
    deleteConfirm: "Permanently delete memory {id}?",
    deleted: "Deleted"
  },
  tags: {
    title: "Tags",
    subtitle: "Tags are an agent-maintained taxonomy; renaming updates every memory that references the tag",
    create: "New Tag",
    colName: "Name",
    colDescription: "Description",
    colMemoryCount: "Memories",
    colLastUsed: "Last used",
    editTitle: "Edit Tag",
    createTitle: "New Tag",
    nameLabel: "Name (unique, ≤100 characters)",
    namePlaceholder: "e.g. rust, project, workflow",
    renameLabel: "Rename to (leave empty to keep unchanged)",
    renamePlaceholder: "Case-only renames also merge spelling drift",
    descLabel: "Description (optional, ≤500 characters)",
    descPlaceholder: "What is this tag used to organize",
    deleteTitle: "Delete Tag",
    deleteBefore: "Tag",
    deleteMiddle: "is used by",
    deleteAfter: "memories. Choose how to delete:",
    detach: "Detach references only (keep all memories)",
    purge: "Also delete the memories (irreversible)",
    purgeConfirmTitle: "Dangerous operation",
    purgeConfirm: 'This permanently deletes tag "{name}" and its {count} associated memories, irreversibly!',
    purgeButton: "Delete permanently",
    saved: "Saved",
    created: "Created",
    deleted: "Deleted"
  },
  ops: {
    title: "Operations",
    subtitle: "Database overview, backup import/export and health checks",
    refresh: "Refresh overview",
    statMemories: "Memories",
    statTags: "Tags",
    statSize: "DB size",
    statNextId: "Next memory ID",
    dbCard: "Database",
    path: "File path",
    lastUpdate: "Last update",
    lastUpdateValue: "{id} ({time})",
    crossPlatform: "Cross-platform migration",
    crossPlatformNote: 'The SQLite file format is platform-independent: copy the .db file to another machine while the server is stopped; use "Export backup" while running.',
    version: "Version",
    export: "Export backup (JSON)",
    import: "Import backup",
    importHint: "Import only works on an empty database (it restores rather than merges); point --db at an empty path before importing.",
    doctorCard: "Health check (doctor)",
    runDoctor: "Run check",
    doctorOk: "Check passed: no orphan references, case conflicts, or empty fields",
    doctorFail: "{count} issues found",
    doctorEmpty: 'Click "Run check" to look for orphan tag references, case-conflicting tags, empty summaries/content',
    imported: "Imported {memories} memories and {tags} tags"
  },
  drawer: {
    title: "Memory {id}",
    rendered: "Rendered",
    source: "Source"
  },
  editor: {
    editTitle: "Edit Memory",
    createTitle: "New Memory",
    summaryLabel: "Summary (one-line intro shown in lists and search)",
    summaryPlaceholder: "A precise, self-contained sentence",
    contentLabel: "Content (Markdown)",
    tabEdit: "Edit",
    tabPreview: "Preview",
    contentPlaceholder: "Markdown supported: headings, lists, code blocks, tables…",
    tagsLabel: "Tags (press Enter to add; new ones allowed)",
    tagsPlaceholder: "Select or type a tag",
    updated: "Updated",
    created: "Created"
  },
  errors: {
    http: "Request failed (HTTP {status})",
    invalidBackup: "Backup file is not valid JSON"
  },
  shell: {
    theme: "Theme",
    themeLight: "Light",
    themeDark: "Dark",
    themeSystem: "System",
    language: "Language"
  }
};
function Re() {
  var t;
  return typeof navigator > "u" || (t = navigator.language) != null && t.toLowerCase().startsWith("zh") ? "zh" : "en";
}
const ue = Ye({
  legacy: !1,
  locale: Re(),
  fallbackLocale: "zh",
  messages: { zh: Ut, en: Mt },
  // 面向宿主组件库，缺 key 时静默回退即可，不刷控制台
  missingWarn: !1,
  fallbackWarn: !1
}), { t: l } = ue.global;
function Vt(t) {
  ue.global.locale.value = t;
}
function $t(t) {
  Vt(t === "auto" ? Re() : t);
}
function Aa() {
  return ue.global.locale.value;
}
const Oe = Symbol("memory-ui-config"), se = {
  baseUrl: "",
  fetch: (...t) => globalThis.fetch(...t),
  defaultPageSize: 20,
  locale: "auto"
};
function Fa(t) {
  t.locale && $t(t.locale), Je(Oe, t);
}
function Be() {
  const t = Ke(Oe);
  return {
    baseUrl: ((t == null ? void 0 : t.baseUrl) ?? se.baseUrl).replace(/\/+$/, ""),
    fetch: (t == null ? void 0 : t.fetch) ?? se.fetch,
    defaultPageSize: (t == null ? void 0 : t.defaultPageSize) ?? se.defaultPageSize,
    locale: (t == null ? void 0 : t.locale) ?? se.locale
  };
}
function oe(t) {
  if (!t) return "—";
  const s = ue.global.locale.value === "en" ? "en-US" : "zh-CN";
  return new Date(t * 1e3).toLocaleString(s, { hour12: !1 });
}
function xt(t) {
  return t == null ? "—" : t < 1024 ? `${t} B` : t < 1024 * 1024 ? `${(t / 1024).toFixed(1)} KB` : `${(t / 1024 / 1024).toFixed(2)} MB`;
}
function Pt(t) {
  async function s(d, p = {}) {
    const f = await t.fetch(t.baseUrl + d, {
      headers: { "Content-Type": "application/json" },
      ...p
    }), b = await f.text();
    let r = null;
    try {
      r = b ? JSON.parse(b) : null;
    } catch {
      r = null;
    }
    if (!f.ok) {
      const S = (r == null ? void 0 : r.error) ?? l("errors.http", { status: f.status });
      throw new Error(S);
    }
    return r;
  }
  async function n(d) {
    var f;
    const p = await t.fetch(t.baseUrl + d);
    if (!p.ok) {
      const b = await p.text().catch(() => "");
      let r = l("errors.http", { status: p.status });
      try {
        r = ((f = JSON.parse(b)) == null ? void 0 : f.error) ?? r;
      } catch {
      }
      throw new Error(r);
    }
    return p.blob();
  }
  return {
    get: (d) => s(d),
    post: (d, p) => s(d, { method: "POST", body: JSON.stringify(p ?? {}) }),
    put: (d, p) => s(d, { method: "PUT", body: JSON.stringify(p ?? {}) }),
    del: (d) => s(d, { method: "DELETE" }),
    getBlob: n
  };
}
function ne() {
  return Pt(Be());
}
function zt(t) {
  const s = pe(t.query), n = new URLSearchParams();
  return s ? (n.set("query", t.query.trim()), t.tagFilter && n.set("tags", t.tagFilter)) : (t.tagFilter && n.set("tag", t.tagFilter), n.set("sort", t.sort), n.set("order", t.order)), n.set("offset", String((t.page - 1) * t.pageSize)), n.set("limit", String(t.pageSize)), n.toString();
}
function pe(t) {
  return t.trim().length > 0;
}
const Dt = new Et();
function It(t) {
  const s = Dt.parse(t, { async: !1 });
  return Ne.sanitize(s);
}
function Lt(t) {
  return Ne.sanitize(t);
}
function ye(t, s) {
  return t.get(`/api/memories/${encodeURIComponent(s)}`);
}
function Ae(t, s) {
  return t.post("/api/memories", s);
}
function Fe(t, s, n) {
  return t.put(`/api/memories/${encodeURIComponent(s)}`, n);
}
function Nt(t, s) {
  return t.del(`/api/memories/${encodeURIComponent(s)}`);
}
function He(t) {
  return t.get("/api/tags");
}
function Rt(t, s, n) {
  return t.post("/api/tags", { name: s, description: n });
}
function Ot(t, s, n, d) {
  const p = { description: d }, f = n.trim();
  return f && f !== s && (p.new_name = f), t.put(`/api/tags/${encodeURIComponent(s)}`, p);
}
function Bt(t, s, n) {
  return t.del(`/api/tags/${encodeURIComponent(s)}?mode=${n}`);
}
function At() {
  const t = ne(), { defaultPageSize: s } = Be(), n = h(""), d = h(""), p = h("updated_at"), f = h("desc"), b = h(1), r = h(s), S = h([]), P = h([]), E = h(0), g = h(""), w = h(!1), M = h([]), x = re(() => pe(n.value));
  function k() {
    return b.value = 1, F();
  }
  function I() {
    return zt({
      query: n.value,
      tagFilter: d.value,
      sort: p.value,
      order: f.value,
      page: b.value,
      pageSize: r.value
    });
  }
  let A = 0;
  async function F() {
    var C;
    const m = ++A;
    w.value = !0;
    try {
      const O = I();
      if (pe(n.value)) {
        const T = await t.get(`/api/memories?${O}`);
        if (m !== A) return;
        P.value = (T.results ?? []).map((L) => ({ ...L, snippet: Lt(L.snippet) })), E.value = T.total_matches ?? 0, g.value = "";
      } else {
        const T = await t.get(`/api/memories?${O}`);
        if (m !== A) return;
        const L = Math.max(1, Math.ceil(T.total / r.value));
        if (((C = T.memories) == null ? void 0 : C.length) === 0 && T.total > 0 && b.value > L)
          return b.value = L, w.value = !1, F();
        S.value = T.memories ?? [], E.value = T.total ?? 0, g.value = T.note ?? "";
      }
    } finally {
      m === A && (w.value = !1);
    }
  }
  async function R() {
    try {
      const m = await He(t);
      M.value = (m.tags ?? []).map((C) => C.name);
    } catch {
    }
  }
  async function V(m) {
    if (m.id) {
      const C = await ye(t, m.id), O = new Set(C.tags), T = new Set(m.tags);
      await Fe(t, m.id, {
        summary: m.summary,
        content: m.content,
        add_tags: [...T].filter((L) => !O.has(L)),
        remove_tags: [...O].filter((L) => !T.has(L))
      });
    } else
      await Ae(t, { summary: m.summary, content: m.content, tags: m.tags });
    await Promise.all([F(), R()]);
  }
  async function _(m) {
    await Nt(t, m), await F();
  }
  return ie(() => {
    F().catch(() => {
    }), R();
  }), {
    query: n,
    tagFilter: d,
    sort: p,
    order: f,
    page: b,
    pageSize: r,
    rows: S,
    searchResults: P,
    total: E,
    note: g,
    loading: w,
    tagOptions: M,
    searching: x,
    onSearch: k,
    reload: F,
    loadTagOptions: R,
    saveMemory: V,
    removeMemory: _
  };
}
function be(t, s = 720) {
  const n = h(0);
  let d = null;
  function p(f) {
    d == null || d.disconnect(), d = null, !(!f || typeof ResizeObserver > "u") && (d = new ResizeObserver((b) => {
      var r;
      n.value = ((r = b[0]) == null ? void 0 : r.contentRect.width) ?? 0;
    }), d.observe(f));
  }
  return ie(() => p(t.value)), fe(t, (f) => p(f)), We(() => d == null ? void 0 : d.disconnect()), { width: n, compact: re(() => n.value > 0 && n.value < s) };
}
const Ft = ["innerHTML"], qe = /* @__PURE__ */ Z({
  __name: "MarkdownView",
  props: {
    source: {}
  },
  setup(t) {
    const s = t, n = re(() => It(s.source));
    return (d, p) => (y(), z("div", {
      class: "md-body",
      innerHTML: n.value
    }, null, 8, Ft));
  }
}), Ht = { class: "content-label" }, qt = /* @__PURE__ */ Z({
  __name: "MemoryEditorDialog",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    tagOptions: {},
    width: {}
  },
  emits: ["update:visible", "saved"],
  setup(t, { emit: s }) {
    const n = t, d = s, p = ne(), f = h(!1), b = h("edit"), r = h({ id: null, summary: "", content: "", tags: [] });
    let S = [];
    fe(
      () => n.visible,
      async (E) => {
        if (E)
          if (b.value = "edit", n.memoryId)
            try {
              const g = await ye(p, n.memoryId);
              r.value = { id: g.id, summary: g.summary, content: g.content, tags: [...g.tags] }, S = [...g.tags];
            } catch (g) {
              B.error(g instanceof Error ? g.message : String(g)), d("update:visible", !1);
            }
          else
            r.value = { id: null, summary: "", content: "", tags: [] }, S = [];
      }
    );
    async function P() {
      f.value = !0;
      try {
        if (r.value.id) {
          const E = new Set(S), g = new Set(r.value.tags);
          await Fe(p, r.value.id, {
            summary: r.value.summary,
            content: r.value.content,
            add_tags: [...g].filter((w) => !E.has(w)),
            remove_tags: [...E].filter((w) => !g.has(w))
          }), B.success(l("editor.updated"));
        } else
          await Ae(p, {
            summary: r.value.summary,
            content: r.value.content,
            tags: r.value.tags
          }), B.success(l("editor.created"));
        d("update:visible", !1), d("saved");
      } catch (E) {
        B.error(E instanceof Error ? E.message : String(E));
      } finally {
        f.value = !1;
      }
    }
    return (E, g) => {
      const w = ge, M = Ee, x = Ue, k = ve, I = Ve, A = Me, F = Te, R = de, V = Se;
      return y(), D(V, {
        "model-value": t.visible,
        title: r.value.id ? e(l)("editor.editTitle") : e(l)("editor.createTitle"),
        width: t.width,
        "onUpdate:modelValue": g[5] || (g[5] = (_) => d("update:visible", _))
      }, {
        footer: o(() => [
          a(R, {
            onClick: g[4] || (g[4] = (_) => d("update:visible", !1))
          }, {
            default: o(() => [
              v(i(e(l)("common.cancel")), 1)
            ]),
            _: 1
          }),
          a(R, {
            type: "primary",
            loading: f.value,
            onClick: P
          }, {
            default: o(() => [
              v(i(e(l)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: o(() => [
          a(F, { "label-position": "top" }, {
            default: o(() => [
              a(M, {
                label: e(l)("editor.summaryLabel")
              }, {
                default: o(() => [
                  a(w, {
                    modelValue: r.value.summary,
                    "onUpdate:modelValue": g[0] || (g[0] = (_) => r.value.summary = _),
                    maxlength: "512",
                    "show-word-limit": "",
                    placeholder: e(l)("editor.summaryPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"]),
              a(M, null, {
                label: o(() => [
                  U("div", Ht, [
                    U("span", null, i(e(l)("editor.contentLabel")), 1),
                    a(k, {
                      modelValue: b.value,
                      "onUpdate:modelValue": g[1] || (g[1] = (_) => b.value = _),
                      size: "small"
                    }, {
                      default: o(() => [
                        a(x, { value: "edit" }, {
                          default: o(() => [
                            v(i(e(l)("editor.tabEdit")), 1)
                          ]),
                          _: 1
                        }),
                        a(x, { value: "preview" }, {
                          default: o(() => [
                            v(i(e(l)("editor.tabPreview")), 1)
                          ]),
                          _: 1
                        })
                      ]),
                      _: 1
                    }, 8, ["modelValue"])
                  ])
                ]),
                default: o(() => [
                  b.value === "edit" ? (y(), D(w, {
                    key: 0,
                    modelValue: r.value.content,
                    "onUpdate:modelValue": g[2] || (g[2] = (_) => r.value.content = _),
                    type: "textarea",
                    rows: 12,
                    maxlength: "200000",
                    "show-word-limit": "",
                    placeholder: e(l)("editor.contentPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])) : (y(), D(qe, {
                    key: 1,
                    class: "content-preview",
                    source: r.value.content
                  }, null, 8, ["source"]))
                ]),
                _: 1
              }),
              a(M, {
                label: e(l)("editor.tagsLabel")
              }, {
                default: o(() => [
                  a(A, {
                    modelValue: r.value.tags,
                    "onUpdate:modelValue": g[3] || (g[3] = (_) => r.value.tags = _),
                    multiple: "",
                    filterable: "",
                    "allow-create": "",
                    "default-first-option": "",
                    placeholder: e(l)("editor.tagsPlaceholder"),
                    class: "tags-select"
                  }, {
                    default: o(() => [
                      (y(!0), z(Y, null, ee(t.tagOptions, (_) => (y(), D(I, {
                        key: _,
                        label: _,
                        value: _
                      }, null, 8, ["label", "value"]))), 128))
                    ]),
                    _: 1
                  }, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"])
            ]),
            _: 1
          })
        ]),
        _: 1
      }, 8, ["model-value", "title", "width"]);
    };
  }
}), ae = (t, s) => {
  const n = t.__vccOpts || t;
  for (const [d, p] of s)
    n[d] = p;
  return n;
}, jt = /* @__PURE__ */ ae(qt, [["__scopeId", "data-v-602bada0"]]), Jt = { class: "detail-summary" }, Kt = { class: "detail-tags" }, Wt = { class: "detail-toolbar" }, Qt = {
  key: 1,
  class: "detail-content"
}, Gt = /* @__PURE__ */ Z({
  __name: "MemoryDetailDrawer",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    size: {}
  },
  emits: ["update:visible"],
  setup(t, { emit: s }) {
    const n = t, d = s, p = ne(), f = h(null), b = h("rendered");
    return fe(
      () => [n.visible, n.memoryId],
      async ([r]) => {
        if (!(!r || !n.memoryId)) {
          b.value = "rendered";
          try {
            f.value = await ye(p, n.memoryId);
          } catch (S) {
            B.error(S instanceof Error ? S.message : String(S)), d("update:visible", !1);
          }
        }
      }
    ), (r, S) => {
      var x;
      const P = _e, E = et, g = Ue, w = ve, M = Ze;
      return y(), D(M, {
        "model-value": t.visible,
        title: e(l)("drawer.title", { id: ((x = f.value) == null ? void 0 : x.id) ?? t.memoryId ?? "" }),
        size: t.size,
        "onUpdate:modelValue": S[1] || (S[1] = (k) => d("update:visible", k))
      }, {
        default: o(() => [
          f.value ? (y(), z(Y, { key: 0 }, [
            U("h3", Jt, i(f.value.summary), 1),
            U("div", Kt, [
              (y(!0), z(Y, null, ee(f.value.tags, (k) => (y(), D(P, {
                key: k,
                size: "small",
                class: "am-tag"
              }, {
                default: o(() => [
                  v(i(k), 1)
                ]),
                _: 2
              }, 1024))), 128))
            ]),
            a(E),
            U("div", Wt, [
              a(w, {
                modelValue: b.value,
                "onUpdate:modelValue": S[0] || (S[0] = (k) => b.value = k),
                size: "small"
              }, {
                default: o(() => [
                  a(g, { value: "rendered" }, {
                    default: o(() => [
                      v(i(e(l)("drawer.rendered")), 1)
                    ]),
                    _: 1
                  }),
                  a(g, { value: "source" }, {
                    default: o(() => [
                      v(i(e(l)("drawer.source")), 1)
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["modelValue"])
            ]),
            b.value === "rendered" ? (y(), D(qe, {
              key: 0,
              source: f.value.content
            }, null, 8, ["source"])) : (y(), z("pre", Qt, i(f.value.content), 1))
          ], 64)) : Q("", !0)
        ]),
        _: 1
      }, 8, ["model-value", "title", "size"]);
    };
  }
}), Xt = /* @__PURE__ */ ae(Gt, [["__scopeId", "data-v-b634d116"]]), Yt = {
  key: 0,
  class: "am-panel-header"
}, Zt = { class: "am-panel-title" }, ea = { class: "am-panel-subtitle" }, ta = { class: "am-toolbar" }, aa = { class: "am-summary" }, la = ["innerHTML"], oa = {
  key: 4,
  class: "am-pager"
}, na = {
  key: 5,
  class: "am-pager"
}, sa = /* @__PURE__ */ Z({
  __name: "MemoriesPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t) {
    const s = t, {
      query: n,
      tagFilter: d,
      sort: p,
      order: f,
      page: b,
      pageSize: r,
      rows: S,
      searchResults: P,
      total: E,
      note: g,
      loading: w,
      tagOptions: M,
      searching: x,
      onSearch: k,
      reload: I,
      loadTagOptions: A,
      removeMemory: F
    } = At(), R = h(null), { compact: V } = be(R), _ = h(!1), m = h(null), C = h(!1), O = h(null);
    function T(N) {
      return N().catch((c) => B.error(c instanceof Error ? c.message : String(c)));
    }
    function L() {
      m.value = null, _.value = !0;
    }
    function G(N) {
      m.value = N, _.value = !0;
    }
    function K(N) {
      O.value = N, C.value = !0;
    }
    function le() {
      T(I), A();
    }
    async function j(N) {
      try {
        await Le.confirm(l("memories.deleteConfirm", { id: N.id }), l("memories.deleteTitle"), {
          type: "warning"
        });
      } catch {
        return;
      }
      try {
        await F(N.id), B.success(l("memories.deleted"));
      } catch (c) {
        B.error(c instanceof Error ? c.message : String(c));
      }
    }
    return (N, c) => {
      const W = de, $ = ge, H = Ve, ce = Me, je = $e, q = Pe, he = _e, we = xe, ke = tt, Ce = ze;
      return y(), z("div", {
        ref_key: "rootRef",
        ref: R,
        class: "am-panel"
      }, [
        t.showHeader ? (y(), z("div", Yt, [
          U("div", null, [
            U("h2", Zt, i(s.title ?? e(l)("memories.title")), 1),
            U("p", ea, i(s.subtitle ?? e(l)("memories.subtitle")), 1)
          ]),
          a(W, {
            type: "primary",
            icon: e(De),
            onClick: L
          }, {
            default: o(() => [
              v(i(e(l)("memories.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : Q("", !0),
        U("div", ta, [
          a($, {
            modelValue: e(n),
            "onUpdate:modelValue": c[1] || (c[1] = (u) => X(n) ? n.value = u : null),
            placeholder: e(l)("memories.searchPlaceholder"),
            clearable: "",
            class: "search",
            onKeyup: c[2] || (c[2] = Qe((u) => T(e(k)), ["enter"])),
            onClear: c[3] || (c[3] = (u) => T(e(k)))
          }, {
            append: o(() => [
              a(W, {
                icon: e(Ie),
                onClick: c[0] || (c[0] = (u) => T(e(k)))
              }, null, 8, ["icon"])
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(ce, {
            modelValue: e(d),
            "onUpdate:modelValue": c[4] || (c[4] = (u) => X(d) ? d.value = u : null),
            placeholder: e(l)("memories.tagFilter"),
            clearable: "",
            filterable: "",
            class: "tag-filter",
            onChange: c[5] || (c[5] = (u) => T(e(k)))
          }, {
            default: o(() => [
              (y(!0), z(Y, null, ee(e(M), (u) => (y(), D(H, {
                key: u,
                label: u,
                value: u
              }, null, 8, ["label", "value"]))), 128))
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(ce, {
            modelValue: e(p),
            "onUpdate:modelValue": c[6] || (c[6] = (u) => X(p) ? p.value = u : null),
            class: "sort",
            onChange: c[7] || (c[7] = (u) => T(e(I)))
          }, {
            default: o(() => [
              a(H, {
                label: e(l)("memories.sortUpdated"),
                value: "updated_at"
              }, null, 8, ["label"]),
              a(H, {
                label: e(l)("memories.sortCreated"),
                value: "created_at"
              }, null, 8, ["label"])
            ]),
            _: 1
          }, 8, ["modelValue"]),
          a(ce, {
            modelValue: e(f),
            "onUpdate:modelValue": c[8] || (c[8] = (u) => X(f) ? f.value = u : null),
            class: "order",
            onChange: c[9] || (c[9] = (u) => T(e(I)))
          }, {
            default: o(() => [
              a(H, {
                label: e(l)("memories.orderDesc"),
                value: "desc"
              }, null, 8, ["label"]),
              a(H, {
                label: e(l)("memories.orderAsc"),
                value: "asc"
              }, null, 8, ["label"])
            ]),
            _: 1
          }, 8, ["modelValue"])
        ]),
        e(g) ? (y(), D(je, {
          key: 1,
          title: e(g),
          type: "info",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : Q("", !0),
        e(x) ? te((y(), D(we, {
          key: 2,
          data: e(P)
        }, {
          default: o(() => [
            a(q, {
              prop: "id",
              label: e(l)("memories.colId"),
              width: "80"
            }, null, 8, ["label"]),
            a(q, {
              label: e(l)("memories.colSummary")
            }, {
              default: o(({ row: u }) => [
                U("div", aa, i(u.summary), 1),
                U("div", {
                  class: "am-snippet",
                  innerHTML: u.snippet
                }, null, 8, la)
              ]),
              _: 1
            }, 8, ["label"]),
            a(q, {
              label: e(l)("memories.colTags"),
              width: "220"
            }, {
              default: o(({ row: u }) => [
                (y(!0), z(Y, null, ee(u.tags, (J) => (y(), D(he, {
                  key: J,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: o(() => [
                    v(i(J), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            a(q, {
              prop: "score",
              label: e(l)("memories.colScore"),
              width: "80",
              sortable: ""
            }, null, 8, ["label"]),
            a(q, {
              label: e(l)("memories.colUpdatedAt"),
              width: "170"
            }, {
              default: o(({ row: u }) => [
                v(i(e(oe)(u.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(q, {
              label: e(l)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: o(({ row: u }) => [
                a(W, {
                  link: "",
                  type: "primary",
                  onClick: (J) => K(u.id)
                }, {
                  default: o(() => [
                    v(i(e(l)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(W, {
                  link: "",
                  type: "primary",
                  onClick: (J) => G(u.id)
                }, {
                  default: o(() => [
                    v(i(e(l)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(W, {
                  link: "",
                  type: "danger",
                  onClick: (J) => j(u)
                }, {
                  default: o(() => [
                    v(i(e(l)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [Ce, e(w)]
        ]) : te((y(), D(we, {
          key: 3,
          data: e(S)
        }, {
          default: o(() => [
            a(q, {
              prop: "id",
              label: e(l)("memories.colId"),
              width: "80"
            }, null, 8, ["label"]),
            a(q, {
              prop: "summary",
              label: e(l)("memories.colSummary"),
              "min-width": "300",
              "show-overflow-tooltip": ""
            }, null, 8, ["label"]),
            a(q, {
              label: e(l)("memories.colTags"),
              width: "220"
            }, {
              default: o(({ row: u }) => [
                (y(!0), z(Y, null, ee(u.tags, (J) => (y(), D(he, {
                  key: J,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: o(() => [
                    v(i(J), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            e(V) ? Q("", !0) : (y(), D(q, {
              key: 0,
              prop: "created_at",
              label: e(l)("memories.colCreatedAt"),
              width: "170"
            }, {
              default: o(({ row: u }) => [
                v(i(e(oe)(u.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label"])),
            a(q, {
              label: e(l)("memories.colUpdatedAt"),
              width: "170"
            }, {
              default: o(({ row: u }) => [
                v(i(e(oe)(u.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(q, {
              label: e(l)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: o(({ row: u }) => [
                a(W, {
                  link: "",
                  type: "primary",
                  onClick: (J) => K(u.id)
                }, {
                  default: o(() => [
                    v(i(e(l)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(W, {
                  link: "",
                  type: "primary",
                  onClick: (J) => G(u.id)
                }, {
                  default: o(() => [
                    v(i(e(l)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(W, {
                  link: "",
                  type: "danger",
                  onClick: (J) => j(u)
                }, {
                  default: o(() => [
                    v(i(e(l)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [Ce, e(w)]
        ]),
        e(x) ? (y(), z("div", na, [
          a(ke, {
            "current-page": e(b),
            "onUpdate:currentPage": c[14] || (c[14] = (u) => X(b) ? b.value = u : null),
            "page-size": e(r),
            "onUpdate:pageSize": c[15] || (c[15] = (u) => X(r) ? r.value = u : null),
            total: e(E),
            "page-sizes": [10, 20, 50],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: c[16] || (c[16] = (u) => T(e(I))),
            onSizeChange: c[17] || (c[17] = (u) => T(e(I)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])) : (y(), z("div", oa, [
          a(ke, {
            "current-page": e(b),
            "onUpdate:currentPage": c[10] || (c[10] = (u) => X(b) ? b.value = u : null),
            "page-size": e(r),
            "onUpdate:pageSize": c[11] || (c[11] = (u) => X(r) ? r.value = u : null),
            total: e(E),
            "page-sizes": [20, 50, 100, 200],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: c[12] || (c[12] = (u) => T(e(I))),
            onSizeChange: c[13] || (c[13] = (u) => T(e(I)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])),
        a(jt, {
          visible: _.value,
          "onUpdate:visible": c[18] || (c[18] = (u) => _.value = u),
          "memory-id": m.value,
          "tag-options": e(M),
          width: e(V) ? "96%" : "640px",
          onSaved: le
        }, null, 8, ["visible", "memory-id", "tag-options", "width"]),
        a(Xt, {
          visible: C.value,
          "onUpdate:visible": c[19] || (c[19] = (u) => C.value = u),
          "memory-id": O.value,
          size: e(V) ? "100%" : "45%"
        }, null, 8, ["visible", "memory-id", "size"])
      ], 512);
    };
  }
}), ra = /* @__PURE__ */ ae(sa, [["__scopeId", "data-v-69f47562"]]);
function ia() {
  const t = ne(), s = h([]), n = h(!1);
  async function d() {
    n.value = !0;
    try {
      const r = await He(t);
      s.value = r.tags ?? [];
    } finally {
      n.value = !1;
    }
  }
  async function p(r, S) {
    await Rt(t, r, S), await d();
  }
  async function f(r, S, P) {
    await Ot(t, r, S, P), await d();
  }
  async function b(r, S) {
    await Bt(t, r, S), await d();
  }
  return ie(() => {
    d().catch(() => {
    });
  }), { rows: s, loading: n, reload: d, create: p, rename: f, remove: b };
}
const da = {
  key: 0,
  class: "am-panel-header"
}, ua = { class: "am-panel-title" }, ca = { class: "am-panel-subtitle" }, ma = { class: "delete-body" }, pa = /* @__PURE__ */ Z({
  __name: "TagsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t) {
    const s = t, { rows: n, loading: d, reload: p, create: f, rename: b, remove: r } = ia(), S = h(null), { compact: P } = be(S), E = h(!1), g = h(!1), w = Ge({ oldName: null, name: "", newName: "", description: "" }), M = h(!1), x = h("detach"), k = h(null);
    function I() {
      Object.assign(w, { oldName: null, name: "", newName: "", description: "" }), g.value = !0;
    }
    function A(_) {
      Object.assign(w, { oldName: _.name, name: _.name, newName: "", description: _.description ?? "" }), g.value = !0;
    }
    async function F() {
      E.value = !0;
      try {
        w.oldName ? (await b(w.oldName, w.newName, w.description), B.success(l("tags.saved"))) : (await f(w.name, w.description), B.success(l("tags.created"))), g.value = !1;
      } catch (_) {
        B.error(_ instanceof Error ? _.message : String(_));
      } finally {
        E.value = !1;
      }
    }
    function R(_) {
      k.value = _, x.value = "detach", M.value = !0;
    }
    async function V() {
      var _, m;
      if (x.value === "purge")
        try {
          await Le.confirm(
            l("tags.purgeConfirm", { name: (_ = k.value) == null ? void 0 : _.name, count: ((m = k.value) == null ? void 0 : m.memory_count) ?? 0 }),
            l("tags.purgeConfirmTitle"),
            { type: "error", confirmButtonText: l("tags.purgeButton") }
          );
        } catch {
          return;
        }
      if (k.value) {
        E.value = !0;
        try {
          await r(k.value.name, x.value), B.success(l("tags.deleted")), M.value = !1;
        } catch (C) {
          B.error(C instanceof Error ? C.message : String(C));
        } finally {
          E.value = !1;
        }
      }
    }
    return (_, m) => {
      const C = de, O = _e, T = Pe, L = xe, G = ge, K = Ee, le = Te, j = Se, N = at, c = ve, W = ze;
      return y(), z("div", {
        ref_key: "rootRef",
        ref: S,
        class: "am-panel"
      }, [
        t.showHeader ? (y(), z("div", da, [
          U("div", null, [
            U("h2", ua, i(s.title ?? e(l)("tags.title")), 1),
            U("p", ca, i(s.subtitle ?? e(l)("tags.subtitle")), 1)
          ]),
          a(C, {
            type: "primary",
            icon: e(De),
            onClick: I
          }, {
            default: o(() => [
              v(i(e(l)("tags.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : Q("", !0),
        te((y(), D(L, { data: e(n) }, {
          default: o(() => [
            a(T, {
              prop: "name",
              label: e(l)("tags.colName"),
              "min-width": "160"
            }, {
              default: o(({ row: $ }) => [
                a(O, null, {
                  default: o(() => [
                    v(i($.name), 1)
                  ]),
                  _: 2
                }, 1024)
              ]),
              _: 1
            }, 8, ["label"]),
            a(T, {
              prop: "description",
              label: e(l)("tags.colDescription"),
              "min-width": "300",
              "show-overflow-tooltip": ""
            }, {
              default: o(({ row: $ }) => [
                v(i($.description || "—"), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(T, {
              prop: "memory_count",
              label: e(l)("tags.colMemoryCount"),
              width: "100",
              sortable: ""
            }, null, 8, ["label"]),
            a(T, {
              label: e(l)("tags.colLastUsed"),
              width: "170"
            }, {
              default: o(({ row: $ }) => [
                v(i(e(oe)($.last_used_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(T, {
              label: e(l)("memories.colActions"),
              width: "150",
              fixed: "right"
            }, {
              default: o(({ row: $ }) => [
                a(C, {
                  link: "",
                  type: "primary",
                  onClick: (H) => A($)
                }, {
                  default: o(() => [
                    v(i(e(l)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(C, {
                  link: "",
                  type: "danger",
                  onClick: (H) => R($)
                }, {
                  default: o(() => [
                    v(i(e(l)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [W, e(d)]
        ]),
        a(j, {
          modelValue: g.value,
          "onUpdate:modelValue": m[4] || (m[4] = ($) => g.value = $),
          title: w.oldName ? e(l)("tags.editTitle") : e(l)("tags.createTitle"),
          width: e(P) ? "96%" : "480px"
        }, {
          footer: o(() => [
            a(C, {
              onClick: m[3] || (m[3] = ($) => g.value = !1)
            }, {
              default: o(() => [
                v(i(e(l)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(C, {
              type: "primary",
              loading: E.value,
              onClick: F
            }, {
              default: o(() => [
                v(i(e(l)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: o(() => [
            a(le, { "label-position": "top" }, {
              default: o(() => [
                a(K, {
                  label: e(l)("tags.nameLabel")
                }, {
                  default: o(() => [
                    a(G, {
                      modelValue: w.name,
                      "onUpdate:modelValue": m[0] || (m[0] = ($) => w.name = $),
                      disabled: !!w.oldName,
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(l)("tags.namePlaceholder")
                    }, null, 8, ["modelValue", "disabled", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                w.oldName ? (y(), D(K, {
                  key: 0,
                  label: e(l)("tags.renameLabel")
                }, {
                  default: o(() => [
                    a(G, {
                      modelValue: w.newName,
                      "onUpdate:modelValue": m[1] || (m[1] = ($) => w.newName = $),
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(l)("tags.renamePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"])) : Q("", !0),
                a(K, {
                  label: e(l)("tags.descLabel")
                }, {
                  default: o(() => [
                    a(G, {
                      modelValue: w.description,
                      "onUpdate:modelValue": m[2] || (m[2] = ($) => w.description = $),
                      type: "textarea",
                      rows: 3,
                      maxlength: "500",
                      "show-word-limit": "",
                      placeholder: e(l)("tags.descPlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"])
              ]),
              _: 1
            })
          ]),
          _: 1
        }, 8, ["modelValue", "title", "width"]),
        a(j, {
          modelValue: M.value,
          "onUpdate:modelValue": m[7] || (m[7] = ($) => M.value = $),
          title: e(l)("tags.deleteTitle"),
          width: e(P) ? "96%" : "480px"
        }, {
          footer: o(() => [
            a(C, {
              onClick: m[6] || (m[6] = ($) => M.value = !1)
            }, {
              default: o(() => [
                v(i(e(l)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(C, {
              type: "danger",
              loading: E.value,
              onClick: V
            }, {
              default: o(() => [
                v(i(e(l)("common.delete")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: o(() => {
            var $;
            return [
              U("p", ma, [
                v(i(e(l)("tags.deleteBefore")) + " ", 1),
                a(O, null, {
                  default: o(() => {
                    var H;
                    return [
                      v(i((H = k.value) == null ? void 0 : H.name), 1)
                    ];
                  }),
                  _: 1
                }),
                v(" " + i(e(l)("tags.deleteMiddle")) + " ", 1),
                U("b", null, i(($ = k.value) == null ? void 0 : $.memory_count), 1),
                v(" " + i(e(l)("tags.deleteAfter")), 1)
              ]),
              a(c, {
                modelValue: x.value,
                "onUpdate:modelValue": m[5] || (m[5] = (H) => x.value = H)
              }, {
                default: o(() => [
                  a(N, { value: "detach" }, {
                    default: o(() => [
                      v(i(e(l)("tags.detach")), 1)
                    ]),
                    _: 1
                  }),
                  a(N, { value: "purge" }, {
                    default: o(() => [
                      v(i(e(l)("tags.purge")), 1)
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["modelValue"])
            ];
          }),
          _: 1
        }, 8, ["modelValue", "title", "width"])
      ], 512);
    };
  }
}), fa = /* @__PURE__ */ ae(pa, [["__scopeId", "data-v-6a537258"]]);
function ga(t) {
  return t.get("/api/stats");
}
function va(t) {
  return t.get("/health");
}
function _a(t) {
  return t.get("/api/doctor");
}
function ya(t) {
  return t.getBlob("/api/export");
}
function ba(t, s) {
  return t.post("/api/import", s);
}
function ha() {
  const t = ne(), s = h({}), n = h(""), d = h({ ok: !0, issues: [] }), p = h(!1), f = h(!1), b = h(!1), r = h(!1), S = re(() => xt(s.value.file_size));
  async function P() {
    s.value = await ga(t), s.value.version = n.value;
  }
  async function E() {
    f.value = !0;
    try {
      d.value = await _a(t), p.value = !0;
    } finally {
      f.value = !1;
    }
  }
  async function g() {
    b.value = !0;
    try {
      const M = await ya(t), x = URL.createObjectURL(M), k = document.createElement("a");
      k.href = x, k.download = "agent-memory-export.json", k.click(), URL.revokeObjectURL(x);
    } finally {
      b.value = !1;
    }
  }
  async function w(M) {
    r.value = !0;
    try {
      const x = await M.text();
      let k;
      try {
        k = JSON.parse(x);
      } catch {
        throw new Error(l("errors.invalidBackup"));
      }
      const I = await ba(t, k);
      return await P(), I;
    } finally {
      r.value = !1;
    }
  }
  return ie(async () => {
    try {
      n.value = (await va(t)).version ?? "";
    } catch {
    }
    await P().catch(() => {
    });
  }), {
    stats: s,
    version: n,
    doctor: d,
    doctorRan: p,
    doctorLoading: f,
    exporting: b,
    importing: r,
    sizeText: S,
    reload: P,
    runDoctor: E,
    exportData: g,
    importFile: w
  };
}
const wa = {
  key: 0,
  class: "am-panel-header"
}, ka = { class: "am-panel-title" }, Ca = { class: "am-panel-subtitle" }, Sa = { class: "actions" }, Ta = { class: "card-header" }, Ea = {
  key: 2,
  class: "issues"
}, Ua = /* @__PURE__ */ Z({
  __name: "OpsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t) {
    const s = t, {
      stats: n,
      version: d,
      doctor: p,
      doctorRan: f,
      doctorLoading: b,
      exporting: r,
      importing: S,
      sizeText: P,
      reload: E,
      runDoctor: g,
      exportData: w,
      importFile: M
    } = ha(), x = h(null), { compact: k } = be(x), I = h(null);
    function A(R) {
      return R().catch((V) => B.error(V instanceof Error ? V.message : String(V)));
    }
    function F(R) {
      var m;
      const V = R.target, _ = (m = V.files) == null ? void 0 : m[0];
      V.value = "", _ && M(_).then((C) => {
        B.success(l("ops.imported", { memories: C.imported_memories, tags: C.imported_tags }));
      }).catch((C) => {
        B.error(C instanceof Error ? C.message : String(C));
      });
    }
    return (R, V) => {
      const _ = de, m = st, C = nt, O = ot, T = lt, L = it, G = rt, K = $e, le = dt;
      return y(), z("div", {
        ref_key: "rootRef",
        ref: x,
        class: "am-panel"
      }, [
        t.showHeader ? (y(), z("div", wa, [
          U("div", null, [
            U("h2", ka, i(s.title ?? e(l)("ops.title")), 1),
            U("p", Ca, i(s.subtitle ?? e(l)("ops.subtitle")), 1)
          ]),
          a(_, {
            icon: e(bt),
            onClick: V[0] || (V[0] = (j) => A(e(E)))
          }, {
            default: o(() => [
              v(i(e(l)("ops.refresh")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : Q("", !0),
        a(T, { gutter: 14 }, {
          default: o(() => [
            a(O, {
              span: e(k) ? 12 : 6
            }, {
              default: o(() => [
                a(C, { shadow: "never" }, {
                  default: o(() => [
                    a(m, {
                      title: e(l)("ops.statMemories"),
                      value: e(n).memories ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(O, {
              span: e(k) ? 12 : 6
            }, {
              default: o(() => [
                a(C, { shadow: "never" }, {
                  default: o(() => [
                    a(m, {
                      title: e(l)("ops.statTags"),
                      value: e(n).tags ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(O, {
              span: e(k) ? 12 : 6
            }, {
              default: o(() => [
                a(C, { shadow: "never" }, {
                  default: o(() => [
                    a(m, {
                      title: e(l)("ops.statSize"),
                      value: e(P)
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(O, {
              span: e(k) ? 12 : 6
            }, {
              default: o(() => [
                a(C, { shadow: "never" }, {
                  default: o(() => [
                    a(m, {
                      title: e(l)("ops.statNextId"),
                      value: e(n).next_id ?? "—"
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"])
          ]),
          _: 1
        }),
        a(C, { shadow: "never" }, {
          header: o(() => [
            v(i(e(l)("ops.dbCard")), 1)
          ]),
          default: o(() => [
            a(G, {
              column: e(k) ? 1 : 2,
              border: ""
            }, {
              default: o(() => [
                a(L, {
                  label: e(l)("ops.path")
                }, {
                  default: o(() => [
                    v(i(e(n).path ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(L, {
                  label: e(l)("ops.lastUpdate")
                }, {
                  default: o(() => [
                    v(i(e(n).newest_update ? e(l)("ops.lastUpdateValue", {
                      id: e(n).newest_update.id,
                      time: e(oe)(e(n).newest_update.updated_at)
                    }) : "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(L, {
                  label: e(l)("ops.crossPlatform")
                }, {
                  default: o(() => [
                    v(i(e(l)("ops.crossPlatformNote")), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(L, {
                  label: e(l)("ops.version")
                }, {
                  default: o(() => [
                    v(i(e(n).version ?? e(d)), 1)
                  ]),
                  _: 1
                }, 8, ["label"])
              ]),
              _: 1
            }, 8, ["column"]),
            U("div", Sa, [
              a(_, {
                icon: e(ht),
                loading: e(r),
                onClick: V[1] || (V[1] = (j) => A(e(w)))
              }, {
                default: o(() => [
                  v(i(e(l)("ops.export")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(_, {
                icon: e(wt),
                loading: e(S),
                onClick: V[2] || (V[2] = (j) => {
                  var N;
                  return (N = I.value) == null ? void 0 : N.click();
                })
              }, {
                default: o(() => [
                  v(i(e(l)("ops.import")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              U("input", {
                ref_key: "importInput",
                ref: I,
                type: "file",
                accept: "application/json,.json",
                style: { display: "none" },
                onChange: F
              }, null, 544)
            ]),
            a(K, {
              title: e(l)("ops.importHint"),
              type: "info",
              "show-icon": "",
              closable: !1,
              class: "import-hint"
            }, null, 8, ["title"])
          ]),
          _: 1
        }),
        a(C, { shadow: "never" }, {
          header: o(() => [
            U("div", Ta, [
              U("span", null, i(e(l)("ops.doctorCard")), 1),
              a(_, {
                size: "small",
                icon: e(Ie),
                loading: e(b),
                onClick: V[3] || (V[3] = (j) => A(e(g)))
              }, {
                default: o(() => [
                  v(i(e(l)("ops.runDoctor")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"])
            ])
          ]),
          default: o(() => [
            e(f) ? (y(), z(Y, { key: 0 }, [
              e(p).ok ? (y(), D(K, {
                key: 0,
                title: e(l)("ops.doctorOk"),
                type: "success",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])) : (y(), D(K, {
                key: 1,
                title: e(l)("ops.doctorFail", { count: e(p).issues.length }),
                type: "error",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])),
              e(p).ok ? Q("", !0) : (y(), z("ul", Ea, [
                (y(!0), z(Y, null, ee(e(p).issues, (j, N) => (y(), z("li", { key: N }, i(j), 1))), 128))
              ]))
            ], 64)) : (y(), D(le, {
              key: 1,
              description: e(l)("ops.doctorEmpty"),
              "image-size": 60
            }, null, 8, ["description"]))
          ]),
          _: 1
        })
      ], 512);
    };
  }
}), Ma = /* @__PURE__ */ ae(Ua, [["__scopeId", "data-v-28e9c5e6"]]), Va = { class: "memory-ui" }, $a = { class: "brand" }, xa = { class: "brand-mark" }, Pa = { class: "aside-footer" }, za = /* @__PURE__ */ Z({
  __name: "MemoryAdmin",
  props: {
    layout: { default: "sidebar" },
    title: { default: "agent-memory" }
  },
  setup(t) {
    const s = h("memories");
    return (n, d) => {
      const p = mt, f = ft, b = pt, r = gt, S = ct, P = yt, E = _t, g = vt, w = ut;
      return y(), z("div", Va, [
        a(w, { class: "layout" }, {
          default: o(() => [
            t.layout === "sidebar" ? (y(), D(S, {
              key: 0,
              width: "200px",
              class: "aside"
            }, {
              default: o(() => [
                U("div", $a, [
                  U("span", xa, [
                    a(p, { size: 16 }, {
                      default: o(() => [
                        a(e(kt))
                      ]),
                      _: 1
                    })
                  ]),
                  U("span", null, i(t.title), 1)
                ]),
                a(b, {
                  "default-active": s.value,
                  class: "menu",
                  onSelect: d[0] || (d[0] = (M) => s.value = M)
                }, {
                  default: o(() => [
                    a(f, { index: "memories" }, {
                      default: o(() => [
                        a(p, null, {
                          default: o(() => [
                            a(e(Ct))
                          ]),
                          _: 1
                        }),
                        U("span", null, i(e(l)("nav.memories")), 1)
                      ]),
                      _: 1
                    }),
                    a(f, { index: "tags" }, {
                      default: o(() => [
                        a(p, null, {
                          default: o(() => [
                            a(e(St))
                          ]),
                          _: 1
                        }),
                        U("span", null, i(e(l)("nav.tags")), 1)
                      ]),
                      _: 1
                    }),
                    a(f, { index: "ops" }, {
                      default: o(() => [
                        a(p, null, {
                          default: o(() => [
                            a(e(Tt))
                          ]),
                          _: 1
                        }),
                        U("span", null, i(e(l)("nav.ops")), 1)
                      ]),
                      _: 1
                    })
                  ]),
                  _: 1
                }, 8, ["default-active"]),
                U("div", Pa, [
                  Xe(n.$slots, "footer", {}, () => [
                    a(r, {
                      size: "small",
                      type: "info"
                    }, {
                      default: o(() => [
                        v(i(e(l)("admin.hint")), 1)
                      ]),
                      _: 1
                    })
                  ], !0)
                ])
              ]),
              _: 3
            })) : Q("", !0),
            a(g, { class: "main" }, {
              default: o(() => [
                t.layout === "tabs" ? (y(), D(E, {
                  key: 0,
                  modelValue: s.value,
                  "onUpdate:modelValue": d[1] || (d[1] = (M) => s.value = M),
                  class: "tabs-bar"
                }, {
                  default: o(() => [
                    a(P, {
                      label: e(l)("nav.memories"),
                      name: "memories"
                    }, null, 8, ["label"]),
                    a(P, {
                      label: e(l)("nav.tags"),
                      name: "tags"
                    }, null, 8, ["label"]),
                    a(P, {
                      label: e(l)("nav.ops"),
                      name: "ops"
                    }, null, 8, ["label"])
                  ]),
                  _: 1
                }, 8, ["modelValue"])) : Q("", !0),
                te(a(ra, null, null, 512), [
                  [me, s.value === "memories"]
                ]),
                te(a(fa, null, null, 512), [
                  [me, s.value === "tags"]
                ]),
                te(a(Ma, null, null, 512), [
                  [me, s.value === "ops"]
                ])
              ]),
              _: 1
            })
          ]),
          _: 3
        })
      ]);
    };
  }
}), Ha = /* @__PURE__ */ ae(za, [["__scopeId", "data-v-71914017"]]);
export {
  qe as MarkdownView,
  ra as MemoriesPanel,
  Ha as MemoryAdmin,
  Xt as MemoryDetailDrawer,
  jt as MemoryEditorDialog,
  Oe as MemoryUIConfigKey,
  Ma as OpsPanel,
  fa as TagsPanel,
  $t as applyMemoryUILocalePreference,
  zt as buildMemoriesQuery,
  Pt as createApiClient,
  Aa as currentMemoryUILocale,
  xt as formatSize,
  oe as formatTime,
  pe as isSearchMode,
  ue as memoryUIi18n,
  Fa as provideMemoryUI,
  It as renderMarkdown,
  Lt as sanitizeHtml,
  Vt as setMemoryUILocale,
  l as t,
  ne as useApiClient,
  At as useMemories,
  Be as useMemoryConfig,
  ha as useOps,
  ia as useTags
};
