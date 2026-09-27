import { inject as bt, provide as yt, ref as f, computed as se, onMounted as we, watch as pe, onUnmounted as ht, defineComponent as H, openBlock as v, createElementBlock as B, createBlock as V, unref as e, withCtx as o, createVNode as n, createElementVNode as A, toDisplayString as u, createTextVNode as k, Fragment as ee, renderList as ce, createCommentVNode as N, withKeys as kt, isRef as re, withDirectives as fe, reactive as et, renderSlot as wt, vShow as Be, withModifiers as ze } from "vue";
import { createI18n as Ct } from "vue-i18n";
import { ElDialog as Ae, ElForm as Ce, ElFormItem as Te, ElInput as _e, ElRadioGroup as Ve, ElRadioButton as Re, ElSelect as tt, ElOption as at, ElButton as G, ElDrawer as Tt, ElTag as Ie, ElDivider as St, ElTooltip as ge, ElIcon as be, ElAlert as Se, ElTable as Oe, ElTableColumn as Ne, ElLoadingDirective as nt, ElPagination as Et, ElRadio as $t, ElRow as At, ElCol as Vt, ElCard as de, ElStatistic as It, ElDescriptions as ot, ElDescriptionsItem as lt, ElContainer as xt, ElAside as Ut, ElMenu as Dt, ElMenuItem as Mt, ElMain as Pt, ElTabs as Bt, ElTabPane as Lt, ElSwitch as st, ElEmpty as it, ElCheckbox as zt } from "element-plus/es";
import { InfoFilled as Ee, Plus as Fe, Search as He, Refresh as rt, Collection as Rt, Notebook as Ot, PriceTag as Nt, Odometer as Ft, Download as Ht, UploadFilled as qt } from "@element-plus/icons-vue";
import { ElMessage as Kt, ElMessageBox as ke } from "element-plus";
import { Marked as jt } from "marked";
import ct from "dompurify";
const Jt = {
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
    modeAuto: "自动",
    modeKeyword: "仅关键词",
    modeHybrid: "关键词 + 语义",
    semanticFallback: "语义搜索服务不可用，本次结果仅来自关键词匹配",
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
    imported: "已导入 {memories} 条记忆、{tags} 个标签",
    embeddingCoverageTitle: "向量覆盖率",
    embeddingModel: "模型",
    embeddingCoverage: "已向量化 / 记忆总数",
    embeddingPending: "{count} 条记忆缺最新向量，点击「补跑向量化」重建",
    runBackfill: "补跑向量化",
    embeddingDisabled: "未启用：开启后此处显示向量覆盖率",
    embeddingDone: "已补跑 {count} 条记忆的向量",
    embeddingUpToDate: "全部记忆已是最新向量，无需补跑",
    embeddingNotConfigured: "尚未配置 embedding 服务",
    embeddingTitle: "语义搜索（embedding）",
    embeddingHint: "配置 OpenAI 兼容的 /embeddings 服务：云端（如 SiliconFlow：https://api.siliconflow.cn/v1 + BAAI/bge-m3）或本地 Ollama（http://127.0.0.1:11434/v1 + bge-m3）。服务不可用时搜索自动回退关键词，写入不中断；缺向量的记忆可点击下方「补跑向量化」重建。",
    embeddingEnabledLabel: "启用语义搜索",
    embeddingBaseUrl: "服务地址（base_url，OpenAI 兼容）",
    embeddingModelLabel: "模型名",
    embeddingApiKeyLabel: "API Key",
    embeddingApiKeyPlaceholder: "本地服务可留空",
    embeddingSaveTest: "保存并测试连接",
    embeddingTestOk: "连接成功：向量维度 {dim}，耗时 {ms} ms",
    embeddingTestFail: "连接失败：{error}"
  }
}, Wt = {
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
    modeAuto: "Auto",
    modeKeyword: "Keyword only",
    modeHybrid: "Keyword + semantic",
    semanticFallback: "Semantic search is unavailable; these results come from keyword matching only",
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
    imported: "Imported {memories} memories and {tags} tags",
    embeddingCoverageTitle: "Vector coverage",
    embeddingModel: "Model",
    embeddingCoverage: "Embedded / total memories",
    embeddingPending: '{count} memories lack up-to-date vectors; click "Backfill embeddings" to rebuild',
    runBackfill: "Backfill embeddings",
    embeddingDisabled: "Not enabled: vector coverage appears here once the service is on",
    embeddingDone: "Backfilled embeddings for {count} memories",
    embeddingUpToDate: "All memories already have up-to-date embeddings",
    embeddingNotConfigured: "Embedding service is not configured",
    embeddingTitle: "Semantic search (embedding)",
    embeddingHint: 'Configure an OpenAI-compatible /embeddings service: cloud (e.g. SiliconFlow: https://api.siliconflow.cn/v1 + BAAI/bge-m3) or local Ollama (http://127.0.0.1:11434/v1 + bge-m3). If the service is unavailable, search falls back to keywords and writes are never blocked; missing vectors can be rebuilt via "Backfill embeddings" below.',
    embeddingEnabledLabel: "Enable semantic search",
    embeddingBaseUrl: "Service base URL (OpenAI-compatible)",
    embeddingModelLabel: "Model name",
    embeddingApiKeyLabel: "API key",
    embeddingApiKeyPlaceholder: "Leave empty for local services",
    embeddingSaveTest: "Save & test connection",
    embeddingTestOk: "Connected: embedding dimension {dim}, {ms} ms",
    embeddingTestFail: "Connection failed: {error}"
  }
};
function dt() {
  var t;
  return typeof navigator > "u" || (t = navigator.language) != null && t.toLowerCase().startsWith("zh") ? "zh" : "en";
}
const xe = Ct({
  legacy: !1,
  locale: dt(),
  fallbackLocale: "zh",
  messages: { zh: Jt, en: Wt },
  // 面向宿主组件库，缺 key 时静默回退即可，不刷控制台
  missingWarn: !1,
  fallbackWarn: !1
}), a = xe.global.t;
function Gt(t) {
  xe.global.locale.value = t;
}
function Qt(t) {
  Gt(t === "auto" ? dt() : t);
}
function Xn() {
  return xe.global.locale.value;
}
const ut = Symbol("memory-ui-config"), $e = {
  baseUrl: "",
  fetch: (...t) => globalThis.fetch(...t),
  defaultPageSize: 20,
  locale: "auto"
};
function Yn(t) {
  t.locale && Qt(t.locale), yt(ut, t);
}
function Ue() {
  const t = bt(ut);
  return {
    baseUrl: ((t == null ? void 0 : t.baseUrl) ?? $e.baseUrl).replace(/\/+$/, ""),
    fetch: (t == null ? void 0 : t.fetch) ?? $e.fetch,
    defaultPageSize: (t == null ? void 0 : t.defaultPageSize) ?? $e.defaultPageSize,
    locale: (t == null ? void 0 : t.locale) ?? $e.locale,
    onIdentityToken: t == null ? void 0 : t.onIdentityToken,
    onAuthChanged: t == null ? void 0 : t.onAuthChanged
  };
}
const Xt = 72;
function mt(t, s) {
  let l;
  return l = Kt({
    type: t,
    message: s,
    offset: Xt,
    showClose: !0,
    grouping: !0,
    ...{ onClick: () => l.close() }
  }), l;
}
function K(t) {
  return mt("success", t);
}
function j(t) {
  return mt("error", t);
}
function ve(t) {
  if (!t) return "—";
  const s = xe.global.locale.value === "en" ? "en-US" : "zh-CN";
  return new Date(t * 1e3).toLocaleString(s, { hour12: !1 });
}
function Yt(t) {
  return t == null ? "—" : t < 1024 ? `${t} B` : t < 1024 * 1024 ? `${(t / 1024).toFixed(1)} KB` : `${(t / 1024 / 1024).toFixed(2)} MB`;
}
function Zt(t) {
  async function s(c, p = {}) {
    const m = await t.fetch(t.baseUrl + c, {
      headers: { "Content-Type": "application/json" },
      ...p
    }), _ = await m.text();
    let d = null;
    try {
      d = _ ? JSON.parse(_) : null;
    } catch {
      d = null;
    }
    if (!m.ok) {
      const r = (d == null ? void 0 : d.error) ?? a("errors.http", { status: m.status });
      throw new Error(r);
    }
    return d;
  }
  async function l(c) {
    var m;
    const p = await t.fetch(t.baseUrl + c);
    if (!p.ok) {
      const _ = await p.text().catch(() => "");
      let d = a("errors.http", { status: p.status });
      try {
        d = ((m = JSON.parse(_)) == null ? void 0 : m.error) ?? d;
      } catch {
      }
      throw new Error(d);
    }
    return p.blob();
  }
  return {
    get: (c) => s(c),
    post: (c, p) => s(c, { method: "POST", body: JSON.stringify(p ?? {}) }),
    put: (c, p) => s(c, { method: "PUT", body: JSON.stringify(p ?? {}) }),
    del: (c) => s(c, { method: "DELETE" }),
    getBlob: l
  };
}
function te() {
  return Zt(Ue());
}
function ea(t) {
  const s = Le(t.query), l = new URLSearchParams();
  return s ? (l.set("query", t.query.trim()), t.tagFilter && l.set("tags", t.tagFilter), t.mode && t.mode !== "auto" && l.set("mode", t.mode)) : (t.tagFilter && l.set("tag", t.tagFilter), l.set("sort", t.sort), l.set("order", t.order)), l.set("offset", String((t.page - 1) * t.pageSize)), l.set("limit", String(t.pageSize)), l.toString();
}
function Le(t) {
  return t.trim().length > 0;
}
const ta = new jt();
function aa(t) {
  const s = ta.parse(t, { async: !1 });
  return ct.sanitize(s);
}
function na(t) {
  return ct.sanitize(t);
}
function qe(t, s) {
  return t.get(`/api/memories/${encodeURIComponent(s)}`);
}
function pt(t, s) {
  return t.post("/api/memories", s);
}
function ft(t, s, l) {
  return t.put(`/api/memories/${encodeURIComponent(s)}`, l);
}
function oa(t, s) {
  return t.del(`/api/memories/${encodeURIComponent(s)}`);
}
function vt(t, s) {
  const l = s ? `?filter=${encodeURIComponent(s)}` : "";
  return t.get(`/api/tags${l}`);
}
function la(t, s, l) {
  return t.post("/api/tags", { name: s, description: l });
}
function sa(t, s, l, c) {
  const p = { description: c }, m = l.trim();
  return m && m !== s && (p.new_name = m), t.put(`/api/tags/${encodeURIComponent(s)}`, p);
}
function ia(t, s, l) {
  return t.del(`/api/tags/${encodeURIComponent(s)}?mode=${l}`);
}
function ra() {
  const t = te(), { defaultPageSize: s } = Ue(), l = f(""), c = f(""), p = f("auto"), m = f("updated_at"), _ = f("desc"), d = f(1), r = f(s), g = f([]), b = f([]), i = f(""), y = f(0), E = f(!1), I = f([]), $ = se(() => Le(l.value));
  function w() {
    return d.value = 1, M();
  }
  function D() {
    return ea({
      query: l.value,
      tagFilter: c.value,
      mode: p.value,
      sort: m.value,
      order: _.value,
      page: d.value,
      pageSize: r.value
    });
  }
  let P = 0;
  async function M() {
    var U;
    const R = ++P;
    E.value = !0;
    try {
      const h = D();
      if (Le(l.value)) {
        const S = await t.get(`/api/memories?${h}`);
        if (R !== P) return;
        b.value = (S.results ?? []).map((x) => ({ ...x, snippet: na(x.snippet) })), y.value = S.total_matches ?? 0, i.value = S.semantic_fallback ? a("memories.semanticFallback") : "";
      } else {
        const S = await t.get(`/api/memories?${h}`);
        if (R !== P) return;
        const x = Math.max(1, Math.ceil(S.total / r.value));
        if (((U = S.memories) == null ? void 0 : U.length) === 0 && S.total > 0 && d.value > x)
          return d.value = x, E.value = !1, M();
        g.value = S.memories ?? [], y.value = S.total ?? 0, i.value = "";
      }
    } finally {
      R === P && (E.value = !1);
    }
  }
  async function O() {
    try {
      const R = await vt(t);
      I.value = (R.tags ?? []).map((U) => U.name);
    } catch {
    }
  }
  async function z(R) {
    if (R.id) {
      const U = await qe(t, R.id), h = new Set(U.tags), S = new Set(R.tags);
      await ft(t, R.id, {
        summary: R.summary,
        content: R.content,
        add_tags: [...S].filter((x) => !h.has(x)),
        remove_tags: [...h].filter((x) => !S.has(x))
      });
    } else
      await pt(t, { summary: R.summary, content: R.content, tags: R.tags });
    await Promise.all([M(), O()]);
  }
  async function oe(R) {
    await oa(t, R), await M();
  }
  return we(() => {
    M().catch(() => {
    }), O();
  }), {
    query: l,
    tagFilter: c,
    mode: p,
    sort: m,
    order: _,
    page: d,
    pageSize: r,
    rows: g,
    searchResults: b,
    note: i,
    total: y,
    loading: E,
    tagOptions: I,
    searching: $,
    onSearch: w,
    reload: M,
    loadTagOptions: O,
    saveMemory: z,
    removeMemory: oe
  };
}
function De(t) {
  const s = f(0);
  let l = null;
  function c(_) {
    l == null || l.disconnect(), l = null, !(!_ || typeof ResizeObserver > "u") && (l = new ResizeObserver((d) => {
      var r;
      s.value = ((r = d[0]) == null ? void 0 : r.contentRect.width) ?? 0;
    }), l.observe(_));
  }
  we(() => c(t.value)), pe(t, (_) => c(_)), ht(() => l == null ? void 0 : l.disconnect());
  const p = se(() => s.value > 0 && s.value < 960), m = se(() => s.value > 0 && s.value < 720);
  return { width: s, compact: p, narrow: m };
}
const ca = ["innerHTML"], _t = /* @__PURE__ */ H({
  __name: "MarkdownView",
  props: {
    source: {}
  },
  setup(t) {
    const s = t, l = se(() => aa(s.source));
    return (c, p) => (v(), B("div", {
      class: "md-body",
      innerHTML: l.value
    }, null, 8, ca));
  }
}), da = { class: "content-label" }, ua = /* @__PURE__ */ H({
  __name: "MemoryEditorDialog",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    tagOptions: {},
    width: {}
  },
  emits: ["update:visible", "saved"],
  setup(t, { emit: s }) {
    const l = t, c = s, p = te(), m = f(!1), _ = f("edit"), d = f({ id: null, summary: "", content: "", tags: [] });
    let r = [];
    pe(
      () => l.visible,
      async (b) => {
        if (b)
          if (_.value = "edit", l.memoryId)
            try {
              const i = await qe(p, l.memoryId);
              d.value = { id: i.id, summary: i.summary, content: i.content, tags: [...i.tags] }, r = [...i.tags];
            } catch (i) {
              j(i instanceof Error ? i.message : String(i)), c("update:visible", !1);
            }
          else
            d.value = { id: null, summary: "", content: "", tags: [] }, r = [];
      }
    );
    async function g() {
      m.value = !0;
      try {
        if (d.value.id) {
          const b = new Set(r), i = new Set(d.value.tags);
          await ft(p, d.value.id, {
            summary: d.value.summary,
            content: d.value.content,
            add_tags: [...i].filter((y) => !b.has(y)),
            remove_tags: [...b].filter((y) => !i.has(y))
          }), K(a("editor.updated"));
        } else
          await pt(p, {
            summary: d.value.summary,
            content: d.value.content,
            tags: d.value.tags
          }), K(a("editor.created"));
        c("update:visible", !1), c("saved");
      } catch (b) {
        j(b instanceof Error ? b.message : String(b));
      } finally {
        m.value = !1;
      }
    }
    return (b, i) => {
      const y = _e, E = Te, I = Re, $ = Ve, w = at, D = tt, P = Ce, M = G, O = Ae;
      return v(), V(O, {
        "model-value": t.visible,
        title: d.value.id ? e(a)("editor.editTitle") : e(a)("editor.createTitle"),
        width: t.width,
        "onUpdate:modelValue": i[5] || (i[5] = (z) => c("update:visible", z))
      }, {
        footer: o(() => [
          n(M, {
            onClick: i[4] || (i[4] = (z) => c("update:visible", !1))
          }, {
            default: o(() => [
              k(u(e(a)("common.cancel")), 1)
            ]),
            _: 1
          }),
          n(M, {
            type: "primary",
            loading: m.value,
            onClick: g
          }, {
            default: o(() => [
              k(u(e(a)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: o(() => [
          n(P, { "label-position": "top" }, {
            default: o(() => [
              n(E, {
                label: e(a)("editor.summaryLabel")
              }, {
                default: o(() => [
                  n(y, {
                    modelValue: d.value.summary,
                    "onUpdate:modelValue": i[0] || (i[0] = (z) => d.value.summary = z),
                    maxlength: "512",
                    "show-word-limit": "",
                    placeholder: e(a)("editor.summaryPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"]),
              n(E, null, {
                label: o(() => [
                  A("div", da, [
                    A("span", null, u(e(a)("editor.contentLabel")), 1),
                    n($, {
                      modelValue: _.value,
                      "onUpdate:modelValue": i[1] || (i[1] = (z) => _.value = z),
                      size: "small"
                    }, {
                      default: o(() => [
                        n(I, { value: "edit" }, {
                          default: o(() => [
                            k(u(e(a)("editor.tabEdit")), 1)
                          ]),
                          _: 1
                        }),
                        n(I, { value: "preview" }, {
                          default: o(() => [
                            k(u(e(a)("editor.tabPreview")), 1)
                          ]),
                          _: 1
                        })
                      ]),
                      _: 1
                    }, 8, ["modelValue"])
                  ])
                ]),
                default: o(() => [
                  _.value === "edit" ? (v(), V(y, {
                    key: 0,
                    modelValue: d.value.content,
                    "onUpdate:modelValue": i[2] || (i[2] = (z) => d.value.content = z),
                    type: "textarea",
                    rows: 12,
                    maxlength: "262144",
                    "show-word-limit": "",
                    placeholder: e(a)("editor.contentPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])) : (v(), V(_t, {
                    key: 1,
                    class: "content-preview",
                    source: d.value.content
                  }, null, 8, ["source"]))
                ]),
                _: 1
              }),
              n(E, {
                label: e(a)("editor.tagsLabel")
              }, {
                default: o(() => [
                  n(D, {
                    modelValue: d.value.tags,
                    "onUpdate:modelValue": i[3] || (i[3] = (z) => d.value.tags = z),
                    multiple: "",
                    filterable: "",
                    "allow-create": "",
                    "default-first-option": "",
                    placeholder: e(a)("editor.tagsPlaceholder"),
                    class: "tags-select"
                  }, {
                    default: o(() => [
                      (v(!0), B(ee, null, ce(t.tagOptions, (z) => (v(), V(w, {
                        key: z,
                        label: z,
                        value: z
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
}), J = (t, s) => {
  const l = t.__vccOpts || t;
  for (const [c, p] of s)
    l[c] = p;
  return l;
}, ma = /* @__PURE__ */ J(ua, [["__scopeId", "data-v-a0fec0dd"]]), pa = { class: "detail-summary" }, fa = { class: "detail-tags" }, va = { class: "detail-toolbar" }, _a = {
  key: 1,
  class: "detail-content"
}, ga = /* @__PURE__ */ H({
  __name: "MemoryDetailDrawer",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    size: {}
  },
  emits: ["update:visible"],
  setup(t, { emit: s }) {
    const l = t, c = s, p = te(), m = f(null), _ = f("rendered");
    return pe(
      () => [l.visible, l.memoryId],
      async ([d]) => {
        if (!(!d || !l.memoryId)) {
          _.value = "rendered";
          try {
            m.value = await qe(p, l.memoryId);
          } catch (r) {
            j(r instanceof Error ? r.message : String(r)), c("update:visible", !1);
          }
        }
      }
    ), (d, r) => {
      var I;
      const g = Ie, b = St, i = Re, y = Ve, E = Tt;
      return v(), V(E, {
        "model-value": t.visible,
        title: e(a)("drawer.title", { id: ((I = m.value) == null ? void 0 : I.id) ?? t.memoryId ?? "" }),
        size: t.size,
        "onUpdate:modelValue": r[1] || (r[1] = ($) => c("update:visible", $))
      }, {
        default: o(() => [
          m.value ? (v(), B(ee, { key: 0 }, [
            A("h3", pa, u(m.value.summary), 1),
            A("div", fa, [
              (v(!0), B(ee, null, ce(m.value.tags, ($) => (v(), V(g, {
                key: $,
                size: "small",
                class: "am-tag"
              }, {
                default: o(() => [
                  k(u($), 1)
                ]),
                _: 2
              }, 1024))), 128))
            ]),
            n(b),
            A("div", va, [
              n(y, {
                modelValue: _.value,
                "onUpdate:modelValue": r[0] || (r[0] = ($) => _.value = $),
                size: "small"
              }, {
                default: o(() => [
                  n(i, { value: "rendered" }, {
                    default: o(() => [
                      k(u(e(a)("drawer.rendered")), 1)
                    ]),
                    _: 1
                  }),
                  n(i, { value: "source" }, {
                    default: o(() => [
                      k(u(e(a)("drawer.source")), 1)
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["modelValue"])
            ]),
            _.value === "rendered" ? (v(), V(_t, {
              key: 0,
              source: m.value.content
            }, null, 8, ["source"])) : (v(), B("pre", _a, u(m.value.content), 1))
          ], 64)) : N("", !0)
        ]),
        _: 1
      }, 8, ["model-value", "title", "size"]);
    };
  }
}), ba = /* @__PURE__ */ J(ga, [["__scopeId", "data-v-613d4260"]]), ya = {
  key: 0,
  class: "am-panel-header"
}, ha = { class: "am-heading" }, ka = { class: "am-panel-title" }, wa = { class: "am-toolbar" }, Ca = { class: "am-summary" }, Ta = ["innerHTML"], Sa = {
  key: 5,
  class: "am-pager"
}, Ea = {
  key: 6,
  class: "am-pager"
}, $a = /* @__PURE__ */ H({
  __name: "MemoriesPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: s }) {
    const l = t, {
      query: c,
      tagFilter: p,
      mode: m,
      sort: _,
      order: d,
      page: r,
      pageSize: g,
      rows: b,
      searchResults: i,
      note: y,
      total: E,
      loading: I,
      tagOptions: $,
      searching: w,
      onSearch: D,
      reload: P,
      loadTagOptions: M,
      removeMemory: O
    } = ra(), z = se(() => {
      if (I.value || E.value !== 0) return "";
      if (w.value) return a("memories.searchEmpty");
      const q = p.value.trim();
      return q ? a("memories.tagEmpty", { tag: q }) : "";
    }), oe = f(null), { compact: R, narrow: U } = De(oe), h = f(!1), S = f(null), x = f(!1), Q = f(null);
    function F(q) {
      return q().catch((C) => j(C instanceof Error ? C.message : String(C)));
    }
    function X() {
      S.value = null, h.value = !0;
    }
    function Y(q) {
      S.value = q, h.value = !0;
    }
    function ye(q) {
      Q.value = q, x.value = !0;
    }
    function ie() {
      F(P), M();
    }
    function Me(q) {
      const { prop: C, order: ue } = q;
      ue && (C === "updated_at" || C === "created_at" || C === "id") ? (_.value = C, d.value = ue === "ascending" ? "asc" : "desc") : (_.value = "updated_at", d.value = "desc"), F(P);
    }
    async function he(q) {
      try {
        await ke.confirm(a("memories.deleteConfirm", { id: q.id }), a("memories.deleteTitle"), {
          type: "warning"
        });
      } catch {
        return;
      }
      try {
        await O(q.id), K(a("memories.deleted"));
      } catch (C) {
        j(C instanceof Error ? C.message : String(C));
      }
    }
    return s({ refresh: () => F(P) }), (q, C) => {
      const ue = be, Pe = ge, le = G, L = _e, ae = at, je = tt, Je = Se, Z = Ne, We = Ie, Ge = Oe, Qe = Et, Xe = nt;
      return v(), B("div", {
        ref_key: "rootRef",
        ref: oe,
        class: "am-panel"
      }, [
        t.showHeader ? (v(), B("div", ya, [
          A("div", ha, [
            A("h2", ka, u(l.title ?? e(a)("memories.title")), 1),
            n(Pe, {
              content: l.subtitle ?? e(a)("memories.subtitle"),
              placement: "top"
            }, {
              default: o(() => [
                n(ue, { class: "am-info" }, {
                  default: o(() => [
                    n(e(Ee))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          n(le, {
            type: "primary",
            icon: e(Fe),
            onClick: X
          }, {
            default: o(() => [
              k(u(e(a)("memories.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : N("", !0),
        A("div", wa, [
          n(L, {
            modelValue: e(c),
            "onUpdate:modelValue": C[1] || (C[1] = (T) => re(c) ? c.value = T : null),
            placeholder: e(a)("memories.searchPlaceholder"),
            clearable: "",
            class: "search",
            onKeyup: C[2] || (C[2] = kt((T) => F(e(D)), ["enter"])),
            onClear: C[3] || (C[3] = (T) => F(e(D)))
          }, {
            append: o(() => [
              n(le, {
                icon: e(He),
                onClick: C[0] || (C[0] = (T) => F(e(D)))
              }, null, 8, ["icon"])
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          n(je, {
            modelValue: e(p),
            "onUpdate:modelValue": C[4] || (C[4] = (T) => re(p) ? p.value = T : null),
            placeholder: e(a)("memories.tagFilter"),
            clearable: "",
            filterable: "",
            class: "tag-filter",
            onChange: C[5] || (C[5] = (T) => F(e(D)))
          }, {
            default: o(() => [
              (v(!0), B(ee, null, ce(e($), (T) => (v(), V(ae, {
                key: T,
                label: T,
                value: T
              }, null, 8, ["label", "value"]))), 128))
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          n(je, {
            modelValue: e(m),
            "onUpdate:modelValue": C[6] || (C[6] = (T) => re(m) ? m.value = T : null),
            class: "mode-select",
            onChange: C[7] || (C[7] = (T) => F(e(D)))
          }, {
            default: o(() => [
              n(ae, {
                label: e(a)("memories.modeAuto"),
                value: "auto"
              }, null, 8, ["label"]),
              n(ae, {
                label: e(a)("memories.modeKeyword"),
                value: "keyword"
              }, null, 8, ["label"]),
              n(ae, {
                label: e(a)("memories.modeHybrid"),
                value: "hybrid"
              }, null, 8, ["label"])
            ]),
            _: 1
          }, 8, ["modelValue"])
        ]),
        e(y) ? (v(), V(Je, {
          key: 1,
          title: e(y),
          type: "warning",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : N("", !0),
        z.value ? (v(), V(Je, {
          key: 2,
          title: z.value,
          type: "info",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : N("", !0),
        e(w) ? fe((v(), V(Ge, {
          key: 3,
          data: e(i)
        }, {
          default: o(() => [
            n(Z, {
              prop: "id",
              label: e(a)("memories.colId"),
              width: "80"
            }, null, 8, ["label"]),
            n(Z, {
              label: e(a)("memories.colSummary")
            }, {
              default: o(({ row: T }) => [
                A("div", Ca, u(T.summary), 1),
                A("div", {
                  class: "am-snippet",
                  innerHTML: T.snippet
                }, null, 8, Ta)
              ]),
              _: 1
            }, 8, ["label"]),
            n(Z, {
              label: e(a)("memories.colTags"),
              "min-width": "150"
            }, {
              default: o(({ row: T }) => [
                (v(!0), B(ee, null, ce(T.tags, (ne) => (v(), V(We, {
                  key: ne,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: o(() => [
                    k(u(ne), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            n(Z, {
              prop: "score",
              label: e(a)("memories.colScore"),
              width: "80",
              sortable: ""
            }, null, 8, ["label"]),
            n(Z, {
              label: e(a)("memories.colUpdatedAt"),
              width: "170"
            }, {
              default: o(({ row: T }) => [
                k(u(e(ve)(T.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            n(Z, {
              label: e(a)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: o(({ row: T }) => [
                n(le, {
                  link: "",
                  type: "primary",
                  onClick: (ne) => ye(T.id)
                }, {
                  default: o(() => [
                    k(u(e(a)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                n(le, {
                  link: "",
                  type: "primary",
                  onClick: (ne) => Y(T.id)
                }, {
                  default: o(() => [
                    k(u(e(a)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                n(le, {
                  link: "",
                  type: "danger",
                  onClick: (ne) => he(T)
                }, {
                  default: o(() => [
                    k(u(e(a)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [Xe, e(I)]
        ]) : fe((v(), V(Ge, {
          key: 4,
          data: e(b),
          "default-sort": { prop: e(_), order: e(d) === "asc" ? "ascending" : "descending" },
          onSortChange: Me
        }, {
          default: o(() => [
            n(Z, {
              prop: "id",
              label: e(a)("memories.colId"),
              width: "80",
              sortable: "custom"
            }, null, 8, ["label"]),
            n(Z, {
              prop: "summary",
              label: e(a)("memories.colSummary"),
              "min-width": "180",
              "show-overflow-tooltip": ""
            }, null, 8, ["label"]),
            n(Z, {
              label: e(a)("memories.colTags"),
              "min-width": "150"
            }, {
              default: o(({ row: T }) => [
                (v(!0), B(ee, null, ce(T.tags, (ne) => (v(), V(We, {
                  key: ne,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: o(() => [
                    k(u(ne), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            e(R) ? N("", !0) : (v(), V(Z, {
              key: 0,
              prop: "created_at",
              label: e(a)("memories.colCreatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: o(({ row: T }) => [
                k(u(e(ve)(T.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label"])),
            n(Z, {
              prop: "updated_at",
              label: e(a)("memories.colUpdatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: o(({ row: T }) => [
                k(u(e(ve)(T.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            n(Z, {
              label: e(a)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: o(({ row: T }) => [
                n(le, {
                  link: "",
                  type: "primary",
                  onClick: (ne) => ye(T.id)
                }, {
                  default: o(() => [
                    k(u(e(a)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                n(le, {
                  link: "",
                  type: "primary",
                  onClick: (ne) => Y(T.id)
                }, {
                  default: o(() => [
                    k(u(e(a)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                n(le, {
                  link: "",
                  type: "danger",
                  onClick: (ne) => he(T)
                }, {
                  default: o(() => [
                    k(u(e(a)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data", "default-sort"])), [
          [Xe, e(I)]
        ]),
        e(w) ? (v(), B("div", Ea, [
          n(Qe, {
            "current-page": e(r),
            "onUpdate:currentPage": C[12] || (C[12] = (T) => re(r) ? r.value = T : null),
            "page-size": e(g),
            "onUpdate:pageSize": C[13] || (C[13] = (T) => re(g) ? g.value = T : null),
            total: e(E),
            "page-sizes": [10, 20, 50],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: C[14] || (C[14] = (T) => F(e(P))),
            onSizeChange: C[15] || (C[15] = (T) => F(e(P)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])) : (v(), B("div", Sa, [
          n(Qe, {
            "current-page": e(r),
            "onUpdate:currentPage": C[8] || (C[8] = (T) => re(r) ? r.value = T : null),
            "page-size": e(g),
            "onUpdate:pageSize": C[9] || (C[9] = (T) => re(g) ? g.value = T : null),
            total: e(E),
            "page-sizes": [20, 50, 100, 200],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: C[10] || (C[10] = (T) => F(e(P))),
            onSizeChange: C[11] || (C[11] = (T) => F(e(P)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])),
        n(ma, {
          visible: h.value,
          "onUpdate:visible": C[16] || (C[16] = (T) => h.value = T),
          "memory-id": S.value,
          "tag-options": e($),
          width: e(U) ? "96%" : "640px",
          onSaved: ie
        }, null, 8, ["visible", "memory-id", "tag-options", "width"]),
        n(ba, {
          visible: x.value,
          "onUpdate:visible": C[17] || (C[17] = (T) => x.value = T),
          "memory-id": Q.value,
          size: e(U) ? "100%" : "45%"
        }, null, 8, ["visible", "memory-id", "size"])
      ], 512);
    };
  }
}), Aa = /* @__PURE__ */ J($a, [["__scopeId", "data-v-8a3e1728"]]);
function Va() {
  const t = te(), s = f([]), l = f(!1), c = f("");
  async function p() {
    l.value = !0;
    try {
      const r = await vt(t, c.value.trim() || void 0);
      s.value = r.tags ?? [];
    } finally {
      l.value = !1;
    }
  }
  async function m(r, g) {
    await la(t, r, g), await p();
  }
  async function _(r, g, b) {
    await sa(t, r, g, b), await p();
  }
  async function d(r, g) {
    await ia(t, r, g), await p();
  }
  return we(() => {
    p().catch(() => {
    });
  }), { rows: s, loading: l, filter: c, reload: p, create: m, rename: _, remove: d };
}
const Ia = {
  key: 0,
  class: "am-panel-header"
}, xa = { class: "am-heading" }, Ua = { class: "am-panel-title" }, Da = { class: "delete-body" }, Ma = /* @__PURE__ */ H({
  __name: "TagsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: s }) {
    const l = t, { rows: c, loading: p, filter: m, reload: _, create: d, rename: r, remove: g } = Va();
    let b = null;
    function i() {
      b && clearTimeout(b), b = setTimeout(() => {
        b = null, _().catch((S) => j(S instanceof Error ? S.message : String(S)));
      }, 300);
    }
    const y = f(null), { narrow: E } = De(y), I = f(!1), $ = f(!1), w = et({ oldName: null, name: "", description: "" }), D = f(!1), P = f("detach"), M = f(null);
    function O(S) {
      return (x, Q) => (x[S] ?? 0) - (Q[S] ?? 0);
    }
    function z() {
      Object.assign(w, { oldName: null, name: "", description: "" }), $.value = !0;
    }
    function oe(S) {
      Object.assign(w, { oldName: S.name, name: S.name, description: S.description ?? "" }), $.value = !0;
    }
    async function R() {
      I.value = !0;
      try {
        w.oldName ? (await r(w.oldName, w.name, w.description), K(a("tags.saved"))) : (await d(w.name, w.description), K(a("tags.created"))), $.value = !1;
      } catch (S) {
        j(S instanceof Error ? S.message : String(S));
      } finally {
        I.value = !1;
      }
    }
    function U(S) {
      M.value = S, P.value = "detach", D.value = !0;
    }
    async function h() {
      var S, x;
      if (P.value === "purge")
        try {
          await ke.confirm(
            a("tags.purgeConfirm", { name: (S = M.value) == null ? void 0 : S.name, count: ((x = M.value) == null ? void 0 : x.memory_count) ?? 0 }),
            a("tags.purgeConfirmTitle"),
            { type: "error", confirmButtonText: a("tags.purgeButton") }
          );
        } catch {
          return;
        }
      if (M.value) {
        I.value = !0;
        try {
          await g(M.value.name, P.value), K(a("tags.deleted")), D.value = !1;
        } catch (Q) {
          j(Q instanceof Error ? Q.message : String(Q));
        } finally {
          I.value = !1;
        }
      }
    }
    return s({
      refresh: () => _().catch((S) => j(S instanceof Error ? S.message : String(S)))
    }), (S, x) => {
      const Q = be, F = ge, X = G, Y = _e, ye = Ie, ie = Ne, Me = Oe, he = Te, q = Ce, C = Ae, ue = $t, Pe = Ve, le = nt;
      return v(), B("div", {
        ref_key: "rootRef",
        ref: y,
        class: "am-panel"
      }, [
        t.showHeader ? (v(), B("div", Ia, [
          A("div", xa, [
            A("h2", Ua, u(l.title ?? e(a)("tags.title")), 1),
            n(F, {
              content: l.subtitle ?? e(a)("tags.subtitle"),
              placement: "top"
            }, {
              default: o(() => [
                n(Q, { class: "am-info" }, {
                  default: o(() => [
                    n(e(Ee))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          n(X, {
            type: "primary",
            icon: e(Fe),
            onClick: z
          }, {
            default: o(() => [
              k(u(e(a)("tags.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : N("", !0),
        n(Y, {
          modelValue: e(m),
          "onUpdate:modelValue": x[0] || (x[0] = (L) => re(m) ? m.value = L : null),
          class: "am-tag-filter",
          placeholder: e(a)("tags.filterPlaceholder"),
          clearable: "",
          "prefix-icon": e(He),
          onInput: i,
          onClear: i
        }, null, 8, ["modelValue", "placeholder", "prefix-icon"]),
        fe((v(), V(Me, { data: e(c) }, {
          default: o(() => [
            n(ie, {
              prop: "name",
              label: e(a)("tags.colName"),
              "min-width": "140",
              sortable: ""
            }, {
              default: o(({ row: L }) => [
                n(ye, null, {
                  default: o(() => [
                    k(u(L.name), 1)
                  ]),
                  _: 2
                }, 1024)
              ]),
              _: 1
            }, 8, ["label"]),
            n(ie, {
              prop: "description",
              label: e(a)("tags.colDescription"),
              "min-width": "150",
              "show-overflow-tooltip": ""
            }, {
              default: o(({ row: L }) => [
                k(u(L.description || "—"), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            n(ie, {
              prop: "memory_count",
              label: e(a)("tags.colMemoryCount"),
              width: "90",
              sortable: ""
            }, null, 8, ["label"]),
            n(ie, {
              prop: "last_used_at",
              label: e(a)("tags.colLastUsed"),
              width: "170",
              sortable: "",
              "sort-method": O("last_used_at")
            }, {
              default: o(({ row: L }) => [
                k(u(e(ve)(L.last_used_at)), 1)
              ]),
              _: 1
            }, 8, ["label", "sort-method"]),
            n(ie, {
              prop: "created_at",
              label: e(a)("tags.colCreatedAt"),
              width: "170",
              sortable: "",
              "sort-method": O("created_at")
            }, {
              default: o(({ row: L }) => [
                k(u(e(ve)(L.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label", "sort-method"]),
            n(ie, {
              label: e(a)("memories.colActions"),
              width: "150",
              fixed: "right"
            }, {
              default: o(({ row: L }) => [
                n(X, {
                  link: "",
                  type: "primary",
                  onClick: (ae) => oe(L)
                }, {
                  default: o(() => [
                    k(u(e(a)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                n(X, {
                  link: "",
                  type: "danger",
                  onClick: (ae) => U(L)
                }, {
                  default: o(() => [
                    k(u(e(a)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [le, e(p)]
        ]),
        n(C, {
          modelValue: $.value,
          "onUpdate:modelValue": x[4] || (x[4] = (L) => $.value = L),
          title: w.oldName ? e(a)("tags.editTitle") : e(a)("tags.createTitle"),
          width: e(E) ? "96%" : "480px"
        }, {
          footer: o(() => [
            n(X, {
              onClick: x[3] || (x[3] = (L) => $.value = !1)
            }, {
              default: o(() => [
                k(u(e(a)("common.cancel")), 1)
              ]),
              _: 1
            }),
            n(X, {
              type: "primary",
              loading: I.value,
              onClick: R
            }, {
              default: o(() => [
                k(u(e(a)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: o(() => [
            n(q, { "label-position": "top" }, {
              default: o(() => [
                n(he, {
                  label: e(a)("tags.nameLabel")
                }, {
                  default: o(() => [
                    n(Y, {
                      modelValue: w.name,
                      "onUpdate:modelValue": x[1] || (x[1] = (L) => w.name = L),
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(a)("tags.namePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                n(he, {
                  label: e(a)("tags.descLabel")
                }, {
                  default: o(() => [
                    n(Y, {
                      modelValue: w.description,
                      "onUpdate:modelValue": x[2] || (x[2] = (L) => w.description = L),
                      type: "textarea",
                      rows: 3,
                      maxlength: "512",
                      "show-word-limit": "",
                      placeholder: e(a)("tags.descPlaceholder")
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
        n(C, {
          modelValue: D.value,
          "onUpdate:modelValue": x[7] || (x[7] = (L) => D.value = L),
          title: e(a)("tags.deleteTitle"),
          width: e(E) ? "96%" : "480px"
        }, {
          footer: o(() => [
            n(X, {
              onClick: x[6] || (x[6] = (L) => D.value = !1)
            }, {
              default: o(() => [
                k(u(e(a)("common.cancel")), 1)
              ]),
              _: 1
            }),
            n(X, {
              type: "danger",
              loading: I.value,
              onClick: h
            }, {
              default: o(() => [
                k(u(e(a)("common.delete")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: o(() => {
            var L;
            return [
              A("p", Da, [
                k(u(e(a)("tags.deleteBefore")) + " ", 1),
                n(ye, null, {
                  default: o(() => {
                    var ae;
                    return [
                      k(u((ae = M.value) == null ? void 0 : ae.name), 1)
                    ];
                  }),
                  _: 1
                }),
                k(" " + u(e(a)("tags.deleteMiddle")) + " ", 1),
                A("b", null, u((L = M.value) == null ? void 0 : L.memory_count), 1),
                k(" " + u(e(a)("tags.deleteAfter")), 1)
              ]),
              n(Pe, {
                modelValue: P.value,
                "onUpdate:modelValue": x[5] || (x[5] = (ae) => P.value = ae)
              }, {
                default: o(() => [
                  n(ue, { value: "detach" }, {
                    default: o(() => [
                      k(u(e(a)("tags.detach")), 1)
                    ]),
                    _: 1
                  }),
                  n(ue, { value: "purge" }, {
                    default: o(() => [
                      k(u(e(a)("tags.purge")), 1)
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
}), Pa = /* @__PURE__ */ J(Ma, [["__scopeId", "data-v-c8aeea57"]]);
function Ba(t) {
  return t.get("/api/stats");
}
function La(t) {
  return t.get("/health");
}
function za(t) {
  return t.get("/api/doctor");
}
function Ra(t) {
  return t.getBlob("/api/export");
}
function Oa(t, s) {
  return t.post("/api/import", s);
}
function Na(t) {
  return t.post("/api/embeddings/backfill", {});
}
function Fa() {
  const t = te(), s = f({}), l = f(""), c = se(() => Yt(s.value.file_size));
  async function p() {
    s.value = await Ba(t), s.value.version = l.value;
  }
  return we(async () => {
    try {
      l.value = (await La(t)).version ?? "";
    } catch {
    }
    await p().catch(() => {
    });
  }), {
    stats: s,
    version: l,
    sizeText: c,
    reload: p
  };
}
const Ha = {
  key: 0,
  class: "am-panel-header"
}, qa = { class: "am-heading" }, Ka = { class: "am-panel-title" }, ja = /* @__PURE__ */ H({
  __name: "OpsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: s }) {
    const l = t, { stats: c, version: p, sizeText: m, reload: _ } = Fa(), d = f(null), { narrow: r } = De(d);
    function g(b) {
      return b().catch((i) => j(i instanceof Error ? i.message : String(i)));
    }
    return s({ refresh: () => g(_) }), (b, i) => {
      const y = be, E = ge, I = G, $ = It, w = de, D = Vt, P = At, M = lt, O = ot;
      return v(), B("div", {
        ref_key: "rootRef",
        ref: d,
        class: "am-panel"
      }, [
        t.showHeader ? (v(), B("div", Ha, [
          A("div", qa, [
            A("h2", Ka, u(l.title ?? e(a)("ops.title")), 1),
            n(E, {
              content: l.subtitle ?? e(a)("ops.subtitle"),
              placement: "top"
            }, {
              default: o(() => [
                n(y, { class: "am-info" }, {
                  default: o(() => [
                    n(e(Ee))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          n(I, {
            icon: e(rt),
            onClick: i[0] || (i[0] = (z) => g(e(_)))
          }, {
            default: o(() => [
              k(u(e(a)("ops.refresh")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : N("", !0),
        n(P, { gutter: 14 }, {
          default: o(() => [
            n(D, {
              span: e(r) ? 12 : 8
            }, {
              default: o(() => [
                n(w, { shadow: "never" }, {
                  default: o(() => [
                    n($, {
                      title: e(a)("ops.statMemories"),
                      value: e(c).memories ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            n(D, {
              span: e(r) ? 12 : 8
            }, {
              default: o(() => [
                n(w, { shadow: "never" }, {
                  default: o(() => [
                    n($, {
                      title: e(a)("ops.statTags"),
                      value: e(c).tags ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            n(D, {
              span: e(r) ? 12 : 8
            }, {
              default: o(() => [
                n(w, { shadow: "never" }, {
                  default: o(() => [
                    n($, {
                      title: e(a)("ops.statSize"),
                      value: e(m)
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
        n(w, { shadow: "never" }, {
          header: o(() => [
            k(u(e(a)("ops.dbCard")), 1)
          ]),
          default: o(() => [
            n(O, {
              column: e(r) ? 1 : 2,
              border: ""
            }, {
              default: o(() => [
                n(M, {
                  label: e(a)("ops.version")
                }, {
                  default: o(() => [
                    k(u(e(c).version ?? e(p)), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                n(M, {
                  label: e(a)("ops.schemaVersion")
                }, {
                  default: o(() => [
                    k(u(e(c).schema_version ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                n(M, {
                  label: e(a)("ops.path"),
                  span: e(r) ? 1 : 2
                }, {
                  default: o(() => [
                    k(u(e(c).path ?? "—"), 1)
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
}), Ja = { class: "memory-ui" }, Wa = { class: "brand" }, Ga = { class: "brand-mark" }, Qa = { class: "aside-footer" }, Xa = /* @__PURE__ */ H({
  __name: "MemoryAdmin",
  props: {
    layout: { default: "sidebar" },
    title: { default: "Agent Memory" }
  },
  setup(t) {
    const s = f("memories");
    return (l, c) => {
      const p = be, m = Mt, _ = Dt, d = Ut, r = Lt, g = Bt, b = Pt, i = xt;
      return v(), B("div", Ja, [
        n(i, { class: "layout" }, {
          default: o(() => [
            t.layout === "sidebar" ? (v(), V(d, {
              key: 0,
              width: "200px",
              class: "aside"
            }, {
              default: o(() => [
                A("div", Wa, [
                  A("span", Ga, [
                    n(p, { size: 16 }, {
                      default: o(() => [
                        n(e(Rt))
                      ]),
                      _: 1
                    })
                  ]),
                  A("span", null, u(t.title), 1)
                ]),
                n(_, {
                  "default-active": s.value,
                  class: "menu",
                  onSelect: c[0] || (c[0] = (y) => s.value = y)
                }, {
                  default: o(() => [
                    n(m, { index: "memories" }, {
                      default: o(() => [
                        n(p, null, {
                          default: o(() => [
                            n(e(Ot))
                          ]),
                          _: 1
                        }),
                        A("span", null, u(e(a)("nav.memories")), 1)
                      ]),
                      _: 1
                    }),
                    n(m, { index: "tags" }, {
                      default: o(() => [
                        n(p, null, {
                          default: o(() => [
                            n(e(Nt))
                          ]),
                          _: 1
                        }),
                        A("span", null, u(e(a)("nav.tags")), 1)
                      ]),
                      _: 1
                    }),
                    n(m, { index: "ops" }, {
                      default: o(() => [
                        n(p, null, {
                          default: o(() => [
                            n(e(Ft))
                          ]),
                          _: 1
                        }),
                        A("span", null, u(e(a)("nav.ops")), 1)
                      ]),
                      _: 1
                    })
                  ]),
                  _: 1
                }, 8, ["default-active"]),
                A("div", Qa, [
                  wt(l.$slots, "footer", {}, void 0, !0)
                ])
              ]),
              _: 3
            })) : N("", !0),
            n(b, { class: "main" }, {
              default: o(() => [
                t.layout === "tabs" ? (v(), V(g, {
                  key: 0,
                  modelValue: s.value,
                  "onUpdate:modelValue": c[1] || (c[1] = (y) => s.value = y),
                  class: "tabs-bar"
                }, {
                  default: o(() => [
                    n(r, {
                      label: e(a)("nav.memories"),
                      name: "memories"
                    }, null, 8, ["label"]),
                    n(r, {
                      label: e(a)("nav.tags"),
                      name: "tags"
                    }, null, 8, ["label"]),
                    n(r, {
                      label: e(a)("nav.ops"),
                      name: "ops"
                    }, null, 8, ["label"])
                  ]),
                  _: 1
                }, 8, ["modelValue"])) : N("", !0),
                fe(n(Aa, null, null, 512), [
                  [Be, s.value === "memories"]
                ]),
                fe(n(Pa, null, null, 512), [
                  [Be, s.value === "tags"]
                ]),
                fe(n(ja, null, null, 512), [
                  [Be, s.value === "ops"]
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
}), Zn = /* @__PURE__ */ J(Xa, [["__scopeId", "data-v-38c91753"]]), me = [
  { key: "read", labelKey: "capRead" },
  { key: "create", labelKey: "capCreate" },
  { key: "update", labelKey: "capUpdate" },
  { key: "delete", labelKey: "capDelete" },
  { key: "tag_manage", labelKey: "capTagManage" },
  { key: "admin", labelKey: "capAdmin" }
];
function Ye() {
  return Object.fromEntries(me.map((t) => [t.key, !1]));
}
const Ya = {
  admin: me.map((t) => t.key),
  member: ["read", "create", "update", "tag_manage"],
  viewer: ["read"]
};
function gt(t) {
  const s = me.find((l) => l.key === t);
  return s ? a(`access.${s.labelKey}`) : t;
}
function Ze(t) {
  return me.map((s) => s.key).filter((s) => {
    var l;
    return ((l = t.permissions) == null ? void 0 : l[s]) === !0;
  });
}
function Za(t) {
  return t ? `…${t}` : "—";
}
async function W(t, s) {
  try {
    const l = await t();
    return s && K(s), l;
  } catch (l) {
    j(l instanceof Error ? l.message : String(l));
    return;
  }
}
const en = { class: "auth-row" }, tn = { class: "auth-text" }, an = { class: "auth-label" }, nn = { class: "auth-hint" }, on = /* @__PURE__ */ H({
  __name: "AuthSwitchCard",
  props: {
    authRequired: { type: Boolean },
    hasAdminIdentity: { type: Boolean }
  },
  setup(t) {
    const s = t, l = f(s.authRequired);
    pe(
      () => s.authRequired,
      (d) => {
        l.value = d;
      }
    );
    const c = f(!1), p = te(), m = Ue();
    async function _() {
      var r;
      const d = !l.value;
      if (d && !s.hasAdminIdentity)
        return j(a("access.enableBlocked")), !1;
      try {
        await ke.confirm(
          a(d ? "access.authEnableConfirm" : "access.authDisableConfirm"),
          a("access.authTitle"),
          {
            type: "warning",
            confirmButtonText: a("common.save"),
            cancelButtonText: a("common.cancel")
          }
        );
      } catch {
        return !1;
      }
      c.value = !0;
      try {
        return await W(() => p.put("/api/settings", { auth_required: d })) !== void 0 && ((r = m.onAuthChanged) == null || r.call(m, d)), !0;
      } finally {
        c.value = !1;
      }
    }
    return (d, r) => {
      const g = st, b = ge, i = de;
      return v(), V(i, { shadow: "never" }, {
        default: o(() => [
          A("div", en, [
            A("div", tn, [
              A("span", an, u(e(a)("access.authTitle")), 1),
              A("span", nn, u(e(a)("access.authHint")), 1)
            ]),
            n(b, {
              disabled: t.hasAdminIdentity,
              content: e(a)("access.enableBlocked"),
              placement: "top"
            }, {
              default: o(() => [
                n(g, {
                  modelValue: l.value,
                  "onUpdate:modelValue": r[0] || (r[0] = (y) => l.value = y),
                  "before-change": _,
                  loading: c.value,
                  disabled: !t.hasAdminIdentity
                }, null, 8, ["modelValue", "loading", "disabled"])
              ]),
              _: 1
            }, 8, ["disabled", "content"])
          ])
        ]),
        _: 1
      });
    };
  }
}), ln = /* @__PURE__ */ J(on, [["__scopeId", "data-v-ca2a1dcb"]]), sn = { class: "token-cell" }, rn = { class: "token-text" }, cn = {
  key: 0,
  class: "muted"
}, dn = /* @__PURE__ */ H({
  __name: "IdentityTableCard",
  props: {
    identities: {},
    compact: { type: Boolean }
  },
  emits: ["edit", "delete", "reset-token"],
  setup(t, { emit: s }) {
    const l = s;
    return (c, p) => {
      const m = it, _ = Ne, d = G, r = Ie, g = Oe, b = de;
      return v(), V(b, { shadow: "never" }, {
        default: o(() => [
          t.identities.length === 0 ? (v(), V(m, {
            key: 0,
            description: e(a)("access.empty")
          }, null, 8, ["description"])) : (v(), V(g, {
            key: 1,
            data: t.identities
          }, {
            default: o(() => [
              n(_, {
                prop: "name",
                label: e(a)("access.colName"),
                "min-width": "120"
              }, null, 8, ["label"]),
              n(_, {
                label: e(a)("access.colToken"),
                "min-width": "200"
              }, {
                default: o(({ row: i }) => [
                  A("div", sn, [
                    A("code", rn, u(e(Za)(i.token_hint)), 1),
                    n(d, {
                      link: "",
                      type: "primary",
                      onClick: (y) => l("reset-token", i)
                    }, {
                      default: o(() => [
                        k(u(e(a)("access.resetToken")), 1)
                      ]),
                      _: 1
                    }, 8, ["onClick"])
                  ])
                ]),
                _: 1
              }, 8, ["label"]),
              n(_, {
                label: e(a)("access.colPermissions"),
                "min-width": "240"
              }, {
                default: o(({ row: i }) => [
                  (v(!0), B(ee, null, ce(e(Ze)(i), (y) => (v(), V(r, {
                    key: y,
                    size: "small",
                    class: "cap-tag",
                    type: y === "admin" ? "danger" : "info"
                  }, {
                    default: o(() => [
                      k(u(e(gt)(y)), 1)
                    ]),
                    _: 2
                  }, 1032, ["type"]))), 128)),
                  e(Ze)(i).length === 0 ? (v(), B("span", cn, "—")) : N("", !0)
                ]),
                _: 1
              }, 8, ["label"]),
              t.compact ? N("", !0) : (v(), V(_, {
                key: 0,
                label: e(a)("access.colCreatedAt"),
                width: "170"
              }, {
                default: o(({ row: i }) => [
                  k(u(e(ve)(i.created_at)), 1)
                ]),
                _: 1
              }, 8, ["label"])),
              n(_, {
                label: e(a)("access.colActions"),
                width: "140",
                fixed: "right"
              }, {
                default: o(({ row: i }) => [
                  n(d, {
                    link: "",
                    type: "primary",
                    onClick: (y) => l("edit", i)
                  }, {
                    default: o(() => [
                      k(u(e(a)("common.edit")), 1)
                    ]),
                    _: 1
                  }, 8, ["onClick"]),
                  n(d, {
                    link: "",
                    type: "danger",
                    onClick: (y) => l("delete", i)
                  }, {
                    default: o(() => [
                      k(u(e(a)("common.delete")), 1)
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
      });
    };
  }
}), un = /* @__PURE__ */ J(dn, [["__scopeId", "data-v-26d3aaad"]]), mn = { class: "presets" }, pn = { class: "presets-label" }, fn = { class: "caps" }, vn = /* @__PURE__ */ H({
  __name: "IdentityFormDialog",
  props: {
    compact: { type: Boolean }
  },
  emits: ["saved", "created"],
  setup(t, { expose: s, emit: l }) {
    const c = l, p = te(), m = f(!1), _ = f(!1), d = f(null), r = f("custom"), g = et({
      name: "",
      caps: Ye()
    });
    function b() {
      if (r.value === "custom") return;
      const $ = new Set(Ya[r.value] ?? []);
      for (const w of me) g.caps[w.key] = $.has(w.key);
    }
    function i() {
      r.value = "custom";
    }
    function y() {
      d.value = null, g.name = "", Object.assign(g.caps, Ye()), r.value = "member", b(), m.value = !0;
    }
    function E($) {
      var w;
      d.value = $, g.name = $.name;
      for (const D of me) g.caps[D.key] = ((w = $.permissions) == null ? void 0 : w[D.key]) === !0;
      r.value = "custom", m.value = !0;
    }
    s({ openCreate: y, openEdit: E });
    async function I() {
      _.value = !0;
      try {
        if (d.value) {
          const $ = d.value.name;
          await W(() => p.put(`/api/identities/${encodeURIComponent($)}`, { permissions: { ...g.caps } })), K(a("access.saved")), c("saved");
        } else {
          const $ = await W(
            () => p.post("/api/identities", {
              name: g.name,
              permissions: { ...g.caps }
            })
          );
          $ != null && $.token && c("created", { name: g.name.trim(), token: $.token });
        }
        m.value = !1;
      } finally {
        _.value = !1;
      }
    }
    return ($, w) => {
      const D = _e, P = Te, M = Re, O = Ve, z = zt, oe = Ce, R = G, U = Ae;
      return v(), V(U, {
        modelValue: m.value,
        "onUpdate:modelValue": w[4] || (w[4] = (h) => m.value = h),
        title: d.value ? e(a)("access.editTitle", { name: d.value.name }) : e(a)("access.createTitle"),
        width: t.compact ? "96%" : "480px"
      }, {
        footer: o(() => [
          n(R, {
            onClick: w[3] || (w[3] = (h) => m.value = !1)
          }, {
            default: o(() => [
              k(u(e(a)("common.cancel")), 1)
            ]),
            _: 1
          }),
          n(R, {
            type: "primary",
            loading: _.value,
            onClick: I
          }, {
            default: o(() => [
              k(u(e(a)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: o(() => [
          n(oe, {
            "label-position": "top",
            onSubmit: w[2] || (w[2] = ze(() => {
            }, ["prevent"]))
          }, {
            default: o(() => [
              d.value ? N("", !0) : (v(), V(P, {
                key: 0,
                label: e(a)("access.nameLabel")
              }, {
                default: o(() => [
                  n(D, {
                    modelValue: g.name,
                    "onUpdate:modelValue": w[0] || (w[0] = (h) => g.name = h),
                    placeholder: e(a)("access.namePlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"])),
              n(P, {
                label: e(a)("access.permsLabel")
              }, {
                default: o(() => [
                  A("div", mn, [
                    A("span", pn, u(e(a)("access.presets")), 1),
                    n(O, {
                      modelValue: r.value,
                      "onUpdate:modelValue": w[1] || (w[1] = (h) => r.value = h),
                      size: "small",
                      onChange: b
                    }, {
                      default: o(() => [
                        n(M, { value: "admin" }, {
                          default: o(() => [
                            k(u(e(a)("access.presetAdmin")), 1)
                          ]),
                          _: 1
                        }),
                        n(M, { value: "member" }, {
                          default: o(() => [
                            k(u(e(a)("access.presetMember")), 1)
                          ]),
                          _: 1
                        }),
                        n(M, { value: "viewer" }, {
                          default: o(() => [
                            k(u(e(a)("access.presetViewer")), 1)
                          ]),
                          _: 1
                        }),
                        n(M, { value: "custom" }, {
                          default: o(() => [
                            k(u(e(a)("access.presetCustom")), 1)
                          ]),
                          _: 1
                        })
                      ]),
                      _: 1
                    }, 8, ["modelValue"])
                  ]),
                  A("div", fn, [
                    (v(!0), B(ee, null, ce(e(me), (h) => (v(), V(z, {
                      key: h.key,
                      modelValue: g.caps[h.key],
                      "onUpdate:modelValue": (S) => g.caps[h.key] = S,
                      onChange: i
                    }, {
                      default: o(() => [
                        k(u(e(gt)(h.key)), 1)
                      ]),
                      _: 2
                    }, 1032, ["modelValue", "onUpdate:modelValue"]))), 128))
                  ])
                ]),
                _: 1
              }, 8, ["label"])
            ]),
            _: 1
          })
        ]),
        _: 1
      }, 8, ["modelValue", "title", "width"]);
    };
  }
}), _n = /* @__PURE__ */ J(vn, [["__scopeId", "data-v-0e04bfb3"]]), gn = { class: "new-token" }, bn = /* @__PURE__ */ H({
  __name: "TokenOnceDialog",
  props: {
    compact: { type: Boolean }
  },
  setup(t, { expose: s }) {
    const l = Ue(), c = f(!1), p = f(""), m = f(""), _ = f("");
    function d(b, i, y) {
      p.value = b, m.value = i, _.value = y, c.value = !0;
    }
    s({ show: d });
    async function r() {
      if (m.value)
        try {
          await l.onIdentityToken(m.value, _.value), c.value = !1, K(a("access.savedToBrowser", { name: m.value }));
        } catch (b) {
          j(b instanceof Error ? b.message : String(b));
        }
    }
    async function g(b) {
      try {
        await navigator.clipboard.writeText(b), K(a("access.copied"));
      } catch {
        j(b);
      }
    }
    return (b, i) => {
      const y = G, E = Ae;
      return v(), V(E, {
        modelValue: c.value,
        "onUpdate:modelValue": i[1] || (i[1] = (I) => c.value = I),
        title: p.value,
        width: t.compact ? "96%" : "560px"
      }, {
        footer: o(() => [
          e(l).onIdentityToken ? (v(), V(y, {
            key: 0,
            type: "primary",
            onClick: r
          }, {
            default: o(() => [
              k(u(e(a)("access.saveToBrowser")), 1)
            ]),
            _: 1
          })) : N("", !0),
          n(y, {
            type: "primary",
            onClick: i[0] || (i[0] = () => {
              g(_.value), c.value = !1;
            })
          }, {
            default: o(() => [
              k(u(e(a)("access.copyToken")), 1)
            ]),
            _: 1
          })
        ]),
        default: o(() => [
          A("p", null, u(e(a)("access.created")), 1),
          A("code", gn, u(_.value), 1)
        ]),
        _: 1
      }, 8, ["modelValue", "title", "width"]);
    };
  }
}), yn = /* @__PURE__ */ J(bn, [["__scopeId", "data-v-cb1c8349"]]), hn = { class: "save-row" }, kn = /* @__PURE__ */ H({
  __name: "EmbeddingSettingsCard",
  props: {
    embeddingEnabled: { type: Boolean },
    embeddingBaseUrl: {},
    embeddingModel: {},
    embeddingApiKey: {}
  },
  setup(t) {
    const s = t, l = f(s.embeddingEnabled), c = f(s.embeddingBaseUrl), p = f(s.embeddingModel), m = f(s.embeddingApiKey);
    pe(
      () => [s.embeddingEnabled, s.embeddingBaseUrl, s.embeddingModel, s.embeddingApiKey],
      ([b, i, y, E]) => {
        l.value = b, c.value = i, p.value = y, m.value = E;
      }
    );
    const _ = te(), d = f(!1), r = f(null);
    async function g() {
      d.value = !0, r.value = null;
      try {
        if (await W(
          () => _.put("/api/settings", {
            embedding_enabled: l.value,
            embedding_base_url: c.value,
            embedding_model: p.value,
            embedding_api_key: m.value
          })
        ) === void 0 || (K(a("access.saved")), !l.value)) return;
        r.value = await W(() => _.post("/api/embeddings/test", {}));
      } finally {
        d.value = !1;
      }
    }
    return (b, i) => {
      const y = Se, E = st, I = Te, $ = _e, w = Ce, D = G, P = de;
      return v(), V(P, { shadow: "never" }, {
        header: o(() => [
          k(u(e(a)("access.embeddingTitle")), 1)
        ]),
        default: o(() => {
          var M;
          return [
            n(y, {
              title: e(a)("access.embeddingHint"),
              type: "info",
              "show-icon": "",
              closable: !1,
              class: "settings-hint"
            }, null, 8, ["title"]),
            n(w, {
              "label-position": "top",
              onSubmit: i[4] || (i[4] = ze(() => {
              }, ["prevent"]))
            }, {
              default: o(() => [
                n(I, {
                  label: e(a)("access.embeddingEnabledLabel")
                }, {
                  default: o(() => [
                    n(E, {
                      modelValue: l.value,
                      "onUpdate:modelValue": i[0] || (i[0] = (O) => l.value = O)
                    }, null, 8, ["modelValue"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                n(I, {
                  label: e(a)("access.embeddingBaseUrl")
                }, {
                  default: o(() => [
                    n($, {
                      modelValue: c.value,
                      "onUpdate:modelValue": i[1] || (i[1] = (O) => c.value = O),
                      placeholder: "https://api.siliconflow.cn/v1 或 http://127.0.0.1:11434/v1"
                    }, null, 8, ["modelValue"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                n(I, {
                  label: e(a)("access.embeddingModelLabel")
                }, {
                  default: o(() => [
                    n($, {
                      modelValue: p.value,
                      "onUpdate:modelValue": i[2] || (i[2] = (O) => p.value = O),
                      placeholder: "BAAI/bge-m3 / bge-m3 / nomic-embed-text"
                    }, null, 8, ["modelValue"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                n(I, {
                  label: e(a)("access.embeddingApiKeyLabel")
                }, {
                  default: o(() => [
                    n($, {
                      modelValue: m.value,
                      "onUpdate:modelValue": i[3] || (i[3] = (O) => m.value = O),
                      "show-password": "",
                      placeholder: e(a)("access.embeddingApiKeyPlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"])
              ]),
              _: 1
            }),
            A("div", hn, [
              n(D, {
                type: "primary",
                loading: d.value,
                onClick: g
              }, {
                default: o(() => [
                  k(u(e(a)("access.embeddingSaveTest")), 1)
                ]),
                _: 1
              }, 8, ["loading"])
            ]),
            (M = r.value) != null && M.ok ? (v(), V(y, {
              key: 0,
              title: e(a)("access.embeddingTestOk", { dim: r.value.dim ?? 0, ms: r.value.elapsed_ms ?? 0 }),
              type: "success",
              "show-icon": "",
              closable: !1,
              class: "settings-hint"
            }, null, 8, ["title"])) : r.value && !r.value.ok ? (v(), V(y, {
              key: 1,
              title: e(a)("access.embeddingTestFail", { error: r.value.error ?? "" }),
              type: "error",
              "show-icon": "",
              closable: !1,
              class: "settings-hint"
            }, null, 8, ["title"])) : N("", !0)
          ];
        }),
        _: 1
      });
    };
  }
}), wn = /* @__PURE__ */ J(kn, [["__scopeId", "data-v-7fdc5381"]]), Cn = { class: "field-hint" }, Tn = {
  key: 0,
  class: "default-view"
}, Sn = { class: "default-text" }, En = { class: "field-hint" }, $n = { class: "save-row" }, An = /* @__PURE__ */ H({
  __name: "PromptSettingsCard",
  props: {
    instructions: {},
    conventions: {},
    defaultInstructions: {}
  },
  setup(t) {
    const s = t, l = f(s.instructions), c = f(s.conventions);
    pe(
      () => [s.instructions, s.conventions],
      ([d, r]) => {
        l.value = d, c.value = r;
      }
    );
    const p = te(), m = f(!1);
    async function _() {
      m.value = !0;
      try {
        await W(
          () => p.put("/api/settings", {
            instructions: l.value,
            conventions: c.value
          })
        ), K(a("access.saved"));
      } finally {
        m.value = !1;
      }
    }
    return (d, r) => {
      const g = _e, b = Te, i = Ce, y = G, E = de;
      return v(), V(E, { shadow: "never" }, {
        header: o(() => [
          k(u(e(a)("access.settingsTitle")), 1)
        ]),
        default: o(() => [
          n(i, {
            "label-position": "top",
            onSubmit: r[2] || (r[2] = ze(() => {
            }, ["prevent"]))
          }, {
            default: o(() => [
              n(b, {
                label: e(a)("access.instructionsLabel")
              }, {
                default: o(() => [
                  n(g, {
                    modelValue: l.value,
                    "onUpdate:modelValue": r[0] || (r[0] = (I) => l.value = I),
                    type: "textarea",
                    rows: 5,
                    placeholder: e(a)("access.instructionsPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"]),
                  A("div", Cn, u(e(a)("access.instructionsHint")), 1),
                  l.value ? (v(), B("details", Tn, [
                    A("summary", null, u(e(a)("access.viewDefault")), 1),
                    A("pre", Sn, u(t.defaultInstructions), 1)
                  ])) : N("", !0)
                ]),
                _: 1
              }, 8, ["label"]),
              n(b, {
                label: e(a)("access.conventionsLabel")
              }, {
                default: o(() => [
                  n(g, {
                    modelValue: c.value,
                    "onUpdate:modelValue": r[1] || (r[1] = (I) => c.value = I),
                    type: "textarea",
                    rows: 4,
                    placeholder: e(a)("access.conventionsPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"]),
                  A("div", En, u(e(a)("access.conventionsHint")), 1)
                ]),
                _: 1
              }, 8, ["label"])
            ]),
            _: 1
          }),
          A("div", $n, [
            n(y, {
              type: "primary",
              loading: m.value,
              onClick: _
            }, {
              default: o(() => [
                k(u(e(a)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ])
        ]),
        _: 1
      });
    };
  }
}), Vn = /* @__PURE__ */ J(An, [["__scopeId", "data-v-3bcdc16f"]]);
function Ke() {
  const t = te(), s = f({ ok: !0, issues: [] }), l = f(!1), c = f(!1), p = f(!1), m = f(!1), _ = f(!1);
  async function d() {
    c.value = !0;
    try {
      s.value = await za(t), l.value = !0;
    } finally {
      c.value = !1;
    }
  }
  async function r() {
    p.value = !0;
    try {
      const i = await Ra(t), y = URL.createObjectURL(i), E = document.createElement("a");
      E.href = y, E.download = "agent-memory-export.json", E.click(), URL.revokeObjectURL(y);
    } finally {
      p.value = !1;
    }
  }
  async function g(i) {
    m.value = !0;
    try {
      const y = await i.text();
      let E;
      try {
        E = JSON.parse(y);
      } catch {
        throw new Error(a("errors.invalidBackup"));
      }
      return await Oa(t, E);
    } finally {
      m.value = !1;
    }
  }
  async function b() {
    _.value = !0;
    try {
      let i = 0;
      for (; ; ) {
        const y = await Na(t);
        if (!y.configured) throw new Error(a("ops.embeddingNotConfigured"));
        if (y.error) throw new Error(y.error);
        if (i += y.processed ?? 0, (y.processed ?? 0) === 0) break;
      }
      return i;
    } finally {
      _.value = !1;
    }
  }
  return {
    doctor: s,
    doctorRan: l,
    doctorLoading: c,
    exporting: p,
    importing: m,
    backfilling: _,
    runDoctor: d,
    exportData: r,
    importFile: g,
    backfill: b
  };
}
const In = { class: "actions" }, xn = /* @__PURE__ */ H({
  __name: "BackupCard",
  setup(t) {
    const { exporting: s, importing: l, exportData: c, importFile: p } = Ke(), m = f(null);
    function _(d) {
      var b;
      const r = d.target, g = (b = r.files) == null ? void 0 : b[0];
      r.value = "", g && p(g).then((i) => {
        K(a("access.imported", { memories: i.imported_memories, tags: i.imported_tags }));
      }).catch((i) => {
        j(i instanceof Error ? i.message : String(i));
      });
    }
    return (d, r) => {
      const g = G, b = be, i = ge, y = de;
      return v(), V(y, { shadow: "never" }, {
        header: o(() => [
          k(u(e(a)("access.backupCard")), 1)
        ]),
        default: o(() => [
          A("div", In, [
            n(g, {
              icon: e(Ht),
              loading: e(s),
              onClick: r[0] || (r[0] = (E) => e(W)(e(c)))
            }, {
              default: o(() => [
                k(u(e(a)("access.export")), 1)
              ]),
              _: 1
            }, 8, ["icon", "loading"]),
            n(g, {
              icon: e(qt),
              loading: e(l),
              onClick: r[1] || (r[1] = (E) => {
                var I;
                return (I = m.value) == null ? void 0 : I.click();
              })
            }, {
              default: o(() => [
                k(u(e(a)("access.import")), 1)
              ]),
              _: 1
            }, 8, ["icon", "loading"]),
            n(i, {
              content: e(a)("access.importHint"),
              placement: "top"
            }, {
              default: o(() => [
                n(b, { class: "am-info" }, {
                  default: o(() => [
                    n(e(Ee))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"]),
            A("input", {
              ref_key: "importInput",
              ref: m,
              type: "file",
              accept: "application/json,.json",
              style: { display: "none" },
              onChange: _
            }, null, 544)
          ])
        ]),
        _: 1
      });
    };
  }
}), Un = /* @__PURE__ */ J(xn, [["__scopeId", "data-v-106154fe"]]), Dn = { class: "card-header" }, Mn = {
  key: 2,
  class: "issues"
}, Pn = /* @__PURE__ */ H({
  __name: "DoctorCard",
  setup(t) {
    const { doctor: s, doctorRan: l, doctorLoading: c, runDoctor: p } = Ke();
    return (m, _) => {
      const d = G, r = Se, g = it, b = de;
      return v(), V(b, { shadow: "never" }, {
        header: o(() => [
          A("div", Dn, [
            A("span", null, u(e(a)("access.doctorCard")), 1),
            n(d, {
              size: "small",
              icon: e(He),
              loading: e(c),
              onClick: _[0] || (_[0] = (i) => e(W)(e(p)))
            }, {
              default: o(() => [
                k(u(e(a)("access.runDoctor")), 1)
              ]),
              _: 1
            }, 8, ["icon", "loading"])
          ])
        ]),
        default: o(() => [
          e(l) ? (v(), B(ee, { key: 0 }, [
            e(s).ok ? (v(), V(r, {
              key: 0,
              title: e(a)("access.doctorOk"),
              type: "success",
              "show-icon": "",
              closable: !1
            }, null, 8, ["title"])) : (v(), V(r, {
              key: 1,
              title: e(a)("access.doctorFail", { count: e(s).issues.length }),
              type: "error",
              "show-icon": "",
              closable: !1
            }, null, 8, ["title"])),
            e(s).ok ? N("", !0) : (v(), B("ul", Mn, [
              (v(!0), B(ee, null, ce(e(s).issues, (i, y) => (v(), B("li", { key: y }, u(i), 1))), 128))
            ]))
          ], 64)) : (v(), V(g, {
            key: 1,
            description: e(a)("access.doctorEmpty"),
            "image-size": 60
          }, null, 8, ["description"]))
        ]),
        _: 1
      });
    };
  }
}), Bn = /* @__PURE__ */ J(Pn, [["__scopeId", "data-v-ae41a618"]]), Ln = { class: "card-header" }, zn = /* @__PURE__ */ H({
  __name: "EmbeddingCoverageCard",
  props: {
    stats: {},
    compact: { type: Boolean }
  },
  emits: ["changed"],
  setup(t, { emit: s }) {
    const l = t;
    te();
    const { backfilling: c, backfill: p } = Ke(), m = s, _ = se(() => {
      var r, g, b, i;
      return (((g = (r = l.stats) == null ? void 0 : r.embedding) == null ? void 0 : g.embedded) ?? 0) + (((i = (b = l.stats) == null ? void 0 : b.embedding) == null ? void 0 : i.pending) ?? 0);
    });
    async function d() {
      const r = await W(p);
      r !== void 0 && (m("changed"), K(r === 0 ? a("access.embeddingUpToDate") : a("access.embeddingDone", { count: r })));
    }
    return (r, g) => {
      const b = G, i = lt, y = ot, E = Se, I = de;
      return v(), V(I, { shadow: "never" }, {
        header: o(() => [
          A("div", Ln, [
            A("span", null, u(e(a)("access.embeddingCard")), 1),
            n(b, {
              size: "small",
              icon: e(rt),
              loading: e(c),
              onClick: d
            }, {
              default: o(() => [
                k(u(e(a)("access.runBackfill")), 1)
              ]),
              _: 1
            }, 8, ["icon", "loading"])
          ])
        ]),
        default: o(() => {
          var $, w;
          return [
            (w = ($ = t.stats) == null ? void 0 : $.embedding) != null && w.enabled ? (v(), B(ee, { key: 0 }, [
              n(y, {
                column: t.compact ? 1 : 2,
                border: ""
              }, {
                default: o(() => [
                  n(i, {
                    label: e(a)("access.embeddingModel")
                  }, {
                    default: o(() => [
                      k(u(t.stats.embedding.model ?? "—"), 1)
                    ]),
                    _: 1
                  }, 8, ["label"]),
                  n(i, {
                    label: e(a)("access.embeddingCoverage")
                  }, {
                    default: o(() => [
                      k(u(t.stats.embedding.embedded ?? 0) + " / " + u(_.value), 1)
                    ]),
                    _: 1
                  }, 8, ["label"])
                ]),
                _: 1
              }, 8, ["column"]),
              (t.stats.embedding.pending ?? 0) > 0 ? (v(), V(E, {
                key: 0,
                title: e(a)("access.embeddingPending", { count: t.stats.embedding.pending }),
                type: "warning",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])) : N("", !0)
            ], 64)) : (v(), V(E, {
              key: 1,
              title: e(a)("access.embeddingDisabled"),
              type: "info",
              "show-icon": "",
              closable: !1
            }, null, 8, ["title"]))
          ];
        }),
        _: 1
      });
    };
  }
}), Rn = /* @__PURE__ */ J(zn, [["__scopeId", "data-v-ea4ac165"]]), On = {
  key: 0,
  class: "am-panel-header"
}, Nn = { class: "am-heading" }, Fn = { class: "am-panel-title" }, Hn = /* @__PURE__ */ H({
  __name: "AdminPanel",
  props: {
    who: { default: null },
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: s }) {
    const l = t, c = te(), p = f([]), m = f(""), _ = f(""), d = f(""), r = f(!1), g = f(!1), b = f(""), i = f(""), y = f(""), E = se(() => {
      var U;
      return !!l.who && ((U = l.who.permissions) == null ? void 0 : U.admin) === !0;
    }), I = f(null), $ = f(null), { compact: w } = De($);
    async function D() {
      await W(async () => {
        const [U, h, S] = await Promise.all([
          c.get("/api/identities"),
          c.get("/api/settings"),
          c.get("/api/stats")
        ]);
        p.value = Array.isArray(U == null ? void 0 : U.identities) ? U.identities : [], m.value = (h == null ? void 0 : h.instructions) ?? "", _.value = (h == null ? void 0 : h.conventions) ?? "", d.value = (h == null ? void 0 : h.default_instructions) ?? "", r.value = (h == null ? void 0 : h.auth_required) === !0, g.value = (h == null ? void 0 : h.embedding_enabled) === !0, b.value = (h == null ? void 0 : h.embedding_base_url) ?? "", i.value = (h == null ? void 0 : h.embedding_model) ?? "", y.value = (h == null ? void 0 : h.embedding_api_key) ?? "", I.value = S ?? null;
      });
    }
    we(() => {
      E.value && D();
    }), pe(E, (U) => {
      U && D();
    }), s({
      refresh: () => {
        E.value && W(D);
      }
    });
    const P = f(null), M = f(null), O = se(() => p.value.some((U) => {
      var h;
      return ((h = U.permissions) == null ? void 0 : h.admin) === !0;
    }));
    function z(U) {
      var h;
      (h = M.value) == null || h.show(a("access.createTitle"), U.name, U.token), D();
    }
    async function oe(U) {
      try {
        await ke.confirm(a("access.deleteConfirm", { name: U.name }), a("access.deleteTitle"), {
          type: "warning",
          confirmButtonText: a("common.delete"),
          cancelButtonText: a("common.cancel")
        });
      } catch {
        return;
      }
      await W(() => c.del(`/api/identities/${encodeURIComponent(U.name)}`)) !== void 0 && (K(a("access.deleted")), await D());
    }
    async function R(U) {
      var S;
      try {
        await ke.confirm(a("access.resetConfirm", { name: U.name }), a("access.resetTitle"), {
          type: "warning",
          confirmButtonText: a("access.resetToken"),
          cancelButtonText: a("common.cancel")
        });
      } catch {
        return;
      }
      const h = await W(
        () => c.post(`/api/identities/${encodeURIComponent(U.name)}/token-reset`, {})
      );
      h !== void 0 && (h != null && h.token && ((S = M.value) == null || S.show(a("access.resetTitle"), U.name, h.token)), await D());
    }
    return (U, h) => {
      const S = be, x = ge, Q = G, F = Se;
      return v(), B("div", {
        ref_key: "rootRef",
        ref: $,
        class: "am-panel admin-panel"
      }, [
        t.showHeader ? (v(), B("div", On, [
          A("div", Nn, [
            A("h2", Fn, u(l.title ?? e(a)("access.title")), 1),
            n(x, {
              content: l.subtitle ?? e(a)("access.subtitle"),
              placement: "top"
            }, {
              default: o(() => [
                n(S, { class: "am-info" }, {
                  default: o(() => [
                    n(e(Ee))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          E.value ? (v(), V(Q, {
            key: 0,
            type: "primary",
            icon: e(Fe),
            onClick: h[0] || (h[0] = (X) => {
              var Y;
              return (Y = P.value) == null ? void 0 : Y.openCreate();
            })
          }, {
            default: o(() => [
              k(u(e(a)("access.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])) : N("", !0)
        ])) : N("", !0),
        t.who ? t.who.mode === "open" ? (v(), V(F, {
          key: 2,
          type: "warning",
          title: e(a)("access.openMode"),
          closable: !1
        }, null, 8, ["title"])) : E.value ? N("", !0) : (v(), V(F, {
          key: 3,
          type: "info",
          title: e(a)("access.needAdmin"),
          closable: !1
        }, null, 8, ["title"])) : (v(), V(F, {
          key: 1,
          type: "info",
          title: e(a)("access.needAdmin"),
          closable: !1
        }, null, 8, ["title"])),
        t.who && E.value ? (v(), B(ee, { key: 4 }, [
          n(ln, {
            "auth-required": r.value,
            "has-admin-identity": O.value
          }, null, 8, ["auth-required", "has-admin-identity"]),
          n(un, {
            identities: p.value,
            compact: e(w),
            onEdit: h[1] || (h[1] = (X) => {
              var Y;
              return (Y = P.value) == null ? void 0 : Y.openEdit(X);
            }),
            onDelete: oe,
            onResetToken: R
          }, null, 8, ["identities", "compact"]),
          n(wn, {
            "embedding-enabled": g.value,
            "embedding-base-url": b.value,
            "embedding-model": i.value,
            "embedding-api-key": y.value
          }, null, 8, ["embedding-enabled", "embedding-base-url", "embedding-model", "embedding-api-key"]),
          n(Vn, {
            instructions: m.value,
            conventions: _.value,
            "default-instructions": d.value
          }, null, 8, ["instructions", "conventions", "default-instructions"]),
          n(Un),
          n(Bn),
          n(Rn, {
            stats: I.value,
            compact: e(w),
            onChanged: D
          }, null, 8, ["stats", "compact"])
        ], 64)) : N("", !0),
        n(_n, {
          ref_key: "formDialog",
          ref: P,
          compact: e(w),
          onSaved: D,
          onCreated: z
        }, null, 8, ["compact"]),
        n(yn, {
          ref_key: "tokenDialog",
          ref: M,
          compact: e(w)
        }, null, 8, ["compact"])
      ], 512);
    };
  }
}), eo = /* @__PURE__ */ J(Hn, [["__scopeId", "data-v-604ebc45"]]);
export {
  eo as AdminPanel,
  _t as MarkdownView,
  Aa as MemoriesPanel,
  Zn as MemoryAdmin,
  ba as MemoryDetailDrawer,
  ma as MemoryEditorDialog,
  ut as MemoryUIConfigKey,
  ja as OpsPanel,
  Pa as TagsPanel,
  Qt as applyMemoryUILocalePreference,
  ea as buildMemoriesQuery,
  Zt as createApiClient,
  Xn as currentMemoryUILocale,
  Yt as formatSize,
  ve as formatTime,
  Le as isSearchMode,
  xe as memoryUIi18n,
  Yn as provideMemoryUI,
  aa as renderMarkdown,
  na as sanitizeHtml,
  Gt as setMemoryUILocale,
  a as t,
  j as toastError,
  K as toastSuccess,
  Ke as useAdmin,
  te as useApiClient,
  ra as useMemories,
  Ue as useMemoryConfig,
  Fa as useOps,
  Va as useTags
};
