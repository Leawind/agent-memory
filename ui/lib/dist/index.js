import { provide as Ge, inject as Xe, ref as w, computed as de, onMounted as ue, watch as ye, onUnmounted as Ze, defineComponent as te, openBlock as b, createElementBlock as z, createBlock as D, unref as e, withCtx as o, createVNode as a, createElementVNode as U, toDisplayString as d, createTextVNode as v, Fragment as Z, renderList as ae, createCommentVNode as W, withKeys as et, isRef as X, withDirectives as le, reactive as tt, renderSlot as at, vShow as ve } from "vue";
import { createI18n as lt } from "vue-i18n";
import { ElDialog as $e, ElForm as Ve, ElFormItem as xe, ElInput as be, ElRadioGroup as he, ElRadioButton as Pe, ElSelect as ze, ElOption as De, ElButton as ce, ElDrawer as ot, ElTag as we, ElDivider as nt, ElTooltip as ke, ElIcon as me, ElAlert as Ie, ElTable as Le, ElTableColumn as Ne, ElLoadingDirective as Re, ElPagination as st, ElRadio as rt, ElRow as it, ElCol as dt, ElCard as ut, ElStatistic as ct, ElDescriptions as mt, ElDescriptionsItem as pt, ElEmpty as ft, ElContainer as gt, ElAside as vt, ElMenu as _t, ElMenuItem as yt, ElMain as bt, ElTabs as ht, ElTabPane as wt } from "element-plus/es";
import { InfoFilled as ne, Plus as Oe, Search as Be, Refresh as kt, Download as Ct, UploadFilled as St, Collection as Tt, Notebook as Et, PriceTag as Ut, Odometer as Mt } from "@element-plus/icons-vue";
import { ElMessage as R, ElMessageBox as Ae } from "element-plus";
import { Marked as $t } from "marked";
import Fe from "dompurify";
const Vt = {
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
    crossPlatformYes: "支持",
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
}, xt = {
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
    crossPlatformYes: "Supported",
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
function He() {
  var t;
  return typeof navigator > "u" || (t = navigator.language) != null && t.toLowerCase().startsWith("zh") ? "zh" : "en";
}
const pe = lt({
  legacy: !1,
  locale: He(),
  fallbackLocale: "zh",
  messages: { zh: Vt, en: xt },
  // 面向宿主组件库，缺 key 时静默回退即可，不刷控制台
  missingWarn: !1,
  fallbackWarn: !1
}), { t: l } = pe.global;
function Pt(t) {
  pe.global.locale.value = t;
}
function zt(t) {
  Pt(t === "auto" ? He() : t);
}
function ja() {
  return pe.global.locale.value;
}
const qe = Symbol("memory-ui-config"), ie = {
  baseUrl: "",
  fetch: (...t) => globalThis.fetch(...t),
  defaultPageSize: 20,
  locale: "auto"
};
function Ja(t) {
  t.locale && zt(t.locale), Ge(qe, t);
}
function je() {
  const t = Xe(qe);
  return {
    baseUrl: ((t == null ? void 0 : t.baseUrl) ?? ie.baseUrl).replace(/\/+$/, ""),
    fetch: (t == null ? void 0 : t.fetch) ?? ie.fetch,
    defaultPageSize: (t == null ? void 0 : t.defaultPageSize) ?? ie.defaultPageSize,
    locale: (t == null ? void 0 : t.locale) ?? ie.locale
  };
}
function se(t) {
  if (!t) return "—";
  const s = pe.global.locale.value === "en" ? "en-US" : "zh-CN";
  return new Date(t * 1e3).toLocaleString(s, { hour12: !1 });
}
function Dt(t) {
  return t == null ? "—" : t < 1024 ? `${t} B` : t < 1024 * 1024 ? `${(t / 1024).toFixed(1)} KB` : `${(t / 1024 / 1024).toFixed(2)} MB`;
}
function It(t) {
  async function s(i, m = {}) {
    const p = await t.fetch(t.baseUrl + i, {
      headers: { "Content-Type": "application/json" },
      ...m
    }), h = await p.text();
    let r = null;
    try {
      r = h ? JSON.parse(h) : null;
    } catch {
      r = null;
    }
    if (!p.ok) {
      const C = (r == null ? void 0 : r.error) ?? l("errors.http", { status: p.status });
      throw new Error(C);
    }
    return r;
  }
  async function n(i) {
    var p;
    const m = await t.fetch(t.baseUrl + i);
    if (!m.ok) {
      const h = await m.text().catch(() => "");
      let r = l("errors.http", { status: m.status });
      try {
        r = ((p = JSON.parse(h)) == null ? void 0 : p.error) ?? r;
      } catch {
      }
      throw new Error(r);
    }
    return m.blob();
  }
  return {
    get: (i) => s(i),
    post: (i, m) => s(i, { method: "POST", body: JSON.stringify(m ?? {}) }),
    put: (i, m) => s(i, { method: "PUT", body: JSON.stringify(m ?? {}) }),
    del: (i) => s(i, { method: "DELETE" }),
    getBlob: n
  };
}
function re() {
  return It(je());
}
function Lt(t) {
  const s = _e(t.query), n = new URLSearchParams();
  return s ? (n.set("query", t.query.trim()), t.tagFilter && n.set("tags", t.tagFilter)) : (t.tagFilter && n.set("tag", t.tagFilter), n.set("sort", t.sort), n.set("order", t.order)), n.set("offset", String((t.page - 1) * t.pageSize)), n.set("limit", String(t.pageSize)), n.toString();
}
function _e(t) {
  return t.trim().length > 0;
}
const Nt = new $t();
function Rt(t) {
  const s = Nt.parse(t, { async: !1 });
  return Fe.sanitize(s);
}
function Ot(t) {
  return Fe.sanitize(t);
}
function Ce(t, s) {
  return t.get(`/api/memories/${encodeURIComponent(s)}`);
}
function Je(t, s) {
  return t.post("/api/memories", s);
}
function Ke(t, s, n) {
  return t.put(`/api/memories/${encodeURIComponent(s)}`, n);
}
function Bt(t, s) {
  return t.del(`/api/memories/${encodeURIComponent(s)}`);
}
function We(t) {
  return t.get("/api/tags");
}
function At(t, s, n) {
  return t.post("/api/tags", { name: s, description: n });
}
function Ft(t, s, n, i) {
  const m = { description: i }, p = n.trim();
  return p && p !== s && (m.new_name = p), t.put(`/api/tags/${encodeURIComponent(s)}`, m);
}
function Ht(t, s, n) {
  return t.del(`/api/tags/${encodeURIComponent(s)}?mode=${n}`);
}
function qt() {
  const t = re(), { defaultPageSize: s } = je(), n = w(""), i = w(""), m = w("updated_at"), p = w("desc"), h = w(1), r = w(s), C = w([]), P = w([]), T = w(0), f = w(""), _ = w(!1), x = w([]), V = de(() => _e(n.value));
  function S() {
    return h.value = 1, A();
  }
  function L() {
    return Lt({
      query: n.value,
      tagFilter: i.value,
      sort: m.value,
      order: p.value,
      page: h.value,
      pageSize: r.value
    });
  }
  let B = 0;
  async function A() {
    var E;
    const g = ++B;
    _.value = !0;
    try {
      const O = L();
      if (_e(n.value)) {
        const k = await t.get(`/api/memories?${O}`);
        if (g !== B) return;
        P.value = (k.results ?? []).map((I) => ({ ...I, snippet: Ot(I.snippet) })), T.value = k.total_matches ?? 0, f.value = "";
      } else {
        const k = await t.get(`/api/memories?${O}`);
        if (g !== B) return;
        const I = Math.max(1, Math.ceil(k.total / r.value));
        if (((E = k.memories) == null ? void 0 : E.length) === 0 && k.total > 0 && h.value > I)
          return h.value = I, _.value = !1, A();
        C.value = k.memories ?? [], T.value = k.total ?? 0, f.value = k.note ?? "";
      }
    } finally {
      g === B && (_.value = !1);
    }
  }
  async function N() {
    try {
      const g = await We(t);
      x.value = (g.tags ?? []).map((E) => E.name);
    } catch {
    }
  }
  async function M(g) {
    if (g.id) {
      const E = await Ce(t, g.id), O = new Set(E.tags), k = new Set(g.tags);
      await Ke(t, g.id, {
        summary: g.summary,
        content: g.content,
        add_tags: [...k].filter((I) => !O.has(I)),
        remove_tags: [...O].filter((I) => !k.has(I))
      });
    } else
      await Je(t, { summary: g.summary, content: g.content, tags: g.tags });
    await Promise.all([A(), N()]);
  }
  async function y(g) {
    await Bt(t, g), await A();
  }
  return ue(() => {
    A().catch(() => {
    }), N();
  }), {
    query: n,
    tagFilter: i,
    sort: m,
    order: p,
    page: h,
    pageSize: r,
    rows: C,
    searchResults: P,
    total: T,
    note: f,
    loading: _,
    tagOptions: x,
    searching: V,
    onSearch: S,
    reload: A,
    loadTagOptions: N,
    saveMemory: M,
    removeMemory: y
  };
}
function Se(t, s = 720) {
  const n = w(0);
  let i = null;
  function m(p) {
    i == null || i.disconnect(), i = null, !(!p || typeof ResizeObserver > "u") && (i = new ResizeObserver((h) => {
      var r;
      n.value = ((r = h[0]) == null ? void 0 : r.contentRect.width) ?? 0;
    }), i.observe(p));
  }
  return ue(() => m(t.value)), ye(t, (p) => m(p)), Ze(() => i == null ? void 0 : i.disconnect()), { width: n, compact: de(() => n.value > 0 && n.value < s) };
}
const jt = ["innerHTML"], Qe = /* @__PURE__ */ te({
  __name: "MarkdownView",
  props: {
    source: {}
  },
  setup(t) {
    const s = t, n = de(() => Rt(s.source));
    return (i, m) => (b(), z("div", {
      class: "md-body",
      innerHTML: n.value
    }, null, 8, jt));
  }
}), Jt = { class: "content-label" }, Kt = /* @__PURE__ */ te({
  __name: "MemoryEditorDialog",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    tagOptions: {},
    width: {}
  },
  emits: ["update:visible", "saved"],
  setup(t, { emit: s }) {
    const n = t, i = s, m = re(), p = w(!1), h = w("edit"), r = w({ id: null, summary: "", content: "", tags: [] });
    let C = [];
    ye(
      () => n.visible,
      async (T) => {
        if (T)
          if (h.value = "edit", n.memoryId)
            try {
              const f = await Ce(m, n.memoryId);
              r.value = { id: f.id, summary: f.summary, content: f.content, tags: [...f.tags] }, C = [...f.tags];
            } catch (f) {
              R.error(f instanceof Error ? f.message : String(f)), i("update:visible", !1);
            }
          else
            r.value = { id: null, summary: "", content: "", tags: [] }, C = [];
      }
    );
    async function P() {
      p.value = !0;
      try {
        if (r.value.id) {
          const T = new Set(C), f = new Set(r.value.tags);
          await Ke(m, r.value.id, {
            summary: r.value.summary,
            content: r.value.content,
            add_tags: [...f].filter((_) => !T.has(_)),
            remove_tags: [...T].filter((_) => !f.has(_))
          }), R.success(l("editor.updated"));
        } else
          await Je(m, {
            summary: r.value.summary,
            content: r.value.content,
            tags: r.value.tags
          }), R.success(l("editor.created"));
        i("update:visible", !1), i("saved");
      } catch (T) {
        R.error(T instanceof Error ? T.message : String(T));
      } finally {
        p.value = !1;
      }
    }
    return (T, f) => {
      const _ = be, x = xe, V = Pe, S = he, L = De, B = ze, A = Ve, N = ce, M = $e;
      return b(), D(M, {
        "model-value": t.visible,
        title: r.value.id ? e(l)("editor.editTitle") : e(l)("editor.createTitle"),
        width: t.width,
        "onUpdate:modelValue": f[5] || (f[5] = (y) => i("update:visible", y))
      }, {
        footer: o(() => [
          a(N, {
            onClick: f[4] || (f[4] = (y) => i("update:visible", !1))
          }, {
            default: o(() => [
              v(d(e(l)("common.cancel")), 1)
            ]),
            _: 1
          }),
          a(N, {
            type: "primary",
            loading: p.value,
            onClick: P
          }, {
            default: o(() => [
              v(d(e(l)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: o(() => [
          a(A, { "label-position": "top" }, {
            default: o(() => [
              a(x, {
                label: e(l)("editor.summaryLabel")
              }, {
                default: o(() => [
                  a(_, {
                    modelValue: r.value.summary,
                    "onUpdate:modelValue": f[0] || (f[0] = (y) => r.value.summary = y),
                    maxlength: "512",
                    "show-word-limit": "",
                    placeholder: e(l)("editor.summaryPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"]),
              a(x, null, {
                label: o(() => [
                  U("div", Jt, [
                    U("span", null, d(e(l)("editor.contentLabel")), 1),
                    a(S, {
                      modelValue: h.value,
                      "onUpdate:modelValue": f[1] || (f[1] = (y) => h.value = y),
                      size: "small"
                    }, {
                      default: o(() => [
                        a(V, { value: "edit" }, {
                          default: o(() => [
                            v(d(e(l)("editor.tabEdit")), 1)
                          ]),
                          _: 1
                        }),
                        a(V, { value: "preview" }, {
                          default: o(() => [
                            v(d(e(l)("editor.tabPreview")), 1)
                          ]),
                          _: 1
                        })
                      ]),
                      _: 1
                    }, 8, ["modelValue"])
                  ])
                ]),
                default: o(() => [
                  h.value === "edit" ? (b(), D(_, {
                    key: 0,
                    modelValue: r.value.content,
                    "onUpdate:modelValue": f[2] || (f[2] = (y) => r.value.content = y),
                    type: "textarea",
                    rows: 12,
                    maxlength: "200000",
                    "show-word-limit": "",
                    placeholder: e(l)("editor.contentPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])) : (b(), D(Qe, {
                    key: 1,
                    class: "content-preview",
                    source: r.value.content
                  }, null, 8, ["source"]))
                ]),
                _: 1
              }),
              a(x, {
                label: e(l)("editor.tagsLabel")
              }, {
                default: o(() => [
                  a(B, {
                    modelValue: r.value.tags,
                    "onUpdate:modelValue": f[3] || (f[3] = (y) => r.value.tags = y),
                    multiple: "",
                    filterable: "",
                    "allow-create": "",
                    "default-first-option": "",
                    placeholder: e(l)("editor.tagsPlaceholder"),
                    class: "tags-select"
                  }, {
                    default: o(() => [
                      (b(!0), z(Z, null, ae(t.tagOptions, (y) => (b(), D(L, {
                        key: y,
                        label: y,
                        value: y
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
}), oe = (t, s) => {
  const n = t.__vccOpts || t;
  for (const [i, m] of s)
    n[i] = m;
  return n;
}, Wt = /* @__PURE__ */ oe(Kt, [["__scopeId", "data-v-602bada0"]]), Qt = { class: "detail-summary" }, Yt = { class: "detail-tags" }, Gt = { class: "detail-toolbar" }, Xt = {
  key: 1,
  class: "detail-content"
}, Zt = /* @__PURE__ */ te({
  __name: "MemoryDetailDrawer",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    size: {}
  },
  emits: ["update:visible"],
  setup(t, { emit: s }) {
    const n = t, i = s, m = re(), p = w(null), h = w("rendered");
    return ye(
      () => [n.visible, n.memoryId],
      async ([r]) => {
        if (!(!r || !n.memoryId)) {
          h.value = "rendered";
          try {
            p.value = await Ce(m, n.memoryId);
          } catch (C) {
            R.error(C instanceof Error ? C.message : String(C)), i("update:visible", !1);
          }
        }
      }
    ), (r, C) => {
      var V;
      const P = we, T = nt, f = Pe, _ = he, x = ot;
      return b(), D(x, {
        "model-value": t.visible,
        title: e(l)("drawer.title", { id: ((V = p.value) == null ? void 0 : V.id) ?? t.memoryId ?? "" }),
        size: t.size,
        "onUpdate:modelValue": C[1] || (C[1] = (S) => i("update:visible", S))
      }, {
        default: o(() => [
          p.value ? (b(), z(Z, { key: 0 }, [
            U("h3", Qt, d(p.value.summary), 1),
            U("div", Yt, [
              (b(!0), z(Z, null, ae(p.value.tags, (S) => (b(), D(P, {
                key: S,
                size: "small",
                class: "am-tag"
              }, {
                default: o(() => [
                  v(d(S), 1)
                ]),
                _: 2
              }, 1024))), 128))
            ]),
            a(T),
            U("div", Gt, [
              a(_, {
                modelValue: h.value,
                "onUpdate:modelValue": C[0] || (C[0] = (S) => h.value = S),
                size: "small"
              }, {
                default: o(() => [
                  a(f, { value: "rendered" }, {
                    default: o(() => [
                      v(d(e(l)("drawer.rendered")), 1)
                    ]),
                    _: 1
                  }),
                  a(f, { value: "source" }, {
                    default: o(() => [
                      v(d(e(l)("drawer.source")), 1)
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["modelValue"])
            ]),
            h.value === "rendered" ? (b(), D(Qe, {
              key: 0,
              source: p.value.content
            }, null, 8, ["source"])) : (b(), z("pre", Xt, d(p.value.content), 1))
          ], 64)) : W("", !0)
        ]),
        _: 1
      }, 8, ["model-value", "title", "size"]);
    };
  }
}), ea = /* @__PURE__ */ oe(Zt, [["__scopeId", "data-v-b634d116"]]), ta = {
  key: 0,
  class: "am-panel-header"
}, aa = { class: "am-heading" }, la = { class: "am-panel-title" }, oa = { class: "am-toolbar" }, na = { class: "am-summary" }, sa = ["innerHTML"], ra = {
  key: 4,
  class: "am-pager"
}, ia = {
  key: 5,
  class: "am-pager"
}, da = /* @__PURE__ */ te({
  __name: "MemoriesPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t) {
    const s = t, {
      query: n,
      tagFilter: i,
      sort: m,
      order: p,
      page: h,
      pageSize: r,
      rows: C,
      searchResults: P,
      total: T,
      note: f,
      loading: _,
      tagOptions: x,
      searching: V,
      onSearch: S,
      reload: L,
      loadTagOptions: B,
      removeMemory: A
    } = qt(), N = w(null), { compact: M } = Se(N), y = w(!1), g = w(null), E = w(!1), O = w(null);
    function k(F) {
      return F().catch((u) => R.error(u instanceof Error ? u.message : String(u)));
    }
    function I() {
      g.value = null, y.value = !0;
    }
    function J(F) {
      g.value = F, y.value = !0;
    }
    function Q(F) {
      O.value = F, E.value = !0;
    }
    function ee() {
      k(L), B();
    }
    async function Y(F) {
      try {
        await Ae.confirm(l("memories.deleteConfirm", { id: F.id }), l("memories.deleteTitle"), {
          type: "warning"
        });
      } catch {
        return;
      }
      try {
        await A(F.id), R.success(l("memories.deleted"));
      } catch (u) {
        R.error(u instanceof Error ? u.message : String(u));
      }
    }
    return (F, u) => {
      const G = me, fe = ke, K = ce, $ = be, H = De, ge = ze, Ye = Ie, q = Ne, Te = we, Ee = Le, Ue = st, Me = Re;
      return b(), z("div", {
        ref_key: "rootRef",
        ref: N,
        class: "am-panel"
      }, [
        t.showHeader ? (b(), z("div", ta, [
          U("div", aa, [
            U("h2", la, d(s.title ?? e(l)("memories.title")), 1),
            a(fe, {
              content: s.subtitle ?? e(l)("memories.subtitle"),
              placement: "top"
            }, {
              default: o(() => [
                a(G, { class: "am-info" }, {
                  default: o(() => [
                    a(e(ne))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(K, {
            type: "primary",
            icon: e(Oe),
            onClick: I
          }, {
            default: o(() => [
              v(d(e(l)("memories.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : W("", !0),
        U("div", oa, [
          a($, {
            modelValue: e(n),
            "onUpdate:modelValue": u[1] || (u[1] = (c) => X(n) ? n.value = c : null),
            placeholder: e(l)("memories.searchPlaceholder"),
            clearable: "",
            class: "search",
            onKeyup: u[2] || (u[2] = et((c) => k(e(S)), ["enter"])),
            onClear: u[3] || (u[3] = (c) => k(e(S)))
          }, {
            append: o(() => [
              a(K, {
                icon: e(Be),
                onClick: u[0] || (u[0] = (c) => k(e(S)))
              }, null, 8, ["icon"])
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(ge, {
            modelValue: e(i),
            "onUpdate:modelValue": u[4] || (u[4] = (c) => X(i) ? i.value = c : null),
            placeholder: e(l)("memories.tagFilter"),
            clearable: "",
            filterable: "",
            class: "tag-filter",
            onChange: u[5] || (u[5] = (c) => k(e(S)))
          }, {
            default: o(() => [
              (b(!0), z(Z, null, ae(e(x), (c) => (b(), D(H, {
                key: c,
                label: c,
                value: c
              }, null, 8, ["label", "value"]))), 128))
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(ge, {
            modelValue: e(m),
            "onUpdate:modelValue": u[6] || (u[6] = (c) => X(m) ? m.value = c : null),
            class: "sort",
            onChange: u[7] || (u[7] = (c) => k(e(L)))
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
          a(ge, {
            modelValue: e(p),
            "onUpdate:modelValue": u[8] || (u[8] = (c) => X(p) ? p.value = c : null),
            class: "order",
            onChange: u[9] || (u[9] = (c) => k(e(L)))
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
        e(f) ? (b(), D(Ye, {
          key: 1,
          title: e(f),
          type: "info",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : W("", !0),
        e(V) ? le((b(), D(Ee, {
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
              default: o(({ row: c }) => [
                U("div", na, d(c.summary), 1),
                U("div", {
                  class: "am-snippet",
                  innerHTML: c.snippet
                }, null, 8, sa)
              ]),
              _: 1
            }, 8, ["label"]),
            a(q, {
              label: e(l)("memories.colTags"),
              width: "220"
            }, {
              default: o(({ row: c }) => [
                (b(!0), z(Z, null, ae(c.tags, (j) => (b(), D(Te, {
                  key: j,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: o(() => [
                    v(d(j), 1)
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
              default: o(({ row: c }) => [
                v(d(e(se)(c.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(q, {
              label: e(l)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: o(({ row: c }) => [
                a(K, {
                  link: "",
                  type: "primary",
                  onClick: (j) => Q(c.id)
                }, {
                  default: o(() => [
                    v(d(e(l)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(K, {
                  link: "",
                  type: "primary",
                  onClick: (j) => J(c.id)
                }, {
                  default: o(() => [
                    v(d(e(l)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(K, {
                  link: "",
                  type: "danger",
                  onClick: (j) => Y(c)
                }, {
                  default: o(() => [
                    v(d(e(l)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [Me, e(_)]
        ]) : le((b(), D(Ee, {
          key: 3,
          data: e(C)
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
              default: o(({ row: c }) => [
                (b(!0), z(Z, null, ae(c.tags, (j) => (b(), D(Te, {
                  key: j,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: o(() => [
                    v(d(j), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            e(M) ? W("", !0) : (b(), D(q, {
              key: 0,
              prop: "created_at",
              label: e(l)("memories.colCreatedAt"),
              width: "170"
            }, {
              default: o(({ row: c }) => [
                v(d(e(se)(c.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label"])),
            a(q, {
              label: e(l)("memories.colUpdatedAt"),
              width: "170"
            }, {
              default: o(({ row: c }) => [
                v(d(e(se)(c.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(q, {
              label: e(l)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: o(({ row: c }) => [
                a(K, {
                  link: "",
                  type: "primary",
                  onClick: (j) => Q(c.id)
                }, {
                  default: o(() => [
                    v(d(e(l)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(K, {
                  link: "",
                  type: "primary",
                  onClick: (j) => J(c.id)
                }, {
                  default: o(() => [
                    v(d(e(l)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(K, {
                  link: "",
                  type: "danger",
                  onClick: (j) => Y(c)
                }, {
                  default: o(() => [
                    v(d(e(l)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [Me, e(_)]
        ]),
        e(V) ? (b(), z("div", ia, [
          a(Ue, {
            "current-page": e(h),
            "onUpdate:currentPage": u[14] || (u[14] = (c) => X(h) ? h.value = c : null),
            "page-size": e(r),
            "onUpdate:pageSize": u[15] || (u[15] = (c) => X(r) ? r.value = c : null),
            total: e(T),
            "page-sizes": [10, 20, 50],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: u[16] || (u[16] = (c) => k(e(L))),
            onSizeChange: u[17] || (u[17] = (c) => k(e(L)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])) : (b(), z("div", ra, [
          a(Ue, {
            "current-page": e(h),
            "onUpdate:currentPage": u[10] || (u[10] = (c) => X(h) ? h.value = c : null),
            "page-size": e(r),
            "onUpdate:pageSize": u[11] || (u[11] = (c) => X(r) ? r.value = c : null),
            total: e(T),
            "page-sizes": [20, 50, 100, 200],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: u[12] || (u[12] = (c) => k(e(L))),
            onSizeChange: u[13] || (u[13] = (c) => k(e(L)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])),
        a(Wt, {
          visible: y.value,
          "onUpdate:visible": u[18] || (u[18] = (c) => y.value = c),
          "memory-id": g.value,
          "tag-options": e(x),
          width: e(M) ? "96%" : "640px",
          onSaved: ee
        }, null, 8, ["visible", "memory-id", "tag-options", "width"]),
        a(ea, {
          visible: E.value,
          "onUpdate:visible": u[19] || (u[19] = (c) => E.value = c),
          "memory-id": O.value,
          size: e(M) ? "100%" : "45%"
        }, null, 8, ["visible", "memory-id", "size"])
      ], 512);
    };
  }
}), ua = /* @__PURE__ */ oe(da, [["__scopeId", "data-v-37a950a7"]]);
function ca() {
  const t = re(), s = w([]), n = w(!1);
  async function i() {
    n.value = !0;
    try {
      const r = await We(t);
      s.value = r.tags ?? [];
    } finally {
      n.value = !1;
    }
  }
  async function m(r, C) {
    await At(t, r, C), await i();
  }
  async function p(r, C, P) {
    await Ft(t, r, C, P), await i();
  }
  async function h(r, C) {
    await Ht(t, r, C), await i();
  }
  return ue(() => {
    i().catch(() => {
    });
  }), { rows: s, loading: n, reload: i, create: m, rename: p, remove: h };
}
const ma = {
  key: 0,
  class: "am-panel-header"
}, pa = { class: "am-heading" }, fa = { class: "am-panel-title" }, ga = { class: "delete-body" }, va = /* @__PURE__ */ te({
  __name: "TagsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t) {
    const s = t, { rows: n, loading: i, reload: m, create: p, rename: h, remove: r } = ca(), C = w(null), { compact: P } = Se(C), T = w(!1), f = w(!1), _ = tt({ oldName: null, name: "", newName: "", description: "" }), x = w(!1), V = w("detach"), S = w(null);
    function L() {
      Object.assign(_, { oldName: null, name: "", newName: "", description: "" }), f.value = !0;
    }
    function B(y) {
      Object.assign(_, { oldName: y.name, name: y.name, newName: "", description: y.description ?? "" }), f.value = !0;
    }
    async function A() {
      T.value = !0;
      try {
        _.oldName ? (await h(_.oldName, _.newName, _.description), R.success(l("tags.saved"))) : (await p(_.name, _.description), R.success(l("tags.created"))), f.value = !1;
      } catch (y) {
        R.error(y instanceof Error ? y.message : String(y));
      } finally {
        T.value = !1;
      }
    }
    function N(y) {
      S.value = y, V.value = "detach", x.value = !0;
    }
    async function M() {
      var y, g;
      if (V.value === "purge")
        try {
          await Ae.confirm(
            l("tags.purgeConfirm", { name: (y = S.value) == null ? void 0 : y.name, count: ((g = S.value) == null ? void 0 : g.memory_count) ?? 0 }),
            l("tags.purgeConfirmTitle"),
            { type: "error", confirmButtonText: l("tags.purgeButton") }
          );
        } catch {
          return;
        }
      if (S.value) {
        T.value = !0;
        try {
          await r(S.value.name, V.value), R.success(l("tags.deleted")), x.value = !1;
        } catch (E) {
          R.error(E instanceof Error ? E.message : String(E));
        } finally {
          T.value = !1;
        }
      }
    }
    return (y, g) => {
      const E = me, O = ke, k = ce, I = we, J = Ne, Q = Le, ee = be, Y = xe, F = Ve, u = $e, G = rt, fe = he, K = Re;
      return b(), z("div", {
        ref_key: "rootRef",
        ref: C,
        class: "am-panel"
      }, [
        t.showHeader ? (b(), z("div", ma, [
          U("div", pa, [
            U("h2", fa, d(s.title ?? e(l)("tags.title")), 1),
            a(O, {
              content: s.subtitle ?? e(l)("tags.subtitle"),
              placement: "top"
            }, {
              default: o(() => [
                a(E, { class: "am-info" }, {
                  default: o(() => [
                    a(e(ne))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(k, {
            type: "primary",
            icon: e(Oe),
            onClick: L
          }, {
            default: o(() => [
              v(d(e(l)("tags.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : W("", !0),
        le((b(), D(Q, { data: e(n) }, {
          default: o(() => [
            a(J, {
              prop: "name",
              label: e(l)("tags.colName"),
              "min-width": "160"
            }, {
              default: o(({ row: $ }) => [
                a(I, null, {
                  default: o(() => [
                    v(d($.name), 1)
                  ]),
                  _: 2
                }, 1024)
              ]),
              _: 1
            }, 8, ["label"]),
            a(J, {
              prop: "description",
              label: e(l)("tags.colDescription"),
              "min-width": "300",
              "show-overflow-tooltip": ""
            }, {
              default: o(({ row: $ }) => [
                v(d($.description || "—"), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(J, {
              prop: "memory_count",
              label: e(l)("tags.colMemoryCount"),
              width: "100",
              sortable: ""
            }, null, 8, ["label"]),
            a(J, {
              label: e(l)("tags.colLastUsed"),
              width: "170"
            }, {
              default: o(({ row: $ }) => [
                v(d(e(se)($.last_used_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(J, {
              label: e(l)("memories.colActions"),
              width: "150",
              fixed: "right"
            }, {
              default: o(({ row: $ }) => [
                a(k, {
                  link: "",
                  type: "primary",
                  onClick: (H) => B($)
                }, {
                  default: o(() => [
                    v(d(e(l)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(k, {
                  link: "",
                  type: "danger",
                  onClick: (H) => N($)
                }, {
                  default: o(() => [
                    v(d(e(l)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [K, e(i)]
        ]),
        a(u, {
          modelValue: f.value,
          "onUpdate:modelValue": g[4] || (g[4] = ($) => f.value = $),
          title: _.oldName ? e(l)("tags.editTitle") : e(l)("tags.createTitle"),
          width: e(P) ? "96%" : "480px"
        }, {
          footer: o(() => [
            a(k, {
              onClick: g[3] || (g[3] = ($) => f.value = !1)
            }, {
              default: o(() => [
                v(d(e(l)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(k, {
              type: "primary",
              loading: T.value,
              onClick: A
            }, {
              default: o(() => [
                v(d(e(l)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: o(() => [
            a(F, { "label-position": "top" }, {
              default: o(() => [
                a(Y, {
                  label: e(l)("tags.nameLabel")
                }, {
                  default: o(() => [
                    a(ee, {
                      modelValue: _.name,
                      "onUpdate:modelValue": g[0] || (g[0] = ($) => _.name = $),
                      disabled: !!_.oldName,
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(l)("tags.namePlaceholder")
                    }, null, 8, ["modelValue", "disabled", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                _.oldName ? (b(), D(Y, {
                  key: 0,
                  label: e(l)("tags.renameLabel")
                }, {
                  default: o(() => [
                    a(ee, {
                      modelValue: _.newName,
                      "onUpdate:modelValue": g[1] || (g[1] = ($) => _.newName = $),
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(l)("tags.renamePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"])) : W("", !0),
                a(Y, {
                  label: e(l)("tags.descLabel")
                }, {
                  default: o(() => [
                    a(ee, {
                      modelValue: _.description,
                      "onUpdate:modelValue": g[2] || (g[2] = ($) => _.description = $),
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
        a(u, {
          modelValue: x.value,
          "onUpdate:modelValue": g[7] || (g[7] = ($) => x.value = $),
          title: e(l)("tags.deleteTitle"),
          width: e(P) ? "96%" : "480px"
        }, {
          footer: o(() => [
            a(k, {
              onClick: g[6] || (g[6] = ($) => x.value = !1)
            }, {
              default: o(() => [
                v(d(e(l)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(k, {
              type: "danger",
              loading: T.value,
              onClick: M
            }, {
              default: o(() => [
                v(d(e(l)("common.delete")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: o(() => {
            var $;
            return [
              U("p", ga, [
                v(d(e(l)("tags.deleteBefore")) + " ", 1),
                a(I, null, {
                  default: o(() => {
                    var H;
                    return [
                      v(d((H = S.value) == null ? void 0 : H.name), 1)
                    ];
                  }),
                  _: 1
                }),
                v(" " + d(e(l)("tags.deleteMiddle")) + " ", 1),
                U("b", null, d(($ = S.value) == null ? void 0 : $.memory_count), 1),
                v(" " + d(e(l)("tags.deleteAfter")), 1)
              ]),
              a(fe, {
                modelValue: V.value,
                "onUpdate:modelValue": g[5] || (g[5] = (H) => V.value = H)
              }, {
                default: o(() => [
                  a(G, { value: "detach" }, {
                    default: o(() => [
                      v(d(e(l)("tags.detach")), 1)
                    ]),
                    _: 1
                  }),
                  a(G, { value: "purge" }, {
                    default: o(() => [
                      v(d(e(l)("tags.purge")), 1)
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
}), _a = /* @__PURE__ */ oe(va, [["__scopeId", "data-v-db74fa31"]]);
function ya(t) {
  return t.get("/api/stats");
}
function ba(t) {
  return t.get("/health");
}
function ha(t) {
  return t.get("/api/doctor");
}
function wa(t) {
  return t.getBlob("/api/export");
}
function ka(t, s) {
  return t.post("/api/import", s);
}
function Ca() {
  const t = re(), s = w({}), n = w(""), i = w({ ok: !0, issues: [] }), m = w(!1), p = w(!1), h = w(!1), r = w(!1), C = de(() => Dt(s.value.file_size));
  async function P() {
    s.value = await ya(t), s.value.version = n.value;
  }
  async function T() {
    p.value = !0;
    try {
      i.value = await ha(t), m.value = !0;
    } finally {
      p.value = !1;
    }
  }
  async function f() {
    h.value = !0;
    try {
      const x = await wa(t), V = URL.createObjectURL(x), S = document.createElement("a");
      S.href = V, S.download = "agent-memory-export.json", S.click(), URL.revokeObjectURL(V);
    } finally {
      h.value = !1;
    }
  }
  async function _(x) {
    r.value = !0;
    try {
      const V = await x.text();
      let S;
      try {
        S = JSON.parse(V);
      } catch {
        throw new Error(l("errors.invalidBackup"));
      }
      const L = await ka(t, S);
      return await P(), L;
    } finally {
      r.value = !1;
    }
  }
  return ue(async () => {
    try {
      n.value = (await ba(t)).version ?? "";
    } catch {
    }
    await P().catch(() => {
    });
  }), {
    stats: s,
    version: n,
    doctor: i,
    doctorRan: m,
    doctorLoading: p,
    exporting: h,
    importing: r,
    sizeText: C,
    reload: P,
    runDoctor: T,
    exportData: f,
    importFile: _
  };
}
const Sa = {
  key: 0,
  class: "am-panel-header"
}, Ta = { class: "am-heading" }, Ea = { class: "am-panel-title" }, Ua = { class: "label-help" }, Ma = { class: "actions" }, $a = { class: "card-header" }, Va = {
  key: 2,
  class: "issues"
}, xa = /* @__PURE__ */ te({
  __name: "OpsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t) {
    const s = t, {
      stats: n,
      version: i,
      doctor: m,
      doctorRan: p,
      doctorLoading: h,
      exporting: r,
      importing: C,
      sizeText: P,
      reload: T,
      runDoctor: f,
      exportData: _,
      importFile: x
    } = Ca(), V = w(null), { compact: S } = Se(V), L = w(null);
    function B(N) {
      return N().catch((M) => R.error(M instanceof Error ? M.message : String(M)));
    }
    function A(N) {
      var g;
      const M = N.target, y = (g = M.files) == null ? void 0 : g[0];
      M.value = "", y && x(y).then((E) => {
        R.success(l("ops.imported", { memories: E.imported_memories, tags: E.imported_tags }));
      }).catch((E) => {
        R.error(E instanceof Error ? E.message : String(E));
      });
    }
    return (N, M) => {
      const y = me, g = ke, E = ce, O = ct, k = ut, I = dt, J = it, Q = pt, ee = mt, Y = Ie, F = ft;
      return b(), z("div", {
        ref_key: "rootRef",
        ref: V,
        class: "am-panel"
      }, [
        t.showHeader ? (b(), z("div", Sa, [
          U("div", Ta, [
            U("h2", Ea, d(s.title ?? e(l)("ops.title")), 1),
            a(g, {
              content: s.subtitle ?? e(l)("ops.subtitle"),
              placement: "top"
            }, {
              default: o(() => [
                a(y, { class: "am-info" }, {
                  default: o(() => [
                    a(e(ne))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(E, {
            icon: e(kt),
            onClick: M[0] || (M[0] = (u) => B(e(T)))
          }, {
            default: o(() => [
              v(d(e(l)("ops.refresh")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : W("", !0),
        a(J, { gutter: 14 }, {
          default: o(() => [
            a(I, {
              span: e(S) ? 12 : 6
            }, {
              default: o(() => [
                a(k, { shadow: "never" }, {
                  default: o(() => [
                    a(O, {
                      title: e(l)("ops.statMemories"),
                      value: e(n).memories ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(I, {
              span: e(S) ? 12 : 6
            }, {
              default: o(() => [
                a(k, { shadow: "never" }, {
                  default: o(() => [
                    a(O, {
                      title: e(l)("ops.statTags"),
                      value: e(n).tags ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(I, {
              span: e(S) ? 12 : 6
            }, {
              default: o(() => [
                a(k, { shadow: "never" }, {
                  default: o(() => [
                    a(O, {
                      title: e(l)("ops.statSize"),
                      value: e(P)
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(I, {
              span: e(S) ? 12 : 6
            }, {
              default: o(() => [
                a(k, { shadow: "never" }, {
                  default: o(() => [
                    a(O, {
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
        a(k, { shadow: "never" }, {
          header: o(() => [
            v(d(e(l)("ops.dbCard")), 1)
          ]),
          default: o(() => [
            a(ee, {
              column: e(S) ? 1 : 2,
              border: ""
            }, {
              default: o(() => [
                a(Q, {
                  label: e(l)("ops.path")
                }, {
                  default: o(() => [
                    v(d(e(n).path ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(Q, {
                  label: e(l)("ops.lastUpdate")
                }, {
                  default: o(() => [
                    v(d(e(n).newest_update ? e(l)("ops.lastUpdateValue", {
                      id: e(n).newest_update.id,
                      time: e(se)(e(n).newest_update.updated_at)
                    }) : "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(Q, null, {
                  label: o(() => [
                    U("span", Ua, [
                      v(d(e(l)("ops.crossPlatform")) + " ", 1),
                      a(g, {
                        content: e(l)("ops.crossPlatformNote"),
                        placement: "top"
                      }, {
                        default: o(() => [
                          a(y, { class: "am-info" }, {
                            default: o(() => [
                              a(e(ne))
                            ]),
                            _: 1
                          })
                        ]),
                        _: 1
                      }, 8, ["content"])
                    ])
                  ]),
                  default: o(() => [
                    v(" " + d(e(l)("ops.crossPlatformYes")), 1)
                  ]),
                  _: 1
                }),
                a(Q, {
                  label: e(l)("ops.version")
                }, {
                  default: o(() => [
                    v(d(e(n).version ?? e(i)), 1)
                  ]),
                  _: 1
                }, 8, ["label"])
              ]),
              _: 1
            }, 8, ["column"]),
            U("div", Ma, [
              a(E, {
                icon: e(Ct),
                loading: e(r),
                onClick: M[1] || (M[1] = (u) => B(e(_)))
              }, {
                default: o(() => [
                  v(d(e(l)("ops.export")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(E, {
                icon: e(St),
                loading: e(C),
                onClick: M[2] || (M[2] = (u) => {
                  var G;
                  return (G = L.value) == null ? void 0 : G.click();
                })
              }, {
                default: o(() => [
                  v(d(e(l)("ops.import")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(g, {
                content: e(l)("ops.importHint"),
                placement: "top"
              }, {
                default: o(() => [
                  a(y, { class: "am-info" }, {
                    default: o(() => [
                      a(e(ne))
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["content"]),
              U("input", {
                ref_key: "importInput",
                ref: L,
                type: "file",
                accept: "application/json,.json",
                style: { display: "none" },
                onChange: A
              }, null, 544)
            ])
          ]),
          _: 1
        }),
        a(k, { shadow: "never" }, {
          header: o(() => [
            U("div", $a, [
              U("span", null, d(e(l)("ops.doctorCard")), 1),
              a(E, {
                size: "small",
                icon: e(Be),
                loading: e(h),
                onClick: M[3] || (M[3] = (u) => B(e(f)))
              }, {
                default: o(() => [
                  v(d(e(l)("ops.runDoctor")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"])
            ])
          ]),
          default: o(() => [
            e(p) ? (b(), z(Z, { key: 0 }, [
              e(m).ok ? (b(), D(Y, {
                key: 0,
                title: e(l)("ops.doctorOk"),
                type: "success",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])) : (b(), D(Y, {
                key: 1,
                title: e(l)("ops.doctorFail", { count: e(m).issues.length }),
                type: "error",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])),
              e(m).ok ? W("", !0) : (b(), z("ul", Va, [
                (b(!0), z(Z, null, ae(e(m).issues, (u, G) => (b(), z("li", { key: G }, d(u), 1))), 128))
              ]))
            ], 64)) : (b(), D(F, {
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
}), Pa = /* @__PURE__ */ oe(xa, [["__scopeId", "data-v-a8956eae"]]), za = { class: "memory-ui" }, Da = { class: "brand" }, Ia = { class: "brand-mark" }, La = { class: "aside-footer" }, Na = /* @__PURE__ */ te({
  __name: "MemoryAdmin",
  props: {
    layout: { default: "sidebar" },
    title: { default: "agent-memory" }
  },
  setup(t) {
    const s = w("memories");
    return (n, i) => {
      const m = me, p = yt, h = _t, r = vt, C = wt, P = ht, T = bt, f = gt;
      return b(), z("div", za, [
        a(f, { class: "layout" }, {
          default: o(() => [
            t.layout === "sidebar" ? (b(), D(r, {
              key: 0,
              width: "200px",
              class: "aside"
            }, {
              default: o(() => [
                U("div", Da, [
                  U("span", Ia, [
                    a(m, { size: 16 }, {
                      default: o(() => [
                        a(e(Tt))
                      ]),
                      _: 1
                    })
                  ]),
                  U("span", null, d(t.title), 1)
                ]),
                a(h, {
                  "default-active": s.value,
                  class: "menu",
                  onSelect: i[0] || (i[0] = (_) => s.value = _)
                }, {
                  default: o(() => [
                    a(p, { index: "memories" }, {
                      default: o(() => [
                        a(m, null, {
                          default: o(() => [
                            a(e(Et))
                          ]),
                          _: 1
                        }),
                        U("span", null, d(e(l)("nav.memories")), 1)
                      ]),
                      _: 1
                    }),
                    a(p, { index: "tags" }, {
                      default: o(() => [
                        a(m, null, {
                          default: o(() => [
                            a(e(Ut))
                          ]),
                          _: 1
                        }),
                        U("span", null, d(e(l)("nav.tags")), 1)
                      ]),
                      _: 1
                    }),
                    a(p, { index: "ops" }, {
                      default: o(() => [
                        a(m, null, {
                          default: o(() => [
                            a(e(Mt))
                          ]),
                          _: 1
                        }),
                        U("span", null, d(e(l)("nav.ops")), 1)
                      ]),
                      _: 1
                    })
                  ]),
                  _: 1
                }, 8, ["default-active"]),
                U("div", La, [
                  at(n.$slots, "footer", {}, void 0, !0)
                ])
              ]),
              _: 3
            })) : W("", !0),
            a(T, { class: "main" }, {
              default: o(() => [
                t.layout === "tabs" ? (b(), D(P, {
                  key: 0,
                  modelValue: s.value,
                  "onUpdate:modelValue": i[1] || (i[1] = (_) => s.value = _),
                  class: "tabs-bar"
                }, {
                  default: o(() => [
                    a(C, {
                      label: e(l)("nav.memories"),
                      name: "memories"
                    }, null, 8, ["label"]),
                    a(C, {
                      label: e(l)("nav.tags"),
                      name: "tags"
                    }, null, 8, ["label"]),
                    a(C, {
                      label: e(l)("nav.ops"),
                      name: "ops"
                    }, null, 8, ["label"])
                  ]),
                  _: 1
                }, 8, ["modelValue"])) : W("", !0),
                le(a(ua, null, null, 512), [
                  [ve, s.value === "memories"]
                ]),
                le(a(_a, null, null, 512), [
                  [ve, s.value === "tags"]
                ]),
                le(a(Pa, null, null, 512), [
                  [ve, s.value === "ops"]
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
}), Ka = /* @__PURE__ */ oe(Na, [["__scopeId", "data-v-6f56cb0b"]]);
export {
  Qe as MarkdownView,
  ua as MemoriesPanel,
  Ka as MemoryAdmin,
  ea as MemoryDetailDrawer,
  Wt as MemoryEditorDialog,
  qe as MemoryUIConfigKey,
  Pa as OpsPanel,
  _a as TagsPanel,
  zt as applyMemoryUILocalePreference,
  Lt as buildMemoriesQuery,
  It as createApiClient,
  ja as currentMemoryUILocale,
  Dt as formatSize,
  se as formatTime,
  _e as isSearchMode,
  pe as memoryUIi18n,
  Ja as provideMemoryUI,
  Rt as renderMarkdown,
  Ot as sanitizeHtml,
  Pt as setMemoryUILocale,
  l as t,
  re as useApiClient,
  qt as useMemories,
  je as useMemoryConfig,
  Ca as useOps,
  ca as useTags
};
