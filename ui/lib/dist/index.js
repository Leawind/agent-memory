import { provide as Ze, inject as et, ref as y, computed as de, onMounted as ue, watch as ye, onUnmounted as tt, defineComponent as Z, openBlock as _, createElementBlock as L, createBlock as A, unref as e, withCtx as l, createVNode as a, createElementVNode as D, toDisplayString as c, createTextVNode as v, Fragment as X, renderList as oe, createCommentVNode as J, withKeys as at, isRef as ae, withDirectives as le, reactive as ot, renderSlot as lt, vShow as ve } from "vue";
import { createI18n as nt } from "vue-i18n";
import { ElDialog as De, ElForm as ze, ElFormItem as $e, ElInput as he, ElRadioGroup as be, ElRadioButton as Ve, ElSelect as Ue, ElOption as xe, ElButton as me, ElDrawer as st, ElTag as ke, ElDivider as it, ElTooltip as we, ElIcon as pe, ElAlert as Le, ElTable as Ae, ElTableColumn as Ie, ElLoadingDirective as Ne, ElPagination as rt, ElRadio as ct, ElRow as dt, ElCol as ut, ElCard as mt, ElStatistic as pt, ElDescriptions as ft, ElDescriptionsItem as gt, ElEmpty as vt, ElContainer as _t, ElAside as yt, ElMenu as ht, ElMenuItem as bt, ElMain as kt, ElTabs as wt, ElTabPane as Ct } from "element-plus/es";
import { InfoFilled as ce, Plus as Re, Search as Be, Refresh as Tt, Download as St, UploadFilled as Et, Collection as Mt, Notebook as Pt, PriceTag as Dt, Odometer as zt } from "@element-plus/icons-vue";
import { ElMessage as B, ElMessageBox as Oe } from "element-plus";
import { Marked as $t } from "marked";
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
    dbCard: "数据库",
    path: "文件路径",
    schemaVersion: "数据库版本",
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
    subtitle: "token 即身份（无账号体系）；每个身份的能力逐项开关",
    needAdmin: "需要 admin 权限才能管理身份与设置",
    openMode: "开放模式：鉴权开关未开启，所有请求免鉴权（适合个人本地部署）。在下方打开开关后，所有 /mcp 与 /api 请求必须携带 Authorization: Bearer <token>。开启前先创建管理员身份并保存其 token。",
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
    created: "token 仅此一次展示（服务端只存哈希），请立即复制保存：",
    resetToken: "重置 Token",
    resetTitle: "重置 Token",
    resetConfirm: "确定重置身份「{name}」的 token？旧 token 立即失效，新 token 仅展示一次。",
    editTitle: "编辑能力：{name}",
    deleteTitle: "删除身份",
    deleteConfirm: "确定删除身份「{name}」？其 token 将立即失效。",
    deleted: "已删除",
    saved: "已保存",
    settingsTitle: "自定义提示词（initialize instructions）",
    settingsHint: "返回给 agent 的 initialize 指令。基础提示词留空时使用内置默认；附加规范会追加在基础之后。",
    instructionsLabel: "基础提示词（覆盖内置默认）",
    instructionsPlaceholder: "留空 = 使用内置默认提示词",
    conventionsLabel: "附加规范（追加在基础之后）",
    conventionsPlaceholder: "如：标签命名约定、摘要书写要求；留空 = 不追加",
    restoreDefault: "恢复默认",
    viewDefault: "查看内置默认",
    empty: "尚无身份。先创建管理员身份并保存其 token，再打开上方的鉴权开关。",
    authTitle: "Token 鉴权",
    authHint: "开启后所有 /mcp 与 /api 请求必须携带 Authorization: Bearer <token>。开启前请先创建身份并保存其 token（开启后本浏览器会弹出令牌输入框）。",
    authEnableConfirm: "开启 token 鉴权？此后所有 /mcp 与 /api 请求都必须携带有效 token。",
    authDisableConfirm: "关闭 token 鉴权？所有请求将免鉴权放行（开放模式）。"
  }
}, Ut = {
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
    dbCard: "Database",
    path: "File path",
    schemaVersion: "Schema version",
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
    subtitle: "A token is an identity (no account system); capabilities are toggled per identity",
    needAdmin: "Admin permission is required to manage identities and settings",
    openMode: "Open mode: the auth switch is off, all requests are unauthenticated (fine for personal local deployments). Turn on the switch below to require Authorization: Bearer <token> on every /mcp and /api request. Create an admin identity and save its token first.",
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
    created: "Shown once only (the server stores a hash) — copy and save it now:",
    resetToken: "Reset token",
    resetTitle: "Reset token",
    resetConfirm: 'Reset the token of identity "{name}"? The old token is revoked immediately; the new one is shown once.',
    editTitle: "Edit capabilities: {name}",
    deleteTitle: "Delete identity",
    deleteConfirm: 'Delete identity "{name}"? Its token is revoked immediately.',
    deleted: "Deleted",
    saved: "Saved",
    settingsTitle: "Custom instructions (initialize instructions)",
    settingsHint: "Instructions returned to agents on initialize. An empty base prompt falls back to the built-in default; the conventions section is appended after the base.",
    instructionsLabel: "Base prompt (overrides the built-in default)",
    instructionsPlaceholder: "Leave empty = built-in default instructions",
    conventionsLabel: "Conventions (appended after the base)",
    conventionsPlaceholder: "e.g. tag naming rules, summary style; leave empty = none",
    restoreDefault: "Restore default",
    viewDefault: "View built-in default",
    empty: "No identities yet. Create an admin identity and save its token, then turn on the auth switch above.",
    authTitle: "Token auth",
    authHint: "When enabled, every /mcp and /api request must carry Authorization: Bearer <token>. Create an identity and save its token first (this browser will show the token prompt after enabling).",
    authEnableConfirm: "Enable token auth? All /mcp and /api requests will then require a valid token.",
    authDisableConfirm: "Disable token auth? All requests will be allowed without credentials (open mode)."
  }
};
function qe() {
  var t;
  return typeof navigator > "u" || (t = navigator.language) != null && t.toLowerCase().startsWith("zh") ? "zh" : "en";
}
const fe = nt({
  legacy: !1,
  locale: qe(),
  fallbackLocale: "zh",
  messages: { zh: Vt, en: Ut },
  // 面向宿主组件库，缺 key 时静默回退即可，不刷控制台
  missingWarn: !1,
  fallbackWarn: !1
}), { t: o } = fe.global;
function xt(t) {
  fe.global.locale.value = t;
}
function Lt(t) {
  xt(t === "auto" ? qe() : t);
}
function Ja() {
  return fe.global.locale.value;
}
const He = Symbol("memory-ui-config"), ie = {
  baseUrl: "",
  fetch: (...t) => globalThis.fetch(...t),
  defaultPageSize: 20,
  locale: "auto"
};
function Wa(t) {
  t.locale && Lt(t.locale), Ze(He, t);
}
function je() {
  const t = et(He);
  return {
    baseUrl: ((t == null ? void 0 : t.baseUrl) ?? ie.baseUrl).replace(/\/+$/, ""),
    fetch: (t == null ? void 0 : t.fetch) ?? ie.fetch,
    defaultPageSize: (t == null ? void 0 : t.defaultPageSize) ?? ie.defaultPageSize,
    locale: (t == null ? void 0 : t.locale) ?? ie.locale
  };
}
function re(t) {
  if (!t) return "—";
  const i = fe.global.locale.value === "en" ? "en-US" : "zh-CN";
  return new Date(t * 1e3).toLocaleString(i, { hour12: !1 });
}
function At(t) {
  return t == null ? "—" : t < 1024 ? `${t} B` : t < 1024 * 1024 ? `${(t / 1024).toFixed(1)} KB` : `${(t / 1024 / 1024).toFixed(2)} MB`;
}
function It(t) {
  async function i(n, f = {}) {
    const u = await t.fetch(t.baseUrl + n, {
      headers: { "Content-Type": "application/json" },
      ...f
    }), b = await u.text();
    let s = null;
    try {
      s = b ? JSON.parse(b) : null;
    } catch {
      s = null;
    }
    if (!u.ok) {
      const h = (s == null ? void 0 : s.error) ?? o("errors.http", { status: u.status });
      throw new Error(h);
    }
    return s;
  }
  async function r(n) {
    var u;
    const f = await t.fetch(t.baseUrl + n);
    if (!f.ok) {
      const b = await f.text().catch(() => "");
      let s = o("errors.http", { status: f.status });
      try {
        s = ((u = JSON.parse(b)) == null ? void 0 : u.error) ?? s;
      } catch {
      }
      throw new Error(s);
    }
    return f.blob();
  }
  return {
    get: (n) => i(n),
    post: (n, f) => i(n, { method: "POST", body: JSON.stringify(f ?? {}) }),
    put: (n, f) => i(n, { method: "PUT", body: JSON.stringify(f ?? {}) }),
    del: (n) => i(n, { method: "DELETE" }),
    getBlob: r
  };
}
function se() {
  return It(je());
}
function Nt(t) {
  const i = _e(t.query), r = new URLSearchParams();
  return i ? (r.set("query", t.query.trim()), t.tagFilter && r.set("tags", t.tagFilter)) : (t.tagFilter && r.set("tag", t.tagFilter), r.set("sort", t.sort), r.set("order", t.order)), r.set("offset", String((t.page - 1) * t.pageSize)), r.set("limit", String(t.pageSize)), r.toString();
}
function _e(t) {
  return t.trim().length > 0;
}
const Rt = new $t();
function Bt(t) {
  const i = Rt.parse(t, { async: !1 });
  return Fe.sanitize(i);
}
function Ot(t) {
  return Fe.sanitize(t);
}
function Ce(t, i) {
  return t.get(`/api/memories/${encodeURIComponent(i)}`);
}
function Je(t, i) {
  return t.post("/api/memories", i);
}
function We(t, i, r) {
  return t.put(`/api/memories/${encodeURIComponent(i)}`, r);
}
function Ft(t, i) {
  return t.del(`/api/memories/${encodeURIComponent(i)}`);
}
function Ke(t) {
  return t.get("/api/tags");
}
function qt(t, i, r) {
  return t.post("/api/tags", { name: i, description: r });
}
function Ht(t, i, r, n) {
  const f = { description: n }, u = r.trim();
  return u && u !== i && (f.new_name = u), t.put(`/api/tags/${encodeURIComponent(i)}`, f);
}
function jt(t, i, r) {
  return t.del(`/api/tags/${encodeURIComponent(i)}?mode=${r}`);
}
function Jt() {
  const t = se(), { defaultPageSize: i } = je(), r = y(""), n = y(""), f = y("updated_at"), u = y("desc"), b = y(1), s = y(i), h = y([]), U = y([]), z = y(0), m = y(""), E = y(!1), C = y([]), $ = de(() => _e(r.value));
  function M() {
    return b.value = 1, R();
  }
  function V() {
    return Nt({
      query: r.value,
      tagFilter: n.value,
      sort: f.value,
      order: u.value,
      page: b.value,
      pageSize: s.value
    });
  }
  let N = 0;
  async function R() {
    var w;
    const g = ++N;
    E.value = !0;
    try {
      const P = V();
      if (_e(r.value)) {
        const x = await t.get(`/api/memories?${P}`);
        if (g !== N) return;
        U.value = (x.results ?? []).map((T) => ({ ...T, snippet: Ot(T.snippet) })), z.value = x.total_matches ?? 0, m.value = "";
      } else {
        const x = await t.get(`/api/memories?${P}`);
        if (g !== N) return;
        const T = Math.max(1, Math.ceil(x.total / s.value));
        if (((w = x.memories) == null ? void 0 : w.length) === 0 && x.total > 0 && b.value > T)
          return b.value = T, E.value = !1, R();
        h.value = x.memories ?? [], z.value = x.total ?? 0, m.value = x.note ?? "";
      }
    } finally {
      g === N && (E.value = !1);
    }
  }
  async function F() {
    try {
      const g = await Ke(t);
      C.value = (g.tags ?? []).map((w) => w.name);
    } catch {
    }
  }
  async function O(g) {
    if (g.id) {
      const w = await Ce(t, g.id), P = new Set(w.tags), x = new Set(g.tags);
      await We(t, g.id, {
        summary: g.summary,
        content: g.content,
        add_tags: [...x].filter((T) => !P.has(T)),
        remove_tags: [...P].filter((T) => !x.has(T))
      });
    } else
      await Je(t, { summary: g.summary, content: g.content, tags: g.tags });
    await Promise.all([R(), F()]);
  }
  async function k(g) {
    await Ft(t, g), await R();
  }
  return ue(() => {
    R().catch(() => {
    }), F();
  }), {
    query: r,
    tagFilter: n,
    sort: f,
    order: u,
    page: b,
    pageSize: s,
    rows: h,
    searchResults: U,
    total: z,
    note: m,
    loading: E,
    tagOptions: C,
    searching: $,
    onSearch: M,
    reload: R,
    loadTagOptions: F,
    saveMemory: O,
    removeMemory: k
  };
}
function Te(t, i = 720) {
  const r = y(0);
  let n = null;
  function f(u) {
    n == null || n.disconnect(), n = null, !(!u || typeof ResizeObserver > "u") && (n = new ResizeObserver((b) => {
      var s;
      r.value = ((s = b[0]) == null ? void 0 : s.contentRect.width) ?? 0;
    }), n.observe(u));
  }
  return ue(() => f(t.value)), ye(t, (u) => f(u)), tt(() => n == null ? void 0 : n.disconnect()), { width: r, compact: de(() => r.value > 0 && r.value < i) };
}
const Wt = ["innerHTML"], Qe = /* @__PURE__ */ Z({
  __name: "MarkdownView",
  props: {
    source: {}
  },
  setup(t) {
    const i = t, r = de(() => Bt(i.source));
    return (n, f) => (_(), L("div", {
      class: "md-body",
      innerHTML: r.value
    }, null, 8, Wt));
  }
}), Kt = { class: "content-label" }, Qt = /* @__PURE__ */ Z({
  __name: "MemoryEditorDialog",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    tagOptions: {},
    width: {}
  },
  emits: ["update:visible", "saved"],
  setup(t, { emit: i }) {
    const r = t, n = i, f = se(), u = y(!1), b = y("edit"), s = y({ id: null, summary: "", content: "", tags: [] });
    let h = [];
    ye(
      () => r.visible,
      async (z) => {
        if (z)
          if (b.value = "edit", r.memoryId)
            try {
              const m = await Ce(f, r.memoryId);
              s.value = { id: m.id, summary: m.summary, content: m.content, tags: [...m.tags] }, h = [...m.tags];
            } catch (m) {
              B.error(m instanceof Error ? m.message : String(m)), n("update:visible", !1);
            }
          else
            s.value = { id: null, summary: "", content: "", tags: [] }, h = [];
      }
    );
    async function U() {
      u.value = !0;
      try {
        if (s.value.id) {
          const z = new Set(h), m = new Set(s.value.tags);
          await We(f, s.value.id, {
            summary: s.value.summary,
            content: s.value.content,
            add_tags: [...m].filter((E) => !z.has(E)),
            remove_tags: [...z].filter((E) => !m.has(E))
          }), B.success(o("editor.updated"));
        } else
          await Je(f, {
            summary: s.value.summary,
            content: s.value.content,
            tags: s.value.tags
          }), B.success(o("editor.created"));
        n("update:visible", !1), n("saved");
      } catch (z) {
        B.error(z instanceof Error ? z.message : String(z));
      } finally {
        u.value = !1;
      }
    }
    return (z, m) => {
      const E = he, C = $e, $ = Ve, M = be, V = xe, N = Ue, R = ze, F = me, O = De;
      return _(), A(O, {
        "model-value": t.visible,
        title: s.value.id ? e(o)("editor.editTitle") : e(o)("editor.createTitle"),
        width: t.width,
        "onUpdate:modelValue": m[5] || (m[5] = (k) => n("update:visible", k))
      }, {
        footer: l(() => [
          a(F, {
            onClick: m[4] || (m[4] = (k) => n("update:visible", !1))
          }, {
            default: l(() => [
              v(c(e(o)("common.cancel")), 1)
            ]),
            _: 1
          }),
          a(F, {
            type: "primary",
            loading: u.value,
            onClick: U
          }, {
            default: l(() => [
              v(c(e(o)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: l(() => [
          a(R, { "label-position": "top" }, {
            default: l(() => [
              a(C, {
                label: e(o)("editor.summaryLabel")
              }, {
                default: l(() => [
                  a(E, {
                    modelValue: s.value.summary,
                    "onUpdate:modelValue": m[0] || (m[0] = (k) => s.value.summary = k),
                    maxlength: "512",
                    "show-word-limit": "",
                    placeholder: e(o)("editor.summaryPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"]),
              a(C, null, {
                label: l(() => [
                  D("div", Kt, [
                    D("span", null, c(e(o)("editor.contentLabel")), 1),
                    a(M, {
                      modelValue: b.value,
                      "onUpdate:modelValue": m[1] || (m[1] = (k) => b.value = k),
                      size: "small"
                    }, {
                      default: l(() => [
                        a($, { value: "edit" }, {
                          default: l(() => [
                            v(c(e(o)("editor.tabEdit")), 1)
                          ]),
                          _: 1
                        }),
                        a($, { value: "preview" }, {
                          default: l(() => [
                            v(c(e(o)("editor.tabPreview")), 1)
                          ]),
                          _: 1
                        })
                      ]),
                      _: 1
                    }, 8, ["modelValue"])
                  ])
                ]),
                default: l(() => [
                  b.value === "edit" ? (_(), A(E, {
                    key: 0,
                    modelValue: s.value.content,
                    "onUpdate:modelValue": m[2] || (m[2] = (k) => s.value.content = k),
                    type: "textarea",
                    rows: 12,
                    maxlength: "200000",
                    "show-word-limit": "",
                    placeholder: e(o)("editor.contentPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])) : (_(), A(Qe, {
                    key: 1,
                    class: "content-preview",
                    source: s.value.content
                  }, null, 8, ["source"]))
                ]),
                _: 1
              }),
              a(C, {
                label: e(o)("editor.tagsLabel")
              }, {
                default: l(() => [
                  a(N, {
                    modelValue: s.value.tags,
                    "onUpdate:modelValue": m[3] || (m[3] = (k) => s.value.tags = k),
                    multiple: "",
                    filterable: "",
                    "allow-create": "",
                    "default-first-option": "",
                    placeholder: e(o)("editor.tagsPlaceholder"),
                    class: "tags-select"
                  }, {
                    default: l(() => [
                      (_(!0), L(X, null, oe(t.tagOptions, (k) => (_(), A(V, {
                        key: k,
                        label: k,
                        value: k
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
}), ne = (t, i) => {
  const r = t.__vccOpts || t;
  for (const [n, f] of i)
    r[n] = f;
  return r;
}, Gt = /* @__PURE__ */ ne(Qt, [["__scopeId", "data-v-602bada0"]]), Xt = { class: "detail-summary" }, Yt = { class: "detail-tags" }, Zt = { class: "detail-toolbar" }, ea = {
  key: 1,
  class: "detail-content"
}, ta = /* @__PURE__ */ Z({
  __name: "MemoryDetailDrawer",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    size: {}
  },
  emits: ["update:visible"],
  setup(t, { emit: i }) {
    const r = t, n = i, f = se(), u = y(null), b = y("rendered");
    return ye(
      () => [r.visible, r.memoryId],
      async ([s]) => {
        if (!(!s || !r.memoryId)) {
          b.value = "rendered";
          try {
            u.value = await Ce(f, r.memoryId);
          } catch (h) {
            B.error(h instanceof Error ? h.message : String(h)), n("update:visible", !1);
          }
        }
      }
    ), (s, h) => {
      var $;
      const U = ke, z = it, m = Ve, E = be, C = st;
      return _(), A(C, {
        "model-value": t.visible,
        title: e(o)("drawer.title", { id: (($ = u.value) == null ? void 0 : $.id) ?? t.memoryId ?? "" }),
        size: t.size,
        "onUpdate:modelValue": h[1] || (h[1] = (M) => n("update:visible", M))
      }, {
        default: l(() => [
          u.value ? (_(), L(X, { key: 0 }, [
            D("h3", Xt, c(u.value.summary), 1),
            D("div", Yt, [
              (_(!0), L(X, null, oe(u.value.tags, (M) => (_(), A(U, {
                key: M,
                size: "small",
                class: "am-tag"
              }, {
                default: l(() => [
                  v(c(M), 1)
                ]),
                _: 2
              }, 1024))), 128))
            ]),
            a(z),
            D("div", Zt, [
              a(E, {
                modelValue: b.value,
                "onUpdate:modelValue": h[0] || (h[0] = (M) => b.value = M),
                size: "small"
              }, {
                default: l(() => [
                  a(m, { value: "rendered" }, {
                    default: l(() => [
                      v(c(e(o)("drawer.rendered")), 1)
                    ]),
                    _: 1
                  }),
                  a(m, { value: "source" }, {
                    default: l(() => [
                      v(c(e(o)("drawer.source")), 1)
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["modelValue"])
            ]),
            b.value === "rendered" ? (_(), A(Qe, {
              key: 0,
              source: u.value.content
            }, null, 8, ["source"])) : (_(), L("pre", ea, c(u.value.content), 1))
          ], 64)) : J("", !0)
        ]),
        _: 1
      }, 8, ["model-value", "title", "size"]);
    };
  }
}), aa = /* @__PURE__ */ ne(ta, [["__scopeId", "data-v-b634d116"]]), oa = {
  key: 0,
  class: "am-panel-header"
}, la = { class: "am-heading" }, na = { class: "am-panel-title" }, sa = { class: "am-toolbar" }, ia = { class: "am-summary" }, ra = ["innerHTML"], ca = {
  key: 4,
  class: "am-pager"
}, da = {
  key: 5,
  class: "am-pager"
}, ua = /* @__PURE__ */ Z({
  __name: "MemoriesPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: i }) {
    const r = t, {
      query: n,
      tagFilter: f,
      sort: u,
      order: b,
      page: s,
      pageSize: h,
      rows: U,
      searchResults: z,
      total: m,
      note: E,
      loading: C,
      tagOptions: $,
      searching: M,
      onSearch: V,
      reload: N,
      loadTagOptions: R,
      removeMemory: F
    } = Jt(), O = y(null), { compact: k } = Te(O), g = y(!1), w = y(null), P = y(!1), x = y(null);
    function T(I) {
      return I().catch((d) => B.error(d instanceof Error ? d.message : String(d)));
    }
    function W() {
      w.value = null, g.value = !0;
    }
    function j(I) {
      w.value = I, g.value = !0;
    }
    function K(I) {
      x.value = I, P.value = !0;
    }
    function Y() {
      T(N), R();
    }
    function Q(I) {
      const { prop: d, order: te } = I;
      te && (d === "updated_at" || d === "created_at") ? (u.value = d, b.value = te === "ascending" ? "asc" : "desc") : (u.value = "updated_at", b.value = "desc"), T(N);
    }
    async function ee(I) {
      try {
        await Oe.confirm(o("memories.deleteConfirm", { id: I.id }), o("memories.deleteTitle"), {
          type: "warning"
        });
      } catch {
        return;
      }
      try {
        await F(I.id), B.success(o("memories.deleted"));
      } catch (d) {
        B.error(d instanceof Error ? d.message : String(d));
      }
    }
    return i({ refresh: () => T(N) }), (I, d) => {
      const te = pe, ge = we, S = me, G = he, Ge = xe, Xe = Ue, Ye = Le, q = Ie, Se = ke, Ee = Ae, Me = rt, Pe = Ne;
      return _(), L("div", {
        ref_key: "rootRef",
        ref: O,
        class: "am-panel"
      }, [
        t.showHeader ? (_(), L("div", oa, [
          D("div", la, [
            D("h2", na, c(r.title ?? e(o)("memories.title")), 1),
            a(ge, {
              content: r.subtitle ?? e(o)("memories.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(te, { class: "am-info" }, {
                  default: l(() => [
                    a(e(ce))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(S, {
            type: "primary",
            icon: e(Re),
            onClick: W
          }, {
            default: l(() => [
              v(c(e(o)("memories.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : J("", !0),
        D("div", sa, [
          a(G, {
            modelValue: e(n),
            "onUpdate:modelValue": d[1] || (d[1] = (p) => ae(n) ? n.value = p : null),
            placeholder: e(o)("memories.searchPlaceholder"),
            clearable: "",
            class: "search",
            onKeyup: d[2] || (d[2] = at((p) => T(e(V)), ["enter"])),
            onClear: d[3] || (d[3] = (p) => T(e(V)))
          }, {
            append: l(() => [
              a(S, {
                icon: e(Be),
                onClick: d[0] || (d[0] = (p) => T(e(V)))
              }, null, 8, ["icon"])
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(Xe, {
            modelValue: e(f),
            "onUpdate:modelValue": d[4] || (d[4] = (p) => ae(f) ? f.value = p : null),
            placeholder: e(o)("memories.tagFilter"),
            clearable: "",
            filterable: "",
            class: "tag-filter",
            onChange: d[5] || (d[5] = (p) => T(e(V)))
          }, {
            default: l(() => [
              (_(!0), L(X, null, oe(e($), (p) => (_(), A(Ge, {
                key: p,
                label: p,
                value: p
              }, null, 8, ["label", "value"]))), 128))
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"])
        ]),
        e(E) ? (_(), A(Ye, {
          key: 1,
          title: e(E),
          type: "info",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : J("", !0),
        e(M) ? le((_(), A(Ee, {
          key: 2,
          data: e(z)
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
              default: l(({ row: p }) => [
                D("div", ia, c(p.summary), 1),
                D("div", {
                  class: "am-snippet",
                  innerHTML: p.snippet
                }, null, 8, ra)
              ]),
              _: 1
            }, 8, ["label"]),
            a(q, {
              label: e(o)("memories.colTags"),
              width: "220"
            }, {
              default: l(({ row: p }) => [
                (_(!0), L(X, null, oe(p.tags, (H) => (_(), A(Se, {
                  key: H,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: l(() => [
                    v(c(H), 1)
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
              default: l(({ row: p }) => [
                v(c(e(re)(p.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(q, {
              label: e(o)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: l(({ row: p }) => [
                a(S, {
                  link: "",
                  type: "primary",
                  onClick: (H) => K(p.id)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(S, {
                  link: "",
                  type: "primary",
                  onClick: (H) => j(p.id)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(S, {
                  link: "",
                  type: "danger",
                  onClick: (H) => ee(p)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [Pe, e(C)]
        ]) : le((_(), A(Ee, {
          key: 3,
          data: e(U),
          "default-sort": { prop: e(u), order: e(b) === "asc" ? "ascending" : "descending" },
          onSortChange: Q
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
              default: l(({ row: p }) => [
                (_(!0), L(X, null, oe(p.tags, (H) => (_(), A(Se, {
                  key: H,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: l(() => [
                    v(c(H), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            e(k) ? J("", !0) : (_(), A(q, {
              key: 0,
              prop: "created_at",
              label: e(o)("memories.colCreatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: l(({ row: p }) => [
                v(c(e(re)(p.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label"])),
            a(q, {
              prop: "updated_at",
              label: e(o)("memories.colUpdatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: l(({ row: p }) => [
                v(c(e(re)(p.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(q, {
              label: e(o)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: l(({ row: p }) => [
                a(S, {
                  link: "",
                  type: "primary",
                  onClick: (H) => K(p.id)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(S, {
                  link: "",
                  type: "primary",
                  onClick: (H) => j(p.id)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(S, {
                  link: "",
                  type: "danger",
                  onClick: (H) => ee(p)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data", "default-sort"])), [
          [Pe, e(C)]
        ]),
        e(M) ? (_(), L("div", da, [
          a(Me, {
            "current-page": e(s),
            "onUpdate:currentPage": d[10] || (d[10] = (p) => ae(s) ? s.value = p : null),
            "page-size": e(h),
            "onUpdate:pageSize": d[11] || (d[11] = (p) => ae(h) ? h.value = p : null),
            total: e(m),
            "page-sizes": [10, 20, 50],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: d[12] || (d[12] = (p) => T(e(N))),
            onSizeChange: d[13] || (d[13] = (p) => T(e(N)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])) : (_(), L("div", ca, [
          a(Me, {
            "current-page": e(s),
            "onUpdate:currentPage": d[6] || (d[6] = (p) => ae(s) ? s.value = p : null),
            "page-size": e(h),
            "onUpdate:pageSize": d[7] || (d[7] = (p) => ae(h) ? h.value = p : null),
            total: e(m),
            "page-sizes": [20, 50, 100, 200],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: d[8] || (d[8] = (p) => T(e(N))),
            onSizeChange: d[9] || (d[9] = (p) => T(e(N)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])),
        a(Gt, {
          visible: g.value,
          "onUpdate:visible": d[14] || (d[14] = (p) => g.value = p),
          "memory-id": w.value,
          "tag-options": e($),
          width: e(k) ? "96%" : "640px",
          onSaved: Y
        }, null, 8, ["visible", "memory-id", "tag-options", "width"]),
        a(aa, {
          visible: P.value,
          "onUpdate:visible": d[15] || (d[15] = (p) => P.value = p),
          "memory-id": x.value,
          size: e(k) ? "100%" : "45%"
        }, null, 8, ["visible", "memory-id", "size"])
      ], 512);
    };
  }
}), ma = /* @__PURE__ */ ne(ua, [["__scopeId", "data-v-1ab68c72"]]);
function pa() {
  const t = se(), i = y([]), r = y(!1);
  async function n() {
    r.value = !0;
    try {
      const s = await Ke(t);
      i.value = s.tags ?? [];
    } finally {
      r.value = !1;
    }
  }
  async function f(s, h) {
    await qt(t, s, h), await n();
  }
  async function u(s, h, U) {
    await Ht(t, s, h, U), await n();
  }
  async function b(s, h) {
    await jt(t, s, h), await n();
  }
  return ue(() => {
    n().catch(() => {
    });
  }), { rows: i, loading: r, reload: n, create: f, rename: u, remove: b };
}
const fa = {
  key: 0,
  class: "am-panel-header"
}, ga = { class: "am-heading" }, va = { class: "am-panel-title" }, _a = { class: "delete-body" }, ya = /* @__PURE__ */ Z({
  __name: "TagsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: i }) {
    const r = t, { rows: n, loading: f, reload: u, create: b, rename: s, remove: h } = pa(), U = y(null), { compact: z } = Te(U), m = y(!1), E = y(!1), C = ot({ oldName: null, name: "", newName: "", description: "" }), $ = y(!1), M = y("detach"), V = y(null);
    function N() {
      Object.assign(C, { oldName: null, name: "", newName: "", description: "" }), E.value = !0;
    }
    function R(g) {
      Object.assign(C, { oldName: g.name, name: g.name, newName: "", description: g.description ?? "" }), E.value = !0;
    }
    async function F() {
      m.value = !0;
      try {
        C.oldName ? (await s(C.oldName, C.newName, C.description), B.success(o("tags.saved"))) : (await b(C.name, C.description), B.success(o("tags.created"))), E.value = !1;
      } catch (g) {
        B.error(g instanceof Error ? g.message : String(g));
      } finally {
        m.value = !1;
      }
    }
    function O(g) {
      V.value = g, M.value = "detach", $.value = !0;
    }
    async function k() {
      var g, w;
      if (M.value === "purge")
        try {
          await Oe.confirm(
            o("tags.purgeConfirm", { name: (g = V.value) == null ? void 0 : g.name, count: ((w = V.value) == null ? void 0 : w.memory_count) ?? 0 }),
            o("tags.purgeConfirmTitle"),
            { type: "error", confirmButtonText: o("tags.purgeButton") }
          );
        } catch {
          return;
        }
      if (V.value) {
        m.value = !0;
        try {
          await h(V.value.name, M.value), B.success(o("tags.deleted")), $.value = !1;
        } catch (P) {
          B.error(P instanceof Error ? P.message : String(P));
        } finally {
          m.value = !1;
        }
      }
    }
    return i({
      refresh: () => u().catch((g) => B.error(g instanceof Error ? g.message : String(g)))
    }), (g, w) => {
      const P = pe, x = we, T = me, W = ke, j = Ie, K = Ae, Y = he, Q = $e, ee = ze, I = De, d = ct, te = be, ge = Ne;
      return _(), L("div", {
        ref_key: "rootRef",
        ref: U,
        class: "am-panel"
      }, [
        t.showHeader ? (_(), L("div", fa, [
          D("div", ga, [
            D("h2", va, c(r.title ?? e(o)("tags.title")), 1),
            a(x, {
              content: r.subtitle ?? e(o)("tags.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(P, { class: "am-info" }, {
                  default: l(() => [
                    a(e(ce))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(T, {
            type: "primary",
            icon: e(Re),
            onClick: N
          }, {
            default: l(() => [
              v(c(e(o)("tags.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : J("", !0),
        le((_(), A(K, { data: e(n) }, {
          default: l(() => [
            a(j, {
              prop: "name",
              label: e(o)("tags.colName"),
              "min-width": "160"
            }, {
              default: l(({ row: S }) => [
                a(W, null, {
                  default: l(() => [
                    v(c(S.name), 1)
                  ]),
                  _: 2
                }, 1024)
              ]),
              _: 1
            }, 8, ["label"]),
            a(j, {
              prop: "description",
              label: e(o)("tags.colDescription"),
              "min-width": "300",
              "show-overflow-tooltip": ""
            }, {
              default: l(({ row: S }) => [
                v(c(S.description || "—"), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(j, {
              prop: "memory_count",
              label: e(o)("tags.colMemoryCount"),
              width: "100",
              sortable: ""
            }, null, 8, ["label"]),
            a(j, {
              label: e(o)("tags.colLastUsed"),
              width: "170"
            }, {
              default: l(({ row: S }) => [
                v(c(e(re)(S.last_used_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(j, {
              label: e(o)("memories.colActions"),
              width: "150",
              fixed: "right"
            }, {
              default: l(({ row: S }) => [
                a(T, {
                  link: "",
                  type: "primary",
                  onClick: (G) => R(S)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(T, {
                  link: "",
                  type: "danger",
                  onClick: (G) => O(S)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [ge, e(f)]
        ]),
        a(I, {
          modelValue: E.value,
          "onUpdate:modelValue": w[4] || (w[4] = (S) => E.value = S),
          title: C.oldName ? e(o)("tags.editTitle") : e(o)("tags.createTitle"),
          width: e(z) ? "96%" : "480px"
        }, {
          footer: l(() => [
            a(T, {
              onClick: w[3] || (w[3] = (S) => E.value = !1)
            }, {
              default: l(() => [
                v(c(e(o)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(T, {
              type: "primary",
              loading: m.value,
              onClick: F
            }, {
              default: l(() => [
                v(c(e(o)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: l(() => [
            a(ee, { "label-position": "top" }, {
              default: l(() => [
                a(Q, {
                  label: e(o)("tags.nameLabel")
                }, {
                  default: l(() => [
                    a(Y, {
                      modelValue: C.name,
                      "onUpdate:modelValue": w[0] || (w[0] = (S) => C.name = S),
                      disabled: !!C.oldName,
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(o)("tags.namePlaceholder")
                    }, null, 8, ["modelValue", "disabled", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                C.oldName ? (_(), A(Q, {
                  key: 0,
                  label: e(o)("tags.renameLabel")
                }, {
                  default: l(() => [
                    a(Y, {
                      modelValue: C.newName,
                      "onUpdate:modelValue": w[1] || (w[1] = (S) => C.newName = S),
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(o)("tags.renamePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"])) : J("", !0),
                a(Q, {
                  label: e(o)("tags.descLabel")
                }, {
                  default: l(() => [
                    a(Y, {
                      modelValue: C.description,
                      "onUpdate:modelValue": w[2] || (w[2] = (S) => C.description = S),
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
        a(I, {
          modelValue: $.value,
          "onUpdate:modelValue": w[7] || (w[7] = (S) => $.value = S),
          title: e(o)("tags.deleteTitle"),
          width: e(z) ? "96%" : "480px"
        }, {
          footer: l(() => [
            a(T, {
              onClick: w[6] || (w[6] = (S) => $.value = !1)
            }, {
              default: l(() => [
                v(c(e(o)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(T, {
              type: "danger",
              loading: m.value,
              onClick: k
            }, {
              default: l(() => [
                v(c(e(o)("common.delete")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: l(() => {
            var S;
            return [
              D("p", _a, [
                v(c(e(o)("tags.deleteBefore")) + " ", 1),
                a(W, null, {
                  default: l(() => {
                    var G;
                    return [
                      v(c((G = V.value) == null ? void 0 : G.name), 1)
                    ];
                  }),
                  _: 1
                }),
                v(" " + c(e(o)("tags.deleteMiddle")) + " ", 1),
                D("b", null, c((S = V.value) == null ? void 0 : S.memory_count), 1),
                v(" " + c(e(o)("tags.deleteAfter")), 1)
              ]),
              a(te, {
                modelValue: M.value,
                "onUpdate:modelValue": w[5] || (w[5] = (G) => M.value = G)
              }, {
                default: l(() => [
                  a(d, { value: "detach" }, {
                    default: l(() => [
                      v(c(e(o)("tags.detach")), 1)
                    ]),
                    _: 1
                  }),
                  a(d, { value: "purge" }, {
                    default: l(() => [
                      v(c(e(o)("tags.purge")), 1)
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
}), ha = /* @__PURE__ */ ne(ya, [["__scopeId", "data-v-9847d5f4"]]);
function ba(t) {
  return t.get("/api/stats");
}
function ka(t) {
  return t.get("/health");
}
function wa(t) {
  return t.get("/api/doctor");
}
function Ca(t) {
  return t.getBlob("/api/export");
}
function Ta(t, i) {
  return t.post("/api/import", i);
}
function Sa() {
  const t = se(), i = y({}), r = y(""), n = y({ ok: !0, issues: [] }), f = y(!1), u = y(!1), b = y(!1), s = y(!1), h = de(() => At(i.value.file_size));
  async function U() {
    i.value = await ba(t), i.value.version = r.value;
  }
  async function z() {
    u.value = !0;
    try {
      n.value = await wa(t), f.value = !0;
    } finally {
      u.value = !1;
    }
  }
  async function m() {
    b.value = !0;
    try {
      const C = await Ca(t), $ = URL.createObjectURL(C), M = document.createElement("a");
      M.href = $, M.download = "agent-memory-export.json", M.click(), URL.revokeObjectURL($);
    } finally {
      b.value = !1;
    }
  }
  async function E(C) {
    s.value = !0;
    try {
      const $ = await C.text();
      let M;
      try {
        M = JSON.parse($);
      } catch {
        throw new Error(o("errors.invalidBackup"));
      }
      const V = await Ta(t, M);
      return await U(), V;
    } finally {
      s.value = !1;
    }
  }
  return ue(async () => {
    try {
      r.value = (await ka(t)).version ?? "";
    } catch {
    }
    await U().catch(() => {
    });
  }), {
    stats: i,
    version: r,
    doctor: n,
    doctorRan: f,
    doctorLoading: u,
    exporting: b,
    importing: s,
    sizeText: h,
    reload: U,
    runDoctor: z,
    exportData: m,
    importFile: E
  };
}
const Ea = {
  key: 0,
  class: "am-panel-header"
}, Ma = { class: "am-heading" }, Pa = { class: "am-panel-title" }, Da = { class: "actions" }, za = { class: "card-header" }, $a = {
  key: 2,
  class: "issues"
}, Va = /* @__PURE__ */ Z({
  __name: "OpsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: i }) {
    const r = t, {
      stats: n,
      version: f,
      doctor: u,
      doctorRan: b,
      doctorLoading: s,
      exporting: h,
      importing: U,
      sizeText: z,
      reload: m,
      runDoctor: E,
      exportData: C,
      importFile: $
    } = Sa(), M = y(null), { compact: V } = Te(M), N = y(null);
    function R(O) {
      return O().catch((k) => B.error(k instanceof Error ? k.message : String(k)));
    }
    function F(O) {
      var w;
      const k = O.target, g = (w = k.files) == null ? void 0 : w[0];
      k.value = "", g && $(g).then((P) => {
        B.success(o("ops.imported", { memories: P.imported_memories, tags: P.imported_tags }));
      }).catch((P) => {
        B.error(P instanceof Error ? P.message : String(P));
      });
    }
    return i({ refresh: () => R(m) }), (O, k) => {
      const g = pe, w = we, P = me, x = pt, T = mt, W = ut, j = dt, K = gt, Y = ft, Q = Le, ee = vt;
      return _(), L("div", {
        ref_key: "rootRef",
        ref: M,
        class: "am-panel"
      }, [
        t.showHeader ? (_(), L("div", Ea, [
          D("div", Ma, [
            D("h2", Pa, c(r.title ?? e(o)("ops.title")), 1),
            a(w, {
              content: r.subtitle ?? e(o)("ops.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(g, { class: "am-info" }, {
                  default: l(() => [
                    a(e(ce))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(P, {
            icon: e(Tt),
            onClick: k[0] || (k[0] = (I) => R(e(m)))
          }, {
            default: l(() => [
              v(c(e(o)("ops.refresh")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : J("", !0),
        a(j, { gutter: 14 }, {
          default: l(() => [
            a(W, {
              span: e(V) ? 12 : 8
            }, {
              default: l(() => [
                a(T, { shadow: "never" }, {
                  default: l(() => [
                    a(x, {
                      title: e(o)("ops.statMemories"),
                      value: e(n).memories ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(W, {
              span: e(V) ? 12 : 8
            }, {
              default: l(() => [
                a(T, { shadow: "never" }, {
                  default: l(() => [
                    a(x, {
                      title: e(o)("ops.statTags"),
                      value: e(n).tags ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(W, {
              span: e(V) ? 12 : 8
            }, {
              default: l(() => [
                a(T, { shadow: "never" }, {
                  default: l(() => [
                    a(x, {
                      title: e(o)("ops.statSize"),
                      value: e(z)
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
        a(T, { shadow: "never" }, {
          header: l(() => [
            v(c(e(o)("ops.dbCard")), 1)
          ]),
          default: l(() => [
            a(Y, {
              column: e(V) ? 1 : 2,
              border: ""
            }, {
              default: l(() => [
                a(K, {
                  label: e(o)("ops.path")
                }, {
                  default: l(() => [
                    v(c(e(n).path ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(K, {
                  label: e(o)("ops.schemaVersion")
                }, {
                  default: l(() => [
                    v(c(e(n).schema_version ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(K, {
                  label: e(o)("ops.version")
                }, {
                  default: l(() => [
                    v(c(e(n).version ?? e(f)), 1)
                  ]),
                  _: 1
                }, 8, ["label"])
              ]),
              _: 1
            }, 8, ["column"]),
            D("div", Da, [
              a(P, {
                icon: e(St),
                loading: e(h),
                onClick: k[1] || (k[1] = (I) => R(e(C)))
              }, {
                default: l(() => [
                  v(c(e(o)("ops.export")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(P, {
                icon: e(Et),
                loading: e(U),
                onClick: k[2] || (k[2] = (I) => {
                  var d;
                  return (d = N.value) == null ? void 0 : d.click();
                })
              }, {
                default: l(() => [
                  v(c(e(o)("ops.import")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(w, {
                content: e(o)("ops.importHint"),
                placement: "top"
              }, {
                default: l(() => [
                  a(g, { class: "am-info" }, {
                    default: l(() => [
                      a(e(ce))
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["content"]),
              D("input", {
                ref_key: "importInput",
                ref: N,
                type: "file",
                accept: "application/json,.json",
                style: { display: "none" },
                onChange: F
              }, null, 544)
            ])
          ]),
          _: 1
        }),
        a(T, { shadow: "never" }, {
          header: l(() => [
            D("div", za, [
              D("span", null, c(e(o)("ops.doctorCard")), 1),
              a(P, {
                size: "small",
                icon: e(Be),
                loading: e(s),
                onClick: k[3] || (k[3] = (I) => R(e(E)))
              }, {
                default: l(() => [
                  v(c(e(o)("ops.runDoctor")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"])
            ])
          ]),
          default: l(() => [
            e(b) ? (_(), L(X, { key: 0 }, [
              e(u).ok ? (_(), A(Q, {
                key: 0,
                title: e(o)("ops.doctorOk"),
                type: "success",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])) : (_(), A(Q, {
                key: 1,
                title: e(o)("ops.doctorFail", { count: e(u).issues.length }),
                type: "error",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])),
              e(u).ok ? J("", !0) : (_(), L("ul", $a, [
                (_(!0), L(X, null, oe(e(u).issues, (I, d) => (_(), L("li", { key: d }, c(I), 1))), 128))
              ]))
            ], 64)) : (_(), A(ee, {
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
}), Ua = /* @__PURE__ */ ne(Va, [["__scopeId", "data-v-27c1e600"]]), xa = { class: "memory-ui" }, La = { class: "brand" }, Aa = { class: "brand-mark" }, Ia = { class: "aside-footer" }, Na = /* @__PURE__ */ Z({
  __name: "MemoryAdmin",
  props: {
    layout: { default: "sidebar" },
    title: { default: "agent-memory" }
  },
  setup(t) {
    const i = y("memories");
    return (r, n) => {
      const f = pe, u = bt, b = ht, s = yt, h = Ct, U = wt, z = kt, m = _t;
      return _(), L("div", xa, [
        a(m, { class: "layout" }, {
          default: l(() => [
            t.layout === "sidebar" ? (_(), A(s, {
              key: 0,
              width: "200px",
              class: "aside"
            }, {
              default: l(() => [
                D("div", La, [
                  D("span", Aa, [
                    a(f, { size: 16 }, {
                      default: l(() => [
                        a(e(Mt))
                      ]),
                      _: 1
                    })
                  ]),
                  D("span", null, c(t.title), 1)
                ]),
                a(b, {
                  "default-active": i.value,
                  class: "menu",
                  onSelect: n[0] || (n[0] = (E) => i.value = E)
                }, {
                  default: l(() => [
                    a(u, { index: "memories" }, {
                      default: l(() => [
                        a(f, null, {
                          default: l(() => [
                            a(e(Pt))
                          ]),
                          _: 1
                        }),
                        D("span", null, c(e(o)("nav.memories")), 1)
                      ]),
                      _: 1
                    }),
                    a(u, { index: "tags" }, {
                      default: l(() => [
                        a(f, null, {
                          default: l(() => [
                            a(e(Dt))
                          ]),
                          _: 1
                        }),
                        D("span", null, c(e(o)("nav.tags")), 1)
                      ]),
                      _: 1
                    }),
                    a(u, { index: "ops" }, {
                      default: l(() => [
                        a(f, null, {
                          default: l(() => [
                            a(e(zt))
                          ]),
                          _: 1
                        }),
                        D("span", null, c(e(o)("nav.ops")), 1)
                      ]),
                      _: 1
                    })
                  ]),
                  _: 1
                }, 8, ["default-active"]),
                D("div", Ia, [
                  lt(r.$slots, "footer", {}, void 0, !0)
                ])
              ]),
              _: 3
            })) : J("", !0),
            a(z, { class: "main" }, {
              default: l(() => [
                t.layout === "tabs" ? (_(), A(U, {
                  key: 0,
                  modelValue: i.value,
                  "onUpdate:modelValue": n[1] || (n[1] = (E) => i.value = E),
                  class: "tabs-bar"
                }, {
                  default: l(() => [
                    a(h, {
                      label: e(o)("nav.memories"),
                      name: "memories"
                    }, null, 8, ["label"]),
                    a(h, {
                      label: e(o)("nav.tags"),
                      name: "tags"
                    }, null, 8, ["label"]),
                    a(h, {
                      label: e(o)("nav.ops"),
                      name: "ops"
                    }, null, 8, ["label"])
                  ]),
                  _: 1
                }, 8, ["modelValue"])) : J("", !0),
                le(a(ma, null, null, 512), [
                  [ve, i.value === "memories"]
                ]),
                le(a(ha, null, null, 512), [
                  [ve, i.value === "tags"]
                ]),
                le(a(Ua, null, null, 512), [
                  [ve, i.value === "ops"]
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
}), Ka = /* @__PURE__ */ ne(Na, [["__scopeId", "data-v-6f56cb0b"]]);
export {
  Qe as MarkdownView,
  ma as MemoriesPanel,
  Ka as MemoryAdmin,
  aa as MemoryDetailDrawer,
  Gt as MemoryEditorDialog,
  He as MemoryUIConfigKey,
  Ua as OpsPanel,
  ha as TagsPanel,
  Lt as applyMemoryUILocalePreference,
  Nt as buildMemoriesQuery,
  It as createApiClient,
  Ja as currentMemoryUILocale,
  At as formatSize,
  re as formatTime,
  _e as isSearchMode,
  fe as memoryUIi18n,
  Wa as provideMemoryUI,
  Bt as renderMarkdown,
  Ot as sanitizeHtml,
  xt as setMemoryUILocale,
  o as t,
  se as useApiClient,
  Jt as useMemories,
  je as useMemoryConfig,
  Sa as useOps,
  pa as useTags
};
