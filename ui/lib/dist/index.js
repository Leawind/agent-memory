import { provide as et, inject as tt, ref as b, computed as re, onMounted as fe, watch as we, onUnmounted as at, defineComponent as ae, openBlock as h, createElementBlock as V, createBlock as z, unref as e, withCtx as l, createVNode as a, createElementVNode as P, toDisplayString as r, createTextVNode as _, Fragment as ee, renderList as ne, createCommentVNode as G, withKeys as ot, isRef as te, withDirectives as se, reactive as lt, renderSlot as nt, vShow as be } from "vue";
import { createI18n as st } from "vue-i18n";
import { ElDialog as ze, ElForm as Le, ElFormItem as Ae, ElInput as Ce, ElRadioGroup as Te, ElRadioButton as Ie, ElSelect as Re, ElOption as Ne, ElButton as ge, ElDrawer as rt, ElTag as Se, ElDivider as it, ElTooltip as Ee, ElIcon as ve, ElAlert as Be, ElTable as Oe, ElTableColumn as Fe, ElLoadingDirective as He, ElPagination as ct, ElRadio as dt, ElRow as ut, ElCol as mt, ElCard as pt, ElStatistic as ft, ElDescriptions as gt, ElDescriptionsItem as vt, ElEmpty as _t, ElContainer as yt, ElAside as ht, ElMenu as bt, ElMenuItem as kt, ElMain as wt, ElTabs as Ct, ElTabPane as Tt } from "element-plus/es";
import { InfoFilled as pe, Plus as qe, Search as Me, Refresh as St, Download as Et, UploadFilled as Mt, Collection as Pt, Notebook as $t, PriceTag as Dt, Odometer as xt } from "@element-plus/icons-vue";
import { ElMessage as B, ElMessageBox as je } from "element-plus";
import { Marked as Ut } from "marked";
import Je from "dompurify";
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
    deleted: "已删除",
    tagEmpty: "标签「{tag}」目前没有关联任何记忆",
    searchEmpty: "没有记忆命中全部关键词，可减少关键词或改用更宽泛的词"
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
    subtitle: "token 即身份，无账号体系；每个身份的能力逐项开关",
    needAdmin: "需要 admin 权限才能管理身份与设置",
    openMode: "开放模式：鉴权开关未开启，所有请求免鉴权，仅适合个人本地部署。启用鉴权前，先在下方创建管理员身份并保存其 token；开启开关后，所有 /mcp 与 /api 请求必须携带有效 token。",
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
    created: "token 仅此一次展示，服务端只存哈希，请立即复制保存：",
    resetToken: "重置 Token",
    resetTitle: "重置 Token",
    resetConfirm: "确定重置身份「{name}」的 token？旧 token 立即失效，新 token 仅展示一次。",
    editTitle: "编辑能力：{name}",
    deleteTitle: "删除身份",
    deleteConfirm: "确定删除身份「{name}」？其 token 将立即失效。",
    deleted: "已删除",
    saved: "已保存",
    settingsTitle: "自定义提示词",
    instructionsLabel: "基础提示词",
    instructionsPlaceholder: "覆盖内置默认的 initialize 提示词",
    instructionsHint: "留空时使用内置默认提示词；清空并保存即回到默认。",
    conventionsLabel: "附加规范",
    conventionsPlaceholder: "如：标签命名约定、摘要书写要求",
    conventionsHint: "追加在基础提示词之后；留空则不追加。",
    viewDefault: "查看内置默认",
    empty: "尚无身份。先创建管理员身份并保存其 token，再打开上方的鉴权开关。",
    authTitle: "Token 鉴权",
    authHint: "开启后，所有 /mcp 与 /api 请求都必须携带 Authorization: Bearer <token>。开启前请先创建身份并保存 token；开启后本浏览器会弹出令牌输入框，输入一次即可。",
    authEnableConfirm: "开启 token 鉴权？此后所有 /mcp 与 /api 请求都必须携带有效 token。",
    authDisableConfirm: "关闭 token 鉴权？所有请求将免鉴权放行（开放模式）。"
  }
}, zt = {
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
    deleted: "Deleted",
    tagEmpty: 'Tag "{tag}" does not have any memories yet',
    searchEmpty: "No memory matched every term; drop some terms or try broader ones"
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
    subtitle: "A token is an identity, no account system; capabilities are toggled per identity",
    needAdmin: "Admin permission is required to manage identities and settings",
    openMode: "Open mode: the auth switch is off and all requests are unauthenticated, only suitable for personal local deployments. Before enabling, create an admin identity below and save its token; once the switch is on, every /mcp and /api request must carry a valid token.",
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
    created: "Shown once only — the server stores only a hash. Copy and save it now:",
    resetToken: "Reset token",
    resetTitle: "Reset token",
    resetConfirm: 'Reset the token of identity "{name}"? The old token is revoked immediately; the new one is shown once.',
    editTitle: "Edit capabilities: {name}",
    deleteTitle: "Delete identity",
    deleteConfirm: 'Delete identity "{name}"? Its token is revoked immediately.',
    deleted: "Deleted",
    saved: "Saved",
    settingsTitle: "Custom instructions",
    instructionsLabel: "Base prompt",
    instructionsPlaceholder: "Custom initialize instructions overriding the built-in default",
    instructionsHint: "Leave empty to use the built-in default; clear and save to go back to it.",
    conventionsLabel: "Conventions",
    conventionsPlaceholder: "e.g. tag naming rules, summary style",
    conventionsHint: "Appended after the base prompt; leave empty to append nothing.",
    viewDefault: "View built-in default",
    empty: "No identities yet. Create an admin identity and save its token, then turn on the auth switch above.",
    authTitle: "Token auth",
    authHint: "When enabled, every /mcp and /api request must carry Authorization: Bearer <token>. Create an identity and save its token first; after enabling, this browser will show the token prompt once.",
    authEnableConfirm: "Enable token auth? All /mcp and /api requests will then require a valid token.",
    authDisableConfirm: "Disable token auth? All requests will be allowed without credentials (open mode)."
  }
};
function We() {
  var t;
  return typeof navigator > "u" || (t = navigator.language) != null && t.toLowerCase().startsWith("zh") ? "zh" : "en";
}
const _e = st({
  legacy: !1,
  locale: We(),
  fallbackLocale: "zh",
  messages: { zh: Vt, en: zt },
  // 面向宿主组件库，缺 key 时静默回退即可，不刷控制台
  missingWarn: !1,
  fallbackWarn: !1
}), { t: o } = _e.global;
function Lt(t) {
  _e.global.locale.value = t;
}
function At(t) {
  Lt(t === "auto" ? We() : t);
}
function Wa() {
  return _e.global.locale.value;
}
const Ke = Symbol("memory-ui-config"), me = {
  baseUrl: "",
  fetch: (...t) => globalThis.fetch(...t),
  defaultPageSize: 20,
  locale: "auto"
};
function Ka(t) {
  t.locale && At(t.locale), et(Ke, t);
}
function Qe() {
  const t = tt(Ke);
  return {
    baseUrl: ((t == null ? void 0 : t.baseUrl) ?? me.baseUrl).replace(/\/+$/, ""),
    fetch: (t == null ? void 0 : t.fetch) ?? me.fetch,
    defaultPageSize: (t == null ? void 0 : t.defaultPageSize) ?? me.defaultPageSize,
    locale: (t == null ? void 0 : t.locale) ?? me.locale
  };
}
function ce(t) {
  if (!t) return "—";
  const n = _e.global.locale.value === "en" ? "en-US" : "zh-CN";
  return new Date(t * 1e3).toLocaleString(n, { hour12: !1 });
}
function It(t) {
  return t == null ? "—" : t < 1024 ? `${t} B` : t < 1024 * 1024 ? `${(t / 1024).toFixed(1)} KB` : `${(t / 1024 / 1024).toFixed(2)} MB`;
}
function Rt(t) {
  async function n(c, u = {}) {
    const m = await t.fetch(t.baseUrl + c, {
      headers: { "Content-Type": "application/json" },
      ...u
    }), y = await m.text();
    let i = null;
    try {
      i = y ? JSON.parse(y) : null;
    } catch {
      i = null;
    }
    if (!m.ok) {
      const v = (i == null ? void 0 : i.error) ?? o("errors.http", { status: m.status });
      throw new Error(v);
    }
    return i;
  }
  async function s(c) {
    var m;
    const u = await t.fetch(t.baseUrl + c);
    if (!u.ok) {
      const y = await u.text().catch(() => "");
      let i = o("errors.http", { status: u.status });
      try {
        i = ((m = JSON.parse(y)) == null ? void 0 : m.error) ?? i;
      } catch {
      }
      throw new Error(i);
    }
    return u.blob();
  }
  return {
    get: (c) => n(c),
    post: (c, u) => n(c, { method: "POST", body: JSON.stringify(u ?? {}) }),
    put: (c, u) => n(c, { method: "PUT", body: JSON.stringify(u ?? {}) }),
    del: (c) => n(c, { method: "DELETE" }),
    getBlob: s
  };
}
function de() {
  return Rt(Qe());
}
function Nt(t) {
  const n = ke(t.query), s = new URLSearchParams();
  return n ? (s.set("query", t.query.trim()), t.tagFilter && s.set("tags", t.tagFilter)) : (t.tagFilter && s.set("tag", t.tagFilter), s.set("sort", t.sort), s.set("order", t.order)), s.set("offset", String((t.page - 1) * t.pageSize)), s.set("limit", String(t.pageSize)), s.toString();
}
function ke(t) {
  return t.trim().length > 0;
}
const Bt = new Ut();
function Ot(t) {
  const n = Bt.parse(t, { async: !1 });
  return Je.sanitize(n);
}
function Ft(t) {
  return Je.sanitize(t);
}
function Pe(t, n) {
  return t.get(`/api/memories/${encodeURIComponent(n)}`);
}
function Ge(t, n) {
  return t.post("/api/memories", n);
}
function Xe(t, n, s) {
  return t.put(`/api/memories/${encodeURIComponent(n)}`, s);
}
function Ht(t, n) {
  return t.del(`/api/memories/${encodeURIComponent(n)}`);
}
function Ye(t, n) {
  const s = n ? `?filter=${encodeURIComponent(n)}` : "";
  return t.get(`/api/tags${s}`);
}
function qt(t, n, s) {
  return t.post("/api/tags", { name: n, description: s });
}
function jt(t, n, s, c) {
  const u = { description: c }, m = s.trim();
  return m && m !== n && (u.new_name = m), t.put(`/api/tags/${encodeURIComponent(n)}`, u);
}
function Jt(t, n, s) {
  return t.del(`/api/tags/${encodeURIComponent(n)}?mode=${s}`);
}
function Wt() {
  const t = de(), { defaultPageSize: n } = Qe(), s = b(""), c = b(""), u = b("updated_at"), m = b("desc"), y = b(1), i = b(n), v = b([]), x = b([]), S = b(0), g = b(!1), $ = b([]), L = re(() => ke(s.value));
  function D() {
    return y.value = 1, A();
  }
  function T() {
    return Nt({
      query: s.value,
      tagFilter: c.value,
      sort: u.value,
      order: m.value,
      page: y.value,
      pageSize: i.value
    });
  }
  let w = 0;
  async function A() {
    var F;
    const d = ++w;
    g.value = !0;
    try {
      const N = T();
      if (ke(s.value)) {
        const E = await t.get(`/api/memories?${N}`);
        if (d !== w) return;
        x.value = (E.results ?? []).map((I) => ({ ...I, snippet: Ft(I.snippet) })), S.value = E.total_matches ?? 0;
      } else {
        const E = await t.get(`/api/memories?${N}`);
        if (d !== w) return;
        const I = Math.max(1, Math.ceil(E.total / i.value));
        if (((F = E.memories) == null ? void 0 : F.length) === 0 && E.total > 0 && y.value > I)
          return y.value = I, g.value = !1, A();
        v.value = E.memories ?? [], S.value = E.total ?? 0;
      }
    } finally {
      d === w && (g.value = !1);
    }
  }
  async function R() {
    try {
      const d = await Ye(t);
      $.value = (d.tags ?? []).map((F) => F.name);
    } catch {
    }
  }
  async function O(d) {
    if (d.id) {
      const F = await Pe(t, d.id), N = new Set(F.tags), E = new Set(d.tags);
      await Xe(t, d.id, {
        summary: d.summary,
        content: d.content,
        add_tags: [...E].filter((I) => !N.has(I)),
        remove_tags: [...N].filter((I) => !E.has(I))
      });
    } else
      await Ge(t, { summary: d.summary, content: d.content, tags: d.tags });
    await Promise.all([A(), R()]);
  }
  async function q(d) {
    await Ht(t, d), await A();
  }
  return fe(() => {
    A().catch(() => {
    }), R();
  }), {
    query: s,
    tagFilter: c,
    sort: u,
    order: m,
    page: y,
    pageSize: i,
    rows: v,
    searchResults: x,
    total: S,
    loading: g,
    tagOptions: $,
    searching: L,
    onSearch: D,
    reload: A,
    loadTagOptions: R,
    saveMemory: O,
    removeMemory: q
  };
}
function $e(t) {
  const n = b(0);
  let s = null;
  function c(y) {
    s == null || s.disconnect(), s = null, !(!y || typeof ResizeObserver > "u") && (s = new ResizeObserver((i) => {
      var v;
      n.value = ((v = i[0]) == null ? void 0 : v.contentRect.width) ?? 0;
    }), s.observe(y));
  }
  fe(() => c(t.value)), we(t, (y) => c(y)), at(() => s == null ? void 0 : s.disconnect());
  const u = re(() => n.value > 0 && n.value < 960), m = re(() => n.value > 0 && n.value < 720);
  return { width: n, compact: u, narrow: m };
}
const Kt = ["innerHTML"], Ze = /* @__PURE__ */ ae({
  __name: "MarkdownView",
  props: {
    source: {}
  },
  setup(t) {
    const n = t, s = re(() => Ot(n.source));
    return (c, u) => (h(), V("div", {
      class: "md-body",
      innerHTML: s.value
    }, null, 8, Kt));
  }
}), Qt = { class: "content-label" }, Gt = /* @__PURE__ */ ae({
  __name: "MemoryEditorDialog",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    tagOptions: {},
    width: {}
  },
  emits: ["update:visible", "saved"],
  setup(t, { emit: n }) {
    const s = t, c = n, u = de(), m = b(!1), y = b("edit"), i = b({ id: null, summary: "", content: "", tags: [] });
    let v = [];
    we(
      () => s.visible,
      async (S) => {
        if (S)
          if (y.value = "edit", s.memoryId)
            try {
              const g = await Pe(u, s.memoryId);
              i.value = { id: g.id, summary: g.summary, content: g.content, tags: [...g.tags] }, v = [...g.tags];
            } catch (g) {
              B.error(g instanceof Error ? g.message : String(g)), c("update:visible", !1);
            }
          else
            i.value = { id: null, summary: "", content: "", tags: [] }, v = [];
      }
    );
    async function x() {
      m.value = !0;
      try {
        if (i.value.id) {
          const S = new Set(v), g = new Set(i.value.tags);
          await Xe(u, i.value.id, {
            summary: i.value.summary,
            content: i.value.content,
            add_tags: [...g].filter(($) => !S.has($)),
            remove_tags: [...S].filter(($) => !g.has($))
          }), B.success(o("editor.updated"));
        } else
          await Ge(u, {
            summary: i.value.summary,
            content: i.value.content,
            tags: i.value.tags
          }), B.success(o("editor.created"));
        c("update:visible", !1), c("saved");
      } catch (S) {
        B.error(S instanceof Error ? S.message : String(S));
      } finally {
        m.value = !1;
      }
    }
    return (S, g) => {
      const $ = Ce, L = Ae, D = Ie, T = Te, w = Ne, A = Re, R = Le, O = ge, q = ze;
      return h(), z(q, {
        "model-value": t.visible,
        title: i.value.id ? e(o)("editor.editTitle") : e(o)("editor.createTitle"),
        width: t.width,
        "onUpdate:modelValue": g[5] || (g[5] = (d) => c("update:visible", d))
      }, {
        footer: l(() => [
          a(O, {
            onClick: g[4] || (g[4] = (d) => c("update:visible", !1))
          }, {
            default: l(() => [
              _(r(e(o)("common.cancel")), 1)
            ]),
            _: 1
          }),
          a(O, {
            type: "primary",
            loading: m.value,
            onClick: x
          }, {
            default: l(() => [
              _(r(e(o)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: l(() => [
          a(R, { "label-position": "top" }, {
            default: l(() => [
              a(L, {
                label: e(o)("editor.summaryLabel")
              }, {
                default: l(() => [
                  a($, {
                    modelValue: i.value.summary,
                    "onUpdate:modelValue": g[0] || (g[0] = (d) => i.value.summary = d),
                    maxlength: "512",
                    "show-word-limit": "",
                    placeholder: e(o)("editor.summaryPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"]),
              a(L, null, {
                label: l(() => [
                  P("div", Qt, [
                    P("span", null, r(e(o)("editor.contentLabel")), 1),
                    a(T, {
                      modelValue: y.value,
                      "onUpdate:modelValue": g[1] || (g[1] = (d) => y.value = d),
                      size: "small"
                    }, {
                      default: l(() => [
                        a(D, { value: "edit" }, {
                          default: l(() => [
                            _(r(e(o)("editor.tabEdit")), 1)
                          ]),
                          _: 1
                        }),
                        a(D, { value: "preview" }, {
                          default: l(() => [
                            _(r(e(o)("editor.tabPreview")), 1)
                          ]),
                          _: 1
                        })
                      ]),
                      _: 1
                    }, 8, ["modelValue"])
                  ])
                ]),
                default: l(() => [
                  y.value === "edit" ? (h(), z($, {
                    key: 0,
                    modelValue: i.value.content,
                    "onUpdate:modelValue": g[2] || (g[2] = (d) => i.value.content = d),
                    type: "textarea",
                    rows: 12,
                    maxlength: "262144",
                    "show-word-limit": "",
                    placeholder: e(o)("editor.contentPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])) : (h(), z(Ze, {
                    key: 1,
                    class: "content-preview",
                    source: i.value.content
                  }, null, 8, ["source"]))
                ]),
                _: 1
              }),
              a(L, {
                label: e(o)("editor.tagsLabel")
              }, {
                default: l(() => [
                  a(A, {
                    modelValue: i.value.tags,
                    "onUpdate:modelValue": g[3] || (g[3] = (d) => i.value.tags = d),
                    multiple: "",
                    filterable: "",
                    "allow-create": "",
                    "default-first-option": "",
                    placeholder: e(o)("editor.tagsPlaceholder"),
                    class: "tags-select"
                  }, {
                    default: l(() => [
                      (h(!0), V(ee, null, ne(t.tagOptions, (d) => (h(), z(w, {
                        key: d,
                        label: d,
                        value: d
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
}), ie = (t, n) => {
  const s = t.__vccOpts || t;
  for (const [c, u] of n)
    s[c] = u;
  return s;
}, Xt = /* @__PURE__ */ ie(Gt, [["__scopeId", "data-v-f7b9d45a"]]), Yt = { class: "detail-summary" }, Zt = { class: "detail-tags" }, ea = { class: "detail-toolbar" }, ta = {
  key: 1,
  class: "detail-content"
}, aa = /* @__PURE__ */ ae({
  __name: "MemoryDetailDrawer",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    size: {}
  },
  emits: ["update:visible"],
  setup(t, { emit: n }) {
    const s = t, c = n, u = de(), m = b(null), y = b("rendered");
    return we(
      () => [s.visible, s.memoryId],
      async ([i]) => {
        if (!(!i || !s.memoryId)) {
          y.value = "rendered";
          try {
            m.value = await Pe(u, s.memoryId);
          } catch (v) {
            B.error(v instanceof Error ? v.message : String(v)), c("update:visible", !1);
          }
        }
      }
    ), (i, v) => {
      var D;
      const x = Se, S = it, g = Ie, $ = Te, L = rt;
      return h(), z(L, {
        "model-value": t.visible,
        title: e(o)("drawer.title", { id: ((D = m.value) == null ? void 0 : D.id) ?? t.memoryId ?? "" }),
        size: t.size,
        "onUpdate:modelValue": v[1] || (v[1] = (T) => c("update:visible", T))
      }, {
        default: l(() => [
          m.value ? (h(), V(ee, { key: 0 }, [
            P("h3", Yt, r(m.value.summary), 1),
            P("div", Zt, [
              (h(!0), V(ee, null, ne(m.value.tags, (T) => (h(), z(x, {
                key: T,
                size: "small",
                class: "am-tag"
              }, {
                default: l(() => [
                  _(r(T), 1)
                ]),
                _: 2
              }, 1024))), 128))
            ]),
            a(S),
            P("div", ea, [
              a($, {
                modelValue: y.value,
                "onUpdate:modelValue": v[0] || (v[0] = (T) => y.value = T),
                size: "small"
              }, {
                default: l(() => [
                  a(g, { value: "rendered" }, {
                    default: l(() => [
                      _(r(e(o)("drawer.rendered")), 1)
                    ]),
                    _: 1
                  }),
                  a(g, { value: "source" }, {
                    default: l(() => [
                      _(r(e(o)("drawer.source")), 1)
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["modelValue"])
            ]),
            y.value === "rendered" ? (h(), z(Ze, {
              key: 0,
              source: m.value.content
            }, null, 8, ["source"])) : (h(), V("pre", ta, r(m.value.content), 1))
          ], 64)) : G("", !0)
        ]),
        _: 1
      }, 8, ["model-value", "title", "size"]);
    };
  }
}), oa = /* @__PURE__ */ ie(aa, [["__scopeId", "data-v-b634d116"]]), la = {
  key: 0,
  class: "am-panel-header"
}, na = { class: "am-heading" }, sa = { class: "am-panel-title" }, ra = { class: "am-toolbar" }, ia = { class: "am-summary" }, ca = ["innerHTML"], da = {
  key: 4,
  class: "am-pager"
}, ua = {
  key: 5,
  class: "am-pager"
}, ma = /* @__PURE__ */ ae({
  __name: "MemoriesPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: n }) {
    const s = t, {
      query: c,
      tagFilter: u,
      sort: m,
      order: y,
      page: i,
      pageSize: v,
      rows: x,
      searchResults: S,
      total: g,
      loading: $,
      tagOptions: L,
      searching: D,
      onSearch: T,
      reload: w,
      loadTagOptions: A,
      removeMemory: R
    } = Wt(), O = re(() => {
      if ($.value || g.value !== 0) return "";
      if (D.value) return o("memories.searchEmpty");
      const U = u.value.trim();
      return U ? o("memories.tagEmpty", { tag: U }) : "";
    }), q = b(null), { compact: d, narrow: F } = $e(q), N = b(!1), E = b(null), I = b(!1), C = b(null);
    function k(U) {
      return U().catch((p) => B.error(p instanceof Error ? p.message : String(p)));
    }
    function W() {
      E.value = null, N.value = !0;
    }
    function X(U) {
      E.value = U, N.value = !0;
    }
    function j(U) {
      C.value = U, I.value = !0;
    }
    function Y() {
      k(w), A();
    }
    function oe(U) {
      const { prop: p, order: le } = U;
      le && (p === "updated_at" || p === "created_at" || p === "id") ? (m.value = p, y.value = le === "ascending" ? "asc" : "desc") : (m.value = "updated_at", y.value = "desc"), k(w);
    }
    async function H(U) {
      try {
        await je.confirm(o("memories.deleteConfirm", { id: U.id }), o("memories.deleteTitle"), {
          type: "warning"
        });
      } catch {
        return;
      }
      try {
        await R(U.id), B.success(o("memories.deleted"));
      } catch (p) {
        B.error(p instanceof Error ? p.message : String(p));
      }
    }
    return n({ refresh: () => k(w) }), (U, p) => {
      const le = ve, ue = Ee, K = ge, ye = Ce, he = Ne, M = Re, Z = Be, J = Fe, De = Se, xe = Oe, Ue = ct, Ve = He;
      return h(), V("div", {
        ref_key: "rootRef",
        ref: q,
        class: "am-panel"
      }, [
        t.showHeader ? (h(), V("div", la, [
          P("div", na, [
            P("h2", sa, r(s.title ?? e(o)("memories.title")), 1),
            a(ue, {
              content: s.subtitle ?? e(o)("memories.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(le, { class: "am-info" }, {
                  default: l(() => [
                    a(e(pe))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(K, {
            type: "primary",
            icon: e(qe),
            onClick: W
          }, {
            default: l(() => [
              _(r(e(o)("memories.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : G("", !0),
        P("div", ra, [
          a(ye, {
            modelValue: e(c),
            "onUpdate:modelValue": p[1] || (p[1] = (f) => te(c) ? c.value = f : null),
            placeholder: e(o)("memories.searchPlaceholder"),
            clearable: "",
            class: "search",
            onKeyup: p[2] || (p[2] = ot((f) => k(e(T)), ["enter"])),
            onClear: p[3] || (p[3] = (f) => k(e(T)))
          }, {
            append: l(() => [
              a(K, {
                icon: e(Me),
                onClick: p[0] || (p[0] = (f) => k(e(T)))
              }, null, 8, ["icon"])
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(M, {
            modelValue: e(u),
            "onUpdate:modelValue": p[4] || (p[4] = (f) => te(u) ? u.value = f : null),
            placeholder: e(o)("memories.tagFilter"),
            clearable: "",
            filterable: "",
            class: "tag-filter",
            onChange: p[5] || (p[5] = (f) => k(e(T)))
          }, {
            default: l(() => [
              (h(!0), V(ee, null, ne(e(L), (f) => (h(), z(he, {
                key: f,
                label: f,
                value: f
              }, null, 8, ["label", "value"]))), 128))
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"])
        ]),
        O.value ? (h(), z(Z, {
          key: 1,
          title: O.value,
          type: "info",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : G("", !0),
        e(D) ? se((h(), z(xe, {
          key: 2,
          data: e(S)
        }, {
          default: l(() => [
            a(J, {
              prop: "id",
              label: e(o)("memories.colId"),
              width: "80"
            }, null, 8, ["label"]),
            a(J, {
              label: e(o)("memories.colSummary")
            }, {
              default: l(({ row: f }) => [
                P("div", ia, r(f.summary), 1),
                P("div", {
                  class: "am-snippet",
                  innerHTML: f.snippet
                }, null, 8, ca)
              ]),
              _: 1
            }, 8, ["label"]),
            a(J, {
              label: e(o)("memories.colTags"),
              "min-width": "150"
            }, {
              default: l(({ row: f }) => [
                (h(!0), V(ee, null, ne(f.tags, (Q) => (h(), z(De, {
                  key: Q,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: l(() => [
                    _(r(Q), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            a(J, {
              prop: "score",
              label: e(o)("memories.colScore"),
              width: "80",
              sortable: ""
            }, null, 8, ["label"]),
            a(J, {
              label: e(o)("memories.colUpdatedAt"),
              width: "170"
            }, {
              default: l(({ row: f }) => [
                _(r(e(ce)(f.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(J, {
              label: e(o)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: l(({ row: f }) => [
                a(K, {
                  link: "",
                  type: "primary",
                  onClick: (Q) => j(f.id)
                }, {
                  default: l(() => [
                    _(r(e(o)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(K, {
                  link: "",
                  type: "primary",
                  onClick: (Q) => X(f.id)
                }, {
                  default: l(() => [
                    _(r(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(K, {
                  link: "",
                  type: "danger",
                  onClick: (Q) => H(f)
                }, {
                  default: l(() => [
                    _(r(e(o)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [Ve, e($)]
        ]) : se((h(), z(xe, {
          key: 3,
          data: e(x),
          "default-sort": { prop: e(m), order: e(y) === "asc" ? "ascending" : "descending" },
          onSortChange: oe
        }, {
          default: l(() => [
            a(J, {
              prop: "id",
              label: e(o)("memories.colId"),
              width: "80",
              sortable: "custom"
            }, null, 8, ["label"]),
            a(J, {
              prop: "summary",
              label: e(o)("memories.colSummary"),
              "min-width": "180",
              "show-overflow-tooltip": ""
            }, null, 8, ["label"]),
            a(J, {
              label: e(o)("memories.colTags"),
              "min-width": "150"
            }, {
              default: l(({ row: f }) => [
                (h(!0), V(ee, null, ne(f.tags, (Q) => (h(), z(De, {
                  key: Q,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: l(() => [
                    _(r(Q), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            e(d) ? G("", !0) : (h(), z(J, {
              key: 0,
              prop: "created_at",
              label: e(o)("memories.colCreatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: l(({ row: f }) => [
                _(r(e(ce)(f.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label"])),
            a(J, {
              prop: "updated_at",
              label: e(o)("memories.colUpdatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: l(({ row: f }) => [
                _(r(e(ce)(f.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(J, {
              label: e(o)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: l(({ row: f }) => [
                a(K, {
                  link: "",
                  type: "primary",
                  onClick: (Q) => j(f.id)
                }, {
                  default: l(() => [
                    _(r(e(o)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(K, {
                  link: "",
                  type: "primary",
                  onClick: (Q) => X(f.id)
                }, {
                  default: l(() => [
                    _(r(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(K, {
                  link: "",
                  type: "danger",
                  onClick: (Q) => H(f)
                }, {
                  default: l(() => [
                    _(r(e(o)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data", "default-sort"])), [
          [Ve, e($)]
        ]),
        e(D) ? (h(), V("div", ua, [
          a(Ue, {
            "current-page": e(i),
            "onUpdate:currentPage": p[10] || (p[10] = (f) => te(i) ? i.value = f : null),
            "page-size": e(v),
            "onUpdate:pageSize": p[11] || (p[11] = (f) => te(v) ? v.value = f : null),
            total: e(g),
            "page-sizes": [10, 20, 50],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: p[12] || (p[12] = (f) => k(e(w))),
            onSizeChange: p[13] || (p[13] = (f) => k(e(w)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])) : (h(), V("div", da, [
          a(Ue, {
            "current-page": e(i),
            "onUpdate:currentPage": p[6] || (p[6] = (f) => te(i) ? i.value = f : null),
            "page-size": e(v),
            "onUpdate:pageSize": p[7] || (p[7] = (f) => te(v) ? v.value = f : null),
            total: e(g),
            "page-sizes": [20, 50, 100, 200],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: p[8] || (p[8] = (f) => k(e(w))),
            onSizeChange: p[9] || (p[9] = (f) => k(e(w)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])),
        a(Xt, {
          visible: N.value,
          "onUpdate:visible": p[14] || (p[14] = (f) => N.value = f),
          "memory-id": E.value,
          "tag-options": e(L),
          width: e(F) ? "96%" : "640px",
          onSaved: Y
        }, null, 8, ["visible", "memory-id", "tag-options", "width"]),
        a(oa, {
          visible: I.value,
          "onUpdate:visible": p[15] || (p[15] = (f) => I.value = f),
          "memory-id": C.value,
          size: e(F) ? "100%" : "45%"
        }, null, 8, ["visible", "memory-id", "size"])
      ], 512);
    };
  }
}), pa = /* @__PURE__ */ ie(ma, [["__scopeId", "data-v-b2f17399"]]);
function fa() {
  const t = de(), n = b([]), s = b(!1), c = b("");
  async function u() {
    s.value = !0;
    try {
      const v = await Ye(t, c.value.trim() || void 0);
      n.value = v.tags ?? [];
    } finally {
      s.value = !1;
    }
  }
  async function m(v, x) {
    await qt(t, v, x), await u();
  }
  async function y(v, x, S) {
    await jt(t, v, x, S), await u();
  }
  async function i(v, x) {
    await Jt(t, v, x), await u();
  }
  return fe(() => {
    u().catch(() => {
    });
  }), { rows: n, loading: s, filter: c, reload: u, create: m, rename: y, remove: i };
}
const ga = {
  key: 0,
  class: "am-panel-header"
}, va = { class: "am-heading" }, _a = { class: "am-panel-title" }, ya = { class: "delete-body" }, ha = /* @__PURE__ */ ae({
  __name: "TagsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: n }) {
    const s = t, { rows: c, loading: u, filter: m, reload: y, create: i, rename: v, remove: x } = fa();
    let S = null;
    function g() {
      S && clearTimeout(S), S = setTimeout(() => {
        S = null, y().catch((C) => B.error(C instanceof Error ? C.message : String(C)));
      }, 300);
    }
    const $ = b(null), { narrow: L } = $e($), D = b(!1), T = b(!1), w = lt({ oldName: null, name: "", description: "" }), A = b(!1), R = b("detach"), O = b(null);
    function q(C) {
      return (k, W) => (k[C] ?? 0) - (W[C] ?? 0);
    }
    function d() {
      Object.assign(w, { oldName: null, name: "", description: "" }), T.value = !0;
    }
    function F(C) {
      Object.assign(w, { oldName: C.name, name: C.name, description: C.description ?? "" }), T.value = !0;
    }
    async function N() {
      D.value = !0;
      try {
        w.oldName ? (await v(w.oldName, w.name, w.description), B.success(o("tags.saved"))) : (await i(w.name, w.description), B.success(o("tags.created"))), T.value = !1;
      } catch (C) {
        B.error(C instanceof Error ? C.message : String(C));
      } finally {
        D.value = !1;
      }
    }
    function E(C) {
      O.value = C, R.value = "detach", A.value = !0;
    }
    async function I() {
      var C, k;
      if (R.value === "purge")
        try {
          await je.confirm(
            o("tags.purgeConfirm", { name: (C = O.value) == null ? void 0 : C.name, count: ((k = O.value) == null ? void 0 : k.memory_count) ?? 0 }),
            o("tags.purgeConfirmTitle"),
            { type: "error", confirmButtonText: o("tags.purgeButton") }
          );
        } catch {
          return;
        }
      if (O.value) {
        D.value = !0;
        try {
          await x(O.value.name, R.value), B.success(o("tags.deleted")), A.value = !1;
        } catch (W) {
          B.error(W instanceof Error ? W.message : String(W));
        } finally {
          D.value = !1;
        }
      }
    }
    return n({
      refresh: () => y().catch((C) => B.error(C instanceof Error ? C.message : String(C)))
    }), (C, k) => {
      const W = ve, X = Ee, j = ge, Y = Ce, oe = Se, H = Fe, U = Oe, p = Ae, le = Le, ue = ze, K = dt, ye = Te, he = He;
      return h(), V("div", {
        ref_key: "rootRef",
        ref: $,
        class: "am-panel"
      }, [
        t.showHeader ? (h(), V("div", ga, [
          P("div", va, [
            P("h2", _a, r(s.title ?? e(o)("tags.title")), 1),
            a(X, {
              content: s.subtitle ?? e(o)("tags.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(W, { class: "am-info" }, {
                  default: l(() => [
                    a(e(pe))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(j, {
            type: "primary",
            icon: e(qe),
            onClick: d
          }, {
            default: l(() => [
              _(r(e(o)("tags.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : G("", !0),
        a(Y, {
          modelValue: e(m),
          "onUpdate:modelValue": k[0] || (k[0] = (M) => te(m) ? m.value = M : null),
          class: "am-tag-filter",
          placeholder: e(o)("tags.filterPlaceholder"),
          clearable: "",
          "prefix-icon": e(Me),
          onInput: g,
          onClear: g
        }, null, 8, ["modelValue", "placeholder", "prefix-icon"]),
        se((h(), z(U, { data: e(c) }, {
          default: l(() => [
            a(H, {
              prop: "name",
              label: e(o)("tags.colName"),
              "min-width": "140",
              sortable: ""
            }, {
              default: l(({ row: M }) => [
                a(oe, null, {
                  default: l(() => [
                    _(r(M.name), 1)
                  ]),
                  _: 2
                }, 1024)
              ]),
              _: 1
            }, 8, ["label"]),
            a(H, {
              prop: "description",
              label: e(o)("tags.colDescription"),
              "min-width": "150",
              "show-overflow-tooltip": ""
            }, {
              default: l(({ row: M }) => [
                _(r(M.description || "—"), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(H, {
              prop: "memory_count",
              label: e(o)("tags.colMemoryCount"),
              width: "90",
              sortable: ""
            }, null, 8, ["label"]),
            a(H, {
              prop: "last_used_at",
              label: e(o)("tags.colLastUsed"),
              width: "170",
              sortable: "",
              "sort-method": q("last_used_at")
            }, {
              default: l(({ row: M }) => [
                _(r(e(ce)(M.last_used_at)), 1)
              ]),
              _: 1
            }, 8, ["label", "sort-method"]),
            a(H, {
              prop: "created_at",
              label: e(o)("tags.colCreatedAt"),
              width: "170",
              sortable: "",
              "sort-method": q("created_at")
            }, {
              default: l(({ row: M }) => [
                _(r(e(ce)(M.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label", "sort-method"]),
            a(H, {
              label: e(o)("memories.colActions"),
              width: "150",
              fixed: "right"
            }, {
              default: l(({ row: M }) => [
                a(j, {
                  link: "",
                  type: "primary",
                  onClick: (Z) => F(M)
                }, {
                  default: l(() => [
                    _(r(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(j, {
                  link: "",
                  type: "danger",
                  onClick: (Z) => E(M)
                }, {
                  default: l(() => [
                    _(r(e(o)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [he, e(u)]
        ]),
        a(ue, {
          modelValue: T.value,
          "onUpdate:modelValue": k[4] || (k[4] = (M) => T.value = M),
          title: w.oldName ? e(o)("tags.editTitle") : e(o)("tags.createTitle"),
          width: e(L) ? "96%" : "480px"
        }, {
          footer: l(() => [
            a(j, {
              onClick: k[3] || (k[3] = (M) => T.value = !1)
            }, {
              default: l(() => [
                _(r(e(o)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(j, {
              type: "primary",
              loading: D.value,
              onClick: N
            }, {
              default: l(() => [
                _(r(e(o)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: l(() => [
            a(le, { "label-position": "top" }, {
              default: l(() => [
                a(p, {
                  label: e(o)("tags.nameLabel")
                }, {
                  default: l(() => [
                    a(Y, {
                      modelValue: w.name,
                      "onUpdate:modelValue": k[1] || (k[1] = (M) => w.name = M),
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(o)("tags.namePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(p, {
                  label: e(o)("tags.descLabel")
                }, {
                  default: l(() => [
                    a(Y, {
                      modelValue: w.description,
                      "onUpdate:modelValue": k[2] || (k[2] = (M) => w.description = M),
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
        a(ue, {
          modelValue: A.value,
          "onUpdate:modelValue": k[7] || (k[7] = (M) => A.value = M),
          title: e(o)("tags.deleteTitle"),
          width: e(L) ? "96%" : "480px"
        }, {
          footer: l(() => [
            a(j, {
              onClick: k[6] || (k[6] = (M) => A.value = !1)
            }, {
              default: l(() => [
                _(r(e(o)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(j, {
              type: "danger",
              loading: D.value,
              onClick: I
            }, {
              default: l(() => [
                _(r(e(o)("common.delete")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: l(() => {
            var M;
            return [
              P("p", ya, [
                _(r(e(o)("tags.deleteBefore")) + " ", 1),
                a(oe, null, {
                  default: l(() => {
                    var Z;
                    return [
                      _(r((Z = O.value) == null ? void 0 : Z.name), 1)
                    ];
                  }),
                  _: 1
                }),
                _(" " + r(e(o)("tags.deleteMiddle")) + " ", 1),
                P("b", null, r((M = O.value) == null ? void 0 : M.memory_count), 1),
                _(" " + r(e(o)("tags.deleteAfter")), 1)
              ]),
              a(ye, {
                modelValue: R.value,
                "onUpdate:modelValue": k[5] || (k[5] = (Z) => R.value = Z)
              }, {
                default: l(() => [
                  a(K, { value: "detach" }, {
                    default: l(() => [
                      _(r(e(o)("tags.detach")), 1)
                    ]),
                    _: 1
                  }),
                  a(K, { value: "purge" }, {
                    default: l(() => [
                      _(r(e(o)("tags.purge")), 1)
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
}), ba = /* @__PURE__ */ ie(ha, [["__scopeId", "data-v-825326f8"]]);
function ka(t) {
  return t.get("/api/stats");
}
function wa(t) {
  return t.get("/health");
}
function Ca(t) {
  return t.get("/api/doctor");
}
function Ta(t) {
  return t.getBlob("/api/export");
}
function Sa(t, n) {
  return t.post("/api/import", n);
}
function Ea() {
  const t = de(), n = b({}), s = b(""), c = b({ ok: !0, issues: [] }), u = b(!1), m = b(!1), y = b(!1), i = b(!1), v = re(() => It(n.value.file_size));
  async function x() {
    n.value = await ka(t), n.value.version = s.value;
  }
  async function S() {
    m.value = !0;
    try {
      c.value = await Ca(t), u.value = !0;
    } finally {
      m.value = !1;
    }
  }
  async function g() {
    y.value = !0;
    try {
      const L = await Ta(t), D = URL.createObjectURL(L), T = document.createElement("a");
      T.href = D, T.download = "agent-memory-export.json", T.click(), URL.revokeObjectURL(D);
    } finally {
      y.value = !1;
    }
  }
  async function $(L) {
    i.value = !0;
    try {
      const D = await L.text();
      let T;
      try {
        T = JSON.parse(D);
      } catch {
        throw new Error(o("errors.invalidBackup"));
      }
      const w = await Sa(t, T);
      return await x(), w;
    } finally {
      i.value = !1;
    }
  }
  return fe(async () => {
    try {
      s.value = (await wa(t)).version ?? "";
    } catch {
    }
    await x().catch(() => {
    });
  }), {
    stats: n,
    version: s,
    doctor: c,
    doctorRan: u,
    doctorLoading: m,
    exporting: y,
    importing: i,
    sizeText: v,
    reload: x,
    runDoctor: S,
    exportData: g,
    importFile: $
  };
}
const Ma = {
  key: 0,
  class: "am-panel-header"
}, Pa = { class: "am-heading" }, $a = { class: "am-panel-title" }, Da = { class: "actions" }, xa = { class: "card-header" }, Ua = {
  key: 2,
  class: "issues"
}, Va = /* @__PURE__ */ ae({
  __name: "OpsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: n }) {
    const s = t, {
      stats: c,
      version: u,
      doctor: m,
      doctorRan: y,
      doctorLoading: i,
      exporting: v,
      importing: x,
      sizeText: S,
      reload: g,
      runDoctor: $,
      exportData: L,
      importFile: D
    } = Ea(), T = b(null), { narrow: w } = $e(T), A = b(null);
    function R(q) {
      return q().catch((d) => B.error(d instanceof Error ? d.message : String(d)));
    }
    function O(q) {
      var N;
      const d = q.target, F = (N = d.files) == null ? void 0 : N[0];
      d.value = "", F && D(F).then((E) => {
        B.success(o("ops.imported", { memories: E.imported_memories, tags: E.imported_tags }));
      }).catch((E) => {
        B.error(E instanceof Error ? E.message : String(E));
      });
    }
    return n({ refresh: () => R(g) }), (q, d) => {
      const F = ve, N = Ee, E = ge, I = ft, C = pt, k = mt, W = ut, X = vt, j = gt, Y = Be, oe = _t;
      return h(), V("div", {
        ref_key: "rootRef",
        ref: T,
        class: "am-panel"
      }, [
        t.showHeader ? (h(), V("div", Ma, [
          P("div", Pa, [
            P("h2", $a, r(s.title ?? e(o)("ops.title")), 1),
            a(N, {
              content: s.subtitle ?? e(o)("ops.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(F, { class: "am-info" }, {
                  default: l(() => [
                    a(e(pe))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(E, {
            icon: e(St),
            onClick: d[0] || (d[0] = (H) => R(e(g)))
          }, {
            default: l(() => [
              _(r(e(o)("ops.refresh")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : G("", !0),
        a(W, { gutter: 14 }, {
          default: l(() => [
            a(k, {
              span: e(w) ? 12 : 8
            }, {
              default: l(() => [
                a(C, { shadow: "never" }, {
                  default: l(() => [
                    a(I, {
                      title: e(o)("ops.statMemories"),
                      value: e(c).memories ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(k, {
              span: e(w) ? 12 : 8
            }, {
              default: l(() => [
                a(C, { shadow: "never" }, {
                  default: l(() => [
                    a(I, {
                      title: e(o)("ops.statTags"),
                      value: e(c).tags ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(k, {
              span: e(w) ? 12 : 8
            }, {
              default: l(() => [
                a(C, { shadow: "never" }, {
                  default: l(() => [
                    a(I, {
                      title: e(o)("ops.statSize"),
                      value: e(S)
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
          header: l(() => [
            _(r(e(o)("ops.dbCard")), 1)
          ]),
          default: l(() => [
            a(j, {
              column: e(w) ? 1 : 2,
              border: ""
            }, {
              default: l(() => [
                a(X, {
                  label: e(o)("ops.version")
                }, {
                  default: l(() => [
                    _(r(e(c).version ?? e(u)), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(X, {
                  label: e(o)("ops.schemaVersion")
                }, {
                  default: l(() => [
                    _(r(e(c).schema_version ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(X, {
                  label: e(o)("ops.path"),
                  span: e(w) ? 1 : 2
                }, {
                  default: l(() => [
                    _(r(e(c).path ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label", "span"])
              ]),
              _: 1
            }, 8, ["column"]),
            P("div", Da, [
              a(E, {
                icon: e(Et),
                loading: e(v),
                onClick: d[1] || (d[1] = (H) => R(e(L)))
              }, {
                default: l(() => [
                  _(r(e(o)("ops.export")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(E, {
                icon: e(Mt),
                loading: e(x),
                onClick: d[2] || (d[2] = (H) => {
                  var U;
                  return (U = A.value) == null ? void 0 : U.click();
                })
              }, {
                default: l(() => [
                  _(r(e(o)("ops.import")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(N, {
                content: e(o)("ops.importHint"),
                placement: "top"
              }, {
                default: l(() => [
                  a(F, { class: "am-info" }, {
                    default: l(() => [
                      a(e(pe))
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["content"]),
              P("input", {
                ref_key: "importInput",
                ref: A,
                type: "file",
                accept: "application/json,.json",
                style: { display: "none" },
                onChange: O
              }, null, 544)
            ])
          ]),
          _: 1
        }),
        a(C, { shadow: "never" }, {
          header: l(() => [
            P("div", xa, [
              P("span", null, r(e(o)("ops.doctorCard")), 1),
              a(E, {
                size: "small",
                icon: e(Me),
                loading: e(i),
                onClick: d[3] || (d[3] = (H) => R(e($)))
              }, {
                default: l(() => [
                  _(r(e(o)("ops.runDoctor")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"])
            ])
          ]),
          default: l(() => [
            e(y) ? (h(), V(ee, { key: 0 }, [
              e(m).ok ? (h(), z(Y, {
                key: 0,
                title: e(o)("ops.doctorOk"),
                type: "success",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])) : (h(), z(Y, {
                key: 1,
                title: e(o)("ops.doctorFail", { count: e(m).issues.length }),
                type: "error",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])),
              e(m).ok ? G("", !0) : (h(), V("ul", Ua, [
                (h(!0), V(ee, null, ne(e(m).issues, (H, U) => (h(), V("li", { key: U }, r(H), 1))), 128))
              ]))
            ], 64)) : (h(), z(oe, {
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
}), za = /* @__PURE__ */ ie(Va, [["__scopeId", "data-v-f13cac5e"]]), La = { class: "memory-ui" }, Aa = { class: "brand" }, Ia = { class: "brand-mark" }, Ra = { class: "aside-footer" }, Na = /* @__PURE__ */ ae({
  __name: "MemoryAdmin",
  props: {
    layout: { default: "sidebar" },
    title: { default: "Agent Memory" }
  },
  setup(t) {
    const n = b("memories");
    return (s, c) => {
      const u = ve, m = kt, y = bt, i = ht, v = Tt, x = Ct, S = wt, g = yt;
      return h(), V("div", La, [
        a(g, { class: "layout" }, {
          default: l(() => [
            t.layout === "sidebar" ? (h(), z(i, {
              key: 0,
              width: "200px",
              class: "aside"
            }, {
              default: l(() => [
                P("div", Aa, [
                  P("span", Ia, [
                    a(u, { size: 16 }, {
                      default: l(() => [
                        a(e(Pt))
                      ]),
                      _: 1
                    })
                  ]),
                  P("span", null, r(t.title), 1)
                ]),
                a(y, {
                  "default-active": n.value,
                  class: "menu",
                  onSelect: c[0] || (c[0] = ($) => n.value = $)
                }, {
                  default: l(() => [
                    a(m, { index: "memories" }, {
                      default: l(() => [
                        a(u, null, {
                          default: l(() => [
                            a(e($t))
                          ]),
                          _: 1
                        }),
                        P("span", null, r(e(o)("nav.memories")), 1)
                      ]),
                      _: 1
                    }),
                    a(m, { index: "tags" }, {
                      default: l(() => [
                        a(u, null, {
                          default: l(() => [
                            a(e(Dt))
                          ]),
                          _: 1
                        }),
                        P("span", null, r(e(o)("nav.tags")), 1)
                      ]),
                      _: 1
                    }),
                    a(m, { index: "ops" }, {
                      default: l(() => [
                        a(u, null, {
                          default: l(() => [
                            a(e(xt))
                          ]),
                          _: 1
                        }),
                        P("span", null, r(e(o)("nav.ops")), 1)
                      ]),
                      _: 1
                    })
                  ]),
                  _: 1
                }, 8, ["default-active"]),
                P("div", Ra, [
                  nt(s.$slots, "footer", {}, void 0, !0)
                ])
              ]),
              _: 3
            })) : G("", !0),
            a(S, { class: "main" }, {
              default: l(() => [
                t.layout === "tabs" ? (h(), z(x, {
                  key: 0,
                  modelValue: n.value,
                  "onUpdate:modelValue": c[1] || (c[1] = ($) => n.value = $),
                  class: "tabs-bar"
                }, {
                  default: l(() => [
                    a(v, {
                      label: e(o)("nav.memories"),
                      name: "memories"
                    }, null, 8, ["label"]),
                    a(v, {
                      label: e(o)("nav.tags"),
                      name: "tags"
                    }, null, 8, ["label"]),
                    a(v, {
                      label: e(o)("nav.ops"),
                      name: "ops"
                    }, null, 8, ["label"])
                  ]),
                  _: 1
                }, 8, ["modelValue"])) : G("", !0),
                se(a(pa, null, null, 512), [
                  [be, n.value === "memories"]
                ]),
                se(a(ba, null, null, 512), [
                  [be, n.value === "tags"]
                ]),
                se(a(za, null, null, 512), [
                  [be, n.value === "ops"]
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
}), Qa = /* @__PURE__ */ ie(Na, [["__scopeId", "data-v-38c91753"]]);
export {
  Ze as MarkdownView,
  pa as MemoriesPanel,
  Qa as MemoryAdmin,
  oa as MemoryDetailDrawer,
  Xt as MemoryEditorDialog,
  Ke as MemoryUIConfigKey,
  za as OpsPanel,
  ba as TagsPanel,
  At as applyMemoryUILocalePreference,
  Nt as buildMemoriesQuery,
  Rt as createApiClient,
  Wa as currentMemoryUILocale,
  It as formatSize,
  ce as formatTime,
  ke as isSearchMode,
  _e as memoryUIi18n,
  Ka as provideMemoryUI,
  Ot as renderMarkdown,
  Ft as sanitizeHtml,
  Lt as setMemoryUILocale,
  o as t,
  de as useApiClient,
  Wt as useMemories,
  Qe as useMemoryConfig,
  Ea as useOps,
  fa as useTags
};
