import { inject as Mt, provide as Pt, ref as _, computed as ve, onMounted as Me, watch as Le, onUnmounted as Ut, defineComponent as ue, openBlock as v, createElementBlock as D, createBlock as $, unref as e, withCtx as n, createVNode as o, createElementVNode as w, toDisplayString as l, createTextVNode as u, Fragment as se, renderList as de, createCommentVNode as J, withKeys as At, isRef as fe, withDirectives as he, reactive as ct, renderSlot as Dt, vShow as je, withModifiers as rt } from "vue";
import { createI18n as It } from "vue-i18n";
import { ElDialog as Je, ElForm as We, ElFormItem as Ge, ElInput as Be, ElRadioGroup as Re, ElRadioButton as Qe, ElSelect as dt, ElOption as ut, ElButton as Pe, ElDrawer as zt, ElTag as Ne, ElDivider as Lt, ElTooltip as Oe, ElIcon as Ue, ElAlert as mt, ElTable as Xe, ElTableColumn as Ye, ElLoadingDirective as pt, ElPagination as Bt, ElRadio as Rt, ElRow as Nt, ElCol as Ot, ElCard as ft, ElStatistic as Ft, ElDescriptions as Ht, ElDescriptionsItem as qt, ElContainer as jt, ElAside as Kt, ElMenu as Jt, ElMenuItem as Wt, ElMain as Gt, ElTabs as Qt, ElTabPane as Xt, ElSwitch as Yt, ElEmpty as Zt, ElCheckbox as ea } from "element-plus/es";
import { InfoFilled as $e, Plus as Ze, Search as et, Refresh as ta, Collection as aa, Notebook as oa, PriceTag as na, Odometer as la, Download as sa, UploadFilled as ia } from "@element-plus/icons-vue";
import { ElMessage as ra, ElMessageBox as ye } from "element-plus";
import { Marked as ca } from "marked";
import vt from "dompurify";
const da = {
  nav: {
    memories: "记忆管理",
    tags: "标签管理",
    ops: "运维",
    admin: "管理"
  },
  common: {
    detail: "详情",
    edit: "编辑",
    delete: "删除",
    cancel: "取消",
    save: "保存",
    ok: "知道了"
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
    subtitle: "数据库概况与版本信息",
    refresh: "刷新概况",
    statMemories: "记忆条数",
    statTags: "标签数",
    statSize: "数据库大小",
    dbCard: "数据库",
    path: "文件路径",
    schemaVersion: "数据库版本",
    version: "版本"
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
    identity: "身份",
    addIdentity: "添加身份",
    addIdentityDesc: "粘贴身份的访问令牌，本浏览器会记住它并切换过去",
    removeIdentity: "移除此浏览器保存的令牌"
  },
  access: {
    title: "管理",
    subtitle: "身份与访问、备份导入导出、数据体检与自定义提示词（admin 专属）",
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
    authHint: "开启后，所有 /mcp 与 /api 请求都必须携带 Authorization: Bearer <token>。开启前请先创建身份并保存 token（创建弹窗里可一键保存到本浏览器）；开启后本浏览器会弹出令牌输入框，输入一次即可。",
    authEnableConfirm: "开启 token 鉴权？此后所有 /mcp 与 /api 请求都必须携带有效 token。",
    authDisableConfirm: "关闭 token 鉴权？所有请求将免鉴权放行（开放模式）。",
    enableBlocked: "尚不具备开启条件：库里还没有具备管理员能力的身份。请先创建身份并妥善保存其 token，再打开开关。",
    saveToBrowser: "保存到本浏览器",
    savedToBrowser: "已保存身份「{name}」的 token 到本浏览器",
    backupCard: "备份导入导出",
    export: "导出备份（JSON）",
    import: "导入备份",
    importHint: "导入仅支持空数据库（导入是恢复而非合并）；当前库已有数据时请换一个空的 --db 路径再导入。",
    doctorCard: "数据体检（doctor）",
    runDoctor: "运行体检",
    doctorOk: "体检通过：未发现孤儿引用、大小写冲突或空字段",
    doctorFail: "发现 {count} 个问题",
    doctorEmpty: "点击「运行体检」检查孤儿标签引用、大小写冲突标签组、空摘要/正文",
    imported: "已导入 {memories} 条记忆、{tags} 个标签"
  }
}, ua = {
  nav: {
    memories: "Memories",
    tags: "Tags",
    ops: "Operations",
    admin: "Admin"
  },
  common: {
    detail: "Details",
    edit: "Edit",
    delete: "Delete",
    cancel: "Cancel",
    save: "Save",
    ok: "Got it"
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
    subtitle: "Database overview and version info",
    refresh: "Refresh overview",
    statMemories: "Memories",
    statTags: "Tags",
    statSize: "DB size",
    dbCard: "Database",
    path: "File path",
    schemaVersion: "Schema version",
    version: "Version"
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
    identity: "Identity",
    addIdentity: "Add identity",
    addIdentityDesc: "Paste an identity token; this browser will remember it and switch to it",
    removeIdentity: "Remove the token saved in this browser"
  },
  access: {
    title: "Admin",
    subtitle: "Identities & access, backup import/export, data checks, and custom prompts (admin only)",
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
    authHint: "When enabled, every /mcp and /api request must carry Authorization: Bearer <token>. Create an identity and save its token first (the creation dialog offers one-click save to this browser); after enabling, this browser will show the token prompt once.",
    authEnableConfirm: "Enable token auth? All /mcp and /api requests will then require a valid token.",
    authDisableConfirm: "Disable token auth? All requests will be allowed without credentials (open mode).",
    enableBlocked: "Not ready to enable: no admin-capable identity exists yet. Create one, keep its token safe, then flip the switch.",
    saveToBrowser: "Save to this browser",
    savedToBrowser: 'Token of identity "{name}" saved to this browser',
    backupCard: "Backup import & export",
    export: "Export backup (JSON)",
    import: "Import backup",
    importHint: "Import only works on an empty database (it restores rather than merges); point --db at an empty path before importing.",
    doctorCard: "Health check (doctor)",
    runDoctor: "Run check",
    doctorOk: "Check passed: no orphan references, case conflicts, or empty fields",
    doctorFail: "{count} issues found",
    doctorEmpty: 'Click "Run check" to look for orphan tag references, case-conflicting tags, empty summaries/content',
    imported: "Imported {memories} memories and {tags} tags"
  }
};
function _t() {
  var a;
  return typeof navigator > "u" || (a = navigator.language) != null && a.toLowerCase().startsWith("zh") ? "zh" : "en";
}
const Fe = It({
  legacy: !1,
  locale: _t(),
  fallbackLocale: "zh",
  messages: { zh: da, en: ua },
  // 面向宿主组件库，缺 key 时静默回退即可，不刷控制台
  missingWarn: !1,
  fallbackWarn: !1
}), { t } = Fe.global;
function ma(a) {
  Fe.global.locale.value = a;
}
function pa(a) {
  ma(a === "auto" ? _t() : a);
}
function Ko() {
  return Fe.global.locale.value;
}
const gt = Symbol("memory-ui-config"), ze = {
  baseUrl: "",
  fetch: (...a) => globalThis.fetch(...a),
  defaultPageSize: 20,
  locale: "auto"
};
function Jo(a) {
  a.locale && pa(a.locale), Pt(gt, a);
}
function tt() {
  const a = Mt(gt);
  return {
    baseUrl: ((a == null ? void 0 : a.baseUrl) ?? ze.baseUrl).replace(/\/+$/, ""),
    fetch: (a == null ? void 0 : a.fetch) ?? ze.fetch,
    defaultPageSize: (a == null ? void 0 : a.defaultPageSize) ?? ze.defaultPageSize,
    locale: (a == null ? void 0 : a.locale) ?? ze.locale,
    onIdentityToken: a == null ? void 0 : a.onIdentityToken,
    onAuthChanged: a == null ? void 0 : a.onAuthChanged
  };
}
const fa = 72;
function yt(a, i) {
  let s;
  return s = ra({
    type: a,
    message: i,
    offset: fa,
    showClose: !0,
    grouping: !0,
    onClick: () => s.close()
  }), s;
}
function ne(a) {
  return yt("success", a);
}
function X(a) {
  return yt("error", a);
}
function be(a) {
  if (!a) return "—";
  const i = Fe.global.locale.value === "en" ? "en-US" : "zh-CN";
  return new Date(a * 1e3).toLocaleString(i, { hour12: !1 });
}
function va(a) {
  return a == null ? "—" : a < 1024 ? `${a} B` : a < 1024 * 1024 ? `${(a / 1024).toFixed(1)} KB` : `${(a / 1024 / 1024).toFixed(2)} MB`;
}
function _a(a) {
  async function i(c, p = {}) {
    const g = await a.fetch(a.baseUrl + c, {
      headers: { "Content-Type": "application/json" },
      ...p
    }), k = await g.text();
    let d = null;
    try {
      d = k ? JSON.parse(k) : null;
    } catch {
      d = null;
    }
    if (!g.ok) {
      const y = (d == null ? void 0 : d.error) ?? t("errors.http", { status: g.status });
      throw new Error(y);
    }
    return d;
  }
  async function s(c) {
    var g;
    const p = await a.fetch(a.baseUrl + c);
    if (!p.ok) {
      const k = await p.text().catch(() => "");
      let d = t("errors.http", { status: p.status });
      try {
        d = ((g = JSON.parse(k)) == null ? void 0 : g.error) ?? d;
      } catch {
      }
      throw new Error(d);
    }
    return p.blob();
  }
  return {
    get: (c) => i(c),
    post: (c, p) => i(c, { method: "POST", body: JSON.stringify(p ?? {}) }),
    put: (c, p) => i(c, { method: "PUT", body: JSON.stringify(p ?? {}) }),
    del: (c) => i(c, { method: "DELETE" }),
    getBlob: s
  };
}
function _e() {
  return _a(tt());
}
function ga(a) {
  const i = Ke(a.query), s = new URLSearchParams();
  return i ? (s.set("query", a.query.trim()), a.tagFilter && s.set("tags", a.tagFilter)) : (a.tagFilter && s.set("tag", a.tagFilter), s.set("sort", a.sort), s.set("order", a.order)), s.set("offset", String((a.page - 1) * a.pageSize)), s.set("limit", String(a.pageSize)), s.toString();
}
function Ke(a) {
  return a.trim().length > 0;
}
const ya = new ca();
function ha(a) {
  const i = ya.parse(a, { async: !1 });
  return vt.sanitize(i);
}
function ba(a) {
  return vt.sanitize(a);
}
function at(a, i) {
  return a.get(`/api/memories/${encodeURIComponent(i)}`);
}
function ht(a, i) {
  return a.post("/api/memories", i);
}
function bt(a, i, s) {
  return a.put(`/api/memories/${encodeURIComponent(i)}`, s);
}
function ka(a, i) {
  return a.del(`/api/memories/${encodeURIComponent(i)}`);
}
function kt(a, i) {
  const s = i ? `?filter=${encodeURIComponent(i)}` : "";
  return a.get(`/api/tags${s}`);
}
function wa(a, i, s) {
  return a.post("/api/tags", { name: i, description: s });
}
function Ca(a, i, s, c) {
  const p = { description: c }, g = s.trim();
  return g && g !== i && (p.new_name = g), a.put(`/api/tags/${encodeURIComponent(i)}`, p);
}
function Ta(a, i, s) {
  return a.del(`/api/tags/${encodeURIComponent(i)}?mode=${s}`);
}
function Sa() {
  const a = _e(), { defaultPageSize: i } = tt(), s = _(""), c = _(""), p = _("updated_at"), g = _("desc"), k = _(1), d = _(i), y = _([]), P = _([]), S = _(0), m = _(!1), U = _([]), B = ve(() => Ke(s.value));
  function z() {
    return k.value = 1, R();
  }
  function A() {
    return ga({
      query: s.value,
      tagFilter: c.value,
      sort: p.value,
      order: g.value,
      page: k.value,
      pageSize: d.value
    });
  }
  let E = 0;
  async function R() {
    var Q;
    const T = ++E;
    m.value = !0;
    try {
      const G = A();
      if (Ke(s.value)) {
        const O = await a.get(`/api/memories?${G}`);
        if (T !== E) return;
        P.value = (O.results ?? []).map((I) => ({ ...I, snippet: ba(I.snippet) })), S.value = O.total_matches ?? 0;
      } else {
        const O = await a.get(`/api/memories?${G}`);
        if (T !== E) return;
        const I = Math.max(1, Math.ceil(O.total / d.value));
        if (((Q = O.memories) == null ? void 0 : Q.length) === 0 && O.total > 0 && k.value > I)
          return k.value = I, m.value = !1, R();
        y.value = O.memories ?? [], S.value = O.total ?? 0;
      }
    } finally {
      T === E && (m.value = !1);
    }
  }
  async function W() {
    try {
      const T = await kt(a);
      U.value = (T.tags ?? []).map((Q) => Q.name);
    } catch {
    }
  }
  async function N(T) {
    if (T.id) {
      const Q = await at(a, T.id), G = new Set(Q.tags), O = new Set(T.tags);
      await bt(a, T.id, {
        summary: T.summary,
        content: T.content,
        add_tags: [...O].filter((I) => !G.has(I)),
        remove_tags: [...G].filter((I) => !O.has(I))
      });
    } else
      await ht(a, { summary: T.summary, content: T.content, tags: T.tags });
    await Promise.all([R(), W()]);
  }
  async function Y(T) {
    await ka(a, T), await R();
  }
  return Me(() => {
    R().catch(() => {
    }), W();
  }), {
    query: s,
    tagFilter: c,
    sort: p,
    order: g,
    page: k,
    pageSize: d,
    rows: y,
    searchResults: P,
    total: S,
    loading: m,
    tagOptions: U,
    searching: B,
    onSearch: z,
    reload: R,
    loadTagOptions: W,
    saveMemory: N,
    removeMemory: Y
  };
}
function He(a) {
  const i = _(0);
  let s = null;
  function c(k) {
    s == null || s.disconnect(), s = null, !(!k || typeof ResizeObserver > "u") && (s = new ResizeObserver((d) => {
      var y;
      i.value = ((y = d[0]) == null ? void 0 : y.contentRect.width) ?? 0;
    }), s.observe(k));
  }
  Me(() => c(a.value)), Le(a, (k) => c(k)), Ut(() => s == null ? void 0 : s.disconnect());
  const p = ve(() => i.value > 0 && i.value < 960), g = ve(() => i.value > 0 && i.value < 720);
  return { width: i, compact: p, narrow: g };
}
const Ea = ["innerHTML"], wt = /* @__PURE__ */ ue({
  __name: "MarkdownView",
  props: {
    source: {}
  },
  setup(a) {
    const i = a, s = ve(() => ha(i.source));
    return (c, p) => (v(), D("div", {
      class: "md-body",
      innerHTML: s.value
    }, null, 8, Ea));
  }
}), Va = { class: "content-label" }, xa = /* @__PURE__ */ ue({
  __name: "MemoryEditorDialog",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    tagOptions: {},
    width: {}
  },
  emits: ["update:visible", "saved"],
  setup(a, { emit: i }) {
    const s = a, c = i, p = _e(), g = _(!1), k = _("edit"), d = _({ id: null, summary: "", content: "", tags: [] });
    let y = [];
    Le(
      () => s.visible,
      async (S) => {
        if (S)
          if (k.value = "edit", s.memoryId)
            try {
              const m = await at(p, s.memoryId);
              d.value = { id: m.id, summary: m.summary, content: m.content, tags: [...m.tags] }, y = [...m.tags];
            } catch (m) {
              X(m instanceof Error ? m.message : String(m)), c("update:visible", !1);
            }
          else
            d.value = { id: null, summary: "", content: "", tags: [] }, y = [];
      }
    );
    async function P() {
      g.value = !0;
      try {
        if (d.value.id) {
          const S = new Set(y), m = new Set(d.value.tags);
          await bt(p, d.value.id, {
            summary: d.value.summary,
            content: d.value.content,
            add_tags: [...m].filter((U) => !S.has(U)),
            remove_tags: [...S].filter((U) => !m.has(U))
          }), ne(t("editor.updated"));
        } else
          await ht(p, {
            summary: d.value.summary,
            content: d.value.content,
            tags: d.value.tags
          }), ne(t("editor.created"));
        c("update:visible", !1), c("saved");
      } catch (S) {
        X(S instanceof Error ? S.message : String(S));
      } finally {
        g.value = !1;
      }
    }
    return (S, m) => {
      const U = Be, B = Ge, z = Qe, A = Re, E = ut, R = dt, W = We, N = Pe, Y = Je;
      return v(), $(Y, {
        "model-value": a.visible,
        title: d.value.id ? e(t)("editor.editTitle") : e(t)("editor.createTitle"),
        width: a.width,
        "onUpdate:modelValue": m[5] || (m[5] = (T) => c("update:visible", T))
      }, {
        footer: n(() => [
          o(N, {
            onClick: m[4] || (m[4] = (T) => c("update:visible", !1))
          }, {
            default: n(() => [
              u(l(e(t)("common.cancel")), 1)
            ]),
            _: 1
          }),
          o(N, {
            type: "primary",
            loading: g.value,
            onClick: P
          }, {
            default: n(() => [
              u(l(e(t)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: n(() => [
          o(W, { "label-position": "top" }, {
            default: n(() => [
              o(B, {
                label: e(t)("editor.summaryLabel")
              }, {
                default: n(() => [
                  o(U, {
                    modelValue: d.value.summary,
                    "onUpdate:modelValue": m[0] || (m[0] = (T) => d.value.summary = T),
                    maxlength: "512",
                    "show-word-limit": "",
                    placeholder: e(t)("editor.summaryPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"]),
              o(B, null, {
                label: n(() => [
                  w("div", Va, [
                    w("span", null, l(e(t)("editor.contentLabel")), 1),
                    o(A, {
                      modelValue: k.value,
                      "onUpdate:modelValue": m[1] || (m[1] = (T) => k.value = T),
                      size: "small"
                    }, {
                      default: n(() => [
                        o(z, { value: "edit" }, {
                          default: n(() => [
                            u(l(e(t)("editor.tabEdit")), 1)
                          ]),
                          _: 1
                        }),
                        o(z, { value: "preview" }, {
                          default: n(() => [
                            u(l(e(t)("editor.tabPreview")), 1)
                          ]),
                          _: 1
                        })
                      ]),
                      _: 1
                    }, 8, ["modelValue"])
                  ])
                ]),
                default: n(() => [
                  k.value === "edit" ? (v(), $(U, {
                    key: 0,
                    modelValue: d.value.content,
                    "onUpdate:modelValue": m[2] || (m[2] = (T) => d.value.content = T),
                    type: "textarea",
                    rows: 12,
                    maxlength: "262144",
                    "show-word-limit": "",
                    placeholder: e(t)("editor.contentPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])) : (v(), $(wt, {
                    key: 1,
                    class: "content-preview",
                    source: d.value.content
                  }, null, 8, ["source"]))
                ]),
                _: 1
              }),
              o(B, {
                label: e(t)("editor.tagsLabel")
              }, {
                default: n(() => [
                  o(R, {
                    modelValue: d.value.tags,
                    "onUpdate:modelValue": m[3] || (m[3] = (T) => d.value.tags = T),
                    multiple: "",
                    filterable: "",
                    "allow-create": "",
                    "default-first-option": "",
                    placeholder: e(t)("editor.tagsPlaceholder"),
                    class: "tags-select"
                  }, {
                    default: n(() => [
                      (v(!0), D(se, null, de(a.tagOptions, (T) => (v(), $(E, {
                        key: T,
                        label: T,
                        value: T
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
}), ke = (a, i) => {
  const s = a.__vccOpts || a;
  for (const [c, p] of i)
    s[c] = p;
  return s;
}, $a = /* @__PURE__ */ ke(xa, [["__scopeId", "data-v-a0fec0dd"]]), Ma = { class: "detail-summary" }, Pa = { class: "detail-tags" }, Ua = { class: "detail-toolbar" }, Aa = {
  key: 1,
  class: "detail-content"
}, Da = /* @__PURE__ */ ue({
  __name: "MemoryDetailDrawer",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    size: {}
  },
  emits: ["update:visible"],
  setup(a, { emit: i }) {
    const s = a, c = i, p = _e(), g = _(null), k = _("rendered");
    return Le(
      () => [s.visible, s.memoryId],
      async ([d]) => {
        if (!(!d || !s.memoryId)) {
          k.value = "rendered";
          try {
            g.value = await at(p, s.memoryId);
          } catch (y) {
            X(y instanceof Error ? y.message : String(y)), c("update:visible", !1);
          }
        }
      }
    ), (d, y) => {
      var z;
      const P = Ne, S = Lt, m = Qe, U = Re, B = zt;
      return v(), $(B, {
        "model-value": a.visible,
        title: e(t)("drawer.title", { id: ((z = g.value) == null ? void 0 : z.id) ?? a.memoryId ?? "" }),
        size: a.size,
        "onUpdate:modelValue": y[1] || (y[1] = (A) => c("update:visible", A))
      }, {
        default: n(() => [
          g.value ? (v(), D(se, { key: 0 }, [
            w("h3", Ma, l(g.value.summary), 1),
            w("div", Pa, [
              (v(!0), D(se, null, de(g.value.tags, (A) => (v(), $(P, {
                key: A,
                size: "small",
                class: "am-tag"
              }, {
                default: n(() => [
                  u(l(A), 1)
                ]),
                _: 2
              }, 1024))), 128))
            ]),
            o(S),
            w("div", Ua, [
              o(U, {
                modelValue: k.value,
                "onUpdate:modelValue": y[0] || (y[0] = (A) => k.value = A),
                size: "small"
              }, {
                default: n(() => [
                  o(m, { value: "rendered" }, {
                    default: n(() => [
                      u(l(e(t)("drawer.rendered")), 1)
                    ]),
                    _: 1
                  }),
                  o(m, { value: "source" }, {
                    default: n(() => [
                      u(l(e(t)("drawer.source")), 1)
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["modelValue"])
            ]),
            k.value === "rendered" ? (v(), $(wt, {
              key: 0,
              source: g.value.content
            }, null, 8, ["source"])) : (v(), D("pre", Aa, l(g.value.content), 1))
          ], 64)) : J("", !0)
        ]),
        _: 1
      }, 8, ["model-value", "title", "size"]);
    };
  }
}), Ia = /* @__PURE__ */ ke(Da, [["__scopeId", "data-v-613d4260"]]), za = {
  key: 0,
  class: "am-panel-header"
}, La = { class: "am-heading" }, Ba = { class: "am-panel-title" }, Ra = { class: "am-toolbar" }, Na = { class: "am-summary" }, Oa = ["innerHTML"], Fa = {
  key: 4,
  class: "am-pager"
}, Ha = {
  key: 5,
  class: "am-pager"
}, qa = /* @__PURE__ */ ue({
  __name: "MemoriesPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(a, { expose: i }) {
    const s = a, {
      query: c,
      tagFilter: p,
      sort: g,
      order: k,
      page: d,
      pageSize: y,
      rows: P,
      searchResults: S,
      total: m,
      loading: U,
      tagOptions: B,
      searching: z,
      onSearch: A,
      reload: E,
      loadTagOptions: R,
      removeMemory: W
    } = Sa(), N = ve(() => {
      if (U.value || m.value !== 0) return "";
      if (z.value) return t("memories.searchEmpty");
      const H = p.value.trim();
      return H ? t("memories.tagEmpty", { tag: H }) : "";
    }), Y = _(null), { compact: T, narrow: Q } = He(Y), G = _(!1), O = _(null), I = _(!1), x = _(null);
    function C(H) {
      return H().catch((b) => X(b instanceof Error ? b.message : String(b)));
    }
    function Z() {
      O.value = null, G.value = !0;
    }
    function oe(H) {
      O.value = H, G.value = !0;
    }
    function j(H) {
      x.value = H, I.value = !0;
    }
    function F() {
      C(E), R();
    }
    function me(H) {
      const { prop: b, order: re } = H;
      re && (b === "updated_at" || b === "created_at" || b === "id") ? (g.value = b, k.value = re === "ascending" ? "asc" : "desc") : (g.value = "updated_at", k.value = "desc"), C(E);
    }
    async function le(H) {
      try {
        await ye.confirm(t("memories.deleteConfirm", { id: H.id }), t("memories.deleteTitle"), {
          type: "warning"
        });
      } catch {
        return;
      }
      try {
        await W(H.id), ne(t("memories.deleted"));
      } catch (b) {
        X(b instanceof Error ? b.message : String(b));
      }
    }
    return i({ refresh: () => C(E) }), (H, b) => {
      const re = Ue, ge = Oe, ee = Pe, we = Be, Ce = ut, M = dt, ie = mt, K = Ye, Te = Ne, Se = Xe, pe = Bt, Ae = pt;
      return v(), D("div", {
        ref_key: "rootRef",
        ref: Y,
        class: "am-panel"
      }, [
        a.showHeader ? (v(), D("div", za, [
          w("div", La, [
            w("h2", Ba, l(s.title ?? e(t)("memories.title")), 1),
            o(ge, {
              content: s.subtitle ?? e(t)("memories.subtitle"),
              placement: "top"
            }, {
              default: n(() => [
                o(re, { class: "am-info" }, {
                  default: n(() => [
                    o(e($e))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          o(ee, {
            type: "primary",
            icon: e(Ze),
            onClick: Z
          }, {
            default: n(() => [
              u(l(e(t)("memories.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : J("", !0),
        w("div", Ra, [
          o(we, {
            modelValue: e(c),
            "onUpdate:modelValue": b[1] || (b[1] = (h) => fe(c) ? c.value = h : null),
            placeholder: e(t)("memories.searchPlaceholder"),
            clearable: "",
            class: "search",
            onKeyup: b[2] || (b[2] = At((h) => C(e(A)), ["enter"])),
            onClear: b[3] || (b[3] = (h) => C(e(A)))
          }, {
            append: n(() => [
              o(ee, {
                icon: e(et),
                onClick: b[0] || (b[0] = (h) => C(e(A)))
              }, null, 8, ["icon"])
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          o(M, {
            modelValue: e(p),
            "onUpdate:modelValue": b[4] || (b[4] = (h) => fe(p) ? p.value = h : null),
            placeholder: e(t)("memories.tagFilter"),
            clearable: "",
            filterable: "",
            class: "tag-filter",
            onChange: b[5] || (b[5] = (h) => C(e(A)))
          }, {
            default: n(() => [
              (v(!0), D(se, null, de(e(B), (h) => (v(), $(Ce, {
                key: h,
                label: h,
                value: h
              }, null, 8, ["label", "value"]))), 128))
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"])
        ]),
        N.value ? (v(), $(ie, {
          key: 1,
          title: N.value,
          type: "info",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : J("", !0),
        e(z) ? he((v(), $(Se, {
          key: 2,
          data: e(S)
        }, {
          default: n(() => [
            o(K, {
              prop: "id",
              label: e(t)("memories.colId"),
              width: "80"
            }, null, 8, ["label"]),
            o(K, {
              label: e(t)("memories.colSummary")
            }, {
              default: n(({ row: h }) => [
                w("div", Na, l(h.summary), 1),
                w("div", {
                  class: "am-snippet",
                  innerHTML: h.snippet
                }, null, 8, Oa)
              ]),
              _: 1
            }, 8, ["label"]),
            o(K, {
              label: e(t)("memories.colTags"),
              "min-width": "150"
            }, {
              default: n(({ row: h }) => [
                (v(!0), D(se, null, de(h.tags, (te) => (v(), $(Te, {
                  key: te,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: n(() => [
                    u(l(te), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            o(K, {
              prop: "score",
              label: e(t)("memories.colScore"),
              width: "80",
              sortable: ""
            }, null, 8, ["label"]),
            o(K, {
              label: e(t)("memories.colUpdatedAt"),
              width: "170"
            }, {
              default: n(({ row: h }) => [
                u(l(e(be)(h.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            o(K, {
              label: e(t)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: n(({ row: h }) => [
                o(ee, {
                  link: "",
                  type: "primary",
                  onClick: (te) => j(h.id)
                }, {
                  default: n(() => [
                    u(l(e(t)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                o(ee, {
                  link: "",
                  type: "primary",
                  onClick: (te) => oe(h.id)
                }, {
                  default: n(() => [
                    u(l(e(t)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                o(ee, {
                  link: "",
                  type: "danger",
                  onClick: (te) => le(h)
                }, {
                  default: n(() => [
                    u(l(e(t)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [Ae, e(U)]
        ]) : he((v(), $(Se, {
          key: 3,
          data: e(P),
          "default-sort": { prop: e(g), order: e(k) === "asc" ? "ascending" : "descending" },
          onSortChange: me
        }, {
          default: n(() => [
            o(K, {
              prop: "id",
              label: e(t)("memories.colId"),
              width: "80",
              sortable: "custom"
            }, null, 8, ["label"]),
            o(K, {
              prop: "summary",
              label: e(t)("memories.colSummary"),
              "min-width": "180",
              "show-overflow-tooltip": ""
            }, null, 8, ["label"]),
            o(K, {
              label: e(t)("memories.colTags"),
              "min-width": "150"
            }, {
              default: n(({ row: h }) => [
                (v(!0), D(se, null, de(h.tags, (te) => (v(), $(Te, {
                  key: te,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: n(() => [
                    u(l(te), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            e(T) ? J("", !0) : (v(), $(K, {
              key: 0,
              prop: "created_at",
              label: e(t)("memories.colCreatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: n(({ row: h }) => [
                u(l(e(be)(h.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label"])),
            o(K, {
              prop: "updated_at",
              label: e(t)("memories.colUpdatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: n(({ row: h }) => [
                u(l(e(be)(h.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            o(K, {
              label: e(t)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: n(({ row: h }) => [
                o(ee, {
                  link: "",
                  type: "primary",
                  onClick: (te) => j(h.id)
                }, {
                  default: n(() => [
                    u(l(e(t)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                o(ee, {
                  link: "",
                  type: "primary",
                  onClick: (te) => oe(h.id)
                }, {
                  default: n(() => [
                    u(l(e(t)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                o(ee, {
                  link: "",
                  type: "danger",
                  onClick: (te) => le(h)
                }, {
                  default: n(() => [
                    u(l(e(t)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data", "default-sort"])), [
          [Ae, e(U)]
        ]),
        e(z) ? (v(), D("div", Ha, [
          o(pe, {
            "current-page": e(d),
            "onUpdate:currentPage": b[10] || (b[10] = (h) => fe(d) ? d.value = h : null),
            "page-size": e(y),
            "onUpdate:pageSize": b[11] || (b[11] = (h) => fe(y) ? y.value = h : null),
            total: e(m),
            "page-sizes": [10, 20, 50],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: b[12] || (b[12] = (h) => C(e(E))),
            onSizeChange: b[13] || (b[13] = (h) => C(e(E)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])) : (v(), D("div", Fa, [
          o(pe, {
            "current-page": e(d),
            "onUpdate:currentPage": b[6] || (b[6] = (h) => fe(d) ? d.value = h : null),
            "page-size": e(y),
            "onUpdate:pageSize": b[7] || (b[7] = (h) => fe(y) ? y.value = h : null),
            total: e(m),
            "page-sizes": [20, 50, 100, 200],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: b[8] || (b[8] = (h) => C(e(E))),
            onSizeChange: b[9] || (b[9] = (h) => C(e(E)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])),
        o($a, {
          visible: G.value,
          "onUpdate:visible": b[14] || (b[14] = (h) => G.value = h),
          "memory-id": O.value,
          "tag-options": e(B),
          width: e(Q) ? "96%" : "640px",
          onSaved: F
        }, null, 8, ["visible", "memory-id", "tag-options", "width"]),
        o(Ia, {
          visible: I.value,
          "onUpdate:visible": b[15] || (b[15] = (h) => I.value = h),
          "memory-id": x.value,
          size: e(Q) ? "100%" : "45%"
        }, null, 8, ["visible", "memory-id", "size"])
      ], 512);
    };
  }
}), ja = /* @__PURE__ */ ke(qa, [["__scopeId", "data-v-5e7dad8e"]]);
function Ka() {
  const a = _e(), i = _([]), s = _(!1), c = _("");
  async function p() {
    s.value = !0;
    try {
      const y = await kt(a, c.value.trim() || void 0);
      i.value = y.tags ?? [];
    } finally {
      s.value = !1;
    }
  }
  async function g(y, P) {
    await wa(a, y, P), await p();
  }
  async function k(y, P, S) {
    await Ca(a, y, P, S), await p();
  }
  async function d(y, P) {
    await Ta(a, y, P), await p();
  }
  return Me(() => {
    p().catch(() => {
    });
  }), { rows: i, loading: s, filter: c, reload: p, create: g, rename: k, remove: d };
}
const Ja = {
  key: 0,
  class: "am-panel-header"
}, Wa = { class: "am-heading" }, Ga = { class: "am-panel-title" }, Qa = { class: "delete-body" }, Xa = /* @__PURE__ */ ue({
  __name: "TagsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(a, { expose: i }) {
    const s = a, { rows: c, loading: p, filter: g, reload: k, create: d, rename: y, remove: P } = Ka();
    let S = null;
    function m() {
      S && clearTimeout(S), S = setTimeout(() => {
        S = null, k().catch((x) => X(x instanceof Error ? x.message : String(x)));
      }, 300);
    }
    const U = _(null), { narrow: B } = He(U), z = _(!1), A = _(!1), E = ct({ oldName: null, name: "", description: "" }), R = _(!1), W = _("detach"), N = _(null);
    function Y(x) {
      return (C, Z) => (C[x] ?? 0) - (Z[x] ?? 0);
    }
    function T() {
      Object.assign(E, { oldName: null, name: "", description: "" }), A.value = !0;
    }
    function Q(x) {
      Object.assign(E, { oldName: x.name, name: x.name, description: x.description ?? "" }), A.value = !0;
    }
    async function G() {
      z.value = !0;
      try {
        E.oldName ? (await y(E.oldName, E.name, E.description), ne(t("tags.saved"))) : (await d(E.name, E.description), ne(t("tags.created"))), A.value = !1;
      } catch (x) {
        X(x instanceof Error ? x.message : String(x));
      } finally {
        z.value = !1;
      }
    }
    function O(x) {
      N.value = x, W.value = "detach", R.value = !0;
    }
    async function I() {
      var x, C;
      if (W.value === "purge")
        try {
          await ye.confirm(
            t("tags.purgeConfirm", { name: (x = N.value) == null ? void 0 : x.name, count: ((C = N.value) == null ? void 0 : C.memory_count) ?? 0 }),
            t("tags.purgeConfirmTitle"),
            { type: "error", confirmButtonText: t("tags.purgeButton") }
          );
        } catch {
          return;
        }
      if (N.value) {
        z.value = !0;
        try {
          await P(N.value.name, W.value), ne(t("tags.deleted")), R.value = !1;
        } catch (Z) {
          X(Z instanceof Error ? Z.message : String(Z));
        } finally {
          z.value = !1;
        }
      }
    }
    return i({
      refresh: () => k().catch((x) => X(x instanceof Error ? x.message : String(x)))
    }), (x, C) => {
      const Z = Ue, oe = Oe, j = Pe, F = Be, me = Ne, le = Ye, H = Xe, b = Ge, re = We, ge = Je, ee = Rt, we = Re, Ce = pt;
      return v(), D("div", {
        ref_key: "rootRef",
        ref: U,
        class: "am-panel"
      }, [
        a.showHeader ? (v(), D("div", Ja, [
          w("div", Wa, [
            w("h2", Ga, l(s.title ?? e(t)("tags.title")), 1),
            o(oe, {
              content: s.subtitle ?? e(t)("tags.subtitle"),
              placement: "top"
            }, {
              default: n(() => [
                o(Z, { class: "am-info" }, {
                  default: n(() => [
                    o(e($e))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          o(j, {
            type: "primary",
            icon: e(Ze),
            onClick: T
          }, {
            default: n(() => [
              u(l(e(t)("tags.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : J("", !0),
        o(F, {
          modelValue: e(g),
          "onUpdate:modelValue": C[0] || (C[0] = (M) => fe(g) ? g.value = M : null),
          class: "am-tag-filter",
          placeholder: e(t)("tags.filterPlaceholder"),
          clearable: "",
          "prefix-icon": e(et),
          onInput: m,
          onClear: m
        }, null, 8, ["modelValue", "placeholder", "prefix-icon"]),
        he((v(), $(H, { data: e(c) }, {
          default: n(() => [
            o(le, {
              prop: "name",
              label: e(t)("tags.colName"),
              "min-width": "140",
              sortable: ""
            }, {
              default: n(({ row: M }) => [
                o(me, null, {
                  default: n(() => [
                    u(l(M.name), 1)
                  ]),
                  _: 2
                }, 1024)
              ]),
              _: 1
            }, 8, ["label"]),
            o(le, {
              prop: "description",
              label: e(t)("tags.colDescription"),
              "min-width": "150",
              "show-overflow-tooltip": ""
            }, {
              default: n(({ row: M }) => [
                u(l(M.description || "—"), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            o(le, {
              prop: "memory_count",
              label: e(t)("tags.colMemoryCount"),
              width: "90",
              sortable: ""
            }, null, 8, ["label"]),
            o(le, {
              prop: "last_used_at",
              label: e(t)("tags.colLastUsed"),
              width: "170",
              sortable: "",
              "sort-method": Y("last_used_at")
            }, {
              default: n(({ row: M }) => [
                u(l(e(be)(M.last_used_at)), 1)
              ]),
              _: 1
            }, 8, ["label", "sort-method"]),
            o(le, {
              prop: "created_at",
              label: e(t)("tags.colCreatedAt"),
              width: "170",
              sortable: "",
              "sort-method": Y("created_at")
            }, {
              default: n(({ row: M }) => [
                u(l(e(be)(M.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label", "sort-method"]),
            o(le, {
              label: e(t)("memories.colActions"),
              width: "150",
              fixed: "right"
            }, {
              default: n(({ row: M }) => [
                o(j, {
                  link: "",
                  type: "primary",
                  onClick: (ie) => Q(M)
                }, {
                  default: n(() => [
                    u(l(e(t)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                o(j, {
                  link: "",
                  type: "danger",
                  onClick: (ie) => O(M)
                }, {
                  default: n(() => [
                    u(l(e(t)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [Ce, e(p)]
        ]),
        o(ge, {
          modelValue: A.value,
          "onUpdate:modelValue": C[4] || (C[4] = (M) => A.value = M),
          title: E.oldName ? e(t)("tags.editTitle") : e(t)("tags.createTitle"),
          width: e(B) ? "96%" : "480px"
        }, {
          footer: n(() => [
            o(j, {
              onClick: C[3] || (C[3] = (M) => A.value = !1)
            }, {
              default: n(() => [
                u(l(e(t)("common.cancel")), 1)
              ]),
              _: 1
            }),
            o(j, {
              type: "primary",
              loading: z.value,
              onClick: G
            }, {
              default: n(() => [
                u(l(e(t)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: n(() => [
            o(re, { "label-position": "top" }, {
              default: n(() => [
                o(b, {
                  label: e(t)("tags.nameLabel")
                }, {
                  default: n(() => [
                    o(F, {
                      modelValue: E.name,
                      "onUpdate:modelValue": C[1] || (C[1] = (M) => E.name = M),
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(t)("tags.namePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                o(b, {
                  label: e(t)("tags.descLabel")
                }, {
                  default: n(() => [
                    o(F, {
                      modelValue: E.description,
                      "onUpdate:modelValue": C[2] || (C[2] = (M) => E.description = M),
                      type: "textarea",
                      rows: 3,
                      maxlength: "512",
                      "show-word-limit": "",
                      placeholder: e(t)("tags.descPlaceholder")
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
        o(ge, {
          modelValue: R.value,
          "onUpdate:modelValue": C[7] || (C[7] = (M) => R.value = M),
          title: e(t)("tags.deleteTitle"),
          width: e(B) ? "96%" : "480px"
        }, {
          footer: n(() => [
            o(j, {
              onClick: C[6] || (C[6] = (M) => R.value = !1)
            }, {
              default: n(() => [
                u(l(e(t)("common.cancel")), 1)
              ]),
              _: 1
            }),
            o(j, {
              type: "danger",
              loading: z.value,
              onClick: I
            }, {
              default: n(() => [
                u(l(e(t)("common.delete")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: n(() => {
            var M;
            return [
              w("p", Qa, [
                u(l(e(t)("tags.deleteBefore")) + " ", 1),
                o(me, null, {
                  default: n(() => {
                    var ie;
                    return [
                      u(l((ie = N.value) == null ? void 0 : ie.name), 1)
                    ];
                  }),
                  _: 1
                }),
                u(" " + l(e(t)("tags.deleteMiddle")) + " ", 1),
                w("b", null, l((M = N.value) == null ? void 0 : M.memory_count), 1),
                u(" " + l(e(t)("tags.deleteAfter")), 1)
              ]),
              o(we, {
                modelValue: W.value,
                "onUpdate:modelValue": C[5] || (C[5] = (ie) => W.value = ie)
              }, {
                default: n(() => [
                  o(ee, { value: "detach" }, {
                    default: n(() => [
                      u(l(e(t)("tags.detach")), 1)
                    ]),
                    _: 1
                  }),
                  o(ee, { value: "purge" }, {
                    default: n(() => [
                      u(l(e(t)("tags.purge")), 1)
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
}), Ya = /* @__PURE__ */ ke(Xa, [["__scopeId", "data-v-c8aeea57"]]);
function Za(a) {
  return a.get("/api/stats");
}
function eo(a) {
  return a.get("/health");
}
function to(a) {
  return a.get("/api/doctor");
}
function ao(a) {
  return a.getBlob("/api/export");
}
function oo(a, i) {
  return a.post("/api/import", i);
}
function no() {
  const a = _e(), i = _({}), s = _(""), c = ve(() => va(i.value.file_size));
  async function p() {
    i.value = await Za(a), i.value.version = s.value;
  }
  return Me(async () => {
    try {
      s.value = (await eo(a)).version ?? "";
    } catch {
    }
    await p().catch(() => {
    });
  }), {
    stats: i,
    version: s,
    sizeText: c,
    reload: p
  };
}
const lo = {
  key: 0,
  class: "am-panel-header"
}, so = { class: "am-heading" }, io = { class: "am-panel-title" }, ro = /* @__PURE__ */ ue({
  __name: "OpsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(a, { expose: i }) {
    const s = a, { stats: c, version: p, sizeText: g, reload: k } = no(), d = _(null), { narrow: y } = He(d);
    function P(S) {
      return S().catch((m) => X(m instanceof Error ? m.message : String(m)));
    }
    return i({ refresh: () => P(k) }), (S, m) => {
      const U = Ue, B = Oe, z = Pe, A = Ft, E = ft, R = Ot, W = Nt, N = qt, Y = Ht;
      return v(), D("div", {
        ref_key: "rootRef",
        ref: d,
        class: "am-panel"
      }, [
        a.showHeader ? (v(), D("div", lo, [
          w("div", so, [
            w("h2", io, l(s.title ?? e(t)("ops.title")), 1),
            o(B, {
              content: s.subtitle ?? e(t)("ops.subtitle"),
              placement: "top"
            }, {
              default: n(() => [
                o(U, { class: "am-info" }, {
                  default: n(() => [
                    o(e($e))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          o(z, {
            icon: e(ta),
            onClick: m[0] || (m[0] = (T) => P(e(k)))
          }, {
            default: n(() => [
              u(l(e(t)("ops.refresh")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : J("", !0),
        o(W, { gutter: 14 }, {
          default: n(() => [
            o(R, {
              span: e(y) ? 12 : 8
            }, {
              default: n(() => [
                o(E, { shadow: "never" }, {
                  default: n(() => [
                    o(A, {
                      title: e(t)("ops.statMemories"),
                      value: e(c).memories ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            o(R, {
              span: e(y) ? 12 : 8
            }, {
              default: n(() => [
                o(E, { shadow: "never" }, {
                  default: n(() => [
                    o(A, {
                      title: e(t)("ops.statTags"),
                      value: e(c).tags ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            o(R, {
              span: e(y) ? 12 : 8
            }, {
              default: n(() => [
                o(E, { shadow: "never" }, {
                  default: n(() => [
                    o(A, {
                      title: e(t)("ops.statSize"),
                      value: e(g)
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
        o(E, { shadow: "never" }, {
          header: n(() => [
            u(l(e(t)("ops.dbCard")), 1)
          ]),
          default: n(() => [
            o(Y, {
              column: e(y) ? 1 : 2,
              border: ""
            }, {
              default: n(() => [
                o(N, {
                  label: e(t)("ops.version")
                }, {
                  default: n(() => [
                    u(l(e(c).version ?? e(p)), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                o(N, {
                  label: e(t)("ops.schemaVersion")
                }, {
                  default: n(() => [
                    u(l(e(c).schema_version ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                o(N, {
                  label: e(t)("ops.path"),
                  span: e(y) ? 1 : 2
                }, {
                  default: n(() => [
                    u(l(e(c).path ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label", "span"])
              ]),
              _: 1
            }, 8, ["column"])
          ]),
          _: 1
        })
      ], 512);
    };
  }
}), co = { class: "memory-ui" }, uo = { class: "brand" }, mo = { class: "brand-mark" }, po = { class: "aside-footer" }, fo = /* @__PURE__ */ ue({
  __name: "MemoryAdmin",
  props: {
    layout: { default: "sidebar" },
    title: { default: "Agent Memory" }
  },
  setup(a) {
    const i = _("memories");
    return (s, c) => {
      const p = Ue, g = Wt, k = Jt, d = Kt, y = Xt, P = Qt, S = Gt, m = jt;
      return v(), D("div", co, [
        o(m, { class: "layout" }, {
          default: n(() => [
            a.layout === "sidebar" ? (v(), $(d, {
              key: 0,
              width: "200px",
              class: "aside"
            }, {
              default: n(() => [
                w("div", uo, [
                  w("span", mo, [
                    o(p, { size: 16 }, {
                      default: n(() => [
                        o(e(aa))
                      ]),
                      _: 1
                    })
                  ]),
                  w("span", null, l(a.title), 1)
                ]),
                o(k, {
                  "default-active": i.value,
                  class: "menu",
                  onSelect: c[0] || (c[0] = (U) => i.value = U)
                }, {
                  default: n(() => [
                    o(g, { index: "memories" }, {
                      default: n(() => [
                        o(p, null, {
                          default: n(() => [
                            o(e(oa))
                          ]),
                          _: 1
                        }),
                        w("span", null, l(e(t)("nav.memories")), 1)
                      ]),
                      _: 1
                    }),
                    o(g, { index: "tags" }, {
                      default: n(() => [
                        o(p, null, {
                          default: n(() => [
                            o(e(na))
                          ]),
                          _: 1
                        }),
                        w("span", null, l(e(t)("nav.tags")), 1)
                      ]),
                      _: 1
                    }),
                    o(g, { index: "ops" }, {
                      default: n(() => [
                        o(p, null, {
                          default: n(() => [
                            o(e(la))
                          ]),
                          _: 1
                        }),
                        w("span", null, l(e(t)("nav.ops")), 1)
                      ]),
                      _: 1
                    })
                  ]),
                  _: 1
                }, 8, ["default-active"]),
                w("div", po, [
                  Dt(s.$slots, "footer", {}, void 0, !0)
                ])
              ]),
              _: 3
            })) : J("", !0),
            o(S, { class: "main" }, {
              default: n(() => [
                a.layout === "tabs" ? (v(), $(P, {
                  key: 0,
                  modelValue: i.value,
                  "onUpdate:modelValue": c[1] || (c[1] = (U) => i.value = U),
                  class: "tabs-bar"
                }, {
                  default: n(() => [
                    o(y, {
                      label: e(t)("nav.memories"),
                      name: "memories"
                    }, null, 8, ["label"]),
                    o(y, {
                      label: e(t)("nav.tags"),
                      name: "tags"
                    }, null, 8, ["label"]),
                    o(y, {
                      label: e(t)("nav.ops"),
                      name: "ops"
                    }, null, 8, ["label"])
                  ]),
                  _: 1
                }, 8, ["modelValue"])) : J("", !0),
                he(o(ja, null, null, 512), [
                  [je, i.value === "memories"]
                ]),
                he(o(Ya, null, null, 512), [
                  [je, i.value === "tags"]
                ]),
                he(o(ro, null, null, 512), [
                  [je, i.value === "ops"]
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
}), Wo = /* @__PURE__ */ ke(fo, [["__scopeId", "data-v-38c91753"]]);
function vo() {
  const a = _e(), i = _({ ok: !0, issues: [] }), s = _(!1), c = _(!1), p = _(!1), g = _(!1);
  async function k() {
    c.value = !0;
    try {
      i.value = await to(a), s.value = !0;
    } finally {
      c.value = !1;
    }
  }
  async function d() {
    p.value = !0;
    try {
      const P = await ao(a), S = URL.createObjectURL(P), m = document.createElement("a");
      m.href = S, m.download = "agent-memory-export.json", m.click(), URL.revokeObjectURL(S);
    } finally {
      p.value = !1;
    }
  }
  async function y(P) {
    g.value = !0;
    try {
      const S = await P.text();
      let m;
      try {
        m = JSON.parse(S);
      } catch {
        throw new Error(t("errors.invalidBackup"));
      }
      return await oo(a, m);
    } finally {
      g.value = !1;
    }
  }
  return {
    doctor: i,
    doctorRan: s,
    doctorLoading: c,
    exporting: p,
    importing: g,
    runDoctor: k,
    exportData: d,
    importFile: y
  };
}
const _o = {
  key: 0,
  class: "am-panel-header"
}, go = { class: "am-heading" }, yo = { class: "am-panel-title" }, ho = { class: "auth-row" }, bo = { class: "auth-text" }, ko = { class: "auth-label" }, wo = { class: "auth-hint" }, Co = { class: "token-cell" }, To = { class: "token-text" }, So = {
  key: 0,
  class: "muted"
}, Eo = { class: "field-hint" }, Vo = {
  key: 0,
  class: "default-view"
}, xo = { class: "default-text" }, $o = { class: "field-hint" }, Mo = { class: "save-row" }, Po = { class: "actions" }, Uo = { class: "card-header" }, Ao = {
  key: 2,
  class: "issues"
}, Do = { class: "presets" }, Io = { class: "presets-label" }, zo = { class: "caps" }, Lo = { class: "new-token" }, Bo = /* @__PURE__ */ ue({
  __name: "AdminPanel",
  props: {
    who: { default: null },
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(a, { expose: i }) {
    const s = a, c = _e(), p = tt(), g = [
      { key: "read", labelKey: "capRead" },
      { key: "create", labelKey: "capCreate" },
      { key: "update", labelKey: "capUpdate" },
      { key: "delete", labelKey: "capDelete" },
      { key: "tag_manage", labelKey: "capTagManage" },
      { key: "admin", labelKey: "capAdmin" }
    ], k = _([]), d = _(""), y = _(""), P = _(""), S = _(!1), m = _(!1), U = _(!1), B = ve(() => {
      var f;
      return !!s.who && ((f = s.who.permissions) == null ? void 0 : f.admin) === !0;
    }), { doctor: z, doctorRan: A, doctorLoading: E, exporting: R, importing: W, runDoctor: N, exportData: Y, importFile: T } = vo(), Q = _(null), { compact: G } = He(Q), O = _(null);
    async function I(f, r) {
      try {
        return await f();
      } catch (L) {
        X(L instanceof Error ? L.message : String(L));
        return;
      }
    }
    async function x() {
      await I(async () => {
        const [f, r] = await Promise.all([
          c.get("/api/identities"),
          c.get("/api/settings")
        ]);
        k.value = Array.isArray(f == null ? void 0 : f.identities) ? f.identities : [], d.value = (r == null ? void 0 : r.instructions) ?? "", y.value = (r == null ? void 0 : r.conventions) ?? "", P.value = (r == null ? void 0 : r.default_instructions) ?? "", m.value = (r == null ? void 0 : r.auth_required) === !0;
      });
    }
    Me(() => {
      B.value && x();
    }), Le(B, (f) => {
      f && x();
    }), i({
      refresh: () => {
        B.value && I(x);
      }
    });
    const C = _(!1), Z = _(!1), oe = _(null), j = _("custom"), F = ct({
      name: "",
      caps: me()
    });
    function me() {
      return Object.fromEntries(g.map((f) => [f.key, !1]));
    }
    const le = {
      admin: g.map((f) => f.key),
      member: ["read", "create", "update", "tag_manage"],
      viewer: ["read"]
    };
    function H() {
      if (j.value === "custom") return;
      const f = new Set(le[j.value] ?? []);
      for (const r of g) F.caps[r.key] = f.has(r.key);
    }
    function b() {
      j.value = "custom";
    }
    function re() {
      oe.value = null, F.name = "", Object.assign(F.caps, me()), j.value = "member", H(), C.value = !0;
    }
    function ge(f) {
      var r;
      oe.value = f, F.name = f.name;
      for (const L of g) F.caps[L.key] = ((r = f.permissions) == null ? void 0 : r[L.key]) === !0;
      j.value = "custom", C.value = !0;
    }
    async function ee() {
      Z.value = !0;
      try {
        if (oe.value)
          await I(
            () => c.put(`/api/identities/${encodeURIComponent(oe.value.name)}`, {
              permissions: { ...F.caps }
            })
          ), ne(t("access.saved"));
        else {
          const f = await I(
            () => c.post("/api/identities", {
              name: F.name,
              permissions: { ...F.caps }
            })
          );
          f != null && f.token && h(t("access.createTitle"), F.name.trim(), f.token);
        }
        C.value = !1, await x();
      } finally {
        Z.value = !1;
      }
    }
    async function we(f) {
      try {
        await ye.confirm(t("access.deleteConfirm", { name: f.name }), t("access.deleteTitle"), {
          type: "warning",
          confirmButtonText: t("common.delete"),
          cancelButtonText: t("common.cancel")
        });
      } catch {
        return;
      }
      await I(() => c.del(`/api/identities/${encodeURIComponent(f.name)}`)) !== void 0 && (ne(t("access.deleted")), await x());
    }
    async function Ce() {
      var r;
      const f = !m.value;
      if (f && !k.value.some((L) => {
        var ce;
        return ((ce = L.permissions) == null ? void 0 : ce.admin) === !0;
      }))
        return ye.alert(t("access.enableBlocked"), t("access.authTitle"), {
          type: "warning",
          confirmButtonText: t("common.ok")
        }).catch(() => {
        }), !1;
      try {
        await ye.confirm(
          t(f ? "access.authEnableConfirm" : "access.authDisableConfirm"),
          t("access.authTitle"),
          {
            type: "warning",
            confirmButtonText: t("common.save"),
            cancelButtonText: t("common.cancel")
          }
        );
      } catch {
        return !1;
      }
      U.value = !0;
      try {
        return await I(() => c.put("/api/settings", { auth_required: f })) !== void 0 && ((r = p.onAuthChanged) == null || r.call(p, f)), !0;
      } finally {
        U.value = !1;
      }
    }
    async function M() {
      S.value = !0;
      try {
        await I(
          () => c.put("/api/settings", {
            instructions: d.value,
            conventions: y.value
          })
        ), ne(t("access.saved"));
      } finally {
        S.value = !1;
      }
    }
    function ie(f) {
      var ce;
      const r = f.target, L = (ce = r.files) == null ? void 0 : ce[0];
      r.value = "", L && T(L).then((q) => {
        ne(t("access.imported", { memories: q.imported_memories, tags: q.imported_tags }));
      }).catch((q) => {
        X(q instanceof Error ? q.message : String(q));
      });
    }
    const K = _(!1), Te = _(""), Se = _(""), pe = _("");
    function Ae(f) {
      return f ? `…${f}` : "—";
    }
    function h(f, r, L) {
      Te.value = f, Se.value = r, pe.value = L, K.value = !0;
    }
    async function te() {
      const f = Se.value;
      if (f)
        try {
          await p.onIdentityToken(f, pe.value), K.value = !1, ne(t("access.savedToBrowser", { name: f }));
        } catch (r) {
          X(r instanceof Error ? r.message : String(r));
        }
    }
    async function Ct(f) {
      try {
        await ye.confirm(t("access.resetConfirm", { name: f.name }), t("access.resetTitle"), {
          type: "warning",
          confirmButtonText: t("access.resetToken"),
          cancelButtonText: t("common.cancel")
        });
      } catch {
        return;
      }
      const r = await I(
        () => c.post(`/api/identities/${encodeURIComponent(f.name)}/token-reset`, {})
      );
      r !== void 0 && (r != null && r.token && h(t("access.resetTitle"), f.name, r.token), await x());
    }
    function ot(f) {
      return g.map((r) => r.key).filter((r) => {
        var L;
        return ((L = f.permissions) == null ? void 0 : L[r]) === !0;
      });
    }
    function nt(f) {
      const r = g.find((L) => L.key === f);
      return r ? t(`access.${r.labelKey}`) : f;
    }
    async function Tt(f) {
      try {
        await navigator.clipboard.writeText(f), ne(t("access.copied"));
      } catch {
        X(f);
      }
    }
    return (f, r) => {
      const L = Ue, ce = Oe, q = Pe, Ee = mt, St = Yt, Ve = ft, lt = Zt, xe = Ye, Et = Ne, Vt = Xe, qe = Be, De = Ge, st = We, Ie = Qe, xt = Re, $t = ea, it = Je;
      return v(), D("div", {
        ref_key: "rootRef",
        ref: Q,
        class: "am-panel admin-panel"
      }, [
        a.showHeader ? (v(), D("div", _o, [
          w("div", go, [
            w("h2", yo, l(s.title ?? e(t)("access.title")), 1),
            o(ce, {
              content: s.subtitle ?? e(t)("access.subtitle"),
              placement: "top"
            }, {
              default: n(() => [
                o(L, { class: "am-info" }, {
                  default: n(() => [
                    o(e($e))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          B.value ? (v(), $(q, {
            key: 0,
            type: "primary",
            icon: e(Ze),
            onClick: re
          }, {
            default: n(() => [
              u(l(e(t)("access.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])) : J("", !0)
        ])) : J("", !0),
        a.who ? a.who.mode === "open" ? (v(), $(Ee, {
          key: 2,
          type: "warning",
          title: e(t)("access.openMode"),
          closable: !1
        }, null, 8, ["title"])) : B.value ? J("", !0) : (v(), $(Ee, {
          key: 3,
          type: "info",
          title: e(t)("access.needAdmin"),
          closable: !1
        }, null, 8, ["title"])) : (v(), $(Ee, {
          key: 1,
          type: "info",
          title: e(t)("access.needAdmin"),
          closable: !1
        }, null, 8, ["title"])),
        a.who && B.value ? (v(), D(se, { key: 4 }, [
          o(Ve, { shadow: "never" }, {
            default: n(() => [
              w("div", ho, [
                w("div", bo, [
                  w("span", ko, l(e(t)("access.authTitle")), 1),
                  w("span", wo, l(e(t)("access.authHint")), 1)
                ]),
                o(St, {
                  modelValue: m.value,
                  "onUpdate:modelValue": r[0] || (r[0] = (V) => m.value = V),
                  "before-change": Ce,
                  loading: U.value
                }, null, 8, ["modelValue", "loading"])
              ])
            ]),
            _: 1
          }),
          o(Ve, { shadow: "never" }, {
            default: n(() => [
              k.value.length === 0 ? (v(), $(lt, {
                key: 0,
                description: e(t)("access.empty")
              }, null, 8, ["description"])) : (v(), $(Vt, {
                key: 1,
                data: k.value
              }, {
                default: n(() => [
                  o(xe, {
                    prop: "name",
                    label: e(t)("access.colName"),
                    "min-width": "120"
                  }, null, 8, ["label"]),
                  o(xe, {
                    label: e(t)("access.colToken"),
                    "min-width": "200"
                  }, {
                    default: n(({ row: V }) => [
                      w("div", Co, [
                        w("code", To, l(Ae(V.token_hint)), 1),
                        o(q, {
                          link: "",
                          type: "primary",
                          onClick: (ae) => Ct(V)
                        }, {
                          default: n(() => [
                            u(l(e(t)("access.resetToken")), 1)
                          ]),
                          _: 1
                        }, 8, ["onClick"])
                      ])
                    ]),
                    _: 1
                  }, 8, ["label"]),
                  o(xe, {
                    label: e(t)("access.colPermissions"),
                    "min-width": "240"
                  }, {
                    default: n(({ row: V }) => [
                      (v(!0), D(se, null, de(ot(V), (ae) => (v(), $(Et, {
                        key: ae,
                        size: "small",
                        class: "cap-tag",
                        type: ae === "admin" ? "danger" : "info"
                      }, {
                        default: n(() => [
                          u(l(nt(ae)), 1)
                        ]),
                        _: 2
                      }, 1032, ["type"]))), 128)),
                      ot(V).length === 0 ? (v(), D("span", So, "—")) : J("", !0)
                    ]),
                    _: 1
                  }, 8, ["label"]),
                  e(G) ? J("", !0) : (v(), $(xe, {
                    key: 0,
                    label: e(t)("access.colCreatedAt"),
                    width: "170"
                  }, {
                    default: n(({ row: V }) => [
                      u(l(e(be)(V.created_at)), 1)
                    ]),
                    _: 1
                  }, 8, ["label"])),
                  o(xe, {
                    label: e(t)("access.colActions"),
                    width: "140",
                    fixed: "right"
                  }, {
                    default: n(({ row: V }) => [
                      o(q, {
                        link: "",
                        type: "primary",
                        onClick: (ae) => ge(V)
                      }, {
                        default: n(() => [
                          u(l(e(t)("common.edit")), 1)
                        ]),
                        _: 1
                      }, 8, ["onClick"]),
                      o(q, {
                        link: "",
                        type: "danger",
                        onClick: (ae) => we(V)
                      }, {
                        default: n(() => [
                          u(l(e(t)("common.delete")), 1)
                        ]),
                        _: 1
                      }, 8, ["onClick"])
                    ]),
                    _: 1
                  }, 8, ["label"])
                ]),
                _: 1
              }, 8, ["data"]))
            ]),
            _: 1
          }),
          o(Ve, { shadow: "never" }, {
            header: n(() => [
              u(l(e(t)("access.settingsTitle")), 1)
            ]),
            default: n(() => [
              o(st, {
                "label-position": "top",
                onSubmit: r[3] || (r[3] = rt(() => {
                }, ["prevent"]))
              }, {
                default: n(() => [
                  o(De, {
                    label: e(t)("access.instructionsLabel")
                  }, {
                    default: n(() => [
                      o(qe, {
                        modelValue: d.value,
                        "onUpdate:modelValue": r[1] || (r[1] = (V) => d.value = V),
                        type: "textarea",
                        rows: 5,
                        placeholder: e(t)("access.instructionsPlaceholder")
                      }, null, 8, ["modelValue", "placeholder"]),
                      w("div", Eo, l(e(t)("access.instructionsHint")), 1),
                      d.value ? (v(), D("details", Vo, [
                        w("summary", null, l(e(t)("access.viewDefault")), 1),
                        w("pre", xo, l(P.value), 1)
                      ])) : J("", !0)
                    ]),
                    _: 1
                  }, 8, ["label"]),
                  o(De, {
                    label: e(t)("access.conventionsLabel")
                  }, {
                    default: n(() => [
                      o(qe, {
                        modelValue: y.value,
                        "onUpdate:modelValue": r[2] || (r[2] = (V) => y.value = V),
                        type: "textarea",
                        rows: 4,
                        placeholder: e(t)("access.conventionsPlaceholder")
                      }, null, 8, ["modelValue", "placeholder"]),
                      w("div", $o, l(e(t)("access.conventionsHint")), 1)
                    ]),
                    _: 1
                  }, 8, ["label"])
                ]),
                _: 1
              }),
              w("div", Mo, [
                o(q, {
                  type: "primary",
                  loading: S.value,
                  onClick: M
                }, {
                  default: n(() => [
                    u(l(e(t)("common.save")), 1)
                  ]),
                  _: 1
                }, 8, ["loading"])
              ])
            ]),
            _: 1
          }),
          o(Ve, { shadow: "never" }, {
            header: n(() => [
              u(l(e(t)("access.backupCard")), 1)
            ]),
            default: n(() => [
              w("div", Po, [
                o(q, {
                  icon: e(sa),
                  loading: e(R),
                  onClick: r[4] || (r[4] = (V) => I(e(Y)))
                }, {
                  default: n(() => [
                    u(l(e(t)("access.export")), 1)
                  ]),
                  _: 1
                }, 8, ["icon", "loading"]),
                o(q, {
                  icon: e(ia),
                  loading: e(W),
                  onClick: r[5] || (r[5] = (V) => {
                    var ae;
                    return (ae = O.value) == null ? void 0 : ae.click();
                  })
                }, {
                  default: n(() => [
                    u(l(e(t)("access.import")), 1)
                  ]),
                  _: 1
                }, 8, ["icon", "loading"]),
                o(ce, {
                  content: e(t)("access.importHint"),
                  placement: "top"
                }, {
                  default: n(() => [
                    o(L, { class: "am-info" }, {
                      default: n(() => [
                        o(e($e))
                      ]),
                      _: 1
                    })
                  ]),
                  _: 1
                }, 8, ["content"]),
                w("input", {
                  ref_key: "importInput",
                  ref: O,
                  type: "file",
                  accept: "application/json,.json",
                  style: { display: "none" },
                  onChange: ie
                }, null, 544)
              ])
            ]),
            _: 1
          }),
          o(Ve, { shadow: "never" }, {
            header: n(() => [
              w("div", Uo, [
                w("span", null, l(e(t)("access.doctorCard")), 1),
                o(q, {
                  size: "small",
                  icon: e(et),
                  loading: e(E),
                  onClick: r[6] || (r[6] = (V) => I(e(N)))
                }, {
                  default: n(() => [
                    u(l(e(t)("access.runDoctor")), 1)
                  ]),
                  _: 1
                }, 8, ["icon", "loading"])
              ])
            ]),
            default: n(() => [
              e(A) ? (v(), D(se, { key: 0 }, [
                e(z).ok ? (v(), $(Ee, {
                  key: 0,
                  title: e(t)("access.doctorOk"),
                  type: "success",
                  "show-icon": "",
                  closable: !1
                }, null, 8, ["title"])) : (v(), $(Ee, {
                  key: 1,
                  title: e(t)("access.doctorFail", { count: e(z).issues.length }),
                  type: "error",
                  "show-icon": "",
                  closable: !1
                }, null, 8, ["title"])),
                e(z).ok ? J("", !0) : (v(), D("ul", Ao, [
                  (v(!0), D(se, null, de(e(z).issues, (V, ae) => (v(), D("li", { key: ae }, l(V), 1))), 128))
                ]))
              ], 64)) : (v(), $(lt, {
                key: 1,
                description: e(t)("access.doctorEmpty"),
                "image-size": 60
              }, null, 8, ["description"]))
            ]),
            _: 1
          })
        ], 64)) : J("", !0),
        o(it, {
          modelValue: C.value,
          "onUpdate:modelValue": r[11] || (r[11] = (V) => C.value = V),
          title: oe.value ? e(t)("access.editTitle", { name: oe.value.name }) : e(t)("access.createTitle"),
          width: e(G) ? "96%" : "480px"
        }, {
          footer: n(() => [
            o(q, {
              onClick: r[10] || (r[10] = (V) => C.value = !1)
            }, {
              default: n(() => [
                u(l(e(t)("common.cancel")), 1)
              ]),
              _: 1
            }),
            o(q, {
              type: "primary",
              loading: Z.value,
              onClick: ee
            }, {
              default: n(() => [
                u(l(e(t)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: n(() => [
            o(st, {
              "label-position": "top",
              onSubmit: r[9] || (r[9] = rt(() => {
              }, ["prevent"]))
            }, {
              default: n(() => [
                oe.value ? J("", !0) : (v(), $(De, {
                  key: 0,
                  label: e(t)("access.nameLabel")
                }, {
                  default: n(() => [
                    o(qe, {
                      modelValue: F.name,
                      "onUpdate:modelValue": r[7] || (r[7] = (V) => F.name = V),
                      placeholder: e(t)("access.namePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"])),
                o(De, {
                  label: e(t)("access.permsLabel")
                }, {
                  default: n(() => [
                    w("div", Do, [
                      w("span", Io, l(e(t)("access.presets")), 1),
                      o(xt, {
                        modelValue: j.value,
                        "onUpdate:modelValue": r[8] || (r[8] = (V) => j.value = V),
                        size: "small",
                        onChange: H
                      }, {
                        default: n(() => [
                          o(Ie, { value: "admin" }, {
                            default: n(() => [
                              u(l(e(t)("access.presetAdmin")), 1)
                            ]),
                            _: 1
                          }),
                          o(Ie, { value: "member" }, {
                            default: n(() => [
                              u(l(e(t)("access.presetMember")), 1)
                            ]),
                            _: 1
                          }),
                          o(Ie, { value: "viewer" }, {
                            default: n(() => [
                              u(l(e(t)("access.presetViewer")), 1)
                            ]),
                            _: 1
                          }),
                          o(Ie, { value: "custom" }, {
                            default: n(() => [
                              u(l(e(t)("access.presetCustom")), 1)
                            ]),
                            _: 1
                          })
                        ]),
                        _: 1
                      }, 8, ["modelValue"])
                    ]),
                    w("div", zo, [
                      (v(), D(se, null, de(g, (V) => o($t, {
                        key: V.key,
                        modelValue: F.caps[V.key],
                        "onUpdate:modelValue": (ae) => F.caps[V.key] = ae,
                        onChange: b
                      }, {
                        default: n(() => [
                          u(l(nt(V.key)), 1)
                        ]),
                        _: 2
                      }, 1032, ["modelValue", "onUpdate:modelValue"])), 64))
                    ])
                  ]),
                  _: 1
                }, 8, ["label"])
              ]),
              _: 1
            })
          ]),
          _: 1
        }, 8, ["modelValue", "title", "width"]),
        o(it, {
          modelValue: K.value,
          "onUpdate:modelValue": r[13] || (r[13] = (V) => K.value = V),
          title: Te.value,
          width: e(G) ? "96%" : "560px"
        }, {
          footer: n(() => [
            e(p).onIdentityToken ? (v(), $(q, {
              key: 0,
              type: "primary",
              onClick: te
            }, {
              default: n(() => [
                u(l(e(t)("access.saveToBrowser")), 1)
              ]),
              _: 1
            })) : J("", !0),
            o(q, {
              type: "primary",
              onClick: r[12] || (r[12] = () => {
                Tt(pe.value), K.value = !1;
              })
            }, {
              default: n(() => [
                u(l(e(t)("access.copyToken")), 1)
              ]),
              _: 1
            })
          ]),
          default: n(() => [
            w("p", null, l(e(t)("access.created")), 1),
            w("code", Lo, l(pe.value), 1)
          ]),
          _: 1
        }, 8, ["modelValue", "title", "width"])
      ], 512);
    };
  }
}), Go = /* @__PURE__ */ ke(Bo, [["__scopeId", "data-v-7b57f976"]]);
export {
  Go as AdminPanel,
  wt as MarkdownView,
  ja as MemoriesPanel,
  Wo as MemoryAdmin,
  Ia as MemoryDetailDrawer,
  $a as MemoryEditorDialog,
  gt as MemoryUIConfigKey,
  ro as OpsPanel,
  Ya as TagsPanel,
  pa as applyMemoryUILocalePreference,
  ga as buildMemoriesQuery,
  _a as createApiClient,
  Ko as currentMemoryUILocale,
  va as formatSize,
  be as formatTime,
  Ke as isSearchMode,
  Fe as memoryUIi18n,
  Jo as provideMemoryUI,
  ha as renderMarkdown,
  ba as sanitizeHtml,
  ma as setMemoryUILocale,
  t,
  X as toastError,
  ne as toastSuccess,
  vo as useAdmin,
  _e as useApiClient,
  Sa as useMemories,
  tt as useMemoryConfig,
  no as useOps,
  Ka as useTags
};
