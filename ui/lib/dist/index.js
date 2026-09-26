import { provide as Ze, inject as et, ref as y, computed as ue, onMounted as me, watch as ke, onUnmounted as tt, defineComponent as ae, openBlock as h, createElementBlock as I, createBlock as B, unref as e, withCtx as l, createVNode as a, createElementVNode as $, toDisplayString as c, createTextVNode as v, Fragment as Y, renderList as oe, createCommentVNode as Q, withKeys as at, isRef as te, withDirectives as le, reactive as ot, renderSlot as lt, vShow as he } from "vue";
import { createI18n as nt } from "vue-i18n";
import { ElDialog as Ue, ElForm as Ve, ElFormItem as Le, ElInput as we, ElRadioGroup as Ce, ElRadioButton as Ae, ElSelect as Ie, ElOption as Re, ElButton as pe, ElDrawer as st, ElTag as Te, ElDivider as it, ElTooltip as Se, ElIcon as fe, ElAlert as Be, ElTable as Ne, ElTableColumn as Oe, ElLoadingDirective as Fe, ElPagination as rt, ElRadio as ct, ElRow as dt, ElCol as ut, ElCard as mt, ElStatistic as pt, ElDescriptions as ft, ElDescriptionsItem as gt, ElEmpty as vt, ElContainer as _t, ElAside as yt, ElMenu as ht, ElMenuItem as bt, ElMain as kt, ElTabs as wt, ElTabPane as Ct } from "element-plus/es";
import { InfoFilled as de, Plus as qe, Search as Ee, Refresh as Tt, Download as St, UploadFilled as Et, Collection as Mt, Notebook as Pt, PriceTag as $t, Odometer as Dt } from "@element-plus/icons-vue";
import { ElMessage as q, ElMessageBox as He } from "element-plus";
import { Marked as xt } from "marked";
import je from "dompurify";
const zt = {
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
    colCreatedAt: "创建时间",
    editTitle: "编辑标签",
    createTitle: "新建标签",
    nameLabel: "名称（唯一，≤100 字符；改名会同步更新所有引用它的记忆）",
    namePlaceholder: "如 rust、项目、工作流",
    descLabel: "描述（可选，≤512 字符）",
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
    settingsHint: "留空即使用默认：基础提示词留空时返回内置默认提示词，附加规范留空则不追加。清空内容并保存即可回到默认行为。",
    instructionsLabel: "基础提示词（覆盖内置默认）",
    instructionsPlaceholder: "留空 = 使用内置默认提示词",
    conventionsLabel: "附加规范（追加在基础之后）",
    conventionsPlaceholder: "如：标签命名约定、摘要书写要求；留空 = 不追加",
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
    filterPlaceholder: "Filter names by regex, e.g. ^proj/ or rust$ (case-sensitive)",
    colName: "Name",
    colDescription: "Description",
    colMemoryCount: "Memories",
    colLastUsed: "Last used",
    colCreatedAt: "Created",
    editTitle: "Edit Tag",
    createTitle: "New Tag",
    nameLabel: "Name (unique, ≤100 characters; renaming updates every memory referencing it)",
    namePlaceholder: "e.g. rust, project, workflow",
    descLabel: "Description (optional, ≤512 characters)",
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
    settingsHint: "Leave a field empty to use the default: an empty base prompt falls back to the built-in instructions, and empty conventions append nothing. Clear a field and save to restore the default behavior.",
    instructionsLabel: "Base prompt (overrides the built-in default)",
    instructionsPlaceholder: "Leave empty = built-in default instructions",
    conventionsLabel: "Conventions (appended after the base)",
    conventionsPlaceholder: "e.g. tag naming rules, summary style; leave empty = none",
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
const ge = nt({
  legacy: !1,
  locale: Je(),
  fallbackLocale: "zh",
  messages: { zh: zt, en: Ut },
  // 面向宿主组件库，缺 key 时静默回退即可，不刷控制台
  missingWarn: !1,
  fallbackWarn: !1
}), { t: o } = ge.global;
function Vt(t) {
  ge.global.locale.value = t;
}
function Lt(t) {
  Vt(t === "auto" ? Je() : t);
}
function Ja() {
  return ge.global.locale.value;
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
function se(t) {
  if (!t) return "—";
  const n = ge.global.locale.value === "en" ? "en-US" : "zh-CN";
  return new Date(t * 1e3).toLocaleString(n, { hour12: !1 });
}
function At(t) {
  return t == null ? "—" : t < 1024 ? `${t} B` : t < 1024 * 1024 ? `${(t / 1024).toFixed(1)} KB` : `${(t / 1024 / 1024).toFixed(2)} MB`;
}
function It(t) {
  async function n(i, p = {}) {
    const d = await t.fetch(t.baseUrl + i, {
      headers: { "Content-Type": "application/json" },
      ...p
    }), b = await d.text();
    let r = null;
    try {
      r = b ? JSON.parse(b) : null;
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
    const p = await t.fetch(t.baseUrl + i);
    if (!p.ok) {
      const b = await p.text().catch(() => "");
      let r = o("errors.http", { status: p.status });
      try {
        r = ((d = JSON.parse(b)) == null ? void 0 : d.error) ?? r;
      } catch {
      }
      throw new Error(r);
    }
    return p.blob();
  }
  return {
    get: (i) => n(i),
    post: (i, p) => n(i, { method: "POST", body: JSON.stringify(p ?? {}) }),
    put: (i, p) => n(i, { method: "PUT", body: JSON.stringify(p ?? {}) }),
    del: (i) => n(i, { method: "DELETE" }),
    getBlob: s
  };
}
function ie() {
  return It(Ke());
}
function Rt(t) {
  const n = be(t.query), s = new URLSearchParams();
  return n ? (s.set("query", t.query.trim()), t.tagFilter && s.set("tags", t.tagFilter)) : (t.tagFilter && s.set("tag", t.tagFilter), s.set("sort", t.sort), s.set("order", t.order)), s.set("offset", String((t.page - 1) * t.pageSize)), s.set("limit", String(t.pageSize)), s.toString();
}
function be(t) {
  return t.trim().length > 0;
}
const Bt = new xt();
function Nt(t) {
  const n = Bt.parse(t, { async: !1 });
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
  const p = { description: i }, d = s.trim();
  return d && d !== n && (p.new_name = d), t.put(`/api/tags/${encodeURIComponent(n)}`, p);
}
function jt(t, n, s) {
  return t.del(`/api/tags/${encodeURIComponent(n)}?mode=${s}`);
}
function Jt() {
  const t = ie(), { defaultPageSize: n } = Ke(), s = y(""), i = y(""), p = y("updated_at"), d = y("desc"), b = y(1), r = y(n), _ = y([]), D = y([]), w = y(0), g = y(""), E = y(!1), R = y([]), x = ue(() => be(s.value));
  function C() {
    return b.value = 1, L();
  }
  function T() {
    return Rt({
      query: s.value,
      tagFilter: i.value,
      sort: p.value,
      order: d.value,
      page: b.value,
      pageSize: r.value
    });
  }
  let U = 0;
  async function L() {
    var F;
    const S = ++U;
    E.value = !0;
    try {
      const V = T();
      if (be(s.value)) {
        const A = await t.get(`/api/memories?${V}`);
        if (S !== U) return;
        D.value = (A.results ?? []).map((u) => ({ ...u, snippet: Ot(u.snippet) })), w.value = A.total_matches ?? 0, g.value = "";
      } else {
        const A = await t.get(`/api/memories?${V}`);
        if (S !== U) return;
        const u = Math.max(1, Math.ceil(A.total / r.value));
        if (((F = A.memories) == null ? void 0 : F.length) === 0 && A.total > 0 && b.value > u)
          return b.value = u, E.value = !1, L();
        _.value = A.memories ?? [], w.value = A.total ?? 0, g.value = A.note ?? "";
      }
    } finally {
      S === U && (E.value = !1);
    }
  }
  async function O() {
    try {
      const S = await Xe(t);
      R.value = (S.tags ?? []).map((F) => F.name);
    } catch {
    }
  }
  async function H(S) {
    if (S.id) {
      const F = await Me(t, S.id), V = new Set(F.tags), A = new Set(S.tags);
      await Ge(t, S.id, {
        summary: S.summary,
        content: S.content,
        add_tags: [...A].filter((u) => !V.has(u)),
        remove_tags: [...V].filter((u) => !A.has(u))
      });
    } else
      await Qe(t, { summary: S.summary, content: S.content, tags: S.tags });
    await Promise.all([L(), O()]);
  }
  async function k(S) {
    await Ft(t, S), await L();
  }
  return me(() => {
    L().catch(() => {
    }), O();
  }), {
    query: s,
    tagFilter: i,
    sort: p,
    order: d,
    page: b,
    pageSize: r,
    rows: _,
    searchResults: D,
    total: w,
    note: g,
    loading: E,
    tagOptions: R,
    searching: x,
    onSearch: C,
    reload: L,
    loadTagOptions: O,
    saveMemory: H,
    removeMemory: k
  };
}
function Pe(t, n = 720) {
  const s = y(0);
  let i = null;
  function p(d) {
    i == null || i.disconnect(), i = null, !(!d || typeof ResizeObserver > "u") && (i = new ResizeObserver((b) => {
      var r;
      s.value = ((r = b[0]) == null ? void 0 : r.contentRect.width) ?? 0;
    }), i.observe(d));
  }
  return me(() => p(t.value)), ke(t, (d) => p(d)), tt(() => i == null ? void 0 : i.disconnect()), { width: s, compact: ue(() => s.value > 0 && s.value < n) };
}
const Wt = ["innerHTML"], Ye = /* @__PURE__ */ ae({
  __name: "MarkdownView",
  props: {
    source: {}
  },
  setup(t) {
    const n = t, s = ue(() => Nt(n.source));
    return (i, p) => (h(), I("div", {
      class: "md-body",
      innerHTML: s.value
    }, null, 8, Wt));
  }
}), Kt = { class: "content-label" }, Qt = /* @__PURE__ */ ae({
  __name: "MemoryEditorDialog",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    tagOptions: {},
    width: {}
  },
  emits: ["update:visible", "saved"],
  setup(t, { emit: n }) {
    const s = t, i = n, p = ie(), d = y(!1), b = y("edit"), r = y({ id: null, summary: "", content: "", tags: [] });
    let _ = [];
    ke(
      () => s.visible,
      async (w) => {
        if (w)
          if (b.value = "edit", s.memoryId)
            try {
              const g = await Me(p, s.memoryId);
              r.value = { id: g.id, summary: g.summary, content: g.content, tags: [...g.tags] }, _ = [...g.tags];
            } catch (g) {
              q.error(g instanceof Error ? g.message : String(g)), i("update:visible", !1);
            }
          else
            r.value = { id: null, summary: "", content: "", tags: [] }, _ = [];
      }
    );
    async function D() {
      d.value = !0;
      try {
        if (r.value.id) {
          const w = new Set(_), g = new Set(r.value.tags);
          await Ge(p, r.value.id, {
            summary: r.value.summary,
            content: r.value.content,
            add_tags: [...g].filter((E) => !w.has(E)),
            remove_tags: [...w].filter((E) => !g.has(E))
          }), q.success(o("editor.updated"));
        } else
          await Qe(p, {
            summary: r.value.summary,
            content: r.value.content,
            tags: r.value.tags
          }), q.success(o("editor.created"));
        i("update:visible", !1), i("saved");
      } catch (w) {
        q.error(w instanceof Error ? w.message : String(w));
      } finally {
        d.value = !1;
      }
    }
    return (w, g) => {
      const E = we, R = Le, x = Ae, C = Ce, T = Re, U = Ie, L = Ve, O = pe, H = Ue;
      return h(), B(H, {
        "model-value": t.visible,
        title: r.value.id ? e(o)("editor.editTitle") : e(o)("editor.createTitle"),
        width: t.width,
        "onUpdate:modelValue": g[5] || (g[5] = (k) => i("update:visible", k))
      }, {
        footer: l(() => [
          a(O, {
            onClick: g[4] || (g[4] = (k) => i("update:visible", !1))
          }, {
            default: l(() => [
              v(c(e(o)("common.cancel")), 1)
            ]),
            _: 1
          }),
          a(O, {
            type: "primary",
            loading: d.value,
            onClick: D
          }, {
            default: l(() => [
              v(c(e(o)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: l(() => [
          a(L, { "label-position": "top" }, {
            default: l(() => [
              a(R, {
                label: e(o)("editor.summaryLabel")
              }, {
                default: l(() => [
                  a(E, {
                    modelValue: r.value.summary,
                    "onUpdate:modelValue": g[0] || (g[0] = (k) => r.value.summary = k),
                    maxlength: "512",
                    "show-word-limit": "",
                    placeholder: e(o)("editor.summaryPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"]),
              a(R, null, {
                label: l(() => [
                  $("div", Kt, [
                    $("span", null, c(e(o)("editor.contentLabel")), 1),
                    a(C, {
                      modelValue: b.value,
                      "onUpdate:modelValue": g[1] || (g[1] = (k) => b.value = k),
                      size: "small"
                    }, {
                      default: l(() => [
                        a(x, { value: "edit" }, {
                          default: l(() => [
                            v(c(e(o)("editor.tabEdit")), 1)
                          ]),
                          _: 1
                        }),
                        a(x, { value: "preview" }, {
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
                  b.value === "edit" ? (h(), B(E, {
                    key: 0,
                    modelValue: r.value.content,
                    "onUpdate:modelValue": g[2] || (g[2] = (k) => r.value.content = k),
                    type: "textarea",
                    rows: 12,
                    maxlength: "262144",
                    "show-word-limit": "",
                    placeholder: e(o)("editor.contentPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])) : (h(), B(Ye, {
                    key: 1,
                    class: "content-preview",
                    source: r.value.content
                  }, null, 8, ["source"]))
                ]),
                _: 1
              }),
              a(R, {
                label: e(o)("editor.tagsLabel")
              }, {
                default: l(() => [
                  a(U, {
                    modelValue: r.value.tags,
                    "onUpdate:modelValue": g[3] || (g[3] = (k) => r.value.tags = k),
                    multiple: "",
                    filterable: "",
                    "allow-create": "",
                    "default-first-option": "",
                    placeholder: e(o)("editor.tagsPlaceholder"),
                    class: "tags-select"
                  }, {
                    default: l(() => [
                      (h(!0), I(Y, null, oe(t.tagOptions, (k) => (h(), B(T, {
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
}), ne = (t, n) => {
  const s = t.__vccOpts || t;
  for (const [i, p] of n)
    s[i] = p;
  return s;
}, Gt = /* @__PURE__ */ ne(Qt, [["__scopeId", "data-v-f7b9d45a"]]), Xt = { class: "detail-summary" }, Yt = { class: "detail-tags" }, Zt = { class: "detail-toolbar" }, ea = {
  key: 1,
  class: "detail-content"
}, ta = /* @__PURE__ */ ae({
  __name: "MemoryDetailDrawer",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    size: {}
  },
  emits: ["update:visible"],
  setup(t, { emit: n }) {
    const s = t, i = n, p = ie(), d = y(null), b = y("rendered");
    return ke(
      () => [s.visible, s.memoryId],
      async ([r]) => {
        if (!(!r || !s.memoryId)) {
          b.value = "rendered";
          try {
            d.value = await Me(p, s.memoryId);
          } catch (_) {
            q.error(_ instanceof Error ? _.message : String(_)), i("update:visible", !1);
          }
        }
      }
    ), (r, _) => {
      var x;
      const D = Te, w = it, g = Ae, E = Ce, R = st;
      return h(), B(R, {
        "model-value": t.visible,
        title: e(o)("drawer.title", { id: ((x = d.value) == null ? void 0 : x.id) ?? t.memoryId ?? "" }),
        size: t.size,
        "onUpdate:modelValue": _[1] || (_[1] = (C) => i("update:visible", C))
      }, {
        default: l(() => [
          d.value ? (h(), I(Y, { key: 0 }, [
            $("h3", Xt, c(d.value.summary), 1),
            $("div", Yt, [
              (h(!0), I(Y, null, oe(d.value.tags, (C) => (h(), B(D, {
                key: C,
                size: "small",
                class: "am-tag"
              }, {
                default: l(() => [
                  v(c(C), 1)
                ]),
                _: 2
              }, 1024))), 128))
            ]),
            a(w),
            $("div", Zt, [
              a(E, {
                modelValue: b.value,
                "onUpdate:modelValue": _[0] || (_[0] = (C) => b.value = C),
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
            b.value === "rendered" ? (h(), B(Ye, {
              key: 0,
              source: d.value.content
            }, null, 8, ["source"])) : (h(), I("pre", ea, c(d.value.content), 1))
          ], 64)) : Q("", !0)
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
}, ua = /* @__PURE__ */ ae({
  __name: "MemoriesPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: n }) {
    const s = t, {
      query: i,
      tagFilter: p,
      sort: d,
      order: b,
      page: r,
      pageSize: _,
      rows: D,
      searchResults: w,
      total: g,
      note: E,
      loading: R,
      tagOptions: x,
      searching: C,
      onSearch: T,
      reload: U,
      loadTagOptions: L,
      removeMemory: O
    } = Jt(), H = y(null), { compact: k } = Pe(H), S = y(!1), F = y(null), V = y(!1), A = y(null);
    function u(z) {
      return z().catch((m) => q.error(m instanceof Error ? m.message : String(m)));
    }
    function M() {
      F.value = null, S.value = !0;
    }
    function j(z) {
      F.value = z, S.value = !0;
    }
    function G(z) {
      A.value = z, V.value = !0;
    }
    function J() {
      u(U), L();
    }
    function X(z) {
      const { prop: m, order: ee } = z;
      ee && (m === "updated_at" || m === "created_at" || m === "id") ? (d.value = m, b.value = ee === "ascending" ? "asc" : "desc") : (d.value = "updated_at", b.value = "desc"), u(U);
    }
    async function Z(z) {
      try {
        await He.confirm(o("memories.deleteConfirm", { id: z.id }), o("memories.deleteTitle"), {
          type: "warning"
        });
      } catch {
        return;
      }
      try {
        await O(z.id), q.success(o("memories.deleted"));
      } catch (m) {
        q.error(m instanceof Error ? m.message : String(m));
      }
    }
    return n({ refresh: () => u(U) }), (z, m) => {
      const ee = fe, ve = Se, W = pe, re = we, _e = Re, ye = Ie, P = Be, N = Oe, $e = Te, De = Ne, xe = rt, ze = Fe;
      return h(), I("div", {
        ref_key: "rootRef",
        ref: H,
        class: "am-panel"
      }, [
        t.showHeader ? (h(), I("div", oa, [
          $("div", la, [
            $("h2", na, c(s.title ?? e(o)("memories.title")), 1),
            a(ve, {
              content: s.subtitle ?? e(o)("memories.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(ee, { class: "am-info" }, {
                  default: l(() => [
                    a(e(de))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(W, {
            type: "primary",
            icon: e(qe),
            onClick: M
          }, {
            default: l(() => [
              v(c(e(o)("memories.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : Q("", !0),
        $("div", sa, [
          a(re, {
            modelValue: e(i),
            "onUpdate:modelValue": m[1] || (m[1] = (f) => te(i) ? i.value = f : null),
            placeholder: e(o)("memories.searchPlaceholder"),
            clearable: "",
            class: "search",
            onKeyup: m[2] || (m[2] = at((f) => u(e(T)), ["enter"])),
            onClear: m[3] || (m[3] = (f) => u(e(T)))
          }, {
            append: l(() => [
              a(W, {
                icon: e(Ee),
                onClick: m[0] || (m[0] = (f) => u(e(T)))
              }, null, 8, ["icon"])
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(ye, {
            modelValue: e(p),
            "onUpdate:modelValue": m[4] || (m[4] = (f) => te(p) ? p.value = f : null),
            placeholder: e(o)("memories.tagFilter"),
            clearable: "",
            filterable: "",
            class: "tag-filter",
            onChange: m[5] || (m[5] = (f) => u(e(T)))
          }, {
            default: l(() => [
              (h(!0), I(Y, null, oe(e(x), (f) => (h(), B(_e, {
                key: f,
                label: f,
                value: f
              }, null, 8, ["label", "value"]))), 128))
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"])
        ]),
        e(E) ? (h(), B(P, {
          key: 1,
          title: e(E),
          type: "info",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : Q("", !0),
        e(C) ? le((h(), B(De, {
          key: 2,
          data: e(w)
        }, {
          default: l(() => [
            a(N, {
              prop: "id",
              label: e(o)("memories.colId"),
              width: "80"
            }, null, 8, ["label"]),
            a(N, {
              label: e(o)("memories.colSummary")
            }, {
              default: l(({ row: f }) => [
                $("div", ia, c(f.summary), 1),
                $("div", {
                  class: "am-snippet",
                  innerHTML: f.snippet
                }, null, 8, ra)
              ]),
              _: 1
            }, 8, ["label"]),
            a(N, {
              label: e(o)("memories.colTags"),
              width: "220"
            }, {
              default: l(({ row: f }) => [
                (h(!0), I(Y, null, oe(f.tags, (K) => (h(), B($e, {
                  key: K,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: l(() => [
                    v(c(K), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            a(N, {
              prop: "score",
              label: e(o)("memories.colScore"),
              width: "80",
              sortable: ""
            }, null, 8, ["label"]),
            a(N, {
              label: e(o)("memories.colUpdatedAt"),
              width: "170"
            }, {
              default: l(({ row: f }) => [
                v(c(e(se)(f.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(N, {
              label: e(o)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: l(({ row: f }) => [
                a(W, {
                  link: "",
                  type: "primary",
                  onClick: (K) => G(f.id)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(W, {
                  link: "",
                  type: "primary",
                  onClick: (K) => j(f.id)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(W, {
                  link: "",
                  type: "danger",
                  onClick: (K) => Z(f)
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
          [ze, e(R)]
        ]) : le((h(), B(De, {
          key: 3,
          data: e(D),
          "default-sort": { prop: e(d), order: e(b) === "asc" ? "ascending" : "descending" },
          onSortChange: X
        }, {
          default: l(() => [
            a(N, {
              prop: "id",
              label: e(o)("memories.colId"),
              width: "80",
              sortable: "custom"
            }, null, 8, ["label"]),
            a(N, {
              prop: "summary",
              label: e(o)("memories.colSummary"),
              "min-width": "300",
              "show-overflow-tooltip": ""
            }, null, 8, ["label"]),
            a(N, {
              label: e(o)("memories.colTags"),
              width: "220"
            }, {
              default: l(({ row: f }) => [
                (h(!0), I(Y, null, oe(f.tags, (K) => (h(), B($e, {
                  key: K,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: l(() => [
                    v(c(K), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            e(k) ? Q("", !0) : (h(), B(N, {
              key: 0,
              prop: "created_at",
              label: e(o)("memories.colCreatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: l(({ row: f }) => [
                v(c(e(se)(f.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label"])),
            a(N, {
              prop: "updated_at",
              label: e(o)("memories.colUpdatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: l(({ row: f }) => [
                v(c(e(se)(f.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(N, {
              label: e(o)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: l(({ row: f }) => [
                a(W, {
                  link: "",
                  type: "primary",
                  onClick: (K) => G(f.id)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(W, {
                  link: "",
                  type: "primary",
                  onClick: (K) => j(f.id)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(W, {
                  link: "",
                  type: "danger",
                  onClick: (K) => Z(f)
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
          [ze, e(R)]
        ]),
        e(C) ? (h(), I("div", da, [
          a(xe, {
            "current-page": e(r),
            "onUpdate:currentPage": m[10] || (m[10] = (f) => te(r) ? r.value = f : null),
            "page-size": e(_),
            "onUpdate:pageSize": m[11] || (m[11] = (f) => te(_) ? _.value = f : null),
            total: e(g),
            "page-sizes": [10, 20, 50],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: m[12] || (m[12] = (f) => u(e(U))),
            onSizeChange: m[13] || (m[13] = (f) => u(e(U)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])) : (h(), I("div", ca, [
          a(xe, {
            "current-page": e(r),
            "onUpdate:currentPage": m[6] || (m[6] = (f) => te(r) ? r.value = f : null),
            "page-size": e(_),
            "onUpdate:pageSize": m[7] || (m[7] = (f) => te(_) ? _.value = f : null),
            total: e(g),
            "page-sizes": [20, 50, 100, 200],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: m[8] || (m[8] = (f) => u(e(U))),
            onSizeChange: m[9] || (m[9] = (f) => u(e(U)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])),
        a(Gt, {
          visible: S.value,
          "onUpdate:visible": m[14] || (m[14] = (f) => S.value = f),
          "memory-id": F.value,
          "tag-options": e(x),
          width: e(k) ? "96%" : "640px",
          onSaved: J
        }, null, 8, ["visible", "memory-id", "tag-options", "width"]),
        a(aa, {
          visible: V.value,
          "onUpdate:visible": m[15] || (m[15] = (f) => V.value = f),
          "memory-id": A.value,
          size: e(k) ? "100%" : "45%"
        }, null, 8, ["visible", "memory-id", "size"])
      ], 512);
    };
  }
}), ma = /* @__PURE__ */ ne(ua, [["__scopeId", "data-v-10d94850"]]);
function pa() {
  const t = ie(), n = y([]), s = y(!1), i = y("");
  async function p() {
    s.value = !0;
    try {
      const _ = await Xe(t, i.value.trim() || void 0);
      n.value = _.tags ?? [];
    } finally {
      s.value = !1;
    }
  }
  async function d(_, D) {
    await qt(t, _, D), await p();
  }
  async function b(_, D, w) {
    await Ht(t, _, D, w), await p();
  }
  async function r(_, D) {
    await jt(t, _, D), await p();
  }
  return me(() => {
    p().catch(() => {
    });
  }), { rows: n, loading: s, filter: i, reload: p, create: d, rename: b, remove: r };
}
const fa = {
  key: 0,
  class: "am-panel-header"
}, ga = { class: "am-heading" }, va = { class: "am-panel-title" }, _a = { class: "delete-body" }, ya = /* @__PURE__ */ ae({
  __name: "TagsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: n }) {
    const s = t, { rows: i, loading: p, filter: d, reload: b, create: r, rename: _, remove: D } = pa();
    let w = null;
    function g() {
      w && clearTimeout(w), w = setTimeout(() => {
        w = null, b().catch((u) => q.error(u instanceof Error ? u.message : String(u)));
      }, 300);
    }
    const E = y(null), { compact: R } = Pe(E), x = y(!1), C = y(!1), T = ot({ oldName: null, name: "", description: "" }), U = y(!1), L = y("detach"), O = y(null);
    function H(u) {
      return (M, j) => (M[u] ?? 0) - (j[u] ?? 0);
    }
    function k() {
      Object.assign(T, { oldName: null, name: "", description: "" }), C.value = !0;
    }
    function S(u) {
      Object.assign(T, { oldName: u.name, name: u.name, description: u.description ?? "" }), C.value = !0;
    }
    async function F() {
      x.value = !0;
      try {
        T.oldName ? (await _(T.oldName, T.name, T.description), q.success(o("tags.saved"))) : (await r(T.name, T.description), q.success(o("tags.created"))), C.value = !1;
      } catch (u) {
        q.error(u instanceof Error ? u.message : String(u));
      } finally {
        x.value = !1;
      }
    }
    function V(u) {
      O.value = u, L.value = "detach", U.value = !0;
    }
    async function A() {
      var u, M;
      if (L.value === "purge")
        try {
          await He.confirm(
            o("tags.purgeConfirm", { name: (u = O.value) == null ? void 0 : u.name, count: ((M = O.value) == null ? void 0 : M.memory_count) ?? 0 }),
            o("tags.purgeConfirmTitle"),
            { type: "error", confirmButtonText: o("tags.purgeButton") }
          );
        } catch {
          return;
        }
      if (O.value) {
        x.value = !0;
        try {
          await D(O.value.name, L.value), q.success(o("tags.deleted")), U.value = !1;
        } catch (j) {
          q.error(j instanceof Error ? j.message : String(j));
        } finally {
          x.value = !1;
        }
      }
    }
    return n({
      refresh: () => b().catch((u) => q.error(u instanceof Error ? u.message : String(u)))
    }), (u, M) => {
      const j = fe, G = Se, J = pe, X = we, Z = Te, z = Oe, m = Ne, ee = Le, ve = Ve, W = Ue, re = ct, _e = Ce, ye = Fe;
      return h(), I("div", {
        ref_key: "rootRef",
        ref: E,
        class: "am-panel"
      }, [
        t.showHeader ? (h(), I("div", fa, [
          $("div", ga, [
            $("h2", va, c(s.title ?? e(o)("tags.title")), 1),
            a(G, {
              content: s.subtitle ?? e(o)("tags.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(j, { class: "am-info" }, {
                  default: l(() => [
                    a(e(de))
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
            onClick: k
          }, {
            default: l(() => [
              v(c(e(o)("tags.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : Q("", !0),
        a(X, {
          modelValue: e(d),
          "onUpdate:modelValue": M[0] || (M[0] = (P) => te(d) ? d.value = P : null),
          class: "am-tag-filter",
          placeholder: e(o)("tags.filterPlaceholder"),
          clearable: "",
          "prefix-icon": e(Ee),
          onInput: g,
          onClear: g
        }, null, 8, ["modelValue", "placeholder", "prefix-icon"]),
        le((h(), B(m, { data: e(i) }, {
          default: l(() => [
            a(z, {
              prop: "name",
              label: e(o)("tags.colName"),
              "min-width": "160",
              sortable: ""
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
            a(z, {
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
            a(z, {
              prop: "memory_count",
              label: e(o)("tags.colMemoryCount"),
              width: "100",
              sortable: ""
            }, null, 8, ["label"]),
            a(z, {
              prop: "last_used_at",
              label: e(o)("tags.colLastUsed"),
              width: "170",
              sortable: "",
              "sort-method": H("last_used_at")
            }, {
              default: l(({ row: P }) => [
                v(c(e(se)(P.last_used_at)), 1)
              ]),
              _: 1
            }, 8, ["label", "sort-method"]),
            a(z, {
              prop: "created_at",
              label: e(o)("tags.colCreatedAt"),
              width: "170",
              sortable: "",
              "sort-method": H("created_at")
            }, {
              default: l(({ row: P }) => [
                v(c(e(se)(P.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label", "sort-method"]),
            a(z, {
              label: e(o)("memories.colActions"),
              width: "150",
              fixed: "right"
            }, {
              default: l(({ row: P }) => [
                a(J, {
                  link: "",
                  type: "primary",
                  onClick: (N) => S(P)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(J, {
                  link: "",
                  type: "danger",
                  onClick: (N) => V(P)
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
          [ye, e(p)]
        ]),
        a(W, {
          modelValue: C.value,
          "onUpdate:modelValue": M[4] || (M[4] = (P) => C.value = P),
          title: T.oldName ? e(o)("tags.editTitle") : e(o)("tags.createTitle"),
          width: e(R) ? "96%" : "480px"
        }, {
          footer: l(() => [
            a(J, {
              onClick: M[3] || (M[3] = (P) => C.value = !1)
            }, {
              default: l(() => [
                v(c(e(o)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(J, {
              type: "primary",
              loading: x.value,
              onClick: F
            }, {
              default: l(() => [
                v(c(e(o)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: l(() => [
            a(ve, { "label-position": "top" }, {
              default: l(() => [
                a(ee, {
                  label: e(o)("tags.nameLabel")
                }, {
                  default: l(() => [
                    a(X, {
                      modelValue: T.name,
                      "onUpdate:modelValue": M[1] || (M[1] = (P) => T.name = P),
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(o)("tags.namePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(ee, {
                  label: e(o)("tags.descLabel")
                }, {
                  default: l(() => [
                    a(X, {
                      modelValue: T.description,
                      "onUpdate:modelValue": M[2] || (M[2] = (P) => T.description = P),
                      type: "textarea",
                      rows: 3,
                      maxlength: "512",
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
        a(W, {
          modelValue: U.value,
          "onUpdate:modelValue": M[7] || (M[7] = (P) => U.value = P),
          title: e(o)("tags.deleteTitle"),
          width: e(R) ? "96%" : "480px"
        }, {
          footer: l(() => [
            a(J, {
              onClick: M[6] || (M[6] = (P) => U.value = !1)
            }, {
              default: l(() => [
                v(c(e(o)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(J, {
              type: "danger",
              loading: x.value,
              onClick: A
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
              $("p", _a, [
                v(c(e(o)("tags.deleteBefore")) + " ", 1),
                a(Z, null, {
                  default: l(() => {
                    var N;
                    return [
                      v(c((N = O.value) == null ? void 0 : N.name), 1)
                    ];
                  }),
                  _: 1
                }),
                v(" " + c(e(o)("tags.deleteMiddle")) + " ", 1),
                $("b", null, c((P = O.value) == null ? void 0 : P.memory_count), 1),
                v(" " + c(e(o)("tags.deleteAfter")), 1)
              ]),
              a(_e, {
                modelValue: L.value,
                "onUpdate:modelValue": M[5] || (M[5] = (N) => L.value = N)
              }, {
                default: l(() => [
                  a(re, { value: "detach" }, {
                    default: l(() => [
                      v(c(e(o)("tags.detach")), 1)
                    ]),
                    _: 1
                  }),
                  a(re, { value: "purge" }, {
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
}), ha = /* @__PURE__ */ ne(ya, [["__scopeId", "data-v-1e7aa745"]]);
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
  const t = ie(), n = y({}), s = y(""), i = y({ ok: !0, issues: [] }), p = y(!1), d = y(!1), b = y(!1), r = y(!1), _ = ue(() => At(n.value.file_size));
  async function D() {
    n.value = await ba(t), n.value.version = s.value;
  }
  async function w() {
    d.value = !0;
    try {
      i.value = await wa(t), p.value = !0;
    } finally {
      d.value = !1;
    }
  }
  async function g() {
    b.value = !0;
    try {
      const R = await Ca(t), x = URL.createObjectURL(R), C = document.createElement("a");
      C.href = x, C.download = "agent-memory-export.json", C.click(), URL.revokeObjectURL(x);
    } finally {
      b.value = !1;
    }
  }
  async function E(R) {
    r.value = !0;
    try {
      const x = await R.text();
      let C;
      try {
        C = JSON.parse(x);
      } catch {
        throw new Error(o("errors.invalidBackup"));
      }
      const T = await Ta(t, C);
      return await D(), T;
    } finally {
      r.value = !1;
    }
  }
  return me(async () => {
    try {
      s.value = (await ka(t)).version ?? "";
    } catch {
    }
    await D().catch(() => {
    });
  }), {
    stats: n,
    version: s,
    doctor: i,
    doctorRan: p,
    doctorLoading: d,
    exporting: b,
    importing: r,
    sizeText: _,
    reload: D,
    runDoctor: w,
    exportData: g,
    importFile: E
  };
}
const Ea = {
  key: 0,
  class: "am-panel-header"
}, Ma = { class: "am-heading" }, Pa = { class: "am-panel-title" }, $a = { class: "actions" }, Da = { class: "card-header" }, xa = {
  key: 2,
  class: "issues"
}, za = /* @__PURE__ */ ae({
  __name: "OpsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: n }) {
    const s = t, {
      stats: i,
      version: p,
      doctor: d,
      doctorRan: b,
      doctorLoading: r,
      exporting: _,
      importing: D,
      sizeText: w,
      reload: g,
      runDoctor: E,
      exportData: R,
      importFile: x
    } = Sa(), C = y(null), { compact: T } = Pe(C), U = y(null);
    function L(H) {
      return H().catch((k) => q.error(k instanceof Error ? k.message : String(k)));
    }
    function O(H) {
      var F;
      const k = H.target, S = (F = k.files) == null ? void 0 : F[0];
      k.value = "", S && x(S).then((V) => {
        q.success(o("ops.imported", { memories: V.imported_memories, tags: V.imported_tags }));
      }).catch((V) => {
        q.error(V instanceof Error ? V.message : String(V));
      });
    }
    return n({ refresh: () => L(g) }), (H, k) => {
      const S = fe, F = Se, V = pe, A = pt, u = mt, M = ut, j = dt, G = gt, J = ft, X = Be, Z = vt;
      return h(), I("div", {
        ref_key: "rootRef",
        ref: C,
        class: "am-panel"
      }, [
        t.showHeader ? (h(), I("div", Ea, [
          $("div", Ma, [
            $("h2", Pa, c(s.title ?? e(o)("ops.title")), 1),
            a(F, {
              content: s.subtitle ?? e(o)("ops.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(S, { class: "am-info" }, {
                  default: l(() => [
                    a(e(de))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(V, {
            icon: e(Tt),
            onClick: k[0] || (k[0] = (z) => L(e(g)))
          }, {
            default: l(() => [
              v(c(e(o)("ops.refresh")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : Q("", !0),
        a(j, { gutter: 14 }, {
          default: l(() => [
            a(M, {
              span: e(T) ? 12 : 8
            }, {
              default: l(() => [
                a(u, { shadow: "never" }, {
                  default: l(() => [
                    a(A, {
                      title: e(o)("ops.statMemories"),
                      value: e(i).memories ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(M, {
              span: e(T) ? 12 : 8
            }, {
              default: l(() => [
                a(u, { shadow: "never" }, {
                  default: l(() => [
                    a(A, {
                      title: e(o)("ops.statTags"),
                      value: e(i).tags ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(M, {
              span: e(T) ? 12 : 8
            }, {
              default: l(() => [
                a(u, { shadow: "never" }, {
                  default: l(() => [
                    a(A, {
                      title: e(o)("ops.statSize"),
                      value: e(w)
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
        a(u, { shadow: "never" }, {
          header: l(() => [
            v(c(e(o)("ops.dbCard")), 1)
          ]),
          default: l(() => [
            a(J, {
              column: e(T) ? 1 : 2,
              border: ""
            }, {
              default: l(() => [
                a(G, {
                  label: e(o)("ops.version")
                }, {
                  default: l(() => [
                    v(c(e(i).version ?? e(p)), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(G, {
                  label: e(o)("ops.schemaVersion")
                }, {
                  default: l(() => [
                    v(c(e(i).schema_version ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(G, {
                  label: e(o)("ops.path"),
                  span: e(T) ? 1 : 2
                }, {
                  default: l(() => [
                    v(c(e(i).path ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label", "span"])
              ]),
              _: 1
            }, 8, ["column"]),
            $("div", $a, [
              a(V, {
                icon: e(St),
                loading: e(_),
                onClick: k[1] || (k[1] = (z) => L(e(R)))
              }, {
                default: l(() => [
                  v(c(e(o)("ops.export")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(V, {
                icon: e(Et),
                loading: e(D),
                onClick: k[2] || (k[2] = (z) => {
                  var m;
                  return (m = U.value) == null ? void 0 : m.click();
                })
              }, {
                default: l(() => [
                  v(c(e(o)("ops.import")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(F, {
                content: e(o)("ops.importHint"),
                placement: "top"
              }, {
                default: l(() => [
                  a(S, { class: "am-info" }, {
                    default: l(() => [
                      a(e(de))
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["content"]),
              $("input", {
                ref_key: "importInput",
                ref: U,
                type: "file",
                accept: "application/json,.json",
                style: { display: "none" },
                onChange: O
              }, null, 544)
            ])
          ]),
          _: 1
        }),
        a(u, { shadow: "never" }, {
          header: l(() => [
            $("div", Da, [
              $("span", null, c(e(o)("ops.doctorCard")), 1),
              a(V, {
                size: "small",
                icon: e(Ee),
                loading: e(r),
                onClick: k[3] || (k[3] = (z) => L(e(E)))
              }, {
                default: l(() => [
                  v(c(e(o)("ops.runDoctor")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"])
            ])
          ]),
          default: l(() => [
            e(b) ? (h(), I(Y, { key: 0 }, [
              e(d).ok ? (h(), B(X, {
                key: 0,
                title: e(o)("ops.doctorOk"),
                type: "success",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])) : (h(), B(X, {
                key: 1,
                title: e(o)("ops.doctorFail", { count: e(d).issues.length }),
                type: "error",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])),
              e(d).ok ? Q("", !0) : (h(), I("ul", xa, [
                (h(!0), I(Y, null, oe(e(d).issues, (z, m) => (h(), I("li", { key: m }, c(z), 1))), 128))
              ]))
            ], 64)) : (h(), B(Z, {
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
}), Ua = /* @__PURE__ */ ne(za, [["__scopeId", "data-v-95205f88"]]), Va = { class: "memory-ui" }, La = { class: "brand" }, Aa = { class: "brand-mark" }, Ia = { class: "aside-footer" }, Ra = /* @__PURE__ */ ae({
  __name: "MemoryAdmin",
  props: {
    layout: { default: "sidebar" },
    title: { default: "Agent Memory" }
  },
  setup(t) {
    const n = y("memories");
    return (s, i) => {
      const p = fe, d = bt, b = ht, r = yt, _ = Ct, D = wt, w = kt, g = _t;
      return h(), I("div", Va, [
        a(g, { class: "layout" }, {
          default: l(() => [
            t.layout === "sidebar" ? (h(), B(r, {
              key: 0,
              width: "200px",
              class: "aside"
            }, {
              default: l(() => [
                $("div", La, [
                  $("span", Aa, [
                    a(p, { size: 16 }, {
                      default: l(() => [
                        a(e(Mt))
                      ]),
                      _: 1
                    })
                  ]),
                  $("span", null, c(t.title), 1)
                ]),
                a(b, {
                  "default-active": n.value,
                  class: "menu",
                  onSelect: i[0] || (i[0] = (E) => n.value = E)
                }, {
                  default: l(() => [
                    a(d, { index: "memories" }, {
                      default: l(() => [
                        a(p, null, {
                          default: l(() => [
                            a(e(Pt))
                          ]),
                          _: 1
                        }),
                        $("span", null, c(e(o)("nav.memories")), 1)
                      ]),
                      _: 1
                    }),
                    a(d, { index: "tags" }, {
                      default: l(() => [
                        a(p, null, {
                          default: l(() => [
                            a(e($t))
                          ]),
                          _: 1
                        }),
                        $("span", null, c(e(o)("nav.tags")), 1)
                      ]),
                      _: 1
                    }),
                    a(d, { index: "ops" }, {
                      default: l(() => [
                        a(p, null, {
                          default: l(() => [
                            a(e(Dt))
                          ]),
                          _: 1
                        }),
                        $("span", null, c(e(o)("nav.ops")), 1)
                      ]),
                      _: 1
                    })
                  ]),
                  _: 1
                }, 8, ["default-active"]),
                $("div", Ia, [
                  lt(s.$slots, "footer", {}, void 0, !0)
                ])
              ]),
              _: 3
            })) : Q("", !0),
            a(w, { class: "main" }, {
              default: l(() => [
                t.layout === "tabs" ? (h(), B(D, {
                  key: 0,
                  modelValue: n.value,
                  "onUpdate:modelValue": i[1] || (i[1] = (E) => n.value = E),
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
                le(a(ma, null, null, 512), [
                  [he, n.value === "memories"]
                ]),
                le(a(ha, null, null, 512), [
                  [he, n.value === "tags"]
                ]),
                le(a(Ua, null, null, 512), [
                  [he, n.value === "ops"]
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
}), Ka = /* @__PURE__ */ ne(Ra, [["__scopeId", "data-v-38c91753"]]);
export {
  Ye as MarkdownView,
  ma as MemoriesPanel,
  Ka as MemoryAdmin,
  aa as MemoryDetailDrawer,
  Gt as MemoryEditorDialog,
  We as MemoryUIConfigKey,
  Ua as OpsPanel,
  ha as TagsPanel,
  Lt as applyMemoryUILocalePreference,
  Rt as buildMemoriesQuery,
  It as createApiClient,
  Ja as currentMemoryUILocale,
  At as formatSize,
  se as formatTime,
  be as isSearchMode,
  ge as memoryUIi18n,
  Wa as provideMemoryUI,
  Nt as renderMarkdown,
  Ot as sanitizeHtml,
  Vt as setMemoryUILocale,
  o as t,
  ie as useApiClient,
  Jt as useMemories,
  Ke as useMemoryConfig,
  Sa as useOps,
  pa as useTags
};
