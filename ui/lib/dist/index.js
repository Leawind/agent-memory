import { provide as Ze, inject as et, ref as y, computed as me, onMounted as pe, watch as ke, onUnmounted as tt, defineComponent as te, openBlock as h, createElementBlock as L, createBlock as I, unref as e, withCtx as l, createVNode as a, createElementVNode as D, toDisplayString as c, createTextVNode as v, Fragment as Y, renderList as le, createCommentVNode as Q, withKeys as at, isRef as ee, withDirectives as ne, reactive as ot, renderSlot as lt, vShow as ye } from "vue";
import { createI18n as nt } from "vue-i18n";
import { ElDialog as xe, ElForm as Ue, ElFormItem as Le, ElInput as we, ElRadioGroup as Ce, ElRadioButton as Ie, ElSelect as Ae, ElOption as Re, ElButton as fe, ElDrawer as st, ElTag as Te, ElDivider as it, ElTooltip as Se, ElIcon as ge, ElAlert as Ne, ElTable as Be, ElTableColumn as Oe, ElLoadingDirective as Fe, ElPagination as rt, ElRadio as ct, ElRow as dt, ElCol as ut, ElCard as mt, ElStatistic as pt, ElDescriptions as ft, ElDescriptionsItem as gt, ElEmpty as vt, ElContainer as _t, ElAside as ht, ElMenu as yt, ElMenuItem as bt, ElMain as kt, ElTabs as wt, ElTabPane as Ct } from "element-plus/es";
import { InfoFilled as ue, Plus as qe, Search as Ee, Refresh as Tt, Download as St, UploadFilled as Et, Collection as Mt, Notebook as Pt, PriceTag as Dt, Odometer as $t } from "@element-plus/icons-vue";
import { ElMessage as O, ElMessageBox as He } from "element-plus";
import { Marked as zt } from "marked";
import je from "dompurify";
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
    filterPlaceholder: "正则过滤标签名，如 ^proj/ 、rust$（大小写敏感）",
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
}, xt = {
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
    filterPlaceholder: "Filter names by regex, e.g. ^proj/ or rust$ (case-sensitive)",
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
function Je() {
  var t;
  return typeof navigator > "u" || (t = navigator.language) != null && t.toLowerCase().startsWith("zh") ? "zh" : "en";
}
const ve = nt({
  legacy: !1,
  locale: Je(),
  fallbackLocale: "zh",
  messages: { zh: Vt, en: xt },
  // 面向宿主组件库，缺 key 时静默回退即可，不刷控制台
  missingWarn: !1,
  fallbackWarn: !1
}), { t: o } = ve.global;
function Ut(t) {
  ve.global.locale.value = t;
}
function Lt(t) {
  Ut(t === "auto" ? Je() : t);
}
function Ja() {
  return ve.global.locale.value;
}
const We = Symbol("memory-ui-config"), ce = {
  baseUrl: "",
  fetch: (...t) => globalThis.fetch(...t),
  defaultPageSize: 20,
  locale: "auto"
};
function Wa(t) {
  t.locale && Lt(t.locale), Ze(We, t);
}
function Ke() {
  const t = et(We);
  return {
    baseUrl: ((t == null ? void 0 : t.baseUrl) ?? ce.baseUrl).replace(/\/+$/, ""),
    fetch: (t == null ? void 0 : t.fetch) ?? ce.fetch,
    defaultPageSize: (t == null ? void 0 : t.defaultPageSize) ?? ce.defaultPageSize,
    locale: (t == null ? void 0 : t.locale) ?? ce.locale
  };
}
function de(t) {
  if (!t) return "—";
  const n = ve.global.locale.value === "en" ? "en-US" : "zh-CN";
  return new Date(t * 1e3).toLocaleString(n, { hour12: !1 });
}
function It(t) {
  return t == null ? "—" : t < 1024 ? `${t} B` : t < 1024 * 1024 ? `${(t / 1024).toFixed(1)} KB` : `${(t / 1024 / 1024).toFixed(2)} MB`;
}
function At(t) {
  async function n(i, m = {}) {
    const d = await t.fetch(t.baseUrl + i, {
      headers: { "Content-Type": "application/json" },
      ...m
    }), k = await d.text();
    let r = null;
    try {
      r = k ? JSON.parse(k) : null;
    } catch {
      r = null;
    }
    if (!d.ok) {
      const _ = (r == null ? void 0 : r.error) ?? o("errors.http", { status: d.status });
      throw new Error(_);
    }
    return r;
  }
  async function s(i) {
    var d;
    const m = await t.fetch(t.baseUrl + i);
    if (!m.ok) {
      const k = await m.text().catch(() => "");
      let r = o("errors.http", { status: m.status });
      try {
        r = ((d = JSON.parse(k)) == null ? void 0 : d.error) ?? r;
      } catch {
      }
      throw new Error(r);
    }
    return m.blob();
  }
  return {
    get: (i) => n(i),
    post: (i, m) => n(i, { method: "POST", body: JSON.stringify(m ?? {}) }),
    put: (i, m) => n(i, { method: "PUT", body: JSON.stringify(m ?? {}) }),
    del: (i) => n(i, { method: "DELETE" }),
    getBlob: s
  };
}
function ie() {
  return At(Ke());
}
function Rt(t) {
  const n = be(t.query), s = new URLSearchParams();
  return n ? (s.set("query", t.query.trim()), t.tagFilter && s.set("tags", t.tagFilter)) : (t.tagFilter && s.set("tag", t.tagFilter), s.set("sort", t.sort), s.set("order", t.order)), s.set("offset", String((t.page - 1) * t.pageSize)), s.set("limit", String(t.pageSize)), s.toString();
}
function be(t) {
  return t.trim().length > 0;
}
const Nt = new zt();
function Bt(t) {
  const n = Nt.parse(t, { async: !1 });
  return je.sanitize(n);
}
function Ot(t) {
  return je.sanitize(t);
}
function Me(t, n) {
  return t.get(`/api/memories/${encodeURIComponent(n)}`);
}
function Qe(t, n) {
  return t.post("/api/memories", n);
}
function Ge(t, n, s) {
  return t.put(`/api/memories/${encodeURIComponent(n)}`, s);
}
function Ft(t, n) {
  return t.del(`/api/memories/${encodeURIComponent(n)}`);
}
function Xe(t, n) {
  const s = n ? `?filter=${encodeURIComponent(n)}` : "";
  return t.get(`/api/tags${s}`);
}
function qt(t, n, s) {
  return t.post("/api/tags", { name: n, description: s });
}
function Ht(t, n, s, i) {
  const m = { description: i }, d = s.trim();
  return d && d !== n && (m.new_name = d), t.put(`/api/tags/${encodeURIComponent(n)}`, m);
}
function jt(t, n, s) {
  return t.del(`/api/tags/${encodeURIComponent(n)}?mode=${s}`);
}
function Jt() {
  const t = ie(), { defaultPageSize: n } = Ke(), s = y(""), i = y(""), m = y("updated_at"), d = y("desc"), k = y(1), r = y(n), _ = y([]), $ = y([]), T = y(0), g = y(""), M = y(!1), A = y([]), z = me(() => be(s.value));
  function S() {
    return k.value = 1, U();
  }
  function C() {
    return Rt({
      query: s.value,
      tagFilter: i.value,
      sort: m.value,
      order: d.value,
      page: k.value,
      pageSize: r.value
    });
  }
  let V = 0;
  async function U() {
    var B;
    const E = ++V;
    M.value = !0;
    try {
      const x = C();
      if (be(s.value)) {
        const b = await t.get(`/api/memories?${x}`);
        if (E !== V) return;
        $.value = (b.results ?? []).map((p) => ({ ...p, snippet: Ot(p.snippet) })), T.value = b.total_matches ?? 0, g.value = "";
      } else {
        const b = await t.get(`/api/memories?${x}`);
        if (E !== V) return;
        const p = Math.max(1, Math.ceil(b.total / r.value));
        if (((B = b.memories) == null ? void 0 : B.length) === 0 && b.total > 0 && k.value > p)
          return k.value = p, M.value = !1, U();
        _.value = b.memories ?? [], T.value = b.total ?? 0, g.value = b.note ?? "";
      }
    } finally {
      E === V && (M.value = !1);
    }
  }
  async function N() {
    try {
      const E = await Xe(t);
      A.value = (E.tags ?? []).map((B) => B.name);
    } catch {
    }
  }
  async function q(E) {
    if (E.id) {
      const B = await Me(t, E.id), x = new Set(B.tags), b = new Set(E.tags);
      await Ge(t, E.id, {
        summary: E.summary,
        content: E.content,
        add_tags: [...b].filter((p) => !x.has(p)),
        remove_tags: [...x].filter((p) => !b.has(p))
      });
    } else
      await Qe(t, { summary: E.summary, content: E.content, tags: E.tags });
    await Promise.all([U(), N()]);
  }
  async function w(E) {
    await Ft(t, E), await U();
  }
  return pe(() => {
    U().catch(() => {
    }), N();
  }), {
    query: s,
    tagFilter: i,
    sort: m,
    order: d,
    page: k,
    pageSize: r,
    rows: _,
    searchResults: $,
    total: T,
    note: g,
    loading: M,
    tagOptions: A,
    searching: z,
    onSearch: S,
    reload: U,
    loadTagOptions: N,
    saveMemory: q,
    removeMemory: w
  };
}
function Pe(t, n = 720) {
  const s = y(0);
  let i = null;
  function m(d) {
    i == null || i.disconnect(), i = null, !(!d || typeof ResizeObserver > "u") && (i = new ResizeObserver((k) => {
      var r;
      s.value = ((r = k[0]) == null ? void 0 : r.contentRect.width) ?? 0;
    }), i.observe(d));
  }
  return pe(() => m(t.value)), ke(t, (d) => m(d)), tt(() => i == null ? void 0 : i.disconnect()), { width: s, compact: me(() => s.value > 0 && s.value < n) };
}
const Wt = ["innerHTML"], Ye = /* @__PURE__ */ te({
  __name: "MarkdownView",
  props: {
    source: {}
  },
  setup(t) {
    const n = t, s = me(() => Bt(n.source));
    return (i, m) => (h(), L("div", {
      class: "md-body",
      innerHTML: s.value
    }, null, 8, Wt));
  }
}), Kt = { class: "content-label" }, Qt = /* @__PURE__ */ te({
  __name: "MemoryEditorDialog",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    tagOptions: {},
    width: {}
  },
  emits: ["update:visible", "saved"],
  setup(t, { emit: n }) {
    const s = t, i = n, m = ie(), d = y(!1), k = y("edit"), r = y({ id: null, summary: "", content: "", tags: [] });
    let _ = [];
    ke(
      () => s.visible,
      async (T) => {
        if (T)
          if (k.value = "edit", s.memoryId)
            try {
              const g = await Me(m, s.memoryId);
              r.value = { id: g.id, summary: g.summary, content: g.content, tags: [...g.tags] }, _ = [...g.tags];
            } catch (g) {
              O.error(g instanceof Error ? g.message : String(g)), i("update:visible", !1);
            }
          else
            r.value = { id: null, summary: "", content: "", tags: [] }, _ = [];
      }
    );
    async function $() {
      d.value = !0;
      try {
        if (r.value.id) {
          const T = new Set(_), g = new Set(r.value.tags);
          await Ge(m, r.value.id, {
            summary: r.value.summary,
            content: r.value.content,
            add_tags: [...g].filter((M) => !T.has(M)),
            remove_tags: [...T].filter((M) => !g.has(M))
          }), O.success(o("editor.updated"));
        } else
          await Qe(m, {
            summary: r.value.summary,
            content: r.value.content,
            tags: r.value.tags
          }), O.success(o("editor.created"));
        i("update:visible", !1), i("saved");
      } catch (T) {
        O.error(T instanceof Error ? T.message : String(T));
      } finally {
        d.value = !1;
      }
    }
    return (T, g) => {
      const M = we, A = Le, z = Ie, S = Ce, C = Re, V = Ae, U = Ue, N = fe, q = xe;
      return h(), I(q, {
        "model-value": t.visible,
        title: r.value.id ? e(o)("editor.editTitle") : e(o)("editor.createTitle"),
        width: t.width,
        "onUpdate:modelValue": g[5] || (g[5] = (w) => i("update:visible", w))
      }, {
        footer: l(() => [
          a(N, {
            onClick: g[4] || (g[4] = (w) => i("update:visible", !1))
          }, {
            default: l(() => [
              v(c(e(o)("common.cancel")), 1)
            ]),
            _: 1
          }),
          a(N, {
            type: "primary",
            loading: d.value,
            onClick: $
          }, {
            default: l(() => [
              v(c(e(o)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: l(() => [
          a(U, { "label-position": "top" }, {
            default: l(() => [
              a(A, {
                label: e(o)("editor.summaryLabel")
              }, {
                default: l(() => [
                  a(M, {
                    modelValue: r.value.summary,
                    "onUpdate:modelValue": g[0] || (g[0] = (w) => r.value.summary = w),
                    maxlength: "512",
                    "show-word-limit": "",
                    placeholder: e(o)("editor.summaryPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"]),
              a(A, null, {
                label: l(() => [
                  D("div", Kt, [
                    D("span", null, c(e(o)("editor.contentLabel")), 1),
                    a(S, {
                      modelValue: k.value,
                      "onUpdate:modelValue": g[1] || (g[1] = (w) => k.value = w),
                      size: "small"
                    }, {
                      default: l(() => [
                        a(z, { value: "edit" }, {
                          default: l(() => [
                            v(c(e(o)("editor.tabEdit")), 1)
                          ]),
                          _: 1
                        }),
                        a(z, { value: "preview" }, {
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
                  k.value === "edit" ? (h(), I(M, {
                    key: 0,
                    modelValue: r.value.content,
                    "onUpdate:modelValue": g[2] || (g[2] = (w) => r.value.content = w),
                    type: "textarea",
                    rows: 12,
                    maxlength: "200000",
                    "show-word-limit": "",
                    placeholder: e(o)("editor.contentPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])) : (h(), I(Ye, {
                    key: 1,
                    class: "content-preview",
                    source: r.value.content
                  }, null, 8, ["source"]))
                ]),
                _: 1
              }),
              a(A, {
                label: e(o)("editor.tagsLabel")
              }, {
                default: l(() => [
                  a(V, {
                    modelValue: r.value.tags,
                    "onUpdate:modelValue": g[3] || (g[3] = (w) => r.value.tags = w),
                    multiple: "",
                    filterable: "",
                    "allow-create": "",
                    "default-first-option": "",
                    placeholder: e(o)("editor.tagsPlaceholder"),
                    class: "tags-select"
                  }, {
                    default: l(() => [
                      (h(!0), L(Y, null, le(t.tagOptions, (w) => (h(), I(C, {
                        key: w,
                        label: w,
                        value: w
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
}), se = (t, n) => {
  const s = t.__vccOpts || t;
  for (const [i, m] of n)
    s[i] = m;
  return s;
}, Gt = /* @__PURE__ */ se(Qt, [["__scopeId", "data-v-602bada0"]]), Xt = { class: "detail-summary" }, Yt = { class: "detail-tags" }, Zt = { class: "detail-toolbar" }, ea = {
  key: 1,
  class: "detail-content"
}, ta = /* @__PURE__ */ te({
  __name: "MemoryDetailDrawer",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    size: {}
  },
  emits: ["update:visible"],
  setup(t, { emit: n }) {
    const s = t, i = n, m = ie(), d = y(null), k = y("rendered");
    return ke(
      () => [s.visible, s.memoryId],
      async ([r]) => {
        if (!(!r || !s.memoryId)) {
          k.value = "rendered";
          try {
            d.value = await Me(m, s.memoryId);
          } catch (_) {
            O.error(_ instanceof Error ? _.message : String(_)), i("update:visible", !1);
          }
        }
      }
    ), (r, _) => {
      var z;
      const $ = Te, T = it, g = Ie, M = Ce, A = st;
      return h(), I(A, {
        "model-value": t.visible,
        title: e(o)("drawer.title", { id: ((z = d.value) == null ? void 0 : z.id) ?? t.memoryId ?? "" }),
        size: t.size,
        "onUpdate:modelValue": _[1] || (_[1] = (S) => i("update:visible", S))
      }, {
        default: l(() => [
          d.value ? (h(), L(Y, { key: 0 }, [
            D("h3", Xt, c(d.value.summary), 1),
            D("div", Yt, [
              (h(!0), L(Y, null, le(d.value.tags, (S) => (h(), I($, {
                key: S,
                size: "small",
                class: "am-tag"
              }, {
                default: l(() => [
                  v(c(S), 1)
                ]),
                _: 2
              }, 1024))), 128))
            ]),
            a(T),
            D("div", Zt, [
              a(M, {
                modelValue: k.value,
                "onUpdate:modelValue": _[0] || (_[0] = (S) => k.value = S),
                size: "small"
              }, {
                default: l(() => [
                  a(g, { value: "rendered" }, {
                    default: l(() => [
                      v(c(e(o)("drawer.rendered")), 1)
                    ]),
                    _: 1
                  }),
                  a(g, { value: "source" }, {
                    default: l(() => [
                      v(c(e(o)("drawer.source")), 1)
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["modelValue"])
            ]),
            k.value === "rendered" ? (h(), I(Ye, {
              key: 0,
              source: d.value.content
            }, null, 8, ["source"])) : (h(), L("pre", ea, c(d.value.content), 1))
          ], 64)) : Q("", !0)
        ]),
        _: 1
      }, 8, ["model-value", "title", "size"]);
    };
  }
}), aa = /* @__PURE__ */ se(ta, [["__scopeId", "data-v-b634d116"]]), oa = {
  key: 0,
  class: "am-panel-header"
}, la = { class: "am-heading" }, na = { class: "am-panel-title" }, sa = { class: "am-toolbar" }, ia = { class: "am-summary" }, ra = ["innerHTML"], ca = {
  key: 4,
  class: "am-pager"
}, da = {
  key: 5,
  class: "am-pager"
}, ua = /* @__PURE__ */ te({
  __name: "MemoriesPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: n }) {
    const s = t, {
      query: i,
      tagFilter: m,
      sort: d,
      order: k,
      page: r,
      pageSize: _,
      rows: $,
      searchResults: T,
      total: g,
      note: M,
      loading: A,
      tagOptions: z,
      searching: S,
      onSearch: C,
      reload: V,
      loadTagOptions: U,
      removeMemory: N
    } = Jt(), q = y(null), { compact: w } = Pe(q), E = y(!1), B = y(null), x = y(!1), b = y(null);
    function p(R) {
      return R().catch((u) => O.error(u instanceof Error ? u.message : String(u)));
    }
    function j() {
      B.value = null, E.value = !0;
    }
    function ae(R) {
      B.value = R, E.value = !0;
    }
    function F(R) {
      b.value = R, x.value = !0;
    }
    function G() {
      p(V), U();
    }
    function Z(R) {
      const { prop: u, order: oe } = R;
      oe && (u === "updated_at" || u === "created_at") ? (d.value = u, k.value = oe === "ascending" ? "asc" : "desc") : (d.value = "updated_at", k.value = "desc"), p(V);
    }
    async function K(R) {
      try {
        await He.confirm(o("memories.deleteConfirm", { id: R.id }), o("memories.deleteTitle"), {
          type: "warning"
        });
      } catch {
        return;
      }
      try {
        await N(R.id), O.success(o("memories.deleted"));
      } catch (u) {
        O.error(u instanceof Error ? u.message : String(u));
      }
    }
    return n({ refresh: () => p(V) }), (R, u) => {
      const oe = ge, re = Se, J = fe, _e = we, he = Re, P = Ae, X = Ne, H = Oe, De = Te, $e = Be, ze = rt, Ve = Fe;
      return h(), L("div", {
        ref_key: "rootRef",
        ref: q,
        class: "am-panel"
      }, [
        t.showHeader ? (h(), L("div", oa, [
          D("div", la, [
            D("h2", na, c(s.title ?? e(o)("memories.title")), 1),
            a(re, {
              content: s.subtitle ?? e(o)("memories.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(oe, { class: "am-info" }, {
                  default: l(() => [
                    a(e(ue))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(J, {
            type: "primary",
            icon: e(qe),
            onClick: j
          }, {
            default: l(() => [
              v(c(e(o)("memories.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : Q("", !0),
        D("div", sa, [
          a(_e, {
            modelValue: e(i),
            "onUpdate:modelValue": u[1] || (u[1] = (f) => ee(i) ? i.value = f : null),
            placeholder: e(o)("memories.searchPlaceholder"),
            clearable: "",
            class: "search",
            onKeyup: u[2] || (u[2] = at((f) => p(e(C)), ["enter"])),
            onClear: u[3] || (u[3] = (f) => p(e(C)))
          }, {
            append: l(() => [
              a(J, {
                icon: e(Ee),
                onClick: u[0] || (u[0] = (f) => p(e(C)))
              }, null, 8, ["icon"])
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(P, {
            modelValue: e(m),
            "onUpdate:modelValue": u[4] || (u[4] = (f) => ee(m) ? m.value = f : null),
            placeholder: e(o)("memories.tagFilter"),
            clearable: "",
            filterable: "",
            class: "tag-filter",
            onChange: u[5] || (u[5] = (f) => p(e(C)))
          }, {
            default: l(() => [
              (h(!0), L(Y, null, le(e(z), (f) => (h(), I(he, {
                key: f,
                label: f,
                value: f
              }, null, 8, ["label", "value"]))), 128))
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"])
        ]),
        e(M) ? (h(), I(X, {
          key: 1,
          title: e(M),
          type: "info",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : Q("", !0),
        e(S) ? ne((h(), I($e, {
          key: 2,
          data: e(T)
        }, {
          default: l(() => [
            a(H, {
              prop: "id",
              label: e(o)("memories.colId"),
              width: "80"
            }, null, 8, ["label"]),
            a(H, {
              label: e(o)("memories.colSummary")
            }, {
              default: l(({ row: f }) => [
                D("div", ia, c(f.summary), 1),
                D("div", {
                  class: "am-snippet",
                  innerHTML: f.snippet
                }, null, 8, ra)
              ]),
              _: 1
            }, 8, ["label"]),
            a(H, {
              label: e(o)("memories.colTags"),
              width: "220"
            }, {
              default: l(({ row: f }) => [
                (h(!0), L(Y, null, le(f.tags, (W) => (h(), I(De, {
                  key: W,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: l(() => [
                    v(c(W), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            a(H, {
              prop: "score",
              label: e(o)("memories.colScore"),
              width: "80",
              sortable: ""
            }, null, 8, ["label"]),
            a(H, {
              label: e(o)("memories.colUpdatedAt"),
              width: "170"
            }, {
              default: l(({ row: f }) => [
                v(c(e(de)(f.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(H, {
              label: e(o)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: l(({ row: f }) => [
                a(J, {
                  link: "",
                  type: "primary",
                  onClick: (W) => F(f.id)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(J, {
                  link: "",
                  type: "primary",
                  onClick: (W) => ae(f.id)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(J, {
                  link: "",
                  type: "danger",
                  onClick: (W) => K(f)
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
          [Ve, e(A)]
        ]) : ne((h(), I($e, {
          key: 3,
          data: e($),
          "default-sort": { prop: e(d), order: e(k) === "asc" ? "ascending" : "descending" },
          onSortChange: Z
        }, {
          default: l(() => [
            a(H, {
              prop: "id",
              label: e(o)("memories.colId"),
              width: "80"
            }, null, 8, ["label"]),
            a(H, {
              prop: "summary",
              label: e(o)("memories.colSummary"),
              "min-width": "300",
              "show-overflow-tooltip": ""
            }, null, 8, ["label"]),
            a(H, {
              label: e(o)("memories.colTags"),
              width: "220"
            }, {
              default: l(({ row: f }) => [
                (h(!0), L(Y, null, le(f.tags, (W) => (h(), I(De, {
                  key: W,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: l(() => [
                    v(c(W), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            e(w) ? Q("", !0) : (h(), I(H, {
              key: 0,
              prop: "created_at",
              label: e(o)("memories.colCreatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: l(({ row: f }) => [
                v(c(e(de)(f.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label"])),
            a(H, {
              prop: "updated_at",
              label: e(o)("memories.colUpdatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: l(({ row: f }) => [
                v(c(e(de)(f.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(H, {
              label: e(o)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: l(({ row: f }) => [
                a(J, {
                  link: "",
                  type: "primary",
                  onClick: (W) => F(f.id)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(J, {
                  link: "",
                  type: "primary",
                  onClick: (W) => ae(f.id)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(J, {
                  link: "",
                  type: "danger",
                  onClick: (W) => K(f)
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
          [Ve, e(A)]
        ]),
        e(S) ? (h(), L("div", da, [
          a(ze, {
            "current-page": e(r),
            "onUpdate:currentPage": u[10] || (u[10] = (f) => ee(r) ? r.value = f : null),
            "page-size": e(_),
            "onUpdate:pageSize": u[11] || (u[11] = (f) => ee(_) ? _.value = f : null),
            total: e(g),
            "page-sizes": [10, 20, 50],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: u[12] || (u[12] = (f) => p(e(V))),
            onSizeChange: u[13] || (u[13] = (f) => p(e(V)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])) : (h(), L("div", ca, [
          a(ze, {
            "current-page": e(r),
            "onUpdate:currentPage": u[6] || (u[6] = (f) => ee(r) ? r.value = f : null),
            "page-size": e(_),
            "onUpdate:pageSize": u[7] || (u[7] = (f) => ee(_) ? _.value = f : null),
            total: e(g),
            "page-sizes": [20, 50, 100, 200],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: u[8] || (u[8] = (f) => p(e(V))),
            onSizeChange: u[9] || (u[9] = (f) => p(e(V)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])),
        a(Gt, {
          visible: E.value,
          "onUpdate:visible": u[14] || (u[14] = (f) => E.value = f),
          "memory-id": B.value,
          "tag-options": e(z),
          width: e(w) ? "96%" : "640px",
          onSaved: G
        }, null, 8, ["visible", "memory-id", "tag-options", "width"]),
        a(aa, {
          visible: x.value,
          "onUpdate:visible": u[15] || (u[15] = (f) => x.value = f),
          "memory-id": b.value,
          size: e(w) ? "100%" : "45%"
        }, null, 8, ["visible", "memory-id", "size"])
      ], 512);
    };
  }
}), ma = /* @__PURE__ */ se(ua, [["__scopeId", "data-v-1ab68c72"]]);
function pa() {
  const t = ie(), n = y([]), s = y(!1), i = y("");
  async function m() {
    s.value = !0;
    try {
      const _ = await Xe(t, i.value.trim() || void 0);
      n.value = _.tags ?? [];
    } finally {
      s.value = !1;
    }
  }
  async function d(_, $) {
    await qt(t, _, $), await m();
  }
  async function k(_, $, T) {
    await Ht(t, _, $, T), await m();
  }
  async function r(_, $) {
    await jt(t, _, $), await m();
  }
  return pe(() => {
    m().catch(() => {
    });
  }), { rows: n, loading: s, filter: i, reload: m, create: d, rename: k, remove: r };
}
const fa = {
  key: 0,
  class: "am-panel-header"
}, ga = { class: "am-heading" }, va = { class: "am-panel-title" }, _a = { class: "delete-body" }, ha = /* @__PURE__ */ te({
  __name: "TagsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: n }) {
    const s = t, { rows: i, loading: m, filter: d, reload: k, create: r, rename: _, remove: $ } = pa();
    let T = null;
    function g() {
      T && clearTimeout(T), T = setTimeout(() => {
        T = null, k().catch((b) => O.error(b instanceof Error ? b.message : String(b)));
      }, 300);
    }
    const M = y(null), { compact: A } = Pe(M), z = y(!1), S = y(!1), C = ot({ oldName: null, name: "", newName: "", description: "" }), V = y(!1), U = y("detach"), N = y(null);
    function q() {
      Object.assign(C, { oldName: null, name: "", newName: "", description: "" }), S.value = !0;
    }
    function w(b) {
      Object.assign(C, { oldName: b.name, name: b.name, newName: "", description: b.description ?? "" }), S.value = !0;
    }
    async function E() {
      z.value = !0;
      try {
        C.oldName ? (await _(C.oldName, C.newName, C.description), O.success(o("tags.saved"))) : (await r(C.name, C.description), O.success(o("tags.created"))), S.value = !1;
      } catch (b) {
        O.error(b instanceof Error ? b.message : String(b));
      } finally {
        z.value = !1;
      }
    }
    function B(b) {
      N.value = b, U.value = "detach", V.value = !0;
    }
    async function x() {
      var b, p;
      if (U.value === "purge")
        try {
          await He.confirm(
            o("tags.purgeConfirm", { name: (b = N.value) == null ? void 0 : b.name, count: ((p = N.value) == null ? void 0 : p.memory_count) ?? 0 }),
            o("tags.purgeConfirmTitle"),
            { type: "error", confirmButtonText: o("tags.purgeButton") }
          );
        } catch {
          return;
        }
      if (N.value) {
        z.value = !0;
        try {
          await $(N.value.name, U.value), O.success(o("tags.deleted")), V.value = !1;
        } catch (j) {
          O.error(j instanceof Error ? j.message : String(j));
        } finally {
          z.value = !1;
        }
      }
    }
    return n({
      refresh: () => k().catch((b) => O.error(b instanceof Error ? b.message : String(b)))
    }), (b, p) => {
      const j = ge, ae = Se, F = fe, G = we, Z = Te, K = Oe, R = Be, u = Le, oe = Ue, re = xe, J = ct, _e = Ce, he = Fe;
      return h(), L("div", {
        ref_key: "rootRef",
        ref: M,
        class: "am-panel"
      }, [
        t.showHeader ? (h(), L("div", fa, [
          D("div", ga, [
            D("h2", va, c(s.title ?? e(o)("tags.title")), 1),
            a(ae, {
              content: s.subtitle ?? e(o)("tags.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(j, { class: "am-info" }, {
                  default: l(() => [
                    a(e(ue))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(F, {
            type: "primary",
            icon: e(qe),
            onClick: q
          }, {
            default: l(() => [
              v(c(e(o)("tags.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : Q("", !0),
        a(G, {
          modelValue: e(d),
          "onUpdate:modelValue": p[0] || (p[0] = (P) => ee(d) ? d.value = P : null),
          class: "am-tag-filter",
          placeholder: e(o)("tags.filterPlaceholder"),
          clearable: "",
          "prefix-icon": e(Ee),
          onInput: g,
          onClear: g
        }, null, 8, ["modelValue", "placeholder", "prefix-icon"]),
        ne((h(), I(R, { data: e(i) }, {
          default: l(() => [
            a(K, {
              prop: "name",
              label: e(o)("tags.colName"),
              "min-width": "160"
            }, {
              default: l(({ row: P }) => [
                a(Z, null, {
                  default: l(() => [
                    v(c(P.name), 1)
                  ]),
                  _: 2
                }, 1024)
              ]),
              _: 1
            }, 8, ["label"]),
            a(K, {
              prop: "description",
              label: e(o)("tags.colDescription"),
              "min-width": "300",
              "show-overflow-tooltip": ""
            }, {
              default: l(({ row: P }) => [
                v(c(P.description || "—"), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(K, {
              prop: "memory_count",
              label: e(o)("tags.colMemoryCount"),
              width: "100",
              sortable: ""
            }, null, 8, ["label"]),
            a(K, {
              label: e(o)("tags.colLastUsed"),
              width: "170"
            }, {
              default: l(({ row: P }) => [
                v(c(e(de)(P.last_used_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(K, {
              label: e(o)("memories.colActions"),
              width: "150",
              fixed: "right"
            }, {
              default: l(({ row: P }) => [
                a(F, {
                  link: "",
                  type: "primary",
                  onClick: (X) => w(P)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(F, {
                  link: "",
                  type: "danger",
                  onClick: (X) => B(P)
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
          [he, e(m)]
        ]),
        a(re, {
          modelValue: S.value,
          "onUpdate:modelValue": p[5] || (p[5] = (P) => S.value = P),
          title: C.oldName ? e(o)("tags.editTitle") : e(o)("tags.createTitle"),
          width: e(A) ? "96%" : "480px"
        }, {
          footer: l(() => [
            a(F, {
              onClick: p[4] || (p[4] = (P) => S.value = !1)
            }, {
              default: l(() => [
                v(c(e(o)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(F, {
              type: "primary",
              loading: z.value,
              onClick: E
            }, {
              default: l(() => [
                v(c(e(o)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: l(() => [
            a(oe, { "label-position": "top" }, {
              default: l(() => [
                a(u, {
                  label: e(o)("tags.nameLabel")
                }, {
                  default: l(() => [
                    a(G, {
                      modelValue: C.name,
                      "onUpdate:modelValue": p[1] || (p[1] = (P) => C.name = P),
                      disabled: !!C.oldName,
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(o)("tags.namePlaceholder")
                    }, null, 8, ["modelValue", "disabled", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                C.oldName ? (h(), I(u, {
                  key: 0,
                  label: e(o)("tags.renameLabel")
                }, {
                  default: l(() => [
                    a(G, {
                      modelValue: C.newName,
                      "onUpdate:modelValue": p[2] || (p[2] = (P) => C.newName = P),
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(o)("tags.renamePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"])) : Q("", !0),
                a(u, {
                  label: e(o)("tags.descLabel")
                }, {
                  default: l(() => [
                    a(G, {
                      modelValue: C.description,
                      "onUpdate:modelValue": p[3] || (p[3] = (P) => C.description = P),
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
        a(re, {
          modelValue: V.value,
          "onUpdate:modelValue": p[8] || (p[8] = (P) => V.value = P),
          title: e(o)("tags.deleteTitle"),
          width: e(A) ? "96%" : "480px"
        }, {
          footer: l(() => [
            a(F, {
              onClick: p[7] || (p[7] = (P) => V.value = !1)
            }, {
              default: l(() => [
                v(c(e(o)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(F, {
              type: "danger",
              loading: z.value,
              onClick: x
            }, {
              default: l(() => [
                v(c(e(o)("common.delete")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: l(() => {
            var P;
            return [
              D("p", _a, [
                v(c(e(o)("tags.deleteBefore")) + " ", 1),
                a(Z, null, {
                  default: l(() => {
                    var X;
                    return [
                      v(c((X = N.value) == null ? void 0 : X.name), 1)
                    ];
                  }),
                  _: 1
                }),
                v(" " + c(e(o)("tags.deleteMiddle")) + " ", 1),
                D("b", null, c((P = N.value) == null ? void 0 : P.memory_count), 1),
                v(" " + c(e(o)("tags.deleteAfter")), 1)
              ]),
              a(_e, {
                modelValue: U.value,
                "onUpdate:modelValue": p[6] || (p[6] = (X) => U.value = X)
              }, {
                default: l(() => [
                  a(J, { value: "detach" }, {
                    default: l(() => [
                      v(c(e(o)("tags.detach")), 1)
                    ]),
                    _: 1
                  }),
                  a(J, { value: "purge" }, {
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
}), ya = /* @__PURE__ */ se(ha, [["__scopeId", "data-v-4a5c5fb0"]]);
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
function Ta(t, n) {
  return t.post("/api/import", n);
}
function Sa() {
  const t = ie(), n = y({}), s = y(""), i = y({ ok: !0, issues: [] }), m = y(!1), d = y(!1), k = y(!1), r = y(!1), _ = me(() => It(n.value.file_size));
  async function $() {
    n.value = await ba(t), n.value.version = s.value;
  }
  async function T() {
    d.value = !0;
    try {
      i.value = await wa(t), m.value = !0;
    } finally {
      d.value = !1;
    }
  }
  async function g() {
    k.value = !0;
    try {
      const A = await Ca(t), z = URL.createObjectURL(A), S = document.createElement("a");
      S.href = z, S.download = "agent-memory-export.json", S.click(), URL.revokeObjectURL(z);
    } finally {
      k.value = !1;
    }
  }
  async function M(A) {
    r.value = !0;
    try {
      const z = await A.text();
      let S;
      try {
        S = JSON.parse(z);
      } catch {
        throw new Error(o("errors.invalidBackup"));
      }
      const C = await Ta(t, S);
      return await $(), C;
    } finally {
      r.value = !1;
    }
  }
  return pe(async () => {
    try {
      s.value = (await ka(t)).version ?? "";
    } catch {
    }
    await $().catch(() => {
    });
  }), {
    stats: n,
    version: s,
    doctor: i,
    doctorRan: m,
    doctorLoading: d,
    exporting: k,
    importing: r,
    sizeText: _,
    reload: $,
    runDoctor: T,
    exportData: g,
    importFile: M
  };
}
const Ea = {
  key: 0,
  class: "am-panel-header"
}, Ma = { class: "am-heading" }, Pa = { class: "am-panel-title" }, Da = { class: "actions" }, $a = { class: "card-header" }, za = {
  key: 2,
  class: "issues"
}, Va = /* @__PURE__ */ te({
  __name: "OpsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: n }) {
    const s = t, {
      stats: i,
      version: m,
      doctor: d,
      doctorRan: k,
      doctorLoading: r,
      exporting: _,
      importing: $,
      sizeText: T,
      reload: g,
      runDoctor: M,
      exportData: A,
      importFile: z
    } = Sa(), S = y(null), { compact: C } = Pe(S), V = y(null);
    function U(q) {
      return q().catch((w) => O.error(w instanceof Error ? w.message : String(w)));
    }
    function N(q) {
      var B;
      const w = q.target, E = (B = w.files) == null ? void 0 : B[0];
      w.value = "", E && z(E).then((x) => {
        O.success(o("ops.imported", { memories: x.imported_memories, tags: x.imported_tags }));
      }).catch((x) => {
        O.error(x instanceof Error ? x.message : String(x));
      });
    }
    return n({ refresh: () => U(g) }), (q, w) => {
      const E = ge, B = Se, x = fe, b = pt, p = mt, j = ut, ae = dt, F = gt, G = ft, Z = Ne, K = vt;
      return h(), L("div", {
        ref_key: "rootRef",
        ref: S,
        class: "am-panel"
      }, [
        t.showHeader ? (h(), L("div", Ea, [
          D("div", Ma, [
            D("h2", Pa, c(s.title ?? e(o)("ops.title")), 1),
            a(B, {
              content: s.subtitle ?? e(o)("ops.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(E, { class: "am-info" }, {
                  default: l(() => [
                    a(e(ue))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(x, {
            icon: e(Tt),
            onClick: w[0] || (w[0] = (R) => U(e(g)))
          }, {
            default: l(() => [
              v(c(e(o)("ops.refresh")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : Q("", !0),
        a(ae, { gutter: 14 }, {
          default: l(() => [
            a(j, {
              span: e(C) ? 12 : 8
            }, {
              default: l(() => [
                a(p, { shadow: "never" }, {
                  default: l(() => [
                    a(b, {
                      title: e(o)("ops.statMemories"),
                      value: e(i).memories ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(j, {
              span: e(C) ? 12 : 8
            }, {
              default: l(() => [
                a(p, { shadow: "never" }, {
                  default: l(() => [
                    a(b, {
                      title: e(o)("ops.statTags"),
                      value: e(i).tags ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(j, {
              span: e(C) ? 12 : 8
            }, {
              default: l(() => [
                a(p, { shadow: "never" }, {
                  default: l(() => [
                    a(b, {
                      title: e(o)("ops.statSize"),
                      value: e(T)
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
        a(p, { shadow: "never" }, {
          header: l(() => [
            v(c(e(o)("ops.dbCard")), 1)
          ]),
          default: l(() => [
            a(G, {
              column: e(C) ? 1 : 2,
              border: ""
            }, {
              default: l(() => [
                a(F, {
                  label: e(o)("ops.path")
                }, {
                  default: l(() => [
                    v(c(e(i).path ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(F, {
                  label: e(o)("ops.schemaVersion")
                }, {
                  default: l(() => [
                    v(c(e(i).schema_version ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(F, {
                  label: e(o)("ops.version")
                }, {
                  default: l(() => [
                    v(c(e(i).version ?? e(m)), 1)
                  ]),
                  _: 1
                }, 8, ["label"])
              ]),
              _: 1
            }, 8, ["column"]),
            D("div", Da, [
              a(x, {
                icon: e(St),
                loading: e(_),
                onClick: w[1] || (w[1] = (R) => U(e(A)))
              }, {
                default: l(() => [
                  v(c(e(o)("ops.export")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(x, {
                icon: e(Et),
                loading: e($),
                onClick: w[2] || (w[2] = (R) => {
                  var u;
                  return (u = V.value) == null ? void 0 : u.click();
                })
              }, {
                default: l(() => [
                  v(c(e(o)("ops.import")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(B, {
                content: e(o)("ops.importHint"),
                placement: "top"
              }, {
                default: l(() => [
                  a(E, { class: "am-info" }, {
                    default: l(() => [
                      a(e(ue))
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["content"]),
              D("input", {
                ref_key: "importInput",
                ref: V,
                type: "file",
                accept: "application/json,.json",
                style: { display: "none" },
                onChange: N
              }, null, 544)
            ])
          ]),
          _: 1
        }),
        a(p, { shadow: "never" }, {
          header: l(() => [
            D("div", $a, [
              D("span", null, c(e(o)("ops.doctorCard")), 1),
              a(x, {
                size: "small",
                icon: e(Ee),
                loading: e(r),
                onClick: w[3] || (w[3] = (R) => U(e(M)))
              }, {
                default: l(() => [
                  v(c(e(o)("ops.runDoctor")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"])
            ])
          ]),
          default: l(() => [
            e(k) ? (h(), L(Y, { key: 0 }, [
              e(d).ok ? (h(), I(Z, {
                key: 0,
                title: e(o)("ops.doctorOk"),
                type: "success",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])) : (h(), I(Z, {
                key: 1,
                title: e(o)("ops.doctorFail", { count: e(d).issues.length }),
                type: "error",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])),
              e(d).ok ? Q("", !0) : (h(), L("ul", za, [
                (h(!0), L(Y, null, le(e(d).issues, (R, u) => (h(), L("li", { key: u }, c(R), 1))), 128))
              ]))
            ], 64)) : (h(), I(K, {
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
}), xa = /* @__PURE__ */ se(Va, [["__scopeId", "data-v-27c1e600"]]), Ua = { class: "memory-ui" }, La = { class: "brand" }, Ia = { class: "brand-mark" }, Aa = { class: "aside-footer" }, Ra = /* @__PURE__ */ te({
  __name: "MemoryAdmin",
  props: {
    layout: { default: "sidebar" },
    title: { default: "agent-memory" }
  },
  setup(t) {
    const n = y("memories");
    return (s, i) => {
      const m = ge, d = bt, k = yt, r = ht, _ = Ct, $ = wt, T = kt, g = _t;
      return h(), L("div", Ua, [
        a(g, { class: "layout" }, {
          default: l(() => [
            t.layout === "sidebar" ? (h(), I(r, {
              key: 0,
              width: "200px",
              class: "aside"
            }, {
              default: l(() => [
                D("div", La, [
                  D("span", Ia, [
                    a(m, { size: 16 }, {
                      default: l(() => [
                        a(e(Mt))
                      ]),
                      _: 1
                    })
                  ]),
                  D("span", null, c(t.title), 1)
                ]),
                a(k, {
                  "default-active": n.value,
                  class: "menu",
                  onSelect: i[0] || (i[0] = (M) => n.value = M)
                }, {
                  default: l(() => [
                    a(d, { index: "memories" }, {
                      default: l(() => [
                        a(m, null, {
                          default: l(() => [
                            a(e(Pt))
                          ]),
                          _: 1
                        }),
                        D("span", null, c(e(o)("nav.memories")), 1)
                      ]),
                      _: 1
                    }),
                    a(d, { index: "tags" }, {
                      default: l(() => [
                        a(m, null, {
                          default: l(() => [
                            a(e(Dt))
                          ]),
                          _: 1
                        }),
                        D("span", null, c(e(o)("nav.tags")), 1)
                      ]),
                      _: 1
                    }),
                    a(d, { index: "ops" }, {
                      default: l(() => [
                        a(m, null, {
                          default: l(() => [
                            a(e($t))
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
                D("div", Aa, [
                  lt(s.$slots, "footer", {}, void 0, !0)
                ])
              ]),
              _: 3
            })) : Q("", !0),
            a(T, { class: "main" }, {
              default: l(() => [
                t.layout === "tabs" ? (h(), I($, {
                  key: 0,
                  modelValue: n.value,
                  "onUpdate:modelValue": i[1] || (i[1] = (M) => n.value = M),
                  class: "tabs-bar"
                }, {
                  default: l(() => [
                    a(_, {
                      label: e(o)("nav.memories"),
                      name: "memories"
                    }, null, 8, ["label"]),
                    a(_, {
                      label: e(o)("nav.tags"),
                      name: "tags"
                    }, null, 8, ["label"]),
                    a(_, {
                      label: e(o)("nav.ops"),
                      name: "ops"
                    }, null, 8, ["label"])
                  ]),
                  _: 1
                }, 8, ["modelValue"])) : Q("", !0),
                ne(a(ma, null, null, 512), [
                  [ye, n.value === "memories"]
                ]),
                ne(a(ya, null, null, 512), [
                  [ye, n.value === "tags"]
                ]),
                ne(a(xa, null, null, 512), [
                  [ye, n.value === "ops"]
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
}), Ka = /* @__PURE__ */ se(Ra, [["__scopeId", "data-v-6f56cb0b"]]);
export {
  Ye as MarkdownView,
  ma as MemoriesPanel,
  Ka as MemoryAdmin,
  aa as MemoryDetailDrawer,
  Gt as MemoryEditorDialog,
  We as MemoryUIConfigKey,
  xa as OpsPanel,
  ya as TagsPanel,
  Lt as applyMemoryUILocalePreference,
  Rt as buildMemoriesQuery,
  At as createApiClient,
  Ja as currentMemoryUILocale,
  It as formatSize,
  de as formatTime,
  be as isSearchMode,
  ve as memoryUIi18n,
  Wa as provideMemoryUI,
  Bt as renderMarkdown,
  Ot as sanitizeHtml,
  Ut as setMemoryUILocale,
  o as t,
  ie as useApiClient,
  Jt as useMemories,
  Ke as useMemoryConfig,
  Sa as useOps,
  pa as useTags
};
