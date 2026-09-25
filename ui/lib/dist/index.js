import { provide as Ge, inject as Xe, ref as w, computed as de, onMounted as ce, watch as ye, onUnmounted as Ze, defineComponent as te, openBlock as b, createElementBlock as x, createBlock as z, unref as e, withCtx as l, createVNode as a, createElementVNode as M, toDisplayString as d, createTextVNode as v, Fragment as Z, renderList as ae, createCommentVNode as W, withKeys as et, isRef as X, withDirectives as oe, reactive as tt, renderSlot as at, vShow as ve } from "vue";
import { createI18n as ot } from "vue-i18n";
import { ElDialog as Ue, ElForm as Ve, ElFormItem as $e, ElInput as be, ElRadioGroup as he, ElRadioButton as De, ElSelect as xe, ElOption as ze, ElButton as ue, ElDrawer as lt, ElTag as we, ElDivider as nt, ElTooltip as ke, ElIcon as me, ElAlert as Ie, ElTable as Le, ElTableColumn as Ne, ElLoadingDirective as Ae, ElPagination as st, ElRadio as it, ElRow as rt, ElCol as dt, ElCard as ct, ElStatistic as ut, ElDescriptions as mt, ElDescriptionsItem as pt, ElEmpty as ft, ElContainer as gt, ElAside as vt, ElMenu as _t, ElMenuItem as yt, ElMain as bt, ElTabs as ht, ElTabPane as wt } from "element-plus/es";
import { InfoFilled as ne, Plus as Re, Search as Be, Refresh as kt, Download as Ct, UploadFilled as Tt, Collection as St, Notebook as Pt, PriceTag as Mt, Odometer as Et } from "@element-plus/icons-vue";
import { ElMessage as A, ElMessageBox as Oe } from "element-plus";
import { Marked as Ut } from "marked";
import Fe from "dompurify";
const Vt = {
  nav: {
    memories: "记忆管理",
    tags: "标签管理",
    ops: "运维",
    access: "身份与访问"
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
    language: "语言",
    tokenPromptTitle: "访问令牌",
    tokenPromptDesc: "此服务器已启用 token 鉴权。粘贴你的访问令牌（每个浏览器只需输入一次，之后自动携带）。",
    tokenPlaceholder: "Bearer token",
    tokenConfirm: "保存并连接",
    tokenInvalid: "令牌无效或已失效，请重试",
    logout: "退出登录"
  },
  access: {
    title: "身份与访问",
    subtitle: "token 即身份（无账号体系）；每个身份可单独开关六项能力",
    needAdmin: "需要 admin 权限才能管理身份与设置",
    openMode: "开放模式：当前未配置任何身份，所有请求免鉴权（适合个人本地部署）。创建第一个身份后，所有 /mcp 与 /api 请求必须携带 Authorization: Bearer <token>。第一个身份建议设为管理员。",
    colName: "名称",
    colToken: "Token",
    colPermissions: "能力",
    colCreatedAt: "创建时间",
    colActions: "操作",
    create: "新建身份",
    createTitle: "新建身份",
    nameLabel: "名称（唯一，≤100 字符）",
    namePlaceholder: "如 alice、ci-bot",
    permsLabel: "能力",
    presets: "预设",
    presetAdmin: "管理员",
    presetMember: "可读写",
    presetViewer: "只读",
    presetCustom: "自定义",
    capRead: "读取",
    capCreate: "新建记忆",
    capUpdate: "修改记忆",
    capDelete: "删除记忆",
    capTagManage: "管理标签",
    capAdmin: "管理员",
    copyToken: "复制 Token",
    copied: "已复制",
    created: "身份已创建，token 可随时在列表中复制：",
    editTitle: "编辑能力：{name}",
    deleteTitle: "删除身份",
    deleteConfirm: "确定删除身份「{name}」？其 token 将立即失效。",
    deleted: "已删除",
    saved: "已保存",
    settingsTitle: "自定义提示词（initialize instructions）",
    settingsHint: "返回给 agent 的 initialize 指令。留空使用内置默认文案；可用于说明这是团队共享库还是个人私有库，以及提交规范。",
    settingsPlaceholder: "留空 = 使用内置默认提示词",
    empty: "尚无身份。创建第一个身份以启用 token 鉴权。"
  }
}, $t = {
  nav: {
    memories: "Memories",
    tags: "Tags",
    ops: "Operations",
    access: "Access"
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
    language: "Language",
    tokenPromptTitle: "Access token",
    tokenPromptDesc: "This server requires token auth. Paste your access token (needed once per browser; it is attached automatically afterwards).",
    tokenPlaceholder: "Bearer token",
    tokenConfirm: "Save & connect",
    tokenInvalid: "Token is invalid or revoked, please retry",
    logout: "Sign out"
  },
  access: {
    title: "Access",
    subtitle: "A token is an identity (no account system); toggle each of the six capabilities per identity",
    needAdmin: "Admin permission is required to manage identities and settings",
    openMode: "Open mode: no identities configured, all requests are unauthenticated (fine for personal local deployments). Once the first identity exists, every /mcp and /api request must carry Authorization: Bearer <token>. Make the first identity an admin.",
    colName: "Name",
    colToken: "Token",
    colPermissions: "Capabilities",
    colCreatedAt: "Created",
    colActions: "Actions",
    create: "New identity",
    createTitle: "New identity",
    nameLabel: "Name (unique, ≤100 chars)",
    namePlaceholder: "e.g. alice, ci-bot",
    permsLabel: "Capabilities",
    presets: "Preset",
    presetAdmin: "Admin",
    presetMember: "Read & write",
    presetViewer: "Read-only",
    presetCustom: "Custom",
    capRead: "Read",
    capCreate: "Create memories",
    capUpdate: "Update memories",
    capDelete: "Delete memories",
    capTagManage: "Manage tags",
    capAdmin: "Admin",
    copyToken: "Copy token",
    copied: "Copied",
    created: "Identity created; its token can be copied from the list anytime:",
    editTitle: "Edit capabilities: {name}",
    deleteTitle: "Delete identity",
    deleteConfirm: 'Delete identity "{name}"? Its token is revoked immediately.',
    deleted: "Deleted",
    saved: "Saved",
    settingsTitle: "Custom instructions (initialize instructions)",
    settingsHint: "Instructions returned to agents on initialize. Leave empty for the built-in default; use it to describe this store (team-shared vs personal) and contribution rules.",
    settingsPlaceholder: "Leave empty = built-in default instructions",
    empty: "No identities yet. Create the first one to enable token auth."
  }
};
function He() {
  var t;
  return typeof navigator > "u" || (t = navigator.language) != null && t.toLowerCase().startsWith("zh") ? "zh" : "en";
}
const pe = ot({
  legacy: !1,
  locale: He(),
  fallbackLocale: "zh",
  messages: { zh: Vt, en: $t },
  // 面向宿主组件库，缺 key 时静默回退即可，不刷控制台
  missingWarn: !1,
  fallbackWarn: !1
}), { t: o } = pe.global;
function Dt(t) {
  pe.global.locale.value = t;
}
function xt(t) {
  Dt(t === "auto" ? He() : t);
}
function ja() {
  return pe.global.locale.value;
}
const qe = Symbol("memory-ui-config"), re = {
  baseUrl: "",
  fetch: (...t) => globalThis.fetch(...t),
  defaultPageSize: 20,
  locale: "auto"
};
function Ja(t) {
  t.locale && xt(t.locale), Ge(qe, t);
}
function je() {
  const t = Xe(qe);
  return {
    baseUrl: ((t == null ? void 0 : t.baseUrl) ?? re.baseUrl).replace(/\/+$/, ""),
    fetch: (t == null ? void 0 : t.fetch) ?? re.fetch,
    defaultPageSize: (t == null ? void 0 : t.defaultPageSize) ?? re.defaultPageSize,
    locale: (t == null ? void 0 : t.locale) ?? re.locale
  };
}
function se(t) {
  if (!t) return "—";
  const s = pe.global.locale.value === "en" ? "en-US" : "zh-CN";
  return new Date(t * 1e3).toLocaleString(s, { hour12: !1 });
}
function zt(t) {
  return t == null ? "—" : t < 1024 ? `${t} B` : t < 1024 * 1024 ? `${(t / 1024).toFixed(1)} KB` : `${(t / 1024 / 1024).toFixed(2)} MB`;
}
function It(t) {
  async function s(r, m = {}) {
    const p = await t.fetch(t.baseUrl + r, {
      headers: { "Content-Type": "application/json" },
      ...m
    }), h = await p.text();
    let i = null;
    try {
      i = h ? JSON.parse(h) : null;
    } catch {
      i = null;
    }
    if (!p.ok) {
      const C = (i == null ? void 0 : i.error) ?? o("errors.http", { status: p.status });
      throw new Error(C);
    }
    return i;
  }
  async function n(r) {
    var p;
    const m = await t.fetch(t.baseUrl + r);
    if (!m.ok) {
      const h = await m.text().catch(() => "");
      let i = o("errors.http", { status: m.status });
      try {
        i = ((p = JSON.parse(h)) == null ? void 0 : p.error) ?? i;
      } catch {
      }
      throw new Error(i);
    }
    return m.blob();
  }
  return {
    get: (r) => s(r),
    post: (r, m) => s(r, { method: "POST", body: JSON.stringify(m ?? {}) }),
    put: (r, m) => s(r, { method: "PUT", body: JSON.stringify(m ?? {}) }),
    del: (r) => s(r, { method: "DELETE" }),
    getBlob: n
  };
}
function ie() {
  return It(je());
}
function Lt(t) {
  const s = _e(t.query), n = new URLSearchParams();
  return s ? (n.set("query", t.query.trim()), t.tagFilter && n.set("tags", t.tagFilter)) : (t.tagFilter && n.set("tag", t.tagFilter), n.set("sort", t.sort), n.set("order", t.order)), n.set("offset", String((t.page - 1) * t.pageSize)), n.set("limit", String(t.pageSize)), n.toString();
}
function _e(t) {
  return t.trim().length > 0;
}
const Nt = new Ut();
function At(t) {
  const s = Nt.parse(t, { async: !1 });
  return Fe.sanitize(s);
}
function Rt(t) {
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
function Ot(t, s, n) {
  return t.post("/api/tags", { name: s, description: n });
}
function Ft(t, s, n, r) {
  const m = { description: r }, p = n.trim();
  return p && p !== s && (m.new_name = p), t.put(`/api/tags/${encodeURIComponent(s)}`, m);
}
function Ht(t, s, n) {
  return t.del(`/api/tags/${encodeURIComponent(s)}?mode=${n}`);
}
function qt() {
  const t = ie(), { defaultPageSize: s } = je(), n = w(""), r = w(""), m = w("updated_at"), p = w("desc"), h = w(1), i = w(s), C = w([]), D = w([]), S = w(0), f = w(""), _ = w(!1), $ = w([]), V = de(() => _e(n.value));
  function T() {
    return h.value = 1, O();
  }
  function L() {
    return Lt({
      query: n.value,
      tagFilter: r.value,
      sort: m.value,
      order: p.value,
      page: h.value,
      pageSize: i.value
    });
  }
  let B = 0;
  async function O() {
    var P;
    const g = ++B;
    _.value = !0;
    try {
      const R = L();
      if (_e(n.value)) {
        const k = await t.get(`/api/memories?${R}`);
        if (g !== B) return;
        D.value = (k.results ?? []).map((I) => ({ ...I, snippet: Rt(I.snippet) })), S.value = k.total_matches ?? 0, f.value = "";
      } else {
        const k = await t.get(`/api/memories?${R}`);
        if (g !== B) return;
        const I = Math.max(1, Math.ceil(k.total / i.value));
        if (((P = k.memories) == null ? void 0 : P.length) === 0 && k.total > 0 && h.value > I)
          return h.value = I, _.value = !1, O();
        C.value = k.memories ?? [], S.value = k.total ?? 0, f.value = k.note ?? "";
      }
    } finally {
      g === B && (_.value = !1);
    }
  }
  async function N() {
    try {
      const g = await We(t);
      $.value = (g.tags ?? []).map((P) => P.name);
    } catch {
    }
  }
  async function E(g) {
    if (g.id) {
      const P = await Ce(t, g.id), R = new Set(P.tags), k = new Set(g.tags);
      await Ke(t, g.id, {
        summary: g.summary,
        content: g.content,
        add_tags: [...k].filter((I) => !R.has(I)),
        remove_tags: [...R].filter((I) => !k.has(I))
      });
    } else
      await Je(t, { summary: g.summary, content: g.content, tags: g.tags });
    await Promise.all([O(), N()]);
  }
  async function y(g) {
    await Bt(t, g), await O();
  }
  return ce(() => {
    O().catch(() => {
    }), N();
  }), {
    query: n,
    tagFilter: r,
    sort: m,
    order: p,
    page: h,
    pageSize: i,
    rows: C,
    searchResults: D,
    total: S,
    note: f,
    loading: _,
    tagOptions: $,
    searching: V,
    onSearch: T,
    reload: O,
    loadTagOptions: N,
    saveMemory: E,
    removeMemory: y
  };
}
function Te(t, s = 720) {
  const n = w(0);
  let r = null;
  function m(p) {
    r == null || r.disconnect(), r = null, !(!p || typeof ResizeObserver > "u") && (r = new ResizeObserver((h) => {
      var i;
      n.value = ((i = h[0]) == null ? void 0 : i.contentRect.width) ?? 0;
    }), r.observe(p));
  }
  return ce(() => m(t.value)), ye(t, (p) => m(p)), Ze(() => r == null ? void 0 : r.disconnect()), { width: n, compact: de(() => n.value > 0 && n.value < s) };
}
const jt = ["innerHTML"], Qe = /* @__PURE__ */ te({
  __name: "MarkdownView",
  props: {
    source: {}
  },
  setup(t) {
    const s = t, n = de(() => At(s.source));
    return (r, m) => (b(), x("div", {
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
    const n = t, r = s, m = ie(), p = w(!1), h = w("edit"), i = w({ id: null, summary: "", content: "", tags: [] });
    let C = [];
    ye(
      () => n.visible,
      async (S) => {
        if (S)
          if (h.value = "edit", n.memoryId)
            try {
              const f = await Ce(m, n.memoryId);
              i.value = { id: f.id, summary: f.summary, content: f.content, tags: [...f.tags] }, C = [...f.tags];
            } catch (f) {
              A.error(f instanceof Error ? f.message : String(f)), r("update:visible", !1);
            }
          else
            i.value = { id: null, summary: "", content: "", tags: [] }, C = [];
      }
    );
    async function D() {
      p.value = !0;
      try {
        if (i.value.id) {
          const S = new Set(C), f = new Set(i.value.tags);
          await Ke(m, i.value.id, {
            summary: i.value.summary,
            content: i.value.content,
            add_tags: [...f].filter((_) => !S.has(_)),
            remove_tags: [...S].filter((_) => !f.has(_))
          }), A.success(o("editor.updated"));
        } else
          await Je(m, {
            summary: i.value.summary,
            content: i.value.content,
            tags: i.value.tags
          }), A.success(o("editor.created"));
        r("update:visible", !1), r("saved");
      } catch (S) {
        A.error(S instanceof Error ? S.message : String(S));
      } finally {
        p.value = !1;
      }
    }
    return (S, f) => {
      const _ = be, $ = $e, V = De, T = he, L = ze, B = xe, O = Ve, N = ue, E = Ue;
      return b(), z(E, {
        "model-value": t.visible,
        title: i.value.id ? e(o)("editor.editTitle") : e(o)("editor.createTitle"),
        width: t.width,
        "onUpdate:modelValue": f[5] || (f[5] = (y) => r("update:visible", y))
      }, {
        footer: l(() => [
          a(N, {
            onClick: f[4] || (f[4] = (y) => r("update:visible", !1))
          }, {
            default: l(() => [
              v(d(e(o)("common.cancel")), 1)
            ]),
            _: 1
          }),
          a(N, {
            type: "primary",
            loading: p.value,
            onClick: D
          }, {
            default: l(() => [
              v(d(e(o)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: l(() => [
          a(O, { "label-position": "top" }, {
            default: l(() => [
              a($, {
                label: e(o)("editor.summaryLabel")
              }, {
                default: l(() => [
                  a(_, {
                    modelValue: i.value.summary,
                    "onUpdate:modelValue": f[0] || (f[0] = (y) => i.value.summary = y),
                    maxlength: "512",
                    "show-word-limit": "",
                    placeholder: e(o)("editor.summaryPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"]),
              a($, null, {
                label: l(() => [
                  M("div", Jt, [
                    M("span", null, d(e(o)("editor.contentLabel")), 1),
                    a(T, {
                      modelValue: h.value,
                      "onUpdate:modelValue": f[1] || (f[1] = (y) => h.value = y),
                      size: "small"
                    }, {
                      default: l(() => [
                        a(V, { value: "edit" }, {
                          default: l(() => [
                            v(d(e(o)("editor.tabEdit")), 1)
                          ]),
                          _: 1
                        }),
                        a(V, { value: "preview" }, {
                          default: l(() => [
                            v(d(e(o)("editor.tabPreview")), 1)
                          ]),
                          _: 1
                        })
                      ]),
                      _: 1
                    }, 8, ["modelValue"])
                  ])
                ]),
                default: l(() => [
                  h.value === "edit" ? (b(), z(_, {
                    key: 0,
                    modelValue: i.value.content,
                    "onUpdate:modelValue": f[2] || (f[2] = (y) => i.value.content = y),
                    type: "textarea",
                    rows: 12,
                    maxlength: "200000",
                    "show-word-limit": "",
                    placeholder: e(o)("editor.contentPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])) : (b(), z(Qe, {
                    key: 1,
                    class: "content-preview",
                    source: i.value.content
                  }, null, 8, ["source"]))
                ]),
                _: 1
              }),
              a($, {
                label: e(o)("editor.tagsLabel")
              }, {
                default: l(() => [
                  a(B, {
                    modelValue: i.value.tags,
                    "onUpdate:modelValue": f[3] || (f[3] = (y) => i.value.tags = y),
                    multiple: "",
                    filterable: "",
                    "allow-create": "",
                    "default-first-option": "",
                    placeholder: e(o)("editor.tagsPlaceholder"),
                    class: "tags-select"
                  }, {
                    default: l(() => [
                      (b(!0), x(Z, null, ae(t.tagOptions, (y) => (b(), z(L, {
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
}), le = (t, s) => {
  const n = t.__vccOpts || t;
  for (const [r, m] of s)
    n[r] = m;
  return n;
}, Wt = /* @__PURE__ */ le(Kt, [["__scopeId", "data-v-602bada0"]]), Qt = { class: "detail-summary" }, Yt = { class: "detail-tags" }, Gt = { class: "detail-toolbar" }, Xt = {
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
    const n = t, r = s, m = ie(), p = w(null), h = w("rendered");
    return ye(
      () => [n.visible, n.memoryId],
      async ([i]) => {
        if (!(!i || !n.memoryId)) {
          h.value = "rendered";
          try {
            p.value = await Ce(m, n.memoryId);
          } catch (C) {
            A.error(C instanceof Error ? C.message : String(C)), r("update:visible", !1);
          }
        }
      }
    ), (i, C) => {
      var V;
      const D = we, S = nt, f = De, _ = he, $ = lt;
      return b(), z($, {
        "model-value": t.visible,
        title: e(o)("drawer.title", { id: ((V = p.value) == null ? void 0 : V.id) ?? t.memoryId ?? "" }),
        size: t.size,
        "onUpdate:modelValue": C[1] || (C[1] = (T) => r("update:visible", T))
      }, {
        default: l(() => [
          p.value ? (b(), x(Z, { key: 0 }, [
            M("h3", Qt, d(p.value.summary), 1),
            M("div", Yt, [
              (b(!0), x(Z, null, ae(p.value.tags, (T) => (b(), z(D, {
                key: T,
                size: "small",
                class: "am-tag"
              }, {
                default: l(() => [
                  v(d(T), 1)
                ]),
                _: 2
              }, 1024))), 128))
            ]),
            a(S),
            M("div", Gt, [
              a(_, {
                modelValue: h.value,
                "onUpdate:modelValue": C[0] || (C[0] = (T) => h.value = T),
                size: "small"
              }, {
                default: l(() => [
                  a(f, { value: "rendered" }, {
                    default: l(() => [
                      v(d(e(o)("drawer.rendered")), 1)
                    ]),
                    _: 1
                  }),
                  a(f, { value: "source" }, {
                    default: l(() => [
                      v(d(e(o)("drawer.source")), 1)
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["modelValue"])
            ]),
            h.value === "rendered" ? (b(), z(Qe, {
              key: 0,
              source: p.value.content
            }, null, 8, ["source"])) : (b(), x("pre", Xt, d(p.value.content), 1))
          ], 64)) : W("", !0)
        ]),
        _: 1
      }, 8, ["model-value", "title", "size"]);
    };
  }
}), ea = /* @__PURE__ */ le(Zt, [["__scopeId", "data-v-b634d116"]]), ta = {
  key: 0,
  class: "am-panel-header"
}, aa = { class: "am-heading" }, oa = { class: "am-panel-title" }, la = { class: "am-toolbar" }, na = { class: "am-summary" }, sa = ["innerHTML"], ia = {
  key: 4,
  class: "am-pager"
}, ra = {
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
      tagFilter: r,
      sort: m,
      order: p,
      page: h,
      pageSize: i,
      rows: C,
      searchResults: D,
      total: S,
      note: f,
      loading: _,
      tagOptions: $,
      searching: V,
      onSearch: T,
      reload: L,
      loadTagOptions: B,
      removeMemory: O
    } = qt(), N = w(null), { compact: E } = Te(N), y = w(!1), g = w(null), P = w(!1), R = w(null);
    function k(F) {
      return F().catch((c) => A.error(c instanceof Error ? c.message : String(c)));
    }
    function I() {
      g.value = null, y.value = !0;
    }
    function J(F) {
      g.value = F, y.value = !0;
    }
    function Q(F) {
      R.value = F, P.value = !0;
    }
    function ee() {
      k(L), B();
    }
    async function Y(F) {
      try {
        await Oe.confirm(o("memories.deleteConfirm", { id: F.id }), o("memories.deleteTitle"), {
          type: "warning"
        });
      } catch {
        return;
      }
      try {
        await O(F.id), A.success(o("memories.deleted"));
      } catch (c) {
        A.error(c instanceof Error ? c.message : String(c));
      }
    }
    return (F, c) => {
      const G = me, fe = ke, K = ue, U = be, H = ze, ge = xe, Ye = Ie, q = Ne, Se = we, Pe = Le, Me = st, Ee = Ae;
      return b(), x("div", {
        ref_key: "rootRef",
        ref: N,
        class: "am-panel"
      }, [
        t.showHeader ? (b(), x("div", ta, [
          M("div", aa, [
            M("h2", oa, d(s.title ?? e(o)("memories.title")), 1),
            a(fe, {
              content: s.subtitle ?? e(o)("memories.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(G, { class: "am-info" }, {
                  default: l(() => [
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
            icon: e(Re),
            onClick: I
          }, {
            default: l(() => [
              v(d(e(o)("memories.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : W("", !0),
        M("div", la, [
          a(U, {
            modelValue: e(n),
            "onUpdate:modelValue": c[1] || (c[1] = (u) => X(n) ? n.value = u : null),
            placeholder: e(o)("memories.searchPlaceholder"),
            clearable: "",
            class: "search",
            onKeyup: c[2] || (c[2] = et((u) => k(e(T)), ["enter"])),
            onClear: c[3] || (c[3] = (u) => k(e(T)))
          }, {
            append: l(() => [
              a(K, {
                icon: e(Be),
                onClick: c[0] || (c[0] = (u) => k(e(T)))
              }, null, 8, ["icon"])
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(ge, {
            modelValue: e(r),
            "onUpdate:modelValue": c[4] || (c[4] = (u) => X(r) ? r.value = u : null),
            placeholder: e(o)("memories.tagFilter"),
            clearable: "",
            filterable: "",
            class: "tag-filter",
            onChange: c[5] || (c[5] = (u) => k(e(T)))
          }, {
            default: l(() => [
              (b(!0), x(Z, null, ae(e($), (u) => (b(), z(H, {
                key: u,
                label: u,
                value: u
              }, null, 8, ["label", "value"]))), 128))
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(ge, {
            modelValue: e(m),
            "onUpdate:modelValue": c[6] || (c[6] = (u) => X(m) ? m.value = u : null),
            class: "sort",
            onChange: c[7] || (c[7] = (u) => k(e(L)))
          }, {
            default: l(() => [
              a(H, {
                label: e(o)("memories.sortUpdated"),
                value: "updated_at"
              }, null, 8, ["label"]),
              a(H, {
                label: e(o)("memories.sortCreated"),
                value: "created_at"
              }, null, 8, ["label"])
            ]),
            _: 1
          }, 8, ["modelValue"]),
          a(ge, {
            modelValue: e(p),
            "onUpdate:modelValue": c[8] || (c[8] = (u) => X(p) ? p.value = u : null),
            class: "order",
            onChange: c[9] || (c[9] = (u) => k(e(L)))
          }, {
            default: l(() => [
              a(H, {
                label: e(o)("memories.orderDesc"),
                value: "desc"
              }, null, 8, ["label"]),
              a(H, {
                label: e(o)("memories.orderAsc"),
                value: "asc"
              }, null, 8, ["label"])
            ]),
            _: 1
          }, 8, ["modelValue"])
        ]),
        e(f) ? (b(), z(Ye, {
          key: 1,
          title: e(f),
          type: "info",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : W("", !0),
        e(V) ? oe((b(), z(Pe, {
          key: 2,
          data: e(D)
        }, {
          default: l(() => [
            a(q, {
              prop: "id",
              label: e(o)("memories.colId"),
              width: "80"
            }, null, 8, ["label"]),
            a(q, {
              label: e(o)("memories.colSummary")
            }, {
              default: l(({ row: u }) => [
                M("div", na, d(u.summary), 1),
                M("div", {
                  class: "am-snippet",
                  innerHTML: u.snippet
                }, null, 8, sa)
              ]),
              _: 1
            }, 8, ["label"]),
            a(q, {
              label: e(o)("memories.colTags"),
              width: "220"
            }, {
              default: l(({ row: u }) => [
                (b(!0), x(Z, null, ae(u.tags, (j) => (b(), z(Se, {
                  key: j,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: l(() => [
                    v(d(j), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            a(q, {
              prop: "score",
              label: e(o)("memories.colScore"),
              width: "80",
              sortable: ""
            }, null, 8, ["label"]),
            a(q, {
              label: e(o)("memories.colUpdatedAt"),
              width: "170"
            }, {
              default: l(({ row: u }) => [
                v(d(e(se)(u.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(q, {
              label: e(o)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: l(({ row: u }) => [
                a(K, {
                  link: "",
                  type: "primary",
                  onClick: (j) => Q(u.id)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(K, {
                  link: "",
                  type: "primary",
                  onClick: (j) => J(u.id)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(K, {
                  link: "",
                  type: "danger",
                  onClick: (j) => Y(u)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [Ee, e(_)]
        ]) : oe((b(), z(Pe, {
          key: 3,
          data: e(C)
        }, {
          default: l(() => [
            a(q, {
              prop: "id",
              label: e(o)("memories.colId"),
              width: "80"
            }, null, 8, ["label"]),
            a(q, {
              prop: "summary",
              label: e(o)("memories.colSummary"),
              "min-width": "300",
              "show-overflow-tooltip": ""
            }, null, 8, ["label"]),
            a(q, {
              label: e(o)("memories.colTags"),
              width: "220"
            }, {
              default: l(({ row: u }) => [
                (b(!0), x(Z, null, ae(u.tags, (j) => (b(), z(Se, {
                  key: j,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: l(() => [
                    v(d(j), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            e(E) ? W("", !0) : (b(), z(q, {
              key: 0,
              prop: "created_at",
              label: e(o)("memories.colCreatedAt"),
              width: "170"
            }, {
              default: l(({ row: u }) => [
                v(d(e(se)(u.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label"])),
            a(q, {
              label: e(o)("memories.colUpdatedAt"),
              width: "170"
            }, {
              default: l(({ row: u }) => [
                v(d(e(se)(u.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(q, {
              label: e(o)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: l(({ row: u }) => [
                a(K, {
                  link: "",
                  type: "primary",
                  onClick: (j) => Q(u.id)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(K, {
                  link: "",
                  type: "primary",
                  onClick: (j) => J(u.id)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(K, {
                  link: "",
                  type: "danger",
                  onClick: (j) => Y(u)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [Ee, e(_)]
        ]),
        e(V) ? (b(), x("div", ra, [
          a(Me, {
            "current-page": e(h),
            "onUpdate:currentPage": c[14] || (c[14] = (u) => X(h) ? h.value = u : null),
            "page-size": e(i),
            "onUpdate:pageSize": c[15] || (c[15] = (u) => X(i) ? i.value = u : null),
            total: e(S),
            "page-sizes": [10, 20, 50],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: c[16] || (c[16] = (u) => k(e(L))),
            onSizeChange: c[17] || (c[17] = (u) => k(e(L)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])) : (b(), x("div", ia, [
          a(Me, {
            "current-page": e(h),
            "onUpdate:currentPage": c[10] || (c[10] = (u) => X(h) ? h.value = u : null),
            "page-size": e(i),
            "onUpdate:pageSize": c[11] || (c[11] = (u) => X(i) ? i.value = u : null),
            total: e(S),
            "page-sizes": [20, 50, 100, 200],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: c[12] || (c[12] = (u) => k(e(L))),
            onSizeChange: c[13] || (c[13] = (u) => k(e(L)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])),
        a(Wt, {
          visible: y.value,
          "onUpdate:visible": c[18] || (c[18] = (u) => y.value = u),
          "memory-id": g.value,
          "tag-options": e($),
          width: e(E) ? "96%" : "640px",
          onSaved: ee
        }, null, 8, ["visible", "memory-id", "tag-options", "width"]),
        a(ea, {
          visible: P.value,
          "onUpdate:visible": c[19] || (c[19] = (u) => P.value = u),
          "memory-id": R.value,
          size: e(E) ? "100%" : "45%"
        }, null, 8, ["visible", "memory-id", "size"])
      ], 512);
    };
  }
}), ca = /* @__PURE__ */ le(da, [["__scopeId", "data-v-37a950a7"]]);
function ua() {
  const t = ie(), s = w([]), n = w(!1);
  async function r() {
    n.value = !0;
    try {
      const i = await We(t);
      s.value = i.tags ?? [];
    } finally {
      n.value = !1;
    }
  }
  async function m(i, C) {
    await Ot(t, i, C), await r();
  }
  async function p(i, C, D) {
    await Ft(t, i, C, D), await r();
  }
  async function h(i, C) {
    await Ht(t, i, C), await r();
  }
  return ce(() => {
    r().catch(() => {
    });
  }), { rows: s, loading: n, reload: r, create: m, rename: p, remove: h };
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
    const s = t, { rows: n, loading: r, reload: m, create: p, rename: h, remove: i } = ua(), C = w(null), { compact: D } = Te(C), S = w(!1), f = w(!1), _ = tt({ oldName: null, name: "", newName: "", description: "" }), $ = w(!1), V = w("detach"), T = w(null);
    function L() {
      Object.assign(_, { oldName: null, name: "", newName: "", description: "" }), f.value = !0;
    }
    function B(y) {
      Object.assign(_, { oldName: y.name, name: y.name, newName: "", description: y.description ?? "" }), f.value = !0;
    }
    async function O() {
      S.value = !0;
      try {
        _.oldName ? (await h(_.oldName, _.newName, _.description), A.success(o("tags.saved"))) : (await p(_.name, _.description), A.success(o("tags.created"))), f.value = !1;
      } catch (y) {
        A.error(y instanceof Error ? y.message : String(y));
      } finally {
        S.value = !1;
      }
    }
    function N(y) {
      T.value = y, V.value = "detach", $.value = !0;
    }
    async function E() {
      var y, g;
      if (V.value === "purge")
        try {
          await Oe.confirm(
            o("tags.purgeConfirm", { name: (y = T.value) == null ? void 0 : y.name, count: ((g = T.value) == null ? void 0 : g.memory_count) ?? 0 }),
            o("tags.purgeConfirmTitle"),
            { type: "error", confirmButtonText: o("tags.purgeButton") }
          );
        } catch {
          return;
        }
      if (T.value) {
        S.value = !0;
        try {
          await i(T.value.name, V.value), A.success(o("tags.deleted")), $.value = !1;
        } catch (P) {
          A.error(P instanceof Error ? P.message : String(P));
        } finally {
          S.value = !1;
        }
      }
    }
    return (y, g) => {
      const P = me, R = ke, k = ue, I = we, J = Ne, Q = Le, ee = be, Y = $e, F = Ve, c = Ue, G = it, fe = he, K = Ae;
      return b(), x("div", {
        ref_key: "rootRef",
        ref: C,
        class: "am-panel"
      }, [
        t.showHeader ? (b(), x("div", ma, [
          M("div", pa, [
            M("h2", fa, d(s.title ?? e(o)("tags.title")), 1),
            a(R, {
              content: s.subtitle ?? e(o)("tags.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(P, { class: "am-info" }, {
                  default: l(() => [
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
            icon: e(Re),
            onClick: L
          }, {
            default: l(() => [
              v(d(e(o)("tags.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : W("", !0),
        oe((b(), z(Q, { data: e(n) }, {
          default: l(() => [
            a(J, {
              prop: "name",
              label: e(o)("tags.colName"),
              "min-width": "160"
            }, {
              default: l(({ row: U }) => [
                a(I, null, {
                  default: l(() => [
                    v(d(U.name), 1)
                  ]),
                  _: 2
                }, 1024)
              ]),
              _: 1
            }, 8, ["label"]),
            a(J, {
              prop: "description",
              label: e(o)("tags.colDescription"),
              "min-width": "300",
              "show-overflow-tooltip": ""
            }, {
              default: l(({ row: U }) => [
                v(d(U.description || "—"), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(J, {
              prop: "memory_count",
              label: e(o)("tags.colMemoryCount"),
              width: "100",
              sortable: ""
            }, null, 8, ["label"]),
            a(J, {
              label: e(o)("tags.colLastUsed"),
              width: "170"
            }, {
              default: l(({ row: U }) => [
                v(d(e(se)(U.last_used_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(J, {
              label: e(o)("memories.colActions"),
              width: "150",
              fixed: "right"
            }, {
              default: l(({ row: U }) => [
                a(k, {
                  link: "",
                  type: "primary",
                  onClick: (H) => B(U)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(k, {
                  link: "",
                  type: "danger",
                  onClick: (H) => N(U)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [K, e(r)]
        ]),
        a(c, {
          modelValue: f.value,
          "onUpdate:modelValue": g[4] || (g[4] = (U) => f.value = U),
          title: _.oldName ? e(o)("tags.editTitle") : e(o)("tags.createTitle"),
          width: e(D) ? "96%" : "480px"
        }, {
          footer: l(() => [
            a(k, {
              onClick: g[3] || (g[3] = (U) => f.value = !1)
            }, {
              default: l(() => [
                v(d(e(o)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(k, {
              type: "primary",
              loading: S.value,
              onClick: O
            }, {
              default: l(() => [
                v(d(e(o)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: l(() => [
            a(F, { "label-position": "top" }, {
              default: l(() => [
                a(Y, {
                  label: e(o)("tags.nameLabel")
                }, {
                  default: l(() => [
                    a(ee, {
                      modelValue: _.name,
                      "onUpdate:modelValue": g[0] || (g[0] = (U) => _.name = U),
                      disabled: !!_.oldName,
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(o)("tags.namePlaceholder")
                    }, null, 8, ["modelValue", "disabled", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                _.oldName ? (b(), z(Y, {
                  key: 0,
                  label: e(o)("tags.renameLabel")
                }, {
                  default: l(() => [
                    a(ee, {
                      modelValue: _.newName,
                      "onUpdate:modelValue": g[1] || (g[1] = (U) => _.newName = U),
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(o)("tags.renamePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"])) : W("", !0),
                a(Y, {
                  label: e(o)("tags.descLabel")
                }, {
                  default: l(() => [
                    a(ee, {
                      modelValue: _.description,
                      "onUpdate:modelValue": g[2] || (g[2] = (U) => _.description = U),
                      type: "textarea",
                      rows: 3,
                      maxlength: "500",
                      "show-word-limit": "",
                      placeholder: e(o)("tags.descPlaceholder")
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
        a(c, {
          modelValue: $.value,
          "onUpdate:modelValue": g[7] || (g[7] = (U) => $.value = U),
          title: e(o)("tags.deleteTitle"),
          width: e(D) ? "96%" : "480px"
        }, {
          footer: l(() => [
            a(k, {
              onClick: g[6] || (g[6] = (U) => $.value = !1)
            }, {
              default: l(() => [
                v(d(e(o)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(k, {
              type: "danger",
              loading: S.value,
              onClick: E
            }, {
              default: l(() => [
                v(d(e(o)("common.delete")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: l(() => {
            var U;
            return [
              M("p", ga, [
                v(d(e(o)("tags.deleteBefore")) + " ", 1),
                a(I, null, {
                  default: l(() => {
                    var H;
                    return [
                      v(d((H = T.value) == null ? void 0 : H.name), 1)
                    ];
                  }),
                  _: 1
                }),
                v(" " + d(e(o)("tags.deleteMiddle")) + " ", 1),
                M("b", null, d((U = T.value) == null ? void 0 : U.memory_count), 1),
                v(" " + d(e(o)("tags.deleteAfter")), 1)
              ]),
              a(fe, {
                modelValue: V.value,
                "onUpdate:modelValue": g[5] || (g[5] = (H) => V.value = H)
              }, {
                default: l(() => [
                  a(G, { value: "detach" }, {
                    default: l(() => [
                      v(d(e(o)("tags.detach")), 1)
                    ]),
                    _: 1
                  }),
                  a(G, { value: "purge" }, {
                    default: l(() => [
                      v(d(e(o)("tags.purge")), 1)
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
}), _a = /* @__PURE__ */ le(va, [["__scopeId", "data-v-db74fa31"]]);
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
  const t = ie(), s = w({}), n = w(""), r = w({ ok: !0, issues: [] }), m = w(!1), p = w(!1), h = w(!1), i = w(!1), C = de(() => zt(s.value.file_size));
  async function D() {
    s.value = await ya(t), s.value.version = n.value;
  }
  async function S() {
    p.value = !0;
    try {
      r.value = await ha(t), m.value = !0;
    } finally {
      p.value = !1;
    }
  }
  async function f() {
    h.value = !0;
    try {
      const $ = await wa(t), V = URL.createObjectURL($), T = document.createElement("a");
      T.href = V, T.download = "agent-memory-export.json", T.click(), URL.revokeObjectURL(V);
    } finally {
      h.value = !1;
    }
  }
  async function _($) {
    i.value = !0;
    try {
      const V = await $.text();
      let T;
      try {
        T = JSON.parse(V);
      } catch {
        throw new Error(o("errors.invalidBackup"));
      }
      const L = await ka(t, T);
      return await D(), L;
    } finally {
      i.value = !1;
    }
  }
  return ce(async () => {
    try {
      n.value = (await ba(t)).version ?? "";
    } catch {
    }
    await D().catch(() => {
    });
  }), {
    stats: s,
    version: n,
    doctor: r,
    doctorRan: m,
    doctorLoading: p,
    exporting: h,
    importing: i,
    sizeText: C,
    reload: D,
    runDoctor: S,
    exportData: f,
    importFile: _
  };
}
const Ta = {
  key: 0,
  class: "am-panel-header"
}, Sa = { class: "am-heading" }, Pa = { class: "am-panel-title" }, Ma = { class: "label-help" }, Ea = { class: "actions" }, Ua = { class: "card-header" }, Va = {
  key: 2,
  class: "issues"
}, $a = /* @__PURE__ */ te({
  __name: "OpsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t) {
    const s = t, {
      stats: n,
      version: r,
      doctor: m,
      doctorRan: p,
      doctorLoading: h,
      exporting: i,
      importing: C,
      sizeText: D,
      reload: S,
      runDoctor: f,
      exportData: _,
      importFile: $
    } = Ca(), V = w(null), { compact: T } = Te(V), L = w(null);
    function B(N) {
      return N().catch((E) => A.error(E instanceof Error ? E.message : String(E)));
    }
    function O(N) {
      var g;
      const E = N.target, y = (g = E.files) == null ? void 0 : g[0];
      E.value = "", y && $(y).then((P) => {
        A.success(o("ops.imported", { memories: P.imported_memories, tags: P.imported_tags }));
      }).catch((P) => {
        A.error(P instanceof Error ? P.message : String(P));
      });
    }
    return (N, E) => {
      const y = me, g = ke, P = ue, R = ut, k = ct, I = dt, J = rt, Q = pt, ee = mt, Y = Ie, F = ft;
      return b(), x("div", {
        ref_key: "rootRef",
        ref: V,
        class: "am-panel"
      }, [
        t.showHeader ? (b(), x("div", Ta, [
          M("div", Sa, [
            M("h2", Pa, d(s.title ?? e(o)("ops.title")), 1),
            a(g, {
              content: s.subtitle ?? e(o)("ops.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(y, { class: "am-info" }, {
                  default: l(() => [
                    a(e(ne))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(P, {
            icon: e(kt),
            onClick: E[0] || (E[0] = (c) => B(e(S)))
          }, {
            default: l(() => [
              v(d(e(o)("ops.refresh")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : W("", !0),
        a(J, { gutter: 14 }, {
          default: l(() => [
            a(I, {
              span: e(T) ? 12 : 6
            }, {
              default: l(() => [
                a(k, { shadow: "never" }, {
                  default: l(() => [
                    a(R, {
                      title: e(o)("ops.statMemories"),
                      value: e(n).memories ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(I, {
              span: e(T) ? 12 : 6
            }, {
              default: l(() => [
                a(k, { shadow: "never" }, {
                  default: l(() => [
                    a(R, {
                      title: e(o)("ops.statTags"),
                      value: e(n).tags ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(I, {
              span: e(T) ? 12 : 6
            }, {
              default: l(() => [
                a(k, { shadow: "never" }, {
                  default: l(() => [
                    a(R, {
                      title: e(o)("ops.statSize"),
                      value: e(D)
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(I, {
              span: e(T) ? 12 : 6
            }, {
              default: l(() => [
                a(k, { shadow: "never" }, {
                  default: l(() => [
                    a(R, {
                      title: e(o)("ops.statNextId"),
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
          header: l(() => [
            v(d(e(o)("ops.dbCard")), 1)
          ]),
          default: l(() => [
            a(ee, {
              column: e(T) ? 1 : 2,
              border: ""
            }, {
              default: l(() => [
                a(Q, {
                  label: e(o)("ops.path")
                }, {
                  default: l(() => [
                    v(d(e(n).path ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(Q, {
                  label: e(o)("ops.lastUpdate")
                }, {
                  default: l(() => [
                    v(d(e(n).newest_update ? e(o)("ops.lastUpdateValue", {
                      id: e(n).newest_update.id,
                      time: e(se)(e(n).newest_update.updated_at)
                    }) : "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(Q, null, {
                  label: l(() => [
                    M("span", Ma, [
                      v(d(e(o)("ops.crossPlatform")) + " ", 1),
                      a(g, {
                        content: e(o)("ops.crossPlatformNote"),
                        placement: "top"
                      }, {
                        default: l(() => [
                          a(y, { class: "am-info" }, {
                            default: l(() => [
                              a(e(ne))
                            ]),
                            _: 1
                          })
                        ]),
                        _: 1
                      }, 8, ["content"])
                    ])
                  ]),
                  default: l(() => [
                    v(" " + d(e(o)("ops.crossPlatformYes")), 1)
                  ]),
                  _: 1
                }),
                a(Q, {
                  label: e(o)("ops.version")
                }, {
                  default: l(() => [
                    v(d(e(n).version ?? e(r)), 1)
                  ]),
                  _: 1
                }, 8, ["label"])
              ]),
              _: 1
            }, 8, ["column"]),
            M("div", Ea, [
              a(P, {
                icon: e(Ct),
                loading: e(i),
                onClick: E[1] || (E[1] = (c) => B(e(_)))
              }, {
                default: l(() => [
                  v(d(e(o)("ops.export")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(P, {
                icon: e(Tt),
                loading: e(C),
                onClick: E[2] || (E[2] = (c) => {
                  var G;
                  return (G = L.value) == null ? void 0 : G.click();
                })
              }, {
                default: l(() => [
                  v(d(e(o)("ops.import")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(g, {
                content: e(o)("ops.importHint"),
                placement: "top"
              }, {
                default: l(() => [
                  a(y, { class: "am-info" }, {
                    default: l(() => [
                      a(e(ne))
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["content"]),
              M("input", {
                ref_key: "importInput",
                ref: L,
                type: "file",
                accept: "application/json,.json",
                style: { display: "none" },
                onChange: O
              }, null, 544)
            ])
          ]),
          _: 1
        }),
        a(k, { shadow: "never" }, {
          header: l(() => [
            M("div", Ua, [
              M("span", null, d(e(o)("ops.doctorCard")), 1),
              a(P, {
                size: "small",
                icon: e(Be),
                loading: e(h),
                onClick: E[3] || (E[3] = (c) => B(e(f)))
              }, {
                default: l(() => [
                  v(d(e(o)("ops.runDoctor")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"])
            ])
          ]),
          default: l(() => [
            e(p) ? (b(), x(Z, { key: 0 }, [
              e(m).ok ? (b(), z(Y, {
                key: 0,
                title: e(o)("ops.doctorOk"),
                type: "success",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])) : (b(), z(Y, {
                key: 1,
                title: e(o)("ops.doctorFail", { count: e(m).issues.length }),
                type: "error",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])),
              e(m).ok ? W("", !0) : (b(), x("ul", Va, [
                (b(!0), x(Z, null, ae(e(m).issues, (c, G) => (b(), x("li", { key: G }, d(c), 1))), 128))
              ]))
            ], 64)) : (b(), z(F, {
              key: 1,
              description: e(o)("ops.doctorEmpty"),
              "image-size": 60
            }, null, 8, ["description"]))
          ]),
          _: 1
        })
      ], 512);
    };
  }
}), Da = /* @__PURE__ */ le($a, [["__scopeId", "data-v-a8956eae"]]), xa = { class: "memory-ui" }, za = { class: "brand" }, Ia = { class: "brand-mark" }, La = { class: "aside-footer" }, Na = /* @__PURE__ */ te({
  __name: "MemoryAdmin",
  props: {
    layout: { default: "sidebar" },
    title: { default: "agent-memory" }
  },
  setup(t) {
    const s = w("memories");
    return (n, r) => {
      const m = me, p = yt, h = _t, i = vt, C = wt, D = ht, S = bt, f = gt;
      return b(), x("div", xa, [
        a(f, { class: "layout" }, {
          default: l(() => [
            t.layout === "sidebar" ? (b(), z(i, {
              key: 0,
              width: "200px",
              class: "aside"
            }, {
              default: l(() => [
                M("div", za, [
                  M("span", Ia, [
                    a(m, { size: 16 }, {
                      default: l(() => [
                        a(e(St))
                      ]),
                      _: 1
                    })
                  ]),
                  M("span", null, d(t.title), 1)
                ]),
                a(h, {
                  "default-active": s.value,
                  class: "menu",
                  onSelect: r[0] || (r[0] = (_) => s.value = _)
                }, {
                  default: l(() => [
                    a(p, { index: "memories" }, {
                      default: l(() => [
                        a(m, null, {
                          default: l(() => [
                            a(e(Pt))
                          ]),
                          _: 1
                        }),
                        M("span", null, d(e(o)("nav.memories")), 1)
                      ]),
                      _: 1
                    }),
                    a(p, { index: "tags" }, {
                      default: l(() => [
                        a(m, null, {
                          default: l(() => [
                            a(e(Mt))
                          ]),
                          _: 1
                        }),
                        M("span", null, d(e(o)("nav.tags")), 1)
                      ]),
                      _: 1
                    }),
                    a(p, { index: "ops" }, {
                      default: l(() => [
                        a(m, null, {
                          default: l(() => [
                            a(e(Et))
                          ]),
                          _: 1
                        }),
                        M("span", null, d(e(o)("nav.ops")), 1)
                      ]),
                      _: 1
                    })
                  ]),
                  _: 1
                }, 8, ["default-active"]),
                M("div", La, [
                  at(n.$slots, "footer", {}, void 0, !0)
                ])
              ]),
              _: 3
            })) : W("", !0),
            a(S, { class: "main" }, {
              default: l(() => [
                t.layout === "tabs" ? (b(), z(D, {
                  key: 0,
                  modelValue: s.value,
                  "onUpdate:modelValue": r[1] || (r[1] = (_) => s.value = _),
                  class: "tabs-bar"
                }, {
                  default: l(() => [
                    a(C, {
                      label: e(o)("nav.memories"),
                      name: "memories"
                    }, null, 8, ["label"]),
                    a(C, {
                      label: e(o)("nav.tags"),
                      name: "tags"
                    }, null, 8, ["label"]),
                    a(C, {
                      label: e(o)("nav.ops"),
                      name: "ops"
                    }, null, 8, ["label"])
                  ]),
                  _: 1
                }, 8, ["modelValue"])) : W("", !0),
                oe(a(ca, null, null, 512), [
                  [ve, s.value === "memories"]
                ]),
                oe(a(_a, null, null, 512), [
                  [ve, s.value === "tags"]
                ]),
                oe(a(Da, null, null, 512), [
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
}), Ka = /* @__PURE__ */ le(Na, [["__scopeId", "data-v-6f56cb0b"]]);
export {
  Qe as MarkdownView,
  ca as MemoriesPanel,
  Ka as MemoryAdmin,
  ea as MemoryDetailDrawer,
  Wt as MemoryEditorDialog,
  qe as MemoryUIConfigKey,
  Da as OpsPanel,
  _a as TagsPanel,
  xt as applyMemoryUILocalePreference,
  Lt as buildMemoriesQuery,
  It as createApiClient,
  ja as currentMemoryUILocale,
  zt as formatSize,
  se as formatTime,
  _e as isSearchMode,
  pe as memoryUIi18n,
  Ja as provideMemoryUI,
  At as renderMarkdown,
  Rt as sanitizeHtml,
  Dt as setMemoryUILocale,
  o as t,
  ie as useApiClient,
  qt as useMemories,
  je as useMemoryConfig,
  Ca as useOps,
  ua as useTags
};
