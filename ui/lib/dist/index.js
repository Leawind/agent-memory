import { provide as Vt, inject as xt, ref as v, computed as me, onMounted as $e, watch as Le, onUnmounted as $t, defineComponent as de, openBlock as f, createElementBlock as I, createBlock as U, unref as e, withCtx as n, createVNode as a, createElementVNode as w, toDisplayString as l, createTextVNode as m, Fragment as le, renderList as ce, createCommentVNode as J, withKeys as Mt, isRef as ue, withDirectives as ge, reactive as st, renderSlot as Pt, vShow as je, withModifiers as lt } from "vue";
import { createI18n as Ut } from "vue-i18n";
import { ElDialog as Je, ElForm as We, ElFormItem as Qe, ElInput as Re, ElRadioGroup as Be, ElRadioButton as Ge, ElSelect as it, ElOption as rt, ElButton as Me, ElDrawer as Dt, ElTag as Oe, ElDivider as At, ElTooltip as Ne, ElIcon as Pe, ElAlert as ct, ElTable as Xe, ElTableColumn as Ye, ElLoadingDirective as dt, ElPagination as It, ElRadio as zt, ElRow as Lt, ElCol as Rt, ElCard as ut, ElStatistic as Bt, ElDescriptions as Ot, ElDescriptionsItem as Nt, ElContainer as Ft, ElAside as Ht, ElMenu as qt, ElMenuItem as jt, ElMain as Kt, ElTabs as Jt, ElTabPane as Wt, ElSwitch as Qt, ElEmpty as Gt, ElCheckbox as Xt } from "element-plus/es";
import { InfoFilled as xe, Plus as Ze, Search as et, Refresh as Yt, Collection as Zt, Notebook as ea, PriceTag as ta, Odometer as aa, Download as oa, UploadFilled as na } from "@element-plus/icons-vue";
import { ElMessage as la, ElMessageBox as Ve } from "element-plus";
import { Marked as sa } from "marked";
import mt from "dompurify";
const ia = {
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
    authHint: "开启后，所有 /mcp 与 /api 请求都必须携带 Authorization: Bearer <token>。开启前请先创建身份并保存 token；开启后本浏览器会弹出令牌输入框，输入一次即可。",
    authEnableConfirm: "开启 token 鉴权？此后所有 /mcp 与 /api 请求都必须携带有效 token。",
    authDisableConfirm: "关闭 token 鉴权？所有请求将免鉴权放行（开放模式）。",
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
}, ra = {
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
    authHint: "When enabled, every /mcp and /api request must carry Authorization: Bearer <token>. Create an identity and save its token first; after enabling, this browser will show the token prompt once.",
    authEnableConfirm: "Enable token auth? All /mcp and /api requests will then require a valid token.",
    authDisableConfirm: "Disable token auth? All requests will be allowed without credentials (open mode).",
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
function pt() {
  var o;
  return typeof navigator > "u" || (o = navigator.language) != null && o.toLowerCase().startsWith("zh") ? "zh" : "en";
}
const Fe = Ut({
  legacy: !1,
  locale: pt(),
  fallbackLocale: "zh",
  messages: { zh: ia, en: ra },
  // 面向宿主组件库，缺 key 时静默回退即可，不刷控制台
  missingWarn: !1,
  fallbackWarn: !1
}), { t } = Fe.global;
function ca(o) {
  Fe.global.locale.value = o;
}
function da(o) {
  ca(o === "auto" ? pt() : o);
}
function Ho() {
  return Fe.global.locale.value;
}
const ft = Symbol("memory-ui-config"), ze = {
  baseUrl: "",
  fetch: (...o) => globalThis.fetch(...o),
  defaultPageSize: 20,
  locale: "auto"
};
function qo(o) {
  o.locale && da(o.locale), Vt(ft, o);
}
function vt() {
  const o = xt(ft);
  return {
    baseUrl: ((o == null ? void 0 : o.baseUrl) ?? ze.baseUrl).replace(/\/+$/, ""),
    fetch: (o == null ? void 0 : o.fetch) ?? ze.fetch,
    defaultPageSize: (o == null ? void 0 : o.defaultPageSize) ?? ze.defaultPageSize,
    locale: (o == null ? void 0 : o.locale) ?? ze.locale
  };
}
const ua = 72;
function _t(o, i) {
  let s;
  return s = la({
    type: o,
    message: i,
    offset: ua,
    showClose: !0,
    grouping: !0,
    onClick: () => s.close()
  }), s;
}
function se(o) {
  return _t("success", o);
}
function ae(o) {
  return _t("error", o);
}
function ye(o) {
  if (!o) return "—";
  const i = Fe.global.locale.value === "en" ? "en-US" : "zh-CN";
  return new Date(o * 1e3).toLocaleString(i, { hour12: !1 });
}
function ma(o) {
  return o == null ? "—" : o < 1024 ? `${o} B` : o < 1024 * 1024 ? `${(o / 1024).toFixed(1)} KB` : `${(o / 1024 / 1024).toFixed(2)} MB`;
}
function pa(o) {
  async function i(c, d = {}) {
    const g = await o.fetch(o.baseUrl + c, {
      headers: { "Content-Type": "application/json" },
      ...d
    }), k = await g.text();
    let u = null;
    try {
      u = k ? JSON.parse(k) : null;
    } catch {
      u = null;
    }
    if (!g.ok) {
      const _ = (u == null ? void 0 : u.error) ?? t("errors.http", { status: g.status });
      throw new Error(_);
    }
    return u;
  }
  async function s(c) {
    var g;
    const d = await o.fetch(o.baseUrl + c);
    if (!d.ok) {
      const k = await d.text().catch(() => "");
      let u = t("errors.http", { status: d.status });
      try {
        u = ((g = JSON.parse(k)) == null ? void 0 : g.error) ?? u;
      } catch {
      }
      throw new Error(u);
    }
    return d.blob();
  }
  return {
    get: (c) => i(c),
    post: (c, d) => i(c, { method: "POST", body: JSON.stringify(d ?? {}) }),
    put: (c, d) => i(c, { method: "PUT", body: JSON.stringify(d ?? {}) }),
    del: (c) => i(c, { method: "DELETE" }),
    getBlob: s
  };
}
function pe() {
  return pa(vt());
}
function fa(o) {
  const i = Ke(o.query), s = new URLSearchParams();
  return i ? (s.set("query", o.query.trim()), o.tagFilter && s.set("tags", o.tagFilter)) : (o.tagFilter && s.set("tag", o.tagFilter), s.set("sort", o.sort), s.set("order", o.order)), s.set("offset", String((o.page - 1) * o.pageSize)), s.set("limit", String(o.pageSize)), s.toString();
}
function Ke(o) {
  return o.trim().length > 0;
}
const va = new sa();
function _a(o) {
  const i = va.parse(o, { async: !1 });
  return mt.sanitize(i);
}
function ga(o) {
  return mt.sanitize(o);
}
function tt(o, i) {
  return o.get(`/api/memories/${encodeURIComponent(i)}`);
}
function gt(o, i) {
  return o.post("/api/memories", i);
}
function yt(o, i, s) {
  return o.put(`/api/memories/${encodeURIComponent(i)}`, s);
}
function ya(o, i) {
  return o.del(`/api/memories/${encodeURIComponent(i)}`);
}
function ht(o, i) {
  const s = i ? `?filter=${encodeURIComponent(i)}` : "";
  return o.get(`/api/tags${s}`);
}
function ha(o, i, s) {
  return o.post("/api/tags", { name: i, description: s });
}
function ba(o, i, s, c) {
  const d = { description: c }, g = s.trim();
  return g && g !== i && (d.new_name = g), o.put(`/api/tags/${encodeURIComponent(i)}`, d);
}
function ka(o, i, s) {
  return o.del(`/api/tags/${encodeURIComponent(i)}?mode=${s}`);
}
function wa() {
  const o = pe(), { defaultPageSize: i } = vt(), s = v(""), c = v(""), d = v("updated_at"), g = v("desc"), k = v(1), u = v(i), _ = v([]), M = v([]), T = v(0), p = v(!1), $ = v([]), N = me(() => Ke(s.value));
  function L() {
    return k.value = 1, R();
  }
  function D() {
    return fa({
      query: s.value,
      tagFilter: c.value,
      sort: d.value,
      order: g.value,
      page: k.value,
      pageSize: u.value
    });
  }
  let E = 0;
  async function R() {
    var W;
    const C = ++E;
    p.value = !0;
    try {
      const Q = D();
      if (Ke(s.value)) {
        const A = await o.get(`/api/memories?${Q}`);
        if (C !== E) return;
        M.value = (A.results ?? []).map((B) => ({ ...B, snippet: ga(B.snippet) })), T.value = A.total_matches ?? 0;
      } else {
        const A = await o.get(`/api/memories?${Q}`);
        if (C !== E) return;
        const B = Math.max(1, Math.ceil(A.total / u.value));
        if (((W = A.memories) == null ? void 0 : W.length) === 0 && A.total > 0 && k.value > B)
          return k.value = B, p.value = !1, R();
        _.value = A.memories ?? [], T.value = A.total ?? 0;
      }
    } finally {
      C === E && (p.value = !1);
    }
  }
  async function j() {
    try {
      const C = await ht(o);
      $.value = (C.tags ?? []).map((W) => W.name);
    } catch {
    }
  }
  async function O(C) {
    if (C.id) {
      const W = await tt(o, C.id), Q = new Set(W.tags), A = new Set(C.tags);
      await yt(o, C.id, {
        summary: C.summary,
        content: C.content,
        add_tags: [...A].filter((B) => !Q.has(B)),
        remove_tags: [...Q].filter((B) => !A.has(B))
      });
    } else
      await gt(o, { summary: C.summary, content: C.content, tags: C.tags });
    await Promise.all([R(), j()]);
  }
  async function Y(C) {
    await ya(o, C), await R();
  }
  return $e(() => {
    R().catch(() => {
    }), j();
  }), {
    query: s,
    tagFilter: c,
    sort: d,
    order: g,
    page: k,
    pageSize: u,
    rows: _,
    searchResults: M,
    total: T,
    loading: p,
    tagOptions: $,
    searching: N,
    onSearch: L,
    reload: R,
    loadTagOptions: j,
    saveMemory: O,
    removeMemory: Y
  };
}
function He(o) {
  const i = v(0);
  let s = null;
  function c(k) {
    s == null || s.disconnect(), s = null, !(!k || typeof ResizeObserver > "u") && (s = new ResizeObserver((u) => {
      var _;
      i.value = ((_ = u[0]) == null ? void 0 : _.contentRect.width) ?? 0;
    }), s.observe(k));
  }
  $e(() => c(o.value)), Le(o, (k) => c(k)), $t(() => s == null ? void 0 : s.disconnect());
  const d = me(() => i.value > 0 && i.value < 960), g = me(() => i.value > 0 && i.value < 720);
  return { width: i, compact: d, narrow: g };
}
const Ca = ["innerHTML"], bt = /* @__PURE__ */ de({
  __name: "MarkdownView",
  props: {
    source: {}
  },
  setup(o) {
    const i = o, s = me(() => _a(i.source));
    return (c, d) => (f(), I("div", {
      class: "md-body",
      innerHTML: s.value
    }, null, 8, Ca));
  }
}), Ta = { class: "content-label" }, Sa = /* @__PURE__ */ de({
  __name: "MemoryEditorDialog",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    tagOptions: {},
    width: {}
  },
  emits: ["update:visible", "saved"],
  setup(o, { emit: i }) {
    const s = o, c = i, d = pe(), g = v(!1), k = v("edit"), u = v({ id: null, summary: "", content: "", tags: [] });
    let _ = [];
    Le(
      () => s.visible,
      async (T) => {
        if (T)
          if (k.value = "edit", s.memoryId)
            try {
              const p = await tt(d, s.memoryId);
              u.value = { id: p.id, summary: p.summary, content: p.content, tags: [...p.tags] }, _ = [...p.tags];
            } catch (p) {
              ae(p instanceof Error ? p.message : String(p)), c("update:visible", !1);
            }
          else
            u.value = { id: null, summary: "", content: "", tags: [] }, _ = [];
      }
    );
    async function M() {
      g.value = !0;
      try {
        if (u.value.id) {
          const T = new Set(_), p = new Set(u.value.tags);
          await yt(d, u.value.id, {
            summary: u.value.summary,
            content: u.value.content,
            add_tags: [...p].filter(($) => !T.has($)),
            remove_tags: [...T].filter(($) => !p.has($))
          }), se(t("editor.updated"));
        } else
          await gt(d, {
            summary: u.value.summary,
            content: u.value.content,
            tags: u.value.tags
          }), se(t("editor.created"));
        c("update:visible", !1), c("saved");
      } catch (T) {
        ae(T instanceof Error ? T.message : String(T));
      } finally {
        g.value = !1;
      }
    }
    return (T, p) => {
      const $ = Re, N = Qe, L = Ge, D = Be, E = rt, R = it, j = We, O = Me, Y = Je;
      return f(), U(Y, {
        "model-value": o.visible,
        title: u.value.id ? e(t)("editor.editTitle") : e(t)("editor.createTitle"),
        width: o.width,
        "onUpdate:modelValue": p[5] || (p[5] = (C) => c("update:visible", C))
      }, {
        footer: n(() => [
          a(O, {
            onClick: p[4] || (p[4] = (C) => c("update:visible", !1))
          }, {
            default: n(() => [
              m(l(e(t)("common.cancel")), 1)
            ]),
            _: 1
          }),
          a(O, {
            type: "primary",
            loading: g.value,
            onClick: M
          }, {
            default: n(() => [
              m(l(e(t)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: n(() => [
          a(j, { "label-position": "top" }, {
            default: n(() => [
              a(N, {
                label: e(t)("editor.summaryLabel")
              }, {
                default: n(() => [
                  a($, {
                    modelValue: u.value.summary,
                    "onUpdate:modelValue": p[0] || (p[0] = (C) => u.value.summary = C),
                    maxlength: "512",
                    "show-word-limit": "",
                    placeholder: e(t)("editor.summaryPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"]),
              a(N, null, {
                label: n(() => [
                  w("div", Ta, [
                    w("span", null, l(e(t)("editor.contentLabel")), 1),
                    a(D, {
                      modelValue: k.value,
                      "onUpdate:modelValue": p[1] || (p[1] = (C) => k.value = C),
                      size: "small"
                    }, {
                      default: n(() => [
                        a(L, { value: "edit" }, {
                          default: n(() => [
                            m(l(e(t)("editor.tabEdit")), 1)
                          ]),
                          _: 1
                        }),
                        a(L, { value: "preview" }, {
                          default: n(() => [
                            m(l(e(t)("editor.tabPreview")), 1)
                          ]),
                          _: 1
                        })
                      ]),
                      _: 1
                    }, 8, ["modelValue"])
                  ])
                ]),
                default: n(() => [
                  k.value === "edit" ? (f(), U($, {
                    key: 0,
                    modelValue: u.value.content,
                    "onUpdate:modelValue": p[2] || (p[2] = (C) => u.value.content = C),
                    type: "textarea",
                    rows: 12,
                    maxlength: "262144",
                    "show-word-limit": "",
                    placeholder: e(t)("editor.contentPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])) : (f(), U(bt, {
                    key: 1,
                    class: "content-preview",
                    source: u.value.content
                  }, null, 8, ["source"]))
                ]),
                _: 1
              }),
              a(N, {
                label: e(t)("editor.tagsLabel")
              }, {
                default: n(() => [
                  a(R, {
                    modelValue: u.value.tags,
                    "onUpdate:modelValue": p[3] || (p[3] = (C) => u.value.tags = C),
                    multiple: "",
                    filterable: "",
                    "allow-create": "",
                    "default-first-option": "",
                    placeholder: e(t)("editor.tagsPlaceholder"),
                    class: "tags-select"
                  }, {
                    default: n(() => [
                      (f(!0), I(le, null, ce(o.tagOptions, (C) => (f(), U(E, {
                        key: C,
                        label: C,
                        value: C
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
}), he = (o, i) => {
  const s = o.__vccOpts || o;
  for (const [c, d] of i)
    s[c] = d;
  return s;
}, Ea = /* @__PURE__ */ he(Sa, [["__scopeId", "data-v-a0fec0dd"]]), Va = { class: "detail-summary" }, xa = { class: "detail-tags" }, $a = { class: "detail-toolbar" }, Ma = {
  key: 1,
  class: "detail-content"
}, Pa = /* @__PURE__ */ de({
  __name: "MemoryDetailDrawer",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    size: {}
  },
  emits: ["update:visible"],
  setup(o, { emit: i }) {
    const s = o, c = i, d = pe(), g = v(null), k = v("rendered");
    return Le(
      () => [s.visible, s.memoryId],
      async ([u]) => {
        if (!(!u || !s.memoryId)) {
          k.value = "rendered";
          try {
            g.value = await tt(d, s.memoryId);
          } catch (_) {
            ae(_ instanceof Error ? _.message : String(_)), c("update:visible", !1);
          }
        }
      }
    ), (u, _) => {
      var L;
      const M = Oe, T = At, p = Ge, $ = Be, N = Dt;
      return f(), U(N, {
        "model-value": o.visible,
        title: e(t)("drawer.title", { id: ((L = g.value) == null ? void 0 : L.id) ?? o.memoryId ?? "" }),
        size: o.size,
        "onUpdate:modelValue": _[1] || (_[1] = (D) => c("update:visible", D))
      }, {
        default: n(() => [
          g.value ? (f(), I(le, { key: 0 }, [
            w("h3", Va, l(g.value.summary), 1),
            w("div", xa, [
              (f(!0), I(le, null, ce(g.value.tags, (D) => (f(), U(M, {
                key: D,
                size: "small",
                class: "am-tag"
              }, {
                default: n(() => [
                  m(l(D), 1)
                ]),
                _: 2
              }, 1024))), 128))
            ]),
            a(T),
            w("div", $a, [
              a($, {
                modelValue: k.value,
                "onUpdate:modelValue": _[0] || (_[0] = (D) => k.value = D),
                size: "small"
              }, {
                default: n(() => [
                  a(p, { value: "rendered" }, {
                    default: n(() => [
                      m(l(e(t)("drawer.rendered")), 1)
                    ]),
                    _: 1
                  }),
                  a(p, { value: "source" }, {
                    default: n(() => [
                      m(l(e(t)("drawer.source")), 1)
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["modelValue"])
            ]),
            k.value === "rendered" ? (f(), U(bt, {
              key: 0,
              source: g.value.content
            }, null, 8, ["source"])) : (f(), I("pre", Ma, l(g.value.content), 1))
          ], 64)) : J("", !0)
        ]),
        _: 1
      }, 8, ["model-value", "title", "size"]);
    };
  }
}), Ua = /* @__PURE__ */ he(Pa, [["__scopeId", "data-v-613d4260"]]), Da = {
  key: 0,
  class: "am-panel-header"
}, Aa = { class: "am-heading" }, Ia = { class: "am-panel-title" }, za = { class: "am-toolbar" }, La = { class: "am-summary" }, Ra = ["innerHTML"], Ba = {
  key: 4,
  class: "am-pager"
}, Oa = {
  key: 5,
  class: "am-pager"
}, Na = /* @__PURE__ */ de({
  __name: "MemoriesPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(o, { expose: i }) {
    const s = o, {
      query: c,
      tagFilter: d,
      sort: g,
      order: k,
      page: u,
      pageSize: _,
      rows: M,
      searchResults: T,
      total: p,
      loading: $,
      tagOptions: N,
      searching: L,
      onSearch: D,
      reload: E,
      loadTagOptions: R,
      removeMemory: j
    } = wa(), O = me(() => {
      if ($.value || p.value !== 0) return "";
      if (L.value) return t("memories.searchEmpty");
      const F = d.value.trim();
      return F ? t("memories.tagEmpty", { tag: F }) : "";
    }), Y = v(null), { compact: C, narrow: W } = He(Y), Q = v(!1), A = v(null), B = v(!1), x = v(null);
    function S(F) {
      return F().catch((b) => ae(b instanceof Error ? b.message : String(b)));
    }
    function K() {
      A.value = null, Q.value = !0;
    }
    function oe(F) {
      A.value = F, Q.value = !0;
    }
    function z(F) {
      x.value = F, B.value = !0;
    }
    function ie() {
      S(E), R();
    }
    function fe(F) {
      const { prop: b, order: re } = F;
      re && (b === "updated_at" || b === "created_at" || b === "id") ? (g.value = b, k.value = re === "ascending" ? "asc" : "desc") : (g.value = "updated_at", k.value = "desc"), S(E);
    }
    async function ne(F) {
      try {
        await Ve.confirm(t("memories.deleteConfirm", { id: F.id }), t("memories.deleteTitle"), {
          type: "warning"
        });
      } catch {
        return;
      }
      try {
        await j(F.id), se(t("memories.deleted"));
      } catch (b) {
        ae(b instanceof Error ? b.message : String(b));
      }
    }
    return i({ refresh: () => S(E) }), (F, b) => {
      const re = Pe, ve = Ne, Z = Me, be = Re, ke = rt, P = it, ee = ct, G = Ye, _e = Oe, Ue = Xe, we = It, De = dt;
      return f(), I("div", {
        ref_key: "rootRef",
        ref: Y,
        class: "am-panel"
      }, [
        o.showHeader ? (f(), I("div", Da, [
          w("div", Aa, [
            w("h2", Ia, l(s.title ?? e(t)("memories.title")), 1),
            a(ve, {
              content: s.subtitle ?? e(t)("memories.subtitle"),
              placement: "top"
            }, {
              default: n(() => [
                a(re, { class: "am-info" }, {
                  default: n(() => [
                    a(e(xe))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(Z, {
            type: "primary",
            icon: e(Ze),
            onClick: K
          }, {
            default: n(() => [
              m(l(e(t)("memories.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : J("", !0),
        w("div", za, [
          a(be, {
            modelValue: e(c),
            "onUpdate:modelValue": b[1] || (b[1] = (y) => ue(c) ? c.value = y : null),
            placeholder: e(t)("memories.searchPlaceholder"),
            clearable: "",
            class: "search",
            onKeyup: b[2] || (b[2] = Mt((y) => S(e(D)), ["enter"])),
            onClear: b[3] || (b[3] = (y) => S(e(D)))
          }, {
            append: n(() => [
              a(Z, {
                icon: e(et),
                onClick: b[0] || (b[0] = (y) => S(e(D)))
              }, null, 8, ["icon"])
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(P, {
            modelValue: e(d),
            "onUpdate:modelValue": b[4] || (b[4] = (y) => ue(d) ? d.value = y : null),
            placeholder: e(t)("memories.tagFilter"),
            clearable: "",
            filterable: "",
            class: "tag-filter",
            onChange: b[5] || (b[5] = (y) => S(e(D)))
          }, {
            default: n(() => [
              (f(!0), I(le, null, ce(e(N), (y) => (f(), U(ke, {
                key: y,
                label: y,
                value: y
              }, null, 8, ["label", "value"]))), 128))
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"])
        ]),
        O.value ? (f(), U(ee, {
          key: 1,
          title: O.value,
          type: "info",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : J("", !0),
        e(L) ? ge((f(), U(Ue, {
          key: 2,
          data: e(T)
        }, {
          default: n(() => [
            a(G, {
              prop: "id",
              label: e(t)("memories.colId"),
              width: "80"
            }, null, 8, ["label"]),
            a(G, {
              label: e(t)("memories.colSummary")
            }, {
              default: n(({ row: y }) => [
                w("div", La, l(y.summary), 1),
                w("div", {
                  class: "am-snippet",
                  innerHTML: y.snippet
                }, null, 8, Ra)
              ]),
              _: 1
            }, 8, ["label"]),
            a(G, {
              label: e(t)("memories.colTags"),
              "min-width": "150"
            }, {
              default: n(({ row: y }) => [
                (f(!0), I(le, null, ce(y.tags, (X) => (f(), U(_e, {
                  key: X,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: n(() => [
                    m(l(X), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            a(G, {
              prop: "score",
              label: e(t)("memories.colScore"),
              width: "80",
              sortable: ""
            }, null, 8, ["label"]),
            a(G, {
              label: e(t)("memories.colUpdatedAt"),
              width: "170"
            }, {
              default: n(({ row: y }) => [
                m(l(e(ye)(y.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(G, {
              label: e(t)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: n(({ row: y }) => [
                a(Z, {
                  link: "",
                  type: "primary",
                  onClick: (X) => z(y.id)
                }, {
                  default: n(() => [
                    m(l(e(t)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(Z, {
                  link: "",
                  type: "primary",
                  onClick: (X) => oe(y.id)
                }, {
                  default: n(() => [
                    m(l(e(t)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(Z, {
                  link: "",
                  type: "danger",
                  onClick: (X) => ne(y)
                }, {
                  default: n(() => [
                    m(l(e(t)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [De, e($)]
        ]) : ge((f(), U(Ue, {
          key: 3,
          data: e(M),
          "default-sort": { prop: e(g), order: e(k) === "asc" ? "ascending" : "descending" },
          onSortChange: fe
        }, {
          default: n(() => [
            a(G, {
              prop: "id",
              label: e(t)("memories.colId"),
              width: "80",
              sortable: "custom"
            }, null, 8, ["label"]),
            a(G, {
              prop: "summary",
              label: e(t)("memories.colSummary"),
              "min-width": "180",
              "show-overflow-tooltip": ""
            }, null, 8, ["label"]),
            a(G, {
              label: e(t)("memories.colTags"),
              "min-width": "150"
            }, {
              default: n(({ row: y }) => [
                (f(!0), I(le, null, ce(y.tags, (X) => (f(), U(_e, {
                  key: X,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: n(() => [
                    m(l(X), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            e(C) ? J("", !0) : (f(), U(G, {
              key: 0,
              prop: "created_at",
              label: e(t)("memories.colCreatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: n(({ row: y }) => [
                m(l(e(ye)(y.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label"])),
            a(G, {
              prop: "updated_at",
              label: e(t)("memories.colUpdatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: n(({ row: y }) => [
                m(l(e(ye)(y.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(G, {
              label: e(t)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: n(({ row: y }) => [
                a(Z, {
                  link: "",
                  type: "primary",
                  onClick: (X) => z(y.id)
                }, {
                  default: n(() => [
                    m(l(e(t)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(Z, {
                  link: "",
                  type: "primary",
                  onClick: (X) => oe(y.id)
                }, {
                  default: n(() => [
                    m(l(e(t)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(Z, {
                  link: "",
                  type: "danger",
                  onClick: (X) => ne(y)
                }, {
                  default: n(() => [
                    m(l(e(t)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data", "default-sort"])), [
          [De, e($)]
        ]),
        e(L) ? (f(), I("div", Oa, [
          a(we, {
            "current-page": e(u),
            "onUpdate:currentPage": b[10] || (b[10] = (y) => ue(u) ? u.value = y : null),
            "page-size": e(_),
            "onUpdate:pageSize": b[11] || (b[11] = (y) => ue(_) ? _.value = y : null),
            total: e(p),
            "page-sizes": [10, 20, 50],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: b[12] || (b[12] = (y) => S(e(E))),
            onSizeChange: b[13] || (b[13] = (y) => S(e(E)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])) : (f(), I("div", Ba, [
          a(we, {
            "current-page": e(u),
            "onUpdate:currentPage": b[6] || (b[6] = (y) => ue(u) ? u.value = y : null),
            "page-size": e(_),
            "onUpdate:pageSize": b[7] || (b[7] = (y) => ue(_) ? _.value = y : null),
            total: e(p),
            "page-sizes": [20, 50, 100, 200],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: b[8] || (b[8] = (y) => S(e(E))),
            onSizeChange: b[9] || (b[9] = (y) => S(e(E)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])),
        a(Ea, {
          visible: Q.value,
          "onUpdate:visible": b[14] || (b[14] = (y) => Q.value = y),
          "memory-id": A.value,
          "tag-options": e(N),
          width: e(W) ? "96%" : "640px",
          onSaved: ie
        }, null, 8, ["visible", "memory-id", "tag-options", "width"]),
        a(Ua, {
          visible: B.value,
          "onUpdate:visible": b[15] || (b[15] = (y) => B.value = y),
          "memory-id": x.value,
          size: e(W) ? "100%" : "45%"
        }, null, 8, ["visible", "memory-id", "size"])
      ], 512);
    };
  }
}), Fa = /* @__PURE__ */ he(Na, [["__scopeId", "data-v-5e7dad8e"]]);
function Ha() {
  const o = pe(), i = v([]), s = v(!1), c = v("");
  async function d() {
    s.value = !0;
    try {
      const _ = await ht(o, c.value.trim() || void 0);
      i.value = _.tags ?? [];
    } finally {
      s.value = !1;
    }
  }
  async function g(_, M) {
    await ha(o, _, M), await d();
  }
  async function k(_, M, T) {
    await ba(o, _, M, T), await d();
  }
  async function u(_, M) {
    await ka(o, _, M), await d();
  }
  return $e(() => {
    d().catch(() => {
    });
  }), { rows: i, loading: s, filter: c, reload: d, create: g, rename: k, remove: u };
}
const qa = {
  key: 0,
  class: "am-panel-header"
}, ja = { class: "am-heading" }, Ka = { class: "am-panel-title" }, Ja = { class: "delete-body" }, Wa = /* @__PURE__ */ de({
  __name: "TagsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(o, { expose: i }) {
    const s = o, { rows: c, loading: d, filter: g, reload: k, create: u, rename: _, remove: M } = Ha();
    let T = null;
    function p() {
      T && clearTimeout(T), T = setTimeout(() => {
        T = null, k().catch((x) => ae(x instanceof Error ? x.message : String(x)));
      }, 300);
    }
    const $ = v(null), { narrow: N } = He($), L = v(!1), D = v(!1), E = st({ oldName: null, name: "", description: "" }), R = v(!1), j = v("detach"), O = v(null);
    function Y(x) {
      return (S, K) => (S[x] ?? 0) - (K[x] ?? 0);
    }
    function C() {
      Object.assign(E, { oldName: null, name: "", description: "" }), D.value = !0;
    }
    function W(x) {
      Object.assign(E, { oldName: x.name, name: x.name, description: x.description ?? "" }), D.value = !0;
    }
    async function Q() {
      L.value = !0;
      try {
        E.oldName ? (await _(E.oldName, E.name, E.description), se(t("tags.saved"))) : (await u(E.name, E.description), se(t("tags.created"))), D.value = !1;
      } catch (x) {
        ae(x instanceof Error ? x.message : String(x));
      } finally {
        L.value = !1;
      }
    }
    function A(x) {
      O.value = x, j.value = "detach", R.value = !0;
    }
    async function B() {
      var x, S;
      if (j.value === "purge")
        try {
          await Ve.confirm(
            t("tags.purgeConfirm", { name: (x = O.value) == null ? void 0 : x.name, count: ((S = O.value) == null ? void 0 : S.memory_count) ?? 0 }),
            t("tags.purgeConfirmTitle"),
            { type: "error", confirmButtonText: t("tags.purgeButton") }
          );
        } catch {
          return;
        }
      if (O.value) {
        L.value = !0;
        try {
          await M(O.value.name, j.value), se(t("tags.deleted")), R.value = !1;
        } catch (K) {
          ae(K instanceof Error ? K.message : String(K));
        } finally {
          L.value = !1;
        }
      }
    }
    return i({
      refresh: () => k().catch((x) => ae(x instanceof Error ? x.message : String(x)))
    }), (x, S) => {
      const K = Pe, oe = Ne, z = Me, ie = Re, fe = Oe, ne = Ye, F = Xe, b = Qe, re = We, ve = Je, Z = zt, be = Be, ke = dt;
      return f(), I("div", {
        ref_key: "rootRef",
        ref: $,
        class: "am-panel"
      }, [
        o.showHeader ? (f(), I("div", qa, [
          w("div", ja, [
            w("h2", Ka, l(s.title ?? e(t)("tags.title")), 1),
            a(oe, {
              content: s.subtitle ?? e(t)("tags.subtitle"),
              placement: "top"
            }, {
              default: n(() => [
                a(K, { class: "am-info" }, {
                  default: n(() => [
                    a(e(xe))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(z, {
            type: "primary",
            icon: e(Ze),
            onClick: C
          }, {
            default: n(() => [
              m(l(e(t)("tags.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : J("", !0),
        a(ie, {
          modelValue: e(g),
          "onUpdate:modelValue": S[0] || (S[0] = (P) => ue(g) ? g.value = P : null),
          class: "am-tag-filter",
          placeholder: e(t)("tags.filterPlaceholder"),
          clearable: "",
          "prefix-icon": e(et),
          onInput: p,
          onClear: p
        }, null, 8, ["modelValue", "placeholder", "prefix-icon"]),
        ge((f(), U(F, { data: e(c) }, {
          default: n(() => [
            a(ne, {
              prop: "name",
              label: e(t)("tags.colName"),
              "min-width": "140",
              sortable: ""
            }, {
              default: n(({ row: P }) => [
                a(fe, null, {
                  default: n(() => [
                    m(l(P.name), 1)
                  ]),
                  _: 2
                }, 1024)
              ]),
              _: 1
            }, 8, ["label"]),
            a(ne, {
              prop: "description",
              label: e(t)("tags.colDescription"),
              "min-width": "150",
              "show-overflow-tooltip": ""
            }, {
              default: n(({ row: P }) => [
                m(l(P.description || "—"), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(ne, {
              prop: "memory_count",
              label: e(t)("tags.colMemoryCount"),
              width: "90",
              sortable: ""
            }, null, 8, ["label"]),
            a(ne, {
              prop: "last_used_at",
              label: e(t)("tags.colLastUsed"),
              width: "170",
              sortable: "",
              "sort-method": Y("last_used_at")
            }, {
              default: n(({ row: P }) => [
                m(l(e(ye)(P.last_used_at)), 1)
              ]),
              _: 1
            }, 8, ["label", "sort-method"]),
            a(ne, {
              prop: "created_at",
              label: e(t)("tags.colCreatedAt"),
              width: "170",
              sortable: "",
              "sort-method": Y("created_at")
            }, {
              default: n(({ row: P }) => [
                m(l(e(ye)(P.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label", "sort-method"]),
            a(ne, {
              label: e(t)("memories.colActions"),
              width: "150",
              fixed: "right"
            }, {
              default: n(({ row: P }) => [
                a(z, {
                  link: "",
                  type: "primary",
                  onClick: (ee) => W(P)
                }, {
                  default: n(() => [
                    m(l(e(t)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(z, {
                  link: "",
                  type: "danger",
                  onClick: (ee) => A(P)
                }, {
                  default: n(() => [
                    m(l(e(t)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [ke, e(d)]
        ]),
        a(ve, {
          modelValue: D.value,
          "onUpdate:modelValue": S[4] || (S[4] = (P) => D.value = P),
          title: E.oldName ? e(t)("tags.editTitle") : e(t)("tags.createTitle"),
          width: e(N) ? "96%" : "480px"
        }, {
          footer: n(() => [
            a(z, {
              onClick: S[3] || (S[3] = (P) => D.value = !1)
            }, {
              default: n(() => [
                m(l(e(t)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(z, {
              type: "primary",
              loading: L.value,
              onClick: Q
            }, {
              default: n(() => [
                m(l(e(t)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: n(() => [
            a(re, { "label-position": "top" }, {
              default: n(() => [
                a(b, {
                  label: e(t)("tags.nameLabel")
                }, {
                  default: n(() => [
                    a(ie, {
                      modelValue: E.name,
                      "onUpdate:modelValue": S[1] || (S[1] = (P) => E.name = P),
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(t)("tags.namePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(b, {
                  label: e(t)("tags.descLabel")
                }, {
                  default: n(() => [
                    a(ie, {
                      modelValue: E.description,
                      "onUpdate:modelValue": S[2] || (S[2] = (P) => E.description = P),
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
        a(ve, {
          modelValue: R.value,
          "onUpdate:modelValue": S[7] || (S[7] = (P) => R.value = P),
          title: e(t)("tags.deleteTitle"),
          width: e(N) ? "96%" : "480px"
        }, {
          footer: n(() => [
            a(z, {
              onClick: S[6] || (S[6] = (P) => R.value = !1)
            }, {
              default: n(() => [
                m(l(e(t)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(z, {
              type: "danger",
              loading: L.value,
              onClick: B
            }, {
              default: n(() => [
                m(l(e(t)("common.delete")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: n(() => {
            var P;
            return [
              w("p", Ja, [
                m(l(e(t)("tags.deleteBefore")) + " ", 1),
                a(fe, null, {
                  default: n(() => {
                    var ee;
                    return [
                      m(l((ee = O.value) == null ? void 0 : ee.name), 1)
                    ];
                  }),
                  _: 1
                }),
                m(" " + l(e(t)("tags.deleteMiddle")) + " ", 1),
                w("b", null, l((P = O.value) == null ? void 0 : P.memory_count), 1),
                m(" " + l(e(t)("tags.deleteAfter")), 1)
              ]),
              a(be, {
                modelValue: j.value,
                "onUpdate:modelValue": S[5] || (S[5] = (ee) => j.value = ee)
              }, {
                default: n(() => [
                  a(Z, { value: "detach" }, {
                    default: n(() => [
                      m(l(e(t)("tags.detach")), 1)
                    ]),
                    _: 1
                  }),
                  a(Z, { value: "purge" }, {
                    default: n(() => [
                      m(l(e(t)("tags.purge")), 1)
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
}), Qa = /* @__PURE__ */ he(Wa, [["__scopeId", "data-v-c8aeea57"]]);
function Ga(o) {
  return o.get("/api/stats");
}
function Xa(o) {
  return o.get("/health");
}
function Ya(o) {
  return o.get("/api/doctor");
}
function Za(o) {
  return o.getBlob("/api/export");
}
function eo(o, i) {
  return o.post("/api/import", i);
}
function to() {
  const o = pe(), i = v({}), s = v(""), c = me(() => ma(i.value.file_size));
  async function d() {
    i.value = await Ga(o), i.value.version = s.value;
  }
  return $e(async () => {
    try {
      s.value = (await Xa(o)).version ?? "";
    } catch {
    }
    await d().catch(() => {
    });
  }), {
    stats: i,
    version: s,
    sizeText: c,
    reload: d
  };
}
const ao = {
  key: 0,
  class: "am-panel-header"
}, oo = { class: "am-heading" }, no = { class: "am-panel-title" }, lo = /* @__PURE__ */ de({
  __name: "OpsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(o, { expose: i }) {
    const s = o, { stats: c, version: d, sizeText: g, reload: k } = to(), u = v(null), { narrow: _ } = He(u);
    function M(T) {
      return T().catch((p) => ae(p instanceof Error ? p.message : String(p)));
    }
    return i({ refresh: () => M(k) }), (T, p) => {
      const $ = Pe, N = Ne, L = Me, D = Bt, E = ut, R = Rt, j = Lt, O = Nt, Y = Ot;
      return f(), I("div", {
        ref_key: "rootRef",
        ref: u,
        class: "am-panel"
      }, [
        o.showHeader ? (f(), I("div", ao, [
          w("div", oo, [
            w("h2", no, l(s.title ?? e(t)("ops.title")), 1),
            a(N, {
              content: s.subtitle ?? e(t)("ops.subtitle"),
              placement: "top"
            }, {
              default: n(() => [
                a($, { class: "am-info" }, {
                  default: n(() => [
                    a(e(xe))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(L, {
            icon: e(Yt),
            onClick: p[0] || (p[0] = (C) => M(e(k)))
          }, {
            default: n(() => [
              m(l(e(t)("ops.refresh")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : J("", !0),
        a(j, { gutter: 14 }, {
          default: n(() => [
            a(R, {
              span: e(_) ? 12 : 8
            }, {
              default: n(() => [
                a(E, { shadow: "never" }, {
                  default: n(() => [
                    a(D, {
                      title: e(t)("ops.statMemories"),
                      value: e(c).memories ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(R, {
              span: e(_) ? 12 : 8
            }, {
              default: n(() => [
                a(E, { shadow: "never" }, {
                  default: n(() => [
                    a(D, {
                      title: e(t)("ops.statTags"),
                      value: e(c).tags ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(R, {
              span: e(_) ? 12 : 8
            }, {
              default: n(() => [
                a(E, { shadow: "never" }, {
                  default: n(() => [
                    a(D, {
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
        a(E, { shadow: "never" }, {
          header: n(() => [
            m(l(e(t)("ops.dbCard")), 1)
          ]),
          default: n(() => [
            a(Y, {
              column: e(_) ? 1 : 2,
              border: ""
            }, {
              default: n(() => [
                a(O, {
                  label: e(t)("ops.version")
                }, {
                  default: n(() => [
                    m(l(e(c).version ?? e(d)), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(O, {
                  label: e(t)("ops.schemaVersion")
                }, {
                  default: n(() => [
                    m(l(e(c).schema_version ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(O, {
                  label: e(t)("ops.path"),
                  span: e(_) ? 1 : 2
                }, {
                  default: n(() => [
                    m(l(e(c).path ?? "—"), 1)
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
}), so = { class: "memory-ui" }, io = { class: "brand" }, ro = { class: "brand-mark" }, co = { class: "aside-footer" }, uo = /* @__PURE__ */ de({
  __name: "MemoryAdmin",
  props: {
    layout: { default: "sidebar" },
    title: { default: "Agent Memory" }
  },
  setup(o) {
    const i = v("memories");
    return (s, c) => {
      const d = Pe, g = jt, k = qt, u = Ht, _ = Wt, M = Jt, T = Kt, p = Ft;
      return f(), I("div", so, [
        a(p, { class: "layout" }, {
          default: n(() => [
            o.layout === "sidebar" ? (f(), U(u, {
              key: 0,
              width: "200px",
              class: "aside"
            }, {
              default: n(() => [
                w("div", io, [
                  w("span", ro, [
                    a(d, { size: 16 }, {
                      default: n(() => [
                        a(e(Zt))
                      ]),
                      _: 1
                    })
                  ]),
                  w("span", null, l(o.title), 1)
                ]),
                a(k, {
                  "default-active": i.value,
                  class: "menu",
                  onSelect: c[0] || (c[0] = ($) => i.value = $)
                }, {
                  default: n(() => [
                    a(g, { index: "memories" }, {
                      default: n(() => [
                        a(d, null, {
                          default: n(() => [
                            a(e(ea))
                          ]),
                          _: 1
                        }),
                        w("span", null, l(e(t)("nav.memories")), 1)
                      ]),
                      _: 1
                    }),
                    a(g, { index: "tags" }, {
                      default: n(() => [
                        a(d, null, {
                          default: n(() => [
                            a(e(ta))
                          ]),
                          _: 1
                        }),
                        w("span", null, l(e(t)("nav.tags")), 1)
                      ]),
                      _: 1
                    }),
                    a(g, { index: "ops" }, {
                      default: n(() => [
                        a(d, null, {
                          default: n(() => [
                            a(e(aa))
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
                w("div", co, [
                  Pt(s.$slots, "footer", {}, void 0, !0)
                ])
              ]),
              _: 3
            })) : J("", !0),
            a(T, { class: "main" }, {
              default: n(() => [
                o.layout === "tabs" ? (f(), U(M, {
                  key: 0,
                  modelValue: i.value,
                  "onUpdate:modelValue": c[1] || (c[1] = ($) => i.value = $),
                  class: "tabs-bar"
                }, {
                  default: n(() => [
                    a(_, {
                      label: e(t)("nav.memories"),
                      name: "memories"
                    }, null, 8, ["label"]),
                    a(_, {
                      label: e(t)("nav.tags"),
                      name: "tags"
                    }, null, 8, ["label"]),
                    a(_, {
                      label: e(t)("nav.ops"),
                      name: "ops"
                    }, null, 8, ["label"])
                  ]),
                  _: 1
                }, 8, ["modelValue"])) : J("", !0),
                ge(a(Fa, null, null, 512), [
                  [je, i.value === "memories"]
                ]),
                ge(a(Qa, null, null, 512), [
                  [je, i.value === "tags"]
                ]),
                ge(a(lo, null, null, 512), [
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
}), jo = /* @__PURE__ */ he(uo, [["__scopeId", "data-v-38c91753"]]);
function mo() {
  const o = pe(), i = v({ ok: !0, issues: [] }), s = v(!1), c = v(!1), d = v(!1), g = v(!1);
  async function k() {
    c.value = !0;
    try {
      i.value = await Ya(o), s.value = !0;
    } finally {
      c.value = !1;
    }
  }
  async function u() {
    d.value = !0;
    try {
      const M = await Za(o), T = URL.createObjectURL(M), p = document.createElement("a");
      p.href = T, p.download = "agent-memory-export.json", p.click(), URL.revokeObjectURL(T);
    } finally {
      d.value = !1;
    }
  }
  async function _(M) {
    g.value = !0;
    try {
      const T = await M.text();
      let p;
      try {
        p = JSON.parse(T);
      } catch {
        throw new Error(t("errors.invalidBackup"));
      }
      return await eo(o, p);
    } finally {
      g.value = !1;
    }
  }
  return {
    doctor: i,
    doctorRan: s,
    doctorLoading: c,
    exporting: d,
    importing: g,
    runDoctor: k,
    exportData: u,
    importFile: _
  };
}
const po = {
  key: 0,
  class: "am-panel-header"
}, fo = { class: "am-heading" }, vo = { class: "am-panel-title" }, _o = { class: "auth-row" }, go = { class: "auth-text" }, yo = { class: "auth-label" }, ho = { class: "auth-hint" }, bo = { class: "token-cell" }, ko = { class: "token-text" }, wo = {
  key: 0,
  class: "muted"
}, Co = { class: "field-hint" }, To = {
  key: 0,
  class: "default-view"
}, So = { class: "default-text" }, Eo = { class: "field-hint" }, Vo = { class: "save-row" }, xo = { class: "actions" }, $o = { class: "card-header" }, Mo = {
  key: 2,
  class: "issues"
}, Po = { class: "presets" }, Uo = { class: "presets-label" }, Do = { class: "caps" }, Ao = { class: "new-token" }, Io = /* @__PURE__ */ de({
  __name: "AdminPanel",
  props: {
    who: { default: null },
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(o, { expose: i }) {
    const s = o, c = pe(), d = [
      { key: "read", labelKey: "capRead" },
      { key: "create", labelKey: "capCreate" },
      { key: "update", labelKey: "capUpdate" },
      { key: "delete", labelKey: "capDelete" },
      { key: "tag_manage", labelKey: "capTagManage" },
      { key: "admin", labelKey: "capAdmin" }
    ], g = v([]), k = v(""), u = v(""), _ = v(""), M = v(!1), T = v(!1), p = v(!1), $ = me(() => {
      var h;
      return !!s.who && ((h = s.who.permissions) == null ? void 0 : h.admin) === !0;
    }), { doctor: N, doctorRan: L, doctorLoading: D, exporting: E, importing: R, runDoctor: j, exportData: O, importFile: Y } = mo(), C = v(null), { compact: W } = He(C), Q = v(null);
    async function A(h, r) {
      try {
        return await h();
      } catch (H) {
        ae(H instanceof Error ? H.message : String(H));
        return;
      }
    }
    async function B() {
      await A(async () => {
        const [h, r] = await Promise.all([
          c.get("/api/identities"),
          c.get("/api/settings")
        ]);
        g.value = Array.isArray(h == null ? void 0 : h.identities) ? h.identities : [], k.value = (r == null ? void 0 : r.instructions) ?? "", u.value = (r == null ? void 0 : r.conventions) ?? "", _.value = (r == null ? void 0 : r.default_instructions) ?? "", T.value = (r == null ? void 0 : r.auth_required) === !0;
      });
    }
    $e(() => {
      $.value && B();
    }), Le($, (h) => {
      h && B();
    }), i({
      refresh: () => {
        $.value && A(B);
      }
    });
    const x = v(!1), S = v(!1), K = v(null), oe = v("custom"), z = st({
      name: "",
      caps: ie()
    });
    function ie() {
      return Object.fromEntries(d.map((h) => [h.key, !1]));
    }
    const fe = {
      admin: d.map((h) => h.key),
      member: ["read", "create", "update", "tag_manage"],
      viewer: ["read"]
    };
    function ne() {
      if (oe.value === "custom") return;
      const h = new Set(fe[oe.value] ?? []);
      for (const r of d) z.caps[r.key] = h.has(r.key);
    }
    function F() {
      oe.value = "custom";
    }
    function b() {
      K.value = null, z.name = "", Object.assign(z.caps, ie()), oe.value = "member", ne(), x.value = !0;
    }
    function re(h) {
      var r;
      K.value = h, z.name = h.name;
      for (const H of d) z.caps[H.key] = ((r = h.permissions) == null ? void 0 : r[H.key]) === !0;
      oe.value = "custom", x.value = !0;
    }
    async function ve() {
      S.value = !0;
      try {
        if (K.value)
          await A(
            () => c.put(`/api/identities/${encodeURIComponent(K.value.name)}`, {
              permissions: { ...z.caps }
            })
          ), se(t("access.saved"));
        else {
          const h = await A(
            () => c.post("/api/identities", {
              name: z.name,
              permissions: { ...z.caps }
            })
          );
          h != null && h.token && we(t("access.createTitle"), h.token);
        }
        x.value = !1, await B();
      } finally {
        S.value = !1;
      }
    }
    async function Z(h) {
      try {
        await Ve.confirm(t("access.deleteConfirm", { name: h.name }), t("access.deleteTitle"), {
          type: "warning",
          confirmButtonText: t("common.delete"),
          cancelButtonText: t("common.cancel")
        });
      } catch {
        return;
      }
      await A(() => c.del(`/api/identities/${encodeURIComponent(h.name)}`)) !== void 0 && (se(t("access.deleted")), await B());
    }
    async function be() {
      const h = !T.value;
      try {
        await Ve.confirm(
          t(h ? "access.authEnableConfirm" : "access.authDisableConfirm"),
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
      p.value = !0;
      try {
        return await A(() => c.put("/api/settings", { auth_required: h })), !0;
      } finally {
        p.value = !1;
      }
    }
    async function ke() {
      M.value = !0;
      try {
        await A(
          () => c.put("/api/settings", {
            instructions: k.value,
            conventions: u.value
          })
        ), se(t("access.saved"));
      } finally {
        M.value = !1;
      }
    }
    function P(h) {
      var Ce;
      const r = h.target, H = (Ce = r.files) == null ? void 0 : Ce[0];
      r.value = "", H && Y(H).then((q) => {
        se(t("access.imported", { memories: q.imported_memories, tags: q.imported_tags }));
      }).catch((q) => {
        ae(q instanceof Error ? q.message : String(q));
      });
    }
    const ee = v(!1), G = v(""), _e = v("");
    function Ue(h) {
      return h ? `…${h}` : "—";
    }
    function we(h, r) {
      G.value = h, _e.value = r, ee.value = !0;
    }
    async function De(h) {
      try {
        await Ve.confirm(t("access.resetConfirm", { name: h.name }), t("access.resetTitle"), {
          type: "warning",
          confirmButtonText: t("access.resetToken"),
          cancelButtonText: t("common.cancel")
        });
      } catch {
        return;
      }
      const r = await A(
        () => c.post(`/api/identities/${encodeURIComponent(h.name)}/token-reset`, {})
      );
      r !== void 0 && (r != null && r.token && we(t("access.resetTitle"), r.token), await B());
    }
    function y(h) {
      return d.map((r) => r.key).filter((r) => {
        var H;
        return ((H = h.permissions) == null ? void 0 : H[r]) === !0;
      });
    }
    function X(h) {
      const r = d.find((H) => H.key === h);
      return r ? t(`access.${r.labelKey}`) : h;
    }
    async function kt(h) {
      try {
        await navigator.clipboard.writeText(h), se(t("access.copied"));
      } catch {
        ae(h);
      }
    }
    return (h, r) => {
      const H = Pe, Ce = Ne, q = Me, Te = ct, wt = Qt, Se = ut, at = Gt, Ee = Ye, Ct = Oe, Tt = Xe, qe = Re, Ae = Qe, ot = We, Ie = Ge, St = Be, Et = Xt, nt = Je;
      return f(), I("div", {
        ref_key: "rootRef",
        ref: C,
        class: "am-panel admin-panel"
      }, [
        o.showHeader ? (f(), I("div", po, [
          w("div", fo, [
            w("h2", vo, l(s.title ?? e(t)("access.title")), 1),
            a(Ce, {
              content: s.subtitle ?? e(t)("access.subtitle"),
              placement: "top"
            }, {
              default: n(() => [
                a(H, { class: "am-info" }, {
                  default: n(() => [
                    a(e(xe))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          $.value ? (f(), U(q, {
            key: 0,
            type: "primary",
            icon: e(Ze),
            onClick: b
          }, {
            default: n(() => [
              m(l(e(t)("access.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])) : J("", !0)
        ])) : J("", !0),
        o.who ? o.who.mode === "open" ? (f(), U(Te, {
          key: 2,
          type: "warning",
          title: e(t)("access.openMode"),
          closable: !1
        }, null, 8, ["title"])) : $.value ? J("", !0) : (f(), U(Te, {
          key: 3,
          type: "info",
          title: e(t)("access.needAdmin"),
          closable: !1
        }, null, 8, ["title"])) : (f(), U(Te, {
          key: 1,
          type: "info",
          title: e(t)("access.needAdmin"),
          closable: !1
        }, null, 8, ["title"])),
        o.who && $.value ? (f(), I(le, { key: 4 }, [
          a(Se, { shadow: "never" }, {
            default: n(() => [
              w("div", _o, [
                w("div", go, [
                  w("span", yo, l(e(t)("access.authTitle")), 1),
                  w("span", ho, l(e(t)("access.authHint")), 1)
                ]),
                a(wt, {
                  modelValue: T.value,
                  "onUpdate:modelValue": r[0] || (r[0] = (V) => T.value = V),
                  "before-change": be,
                  loading: p.value
                }, null, 8, ["modelValue", "loading"])
              ])
            ]),
            _: 1
          }),
          a(Se, { shadow: "never" }, {
            default: n(() => [
              g.value.length === 0 ? (f(), U(at, {
                key: 0,
                description: e(t)("access.empty")
              }, null, 8, ["description"])) : (f(), U(Tt, {
                key: 1,
                data: g.value
              }, {
                default: n(() => [
                  a(Ee, {
                    prop: "name",
                    label: e(t)("access.colName"),
                    "min-width": "120"
                  }, null, 8, ["label"]),
                  a(Ee, {
                    label: e(t)("access.colToken"),
                    "min-width": "200"
                  }, {
                    default: n(({ row: V }) => [
                      w("div", bo, [
                        w("code", ko, l(Ue(V.token_hint)), 1),
                        a(q, {
                          link: "",
                          type: "primary",
                          onClick: (te) => De(V)
                        }, {
                          default: n(() => [
                            m(l(e(t)("access.resetToken")), 1)
                          ]),
                          _: 1
                        }, 8, ["onClick"])
                      ])
                    ]),
                    _: 1
                  }, 8, ["label"]),
                  a(Ee, {
                    label: e(t)("access.colPermissions"),
                    "min-width": "240"
                  }, {
                    default: n(({ row: V }) => [
                      (f(!0), I(le, null, ce(y(V), (te) => (f(), U(Ct, {
                        key: te,
                        size: "small",
                        class: "cap-tag",
                        type: te === "admin" ? "danger" : "info"
                      }, {
                        default: n(() => [
                          m(l(X(te)), 1)
                        ]),
                        _: 2
                      }, 1032, ["type"]))), 128)),
                      y(V).length === 0 ? (f(), I("span", wo, "—")) : J("", !0)
                    ]),
                    _: 1
                  }, 8, ["label"]),
                  e(W) ? J("", !0) : (f(), U(Ee, {
                    key: 0,
                    label: e(t)("access.colCreatedAt"),
                    width: "170"
                  }, {
                    default: n(({ row: V }) => [
                      m(l(e(ye)(V.created_at)), 1)
                    ]),
                    _: 1
                  }, 8, ["label"])),
                  a(Ee, {
                    label: e(t)("access.colActions"),
                    width: "140",
                    fixed: "right"
                  }, {
                    default: n(({ row: V }) => [
                      a(q, {
                        link: "",
                        type: "primary",
                        onClick: (te) => re(V)
                      }, {
                        default: n(() => [
                          m(l(e(t)("common.edit")), 1)
                        ]),
                        _: 1
                      }, 8, ["onClick"]),
                      a(q, {
                        link: "",
                        type: "danger",
                        onClick: (te) => Z(V)
                      }, {
                        default: n(() => [
                          m(l(e(t)("common.delete")), 1)
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
          a(Se, { shadow: "never" }, {
            header: n(() => [
              m(l(e(t)("access.settingsTitle")), 1)
            ]),
            default: n(() => [
              a(ot, {
                "label-position": "top",
                onSubmit: r[3] || (r[3] = lt(() => {
                }, ["prevent"]))
              }, {
                default: n(() => [
                  a(Ae, {
                    label: e(t)("access.instructionsLabel")
                  }, {
                    default: n(() => [
                      a(qe, {
                        modelValue: k.value,
                        "onUpdate:modelValue": r[1] || (r[1] = (V) => k.value = V),
                        type: "textarea",
                        rows: 5,
                        placeholder: e(t)("access.instructionsPlaceholder")
                      }, null, 8, ["modelValue", "placeholder"]),
                      w("div", Co, l(e(t)("access.instructionsHint")), 1),
                      k.value ? (f(), I("details", To, [
                        w("summary", null, l(e(t)("access.viewDefault")), 1),
                        w("pre", So, l(_.value), 1)
                      ])) : J("", !0)
                    ]),
                    _: 1
                  }, 8, ["label"]),
                  a(Ae, {
                    label: e(t)("access.conventionsLabel")
                  }, {
                    default: n(() => [
                      a(qe, {
                        modelValue: u.value,
                        "onUpdate:modelValue": r[2] || (r[2] = (V) => u.value = V),
                        type: "textarea",
                        rows: 4,
                        placeholder: e(t)("access.conventionsPlaceholder")
                      }, null, 8, ["modelValue", "placeholder"]),
                      w("div", Eo, l(e(t)("access.conventionsHint")), 1)
                    ]),
                    _: 1
                  }, 8, ["label"])
                ]),
                _: 1
              }),
              w("div", Vo, [
                a(q, {
                  type: "primary",
                  loading: M.value,
                  onClick: ke
                }, {
                  default: n(() => [
                    m(l(e(t)("common.save")), 1)
                  ]),
                  _: 1
                }, 8, ["loading"])
              ])
            ]),
            _: 1
          }),
          a(Se, { shadow: "never" }, {
            header: n(() => [
              m(l(e(t)("access.backupCard")), 1)
            ]),
            default: n(() => [
              w("div", xo, [
                a(q, {
                  icon: e(oa),
                  loading: e(E),
                  onClick: r[4] || (r[4] = (V) => A(e(O)))
                }, {
                  default: n(() => [
                    m(l(e(t)("access.export")), 1)
                  ]),
                  _: 1
                }, 8, ["icon", "loading"]),
                a(q, {
                  icon: e(na),
                  loading: e(R),
                  onClick: r[5] || (r[5] = (V) => {
                    var te;
                    return (te = Q.value) == null ? void 0 : te.click();
                  })
                }, {
                  default: n(() => [
                    m(l(e(t)("access.import")), 1)
                  ]),
                  _: 1
                }, 8, ["icon", "loading"]),
                a(Ce, {
                  content: e(t)("access.importHint"),
                  placement: "top"
                }, {
                  default: n(() => [
                    a(H, { class: "am-info" }, {
                      default: n(() => [
                        a(e(xe))
                      ]),
                      _: 1
                    })
                  ]),
                  _: 1
                }, 8, ["content"]),
                w("input", {
                  ref_key: "importInput",
                  ref: Q,
                  type: "file",
                  accept: "application/json,.json",
                  style: { display: "none" },
                  onChange: P
                }, null, 544)
              ])
            ]),
            _: 1
          }),
          a(Se, { shadow: "never" }, {
            header: n(() => [
              w("div", $o, [
                w("span", null, l(e(t)("access.doctorCard")), 1),
                a(q, {
                  size: "small",
                  icon: e(et),
                  loading: e(D),
                  onClick: r[6] || (r[6] = (V) => A(e(j)))
                }, {
                  default: n(() => [
                    m(l(e(t)("access.runDoctor")), 1)
                  ]),
                  _: 1
                }, 8, ["icon", "loading"])
              ])
            ]),
            default: n(() => [
              e(L) ? (f(), I(le, { key: 0 }, [
                e(N).ok ? (f(), U(Te, {
                  key: 0,
                  title: e(t)("access.doctorOk"),
                  type: "success",
                  "show-icon": "",
                  closable: !1
                }, null, 8, ["title"])) : (f(), U(Te, {
                  key: 1,
                  title: e(t)("access.doctorFail", { count: e(N).issues.length }),
                  type: "error",
                  "show-icon": "",
                  closable: !1
                }, null, 8, ["title"])),
                e(N).ok ? J("", !0) : (f(), I("ul", Mo, [
                  (f(!0), I(le, null, ce(e(N).issues, (V, te) => (f(), I("li", { key: te }, l(V), 1))), 128))
                ]))
              ], 64)) : (f(), U(at, {
                key: 1,
                description: e(t)("access.doctorEmpty"),
                "image-size": 60
              }, null, 8, ["description"]))
            ]),
            _: 1
          })
        ], 64)) : J("", !0),
        a(nt, {
          modelValue: x.value,
          "onUpdate:modelValue": r[11] || (r[11] = (V) => x.value = V),
          title: K.value ? e(t)("access.editTitle", { name: K.value.name }) : e(t)("access.createTitle"),
          width: e(W) ? "96%" : "480px"
        }, {
          footer: n(() => [
            a(q, {
              onClick: r[10] || (r[10] = (V) => x.value = !1)
            }, {
              default: n(() => [
                m(l(e(t)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(q, {
              type: "primary",
              loading: S.value,
              onClick: ve
            }, {
              default: n(() => [
                m(l(e(t)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: n(() => [
            a(ot, {
              "label-position": "top",
              onSubmit: r[9] || (r[9] = lt(() => {
              }, ["prevent"]))
            }, {
              default: n(() => [
                K.value ? J("", !0) : (f(), U(Ae, {
                  key: 0,
                  label: e(t)("access.nameLabel")
                }, {
                  default: n(() => [
                    a(qe, {
                      modelValue: z.name,
                      "onUpdate:modelValue": r[7] || (r[7] = (V) => z.name = V),
                      placeholder: e(t)("access.namePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"])),
                a(Ae, {
                  label: e(t)("access.permsLabel")
                }, {
                  default: n(() => [
                    w("div", Po, [
                      w("span", Uo, l(e(t)("access.presets")), 1),
                      a(St, {
                        modelValue: oe.value,
                        "onUpdate:modelValue": r[8] || (r[8] = (V) => oe.value = V),
                        size: "small",
                        onChange: ne
                      }, {
                        default: n(() => [
                          a(Ie, { value: "admin" }, {
                            default: n(() => [
                              m(l(e(t)("access.presetAdmin")), 1)
                            ]),
                            _: 1
                          }),
                          a(Ie, { value: "member" }, {
                            default: n(() => [
                              m(l(e(t)("access.presetMember")), 1)
                            ]),
                            _: 1
                          }),
                          a(Ie, { value: "viewer" }, {
                            default: n(() => [
                              m(l(e(t)("access.presetViewer")), 1)
                            ]),
                            _: 1
                          }),
                          a(Ie, { value: "custom" }, {
                            default: n(() => [
                              m(l(e(t)("access.presetCustom")), 1)
                            ]),
                            _: 1
                          })
                        ]),
                        _: 1
                      }, 8, ["modelValue"])
                    ]),
                    w("div", Do, [
                      (f(), I(le, null, ce(d, (V) => a(Et, {
                        key: V.key,
                        modelValue: z.caps[V.key],
                        "onUpdate:modelValue": (te) => z.caps[V.key] = te,
                        onChange: F
                      }, {
                        default: n(() => [
                          m(l(X(V.key)), 1)
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
        a(nt, {
          modelValue: ee.value,
          "onUpdate:modelValue": r[13] || (r[13] = (V) => ee.value = V),
          title: G.value,
          width: e(W) ? "96%" : "560px"
        }, {
          footer: n(() => [
            a(q, {
              type: "primary",
              onClick: r[12] || (r[12] = () => {
                kt(_e.value), ee.value = !1;
              })
            }, {
              default: n(() => [
                m(l(e(t)("access.copyToken")), 1)
              ]),
              _: 1
            })
          ]),
          default: n(() => [
            w("p", null, l(e(t)("access.created")), 1),
            w("code", Ao, l(_e.value), 1)
          ]),
          _: 1
        }, 8, ["modelValue", "title", "width"])
      ], 512);
    };
  }
}), Ko = /* @__PURE__ */ he(Io, [["__scopeId", "data-v-85d0fef7"]]);
export {
  Ko as AdminPanel,
  bt as MarkdownView,
  Fa as MemoriesPanel,
  jo as MemoryAdmin,
  Ua as MemoryDetailDrawer,
  Ea as MemoryEditorDialog,
  ft as MemoryUIConfigKey,
  lo as OpsPanel,
  Qa as TagsPanel,
  da as applyMemoryUILocalePreference,
  fa as buildMemoriesQuery,
  pa as createApiClient,
  Ho as currentMemoryUILocale,
  ma as formatSize,
  ye as formatTime,
  Ke as isSearchMode,
  Fe as memoryUIi18n,
  qo as provideMemoryUI,
  _a as renderMarkdown,
  ga as sanitizeHtml,
  ca as setMemoryUILocale,
  t,
  ae as toastError,
  se as toastSuccess,
  mo as useAdmin,
  pe as useApiClient,
  wa as useMemories,
  vt as useMemoryConfig,
  to as useOps,
  Ha as useTags
};
