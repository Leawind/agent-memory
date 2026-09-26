import { inject as gt, provide as bt, ref as f, computed as le, onMounted as he, watch as pe, onUnmounted as yt, defineComponent as q, openBlock as _, createElementBlock as P, createBlock as V, unref as e, withCtx as o, createVNode as n, createElementVNode as E, toDisplayString as c, createTextVNode as k, Fragment as ee, renderList as re, createCommentVNode as F, withKeys as ht, isRef as ie, withDirectives as fe, reactive as Ye, renderSlot as kt, vShow as Be, withModifiers as ze } from "vue";
import { createI18n as wt } from "vue-i18n";
import { ElDialog as Ve, ElForm as ke, ElFormItem as we, ElInput as _e, ElRadioGroup as Ie, ElRadioButton as Re, ElSelect as Ze, ElOption as et, ElButton as X, ElDrawer as Ct, ElTag as xe, ElDivider as Tt, ElTooltip as ge, ElIcon as be, ElAlert as Ce, ElTable as Oe, ElTableColumn as Ne, ElLoadingDirective as tt, ElPagination as St, ElRadio as Et, ElRow as $t, ElCol as At, ElCard as de, ElStatistic as Vt, ElDescriptions as at, ElDescriptionsItem as nt, ElContainer as It, ElAside as xt, ElMenu as Ut, ElMenuItem as Dt, ElMain as Mt, ElTabs as Pt, ElTabPane as Bt, ElSwitch as ot, ElEmpty as lt, ElCheckbox as Lt } from "element-plus/es";
import { InfoFilled as Te, Plus as Fe, Search as He, Refresh as st, Collection as zt, Notebook as Rt, PriceTag as Ot, Odometer as Nt, Download as Ft, UploadFilled as Ht } from "@element-plus/icons-vue";
import { ElMessage as qt, ElMessageBox as ye } from "element-plus";
import { Marked as Kt } from "marked";
import it from "dompurify";
const jt = {
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
}, Jt = {
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
function rt() {
  var t;
  return typeof navigator > "u" || (t = navigator.language) != null && t.toLowerCase().startsWith("zh") ? "zh" : "en";
}
const Ue = wt({
  legacy: !1,
  locale: rt(),
  fallbackLocale: "zh",
  messages: { zh: jt, en: Jt },
  // 面向宿主组件库，缺 key 时静默回退即可，不刷控制台
  missingWarn: !1,
  fallbackWarn: !1
}), { t: a } = Ue.global;
function Wt(t) {
  Ue.global.locale.value = t;
}
function Gt(t) {
  Wt(t === "auto" ? rt() : t);
}
function Qn() {
  return Ue.global.locale.value;
}
const dt = Symbol("memory-ui-config"), Ae = {
  baseUrl: "",
  fetch: (...t) => globalThis.fetch(...t),
  defaultPageSize: 20,
  locale: "auto"
};
function Xn(t) {
  t.locale && Gt(t.locale), bt(dt, t);
}
function De() {
  const t = gt(dt);
  return {
    baseUrl: ((t == null ? void 0 : t.baseUrl) ?? Ae.baseUrl).replace(/\/+$/, ""),
    fetch: (t == null ? void 0 : t.fetch) ?? Ae.fetch,
    defaultPageSize: (t == null ? void 0 : t.defaultPageSize) ?? Ae.defaultPageSize,
    locale: (t == null ? void 0 : t.locale) ?? Ae.locale,
    onIdentityToken: t == null ? void 0 : t.onIdentityToken,
    onAuthChanged: t == null ? void 0 : t.onAuthChanged
  };
}
const Qt = 72;
function ct(t, s) {
  let l;
  return l = qt({
    type: t,
    message: s,
    offset: Qt,
    showClose: !0,
    grouping: !0,
    onClick: () => l.close()
  }), l;
}
function j(t) {
  return ct("success", t);
}
function J(t) {
  return ct("error", t);
}
function ve(t) {
  if (!t) return "—";
  const s = Ue.global.locale.value === "en" ? "en-US" : "zh-CN";
  return new Date(t * 1e3).toLocaleString(s, { hour12: !1 });
}
function Xt(t) {
  return t == null ? "—" : t < 1024 ? `${t} B` : t < 1024 * 1024 ? `${(t / 1024).toFixed(1)} KB` : `${(t / 1024 / 1024).toFixed(2)} MB`;
}
function Yt(t) {
  async function s(u, p = {}) {
    const m = await t.fetch(t.baseUrl + u, {
      headers: { "Content-Type": "application/json" },
      ...p
    }), g = await m.text();
    let d = null;
    try {
      d = g ? JSON.parse(g) : null;
    } catch {
      d = null;
    }
    if (!m.ok) {
      const r = (d == null ? void 0 : d.error) ?? a("errors.http", { status: m.status });
      throw new Error(r);
    }
    return d;
  }
  async function l(u) {
    var m;
    const p = await t.fetch(t.baseUrl + u);
    if (!p.ok) {
      const g = await p.text().catch(() => "");
      let d = a("errors.http", { status: p.status });
      try {
        d = ((m = JSON.parse(g)) == null ? void 0 : m.error) ?? d;
      } catch {
      }
      throw new Error(d);
    }
    return p.blob();
  }
  return {
    get: (u) => s(u),
    post: (u, p) => s(u, { method: "POST", body: JSON.stringify(p ?? {}) }),
    put: (u, p) => s(u, { method: "PUT", body: JSON.stringify(p ?? {}) }),
    del: (u) => s(u, { method: "DELETE" }),
    getBlob: l
  };
}
function te() {
  return Yt(De());
}
function Zt(t) {
  const s = Le(t.query), l = new URLSearchParams();
  return s ? (l.set("query", t.query.trim()), t.tagFilter && l.set("tags", t.tagFilter), t.mode && t.mode !== "auto" && l.set("mode", t.mode)) : (t.tagFilter && l.set("tag", t.tagFilter), l.set("sort", t.sort), l.set("order", t.order)), l.set("offset", String((t.page - 1) * t.pageSize)), l.set("limit", String(t.pageSize)), l.toString();
}
function Le(t) {
  return t.trim().length > 0;
}
const ea = new Kt();
function ta(t) {
  const s = ea.parse(t, { async: !1 });
  return it.sanitize(s);
}
function aa(t) {
  return it.sanitize(t);
}
function qe(t, s) {
  return t.get(`/api/memories/${encodeURIComponent(s)}`);
}
function ut(t, s) {
  return t.post("/api/memories", s);
}
function mt(t, s, l) {
  return t.put(`/api/memories/${encodeURIComponent(s)}`, l);
}
function na(t, s) {
  return t.del(`/api/memories/${encodeURIComponent(s)}`);
}
function pt(t, s) {
  const l = s ? `?filter=${encodeURIComponent(s)}` : "";
  return t.get(`/api/tags${l}`);
}
function oa(t, s, l) {
  return t.post("/api/tags", { name: s, description: l });
}
function la(t, s, l, u) {
  const p = { description: u }, m = l.trim();
  return m && m !== s && (p.new_name = m), t.put(`/api/tags/${encodeURIComponent(s)}`, p);
}
function sa(t, s, l) {
  return t.del(`/api/tags/${encodeURIComponent(s)}?mode=${l}`);
}
function ia() {
  const t = te(), { defaultPageSize: s } = De(), l = f(""), u = f(""), p = f("auto"), m = f("updated_at"), g = f("desc"), d = f(1), r = f(s), y = f([]), h = f([]), i = f(0), b = f(!1), A = f([]), I = le(() => Le(l.value));
  function S() {
    return d.value = 1, B();
  }
  function w() {
    return Zt({
      query: l.value,
      tagFilter: u.value,
      mode: p.value,
      sort: m.value,
      order: g.value,
      page: d.value,
      pageSize: r.value
    });
  }
  let x = 0;
  async function B() {
    var K;
    const R = ++x;
    b.value = !0;
    try {
      const U = w();
      if (Le(l.value)) {
        const v = await t.get(`/api/memories?${U}`);
        if (R !== x) return;
        h.value = (v.results ?? []).map(($) => ({ ...$, snippet: aa($.snippet) })), i.value = v.total_matches ?? 0, note.value = v.semantic_fallback ? a("memories.semanticFallback") : "";
      } else {
        const v = await t.get(`/api/memories?${U}`);
        if (R !== x) return;
        const $ = Math.max(1, Math.ceil(v.total / r.value));
        if (((K = v.memories) == null ? void 0 : K.length) === 0 && v.total > 0 && d.value > $)
          return d.value = $, b.value = !1, B();
        y.value = v.memories ?? [], i.value = v.total ?? 0;
      }
    } finally {
      R === x && (b.value = !1);
    }
  }
  async function D() {
    try {
      const R = await pt(t);
      A.value = (R.tags ?? []).map((K) => K.name);
    } catch {
    }
  }
  async function O(R) {
    if (R.id) {
      const K = await qe(t, R.id), U = new Set(K.tags), v = new Set(R.tags);
      await mt(t, R.id, {
        summary: R.summary,
        content: R.content,
        add_tags: [...v].filter(($) => !U.has($)),
        remove_tags: [...U].filter(($) => !v.has($))
      });
    } else
      await ut(t, { summary: R.summary, content: R.content, tags: R.tags });
    await Promise.all([B(), D()]);
  }
  async function L(R) {
    await na(t, R), await B();
  }
  return he(() => {
    B().catch(() => {
    }), D();
  }), {
    query: l,
    tagFilter: u,
    mode: p,
    sort: m,
    order: g,
    page: d,
    pageSize: r,
    rows: y,
    searchResults: h,
    total: i,
    loading: b,
    tagOptions: A,
    searching: I,
    onSearch: S,
    reload: B,
    loadTagOptions: D,
    saveMemory: O,
    removeMemory: L
  };
}
function Me(t) {
  const s = f(0);
  let l = null;
  function u(g) {
    l == null || l.disconnect(), l = null, !(!g || typeof ResizeObserver > "u") && (l = new ResizeObserver((d) => {
      var r;
      s.value = ((r = d[0]) == null ? void 0 : r.contentRect.width) ?? 0;
    }), l.observe(g));
  }
  he(() => u(t.value)), pe(t, (g) => u(g)), yt(() => l == null ? void 0 : l.disconnect());
  const p = le(() => s.value > 0 && s.value < 960), m = le(() => s.value > 0 && s.value < 720);
  return { width: s, compact: p, narrow: m };
}
const ra = ["innerHTML"], ft = /* @__PURE__ */ q({
  __name: "MarkdownView",
  props: {
    source: {}
  },
  setup(t) {
    const s = t, l = le(() => ta(s.source));
    return (u, p) => (_(), P("div", {
      class: "md-body",
      innerHTML: l.value
    }, null, 8, ra));
  }
}), da = { class: "content-label" }, ca = /* @__PURE__ */ q({
  __name: "MemoryEditorDialog",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    tagOptions: {},
    width: {}
  },
  emits: ["update:visible", "saved"],
  setup(t, { emit: s }) {
    const l = t, u = s, p = te(), m = f(!1), g = f("edit"), d = f({ id: null, summary: "", content: "", tags: [] });
    let r = [];
    pe(
      () => l.visible,
      async (h) => {
        if (h)
          if (g.value = "edit", l.memoryId)
            try {
              const i = await qe(p, l.memoryId);
              d.value = { id: i.id, summary: i.summary, content: i.content, tags: [...i.tags] }, r = [...i.tags];
            } catch (i) {
              J(i instanceof Error ? i.message : String(i)), u("update:visible", !1);
            }
          else
            d.value = { id: null, summary: "", content: "", tags: [] }, r = [];
      }
    );
    async function y() {
      m.value = !0;
      try {
        if (d.value.id) {
          const h = new Set(r), i = new Set(d.value.tags);
          await mt(p, d.value.id, {
            summary: d.value.summary,
            content: d.value.content,
            add_tags: [...i].filter((b) => !h.has(b)),
            remove_tags: [...h].filter((b) => !i.has(b))
          }), j(a("editor.updated"));
        } else
          await ut(p, {
            summary: d.value.summary,
            content: d.value.content,
            tags: d.value.tags
          }), j(a("editor.created"));
        u("update:visible", !1), u("saved");
      } catch (h) {
        J(h instanceof Error ? h.message : String(h));
      } finally {
        m.value = !1;
      }
    }
    return (h, i) => {
      const b = _e, A = we, I = Re, S = Ie, w = et, x = Ze, B = ke, D = X, O = Ve;
      return _(), V(O, {
        "model-value": t.visible,
        title: d.value.id ? e(a)("editor.editTitle") : e(a)("editor.createTitle"),
        width: t.width,
        "onUpdate:modelValue": i[5] || (i[5] = (L) => u("update:visible", L))
      }, {
        footer: o(() => [
          n(D, {
            onClick: i[4] || (i[4] = (L) => u("update:visible", !1))
          }, {
            default: o(() => [
              k(c(e(a)("common.cancel")), 1)
            ]),
            _: 1
          }),
          n(D, {
            type: "primary",
            loading: m.value,
            onClick: y
          }, {
            default: o(() => [
              k(c(e(a)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: o(() => [
          n(B, { "label-position": "top" }, {
            default: o(() => [
              n(A, {
                label: e(a)("editor.summaryLabel")
              }, {
                default: o(() => [
                  n(b, {
                    modelValue: d.value.summary,
                    "onUpdate:modelValue": i[0] || (i[0] = (L) => d.value.summary = L),
                    maxlength: "512",
                    "show-word-limit": "",
                    placeholder: e(a)("editor.summaryPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"]),
              n(A, null, {
                label: o(() => [
                  E("div", da, [
                    E("span", null, c(e(a)("editor.contentLabel")), 1),
                    n(S, {
                      modelValue: g.value,
                      "onUpdate:modelValue": i[1] || (i[1] = (L) => g.value = L),
                      size: "small"
                    }, {
                      default: o(() => [
                        n(I, { value: "edit" }, {
                          default: o(() => [
                            k(c(e(a)("editor.tabEdit")), 1)
                          ]),
                          _: 1
                        }),
                        n(I, { value: "preview" }, {
                          default: o(() => [
                            k(c(e(a)("editor.tabPreview")), 1)
                          ]),
                          _: 1
                        })
                      ]),
                      _: 1
                    }, 8, ["modelValue"])
                  ])
                ]),
                default: o(() => [
                  g.value === "edit" ? (_(), V(b, {
                    key: 0,
                    modelValue: d.value.content,
                    "onUpdate:modelValue": i[2] || (i[2] = (L) => d.value.content = L),
                    type: "textarea",
                    rows: 12,
                    maxlength: "262144",
                    "show-word-limit": "",
                    placeholder: e(a)("editor.contentPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])) : (_(), V(ft, {
                    key: 1,
                    class: "content-preview",
                    source: d.value.content
                  }, null, 8, ["source"]))
                ]),
                _: 1
              }),
              n(A, {
                label: e(a)("editor.tagsLabel")
              }, {
                default: o(() => [
                  n(x, {
                    modelValue: d.value.tags,
                    "onUpdate:modelValue": i[3] || (i[3] = (L) => d.value.tags = L),
                    multiple: "",
                    filterable: "",
                    "allow-create": "",
                    "default-first-option": "",
                    placeholder: e(a)("editor.tagsPlaceholder"),
                    class: "tags-select"
                  }, {
                    default: o(() => [
                      (_(!0), P(ee, null, re(t.tagOptions, (L) => (_(), V(w, {
                        key: L,
                        label: L,
                        value: L
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
}), W = (t, s) => {
  const l = t.__vccOpts || t;
  for (const [u, p] of s)
    l[u] = p;
  return l;
}, ua = /* @__PURE__ */ W(ca, [["__scopeId", "data-v-a0fec0dd"]]), ma = { class: "detail-summary" }, pa = { class: "detail-tags" }, fa = { class: "detail-toolbar" }, va = {
  key: 1,
  class: "detail-content"
}, _a = /* @__PURE__ */ q({
  __name: "MemoryDetailDrawer",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    size: {}
  },
  emits: ["update:visible"],
  setup(t, { emit: s }) {
    const l = t, u = s, p = te(), m = f(null), g = f("rendered");
    return pe(
      () => [l.visible, l.memoryId],
      async ([d]) => {
        if (!(!d || !l.memoryId)) {
          g.value = "rendered";
          try {
            m.value = await qe(p, l.memoryId);
          } catch (r) {
            J(r instanceof Error ? r.message : String(r)), u("update:visible", !1);
          }
        }
      }
    ), (d, r) => {
      var I;
      const y = xe, h = Tt, i = Re, b = Ie, A = Ct;
      return _(), V(A, {
        "model-value": t.visible,
        title: e(a)("drawer.title", { id: ((I = m.value) == null ? void 0 : I.id) ?? t.memoryId ?? "" }),
        size: t.size,
        "onUpdate:modelValue": r[1] || (r[1] = (S) => u("update:visible", S))
      }, {
        default: o(() => [
          m.value ? (_(), P(ee, { key: 0 }, [
            E("h3", ma, c(m.value.summary), 1),
            E("div", pa, [
              (_(!0), P(ee, null, re(m.value.tags, (S) => (_(), V(y, {
                key: S,
                size: "small",
                class: "am-tag"
              }, {
                default: o(() => [
                  k(c(S), 1)
                ]),
                _: 2
              }, 1024))), 128))
            ]),
            n(h),
            E("div", fa, [
              n(b, {
                modelValue: g.value,
                "onUpdate:modelValue": r[0] || (r[0] = (S) => g.value = S),
                size: "small"
              }, {
                default: o(() => [
                  n(i, { value: "rendered" }, {
                    default: o(() => [
                      k(c(e(a)("drawer.rendered")), 1)
                    ]),
                    _: 1
                  }),
                  n(i, { value: "source" }, {
                    default: o(() => [
                      k(c(e(a)("drawer.source")), 1)
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["modelValue"])
            ]),
            g.value === "rendered" ? (_(), V(ft, {
              key: 0,
              source: m.value.content
            }, null, 8, ["source"])) : (_(), P("pre", va, c(m.value.content), 1))
          ], 64)) : F("", !0)
        ]),
        _: 1
      }, 8, ["model-value", "title", "size"]);
    };
  }
}), ga = /* @__PURE__ */ W(_a, [["__scopeId", "data-v-613d4260"]]), ba = {
  key: 0,
  class: "am-panel-header"
}, ya = { class: "am-heading" }, ha = { class: "am-panel-title" }, ka = { class: "am-toolbar" }, wa = { class: "am-summary" }, Ca = ["innerHTML"], Ta = {
  key: 4,
  class: "am-pager"
}, Sa = {
  key: 5,
  class: "am-pager"
}, Ea = /* @__PURE__ */ q({
  __name: "MemoriesPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: s }) {
    const l = t, {
      query: u,
      tagFilter: p,
      mode: m,
      sort: g,
      order: d,
      page: r,
      pageSize: y,
      rows: h,
      searchResults: i,
      total: b,
      loading: A,
      tagOptions: I,
      searching: S,
      onSearch: w,
      reload: x,
      loadTagOptions: B,
      removeMemory: D
    } = ia(), O = le(() => {
      if (A.value || b.value !== 0) return "";
      if (S.value) return a("memories.searchEmpty");
      const H = p.value.trim();
      return H ? a("memories.tagEmpty", { tag: H }) : "";
    }), L = f(null), { compact: R, narrow: K } = Me(L), U = f(!1), v = f(null), $ = f(!1), z = f(null);
    function N(H) {
      return H().catch((C) => J(C instanceof Error ? C.message : String(C)));
    }
    function ce() {
      v.value = null, U.value = !0;
    }
    function G(H) {
      v.value = H, U.value = !0;
    }
    function Y(H) {
      z.value = H, $.value = !0;
    }
    function Se() {
      N(x), B();
    }
    function se(H) {
      const { prop: C, order: ue } = H;
      ue && (C === "updated_at" || C === "created_at" || C === "id") ? (g.value = C, d.value = ue === "ascending" ? "asc" : "desc") : (g.value = "updated_at", d.value = "desc"), N(x);
    }
    async function Ee(H) {
      try {
        await ye.confirm(a("memories.deleteConfirm", { id: H.id }), a("memories.deleteTitle"), {
          type: "warning"
        });
      } catch {
        return;
      }
      try {
        await D(H.id), j(a("memories.deleted"));
      } catch (C) {
        J(C instanceof Error ? C.message : String(C));
      }
    }
    return s({ refresh: () => N(x) }), (H, C) => {
      const ue = be, $e = ge, ne = X, Pe = _e, M = et, oe = Ze, _t = Ce, Z = Ne, je = xe, Je = Oe, We = St, Ge = tt;
      return _(), P("div", {
        ref_key: "rootRef",
        ref: L,
        class: "am-panel"
      }, [
        t.showHeader ? (_(), P("div", ba, [
          E("div", ya, [
            E("h2", ha, c(l.title ?? e(a)("memories.title")), 1),
            n($e, {
              content: l.subtitle ?? e(a)("memories.subtitle"),
              placement: "top"
            }, {
              default: o(() => [
                n(ue, { class: "am-info" }, {
                  default: o(() => [
                    n(e(Te))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          n(ne, {
            type: "primary",
            icon: e(Fe),
            onClick: ce
          }, {
            default: o(() => [
              k(c(e(a)("memories.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : F("", !0),
        E("div", ka, [
          n(Pe, {
            modelValue: e(u),
            "onUpdate:modelValue": C[1] || (C[1] = (T) => ie(u) ? u.value = T : null),
            placeholder: e(a)("memories.searchPlaceholder"),
            clearable: "",
            class: "search",
            onKeyup: C[2] || (C[2] = ht((T) => N(e(w)), ["enter"])),
            onClear: C[3] || (C[3] = (T) => N(e(w)))
          }, {
            append: o(() => [
              n(ne, {
                icon: e(He),
                onClick: C[0] || (C[0] = (T) => N(e(w)))
              }, null, 8, ["icon"])
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          n(oe, {
            modelValue: e(p),
            "onUpdate:modelValue": C[4] || (C[4] = (T) => ie(p) ? p.value = T : null),
            placeholder: e(a)("memories.tagFilter"),
            clearable: "",
            filterable: "",
            class: "tag-filter",
            onChange: C[5] || (C[5] = (T) => N(e(w)))
          }, {
            default: o(() => [
              (_(!0), P(ee, null, re(e(I), (T) => (_(), V(M, {
                key: T,
                label: T,
                value: T
              }, null, 8, ["label", "value"]))), 128))
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          n(oe, {
            modelValue: e(m),
            "onUpdate:modelValue": C[6] || (C[6] = (T) => ie(m) ? m.value = T : null),
            class: "mode-select",
            onChange: C[7] || (C[7] = (T) => N(e(w)))
          }, {
            default: o(() => [
              n(M, {
                label: e(a)("memories.modeAuto"),
                value: "auto"
              }, null, 8, ["label"]),
              n(M, {
                label: e(a)("memories.modeKeyword"),
                value: "keyword"
              }, null, 8, ["label"]),
              n(M, {
                label: e(a)("memories.modeHybrid"),
                value: "hybrid"
              }, null, 8, ["label"])
            ]),
            _: 1
          }, 8, ["modelValue"])
        ]),
        O.value ? (_(), V(_t, {
          key: 1,
          title: O.value,
          type: "info",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : F("", !0),
        e(S) ? fe((_(), V(Je, {
          key: 2,
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
                E("div", wa, c(T.summary), 1),
                E("div", {
                  class: "am-snippet",
                  innerHTML: T.snippet
                }, null, 8, Ca)
              ]),
              _: 1
            }, 8, ["label"]),
            n(Z, {
              label: e(a)("memories.colTags"),
              "min-width": "150"
            }, {
              default: o(({ row: T }) => [
                (_(!0), P(ee, null, re(T.tags, (ae) => (_(), V(je, {
                  key: ae,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: o(() => [
                    k(c(ae), 1)
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
                k(c(e(ve)(T.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            n(Z, {
              label: e(a)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: o(({ row: T }) => [
                n(ne, {
                  link: "",
                  type: "primary",
                  onClick: (ae) => Y(T.id)
                }, {
                  default: o(() => [
                    k(c(e(a)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                n(ne, {
                  link: "",
                  type: "primary",
                  onClick: (ae) => G(T.id)
                }, {
                  default: o(() => [
                    k(c(e(a)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                n(ne, {
                  link: "",
                  type: "danger",
                  onClick: (ae) => Ee(T)
                }, {
                  default: o(() => [
                    k(c(e(a)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [Ge, e(A)]
        ]) : fe((_(), V(Je, {
          key: 3,
          data: e(h),
          "default-sort": { prop: e(g), order: e(d) === "asc" ? "ascending" : "descending" },
          onSortChange: se
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
                (_(!0), P(ee, null, re(T.tags, (ae) => (_(), V(je, {
                  key: ae,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: o(() => [
                    k(c(ae), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            e(R) ? F("", !0) : (_(), V(Z, {
              key: 0,
              prop: "created_at",
              label: e(a)("memories.colCreatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: o(({ row: T }) => [
                k(c(e(ve)(T.created_at)), 1)
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
                k(c(e(ve)(T.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            n(Z, {
              label: e(a)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: o(({ row: T }) => [
                n(ne, {
                  link: "",
                  type: "primary",
                  onClick: (ae) => Y(T.id)
                }, {
                  default: o(() => [
                    k(c(e(a)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                n(ne, {
                  link: "",
                  type: "primary",
                  onClick: (ae) => G(T.id)
                }, {
                  default: o(() => [
                    k(c(e(a)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                n(ne, {
                  link: "",
                  type: "danger",
                  onClick: (ae) => Ee(T)
                }, {
                  default: o(() => [
                    k(c(e(a)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data", "default-sort"])), [
          [Ge, e(A)]
        ]),
        e(S) ? (_(), P("div", Sa, [
          n(We, {
            "current-page": e(r),
            "onUpdate:currentPage": C[12] || (C[12] = (T) => ie(r) ? r.value = T : null),
            "page-size": e(y),
            "onUpdate:pageSize": C[13] || (C[13] = (T) => ie(y) ? y.value = T : null),
            total: e(b),
            "page-sizes": [10, 20, 50],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: C[14] || (C[14] = (T) => N(e(x))),
            onSizeChange: C[15] || (C[15] = (T) => N(e(x)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])) : (_(), P("div", Ta, [
          n(We, {
            "current-page": e(r),
            "onUpdate:currentPage": C[8] || (C[8] = (T) => ie(r) ? r.value = T : null),
            "page-size": e(y),
            "onUpdate:pageSize": C[9] || (C[9] = (T) => ie(y) ? y.value = T : null),
            total: e(b),
            "page-sizes": [20, 50, 100, 200],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: C[10] || (C[10] = (T) => N(e(x))),
            onSizeChange: C[11] || (C[11] = (T) => N(e(x)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])),
        n(ua, {
          visible: U.value,
          "onUpdate:visible": C[16] || (C[16] = (T) => U.value = T),
          "memory-id": v.value,
          "tag-options": e(I),
          width: e(K) ? "96%" : "640px",
          onSaved: Se
        }, null, 8, ["visible", "memory-id", "tag-options", "width"]),
        n(ga, {
          visible: $.value,
          "onUpdate:visible": C[17] || (C[17] = (T) => $.value = T),
          "memory-id": z.value,
          size: e(K) ? "100%" : "45%"
        }, null, 8, ["visible", "memory-id", "size"])
      ], 512);
    };
  }
}), $a = /* @__PURE__ */ W(Ea, [["__scopeId", "data-v-a269dc91"]]);
function Aa() {
  const t = te(), s = f([]), l = f(!1), u = f("");
  async function p() {
    l.value = !0;
    try {
      const r = await pt(t, u.value.trim() || void 0);
      s.value = r.tags ?? [];
    } finally {
      l.value = !1;
    }
  }
  async function m(r, y) {
    await oa(t, r, y), await p();
  }
  async function g(r, y, h) {
    await la(t, r, y, h), await p();
  }
  async function d(r, y) {
    await sa(t, r, y), await p();
  }
  return he(() => {
    p().catch(() => {
    });
  }), { rows: s, loading: l, filter: u, reload: p, create: m, rename: g, remove: d };
}
const Va = {
  key: 0,
  class: "am-panel-header"
}, Ia = { class: "am-heading" }, xa = { class: "am-panel-title" }, Ua = { class: "delete-body" }, Da = /* @__PURE__ */ q({
  __name: "TagsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: s }) {
    const l = t, { rows: u, loading: p, filter: m, reload: g, create: d, rename: r, remove: y } = Aa();
    let h = null;
    function i() {
      h && clearTimeout(h), h = setTimeout(() => {
        h = null, g().catch(($) => J($ instanceof Error ? $.message : String($)));
      }, 300);
    }
    const b = f(null), { narrow: A } = Me(b), I = f(!1), S = f(!1), w = Ye({ oldName: null, name: "", description: "" }), x = f(!1), B = f("detach"), D = f(null);
    function O($) {
      return (z, N) => (z[$] ?? 0) - (N[$] ?? 0);
    }
    function L() {
      Object.assign(w, { oldName: null, name: "", description: "" }), S.value = !0;
    }
    function R($) {
      Object.assign(w, { oldName: $.name, name: $.name, description: $.description ?? "" }), S.value = !0;
    }
    async function K() {
      I.value = !0;
      try {
        w.oldName ? (await r(w.oldName, w.name, w.description), j(a("tags.saved"))) : (await d(w.name, w.description), j(a("tags.created"))), S.value = !1;
      } catch ($) {
        J($ instanceof Error ? $.message : String($));
      } finally {
        I.value = !1;
      }
    }
    function U($) {
      D.value = $, B.value = "detach", x.value = !0;
    }
    async function v() {
      var $, z;
      if (B.value === "purge")
        try {
          await ye.confirm(
            a("tags.purgeConfirm", { name: ($ = D.value) == null ? void 0 : $.name, count: ((z = D.value) == null ? void 0 : z.memory_count) ?? 0 }),
            a("tags.purgeConfirmTitle"),
            { type: "error", confirmButtonText: a("tags.purgeButton") }
          );
        } catch {
          return;
        }
      if (D.value) {
        I.value = !0;
        try {
          await y(D.value.name, B.value), j(a("tags.deleted")), x.value = !1;
        } catch (N) {
          J(N instanceof Error ? N.message : String(N));
        } finally {
          I.value = !1;
        }
      }
    }
    return s({
      refresh: () => g().catch(($) => J($ instanceof Error ? $.message : String($)))
    }), ($, z) => {
      const N = be, ce = ge, G = X, Y = _e, Se = xe, se = Ne, Ee = Oe, H = we, C = ke, ue = Ve, $e = Et, ne = Ie, Pe = tt;
      return _(), P("div", {
        ref_key: "rootRef",
        ref: b,
        class: "am-panel"
      }, [
        t.showHeader ? (_(), P("div", Va, [
          E("div", Ia, [
            E("h2", xa, c(l.title ?? e(a)("tags.title")), 1),
            n(ce, {
              content: l.subtitle ?? e(a)("tags.subtitle"),
              placement: "top"
            }, {
              default: o(() => [
                n(N, { class: "am-info" }, {
                  default: o(() => [
                    n(e(Te))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          n(G, {
            type: "primary",
            icon: e(Fe),
            onClick: L
          }, {
            default: o(() => [
              k(c(e(a)("tags.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : F("", !0),
        n(Y, {
          modelValue: e(m),
          "onUpdate:modelValue": z[0] || (z[0] = (M) => ie(m) ? m.value = M : null),
          class: "am-tag-filter",
          placeholder: e(a)("tags.filterPlaceholder"),
          clearable: "",
          "prefix-icon": e(He),
          onInput: i,
          onClear: i
        }, null, 8, ["modelValue", "placeholder", "prefix-icon"]),
        fe((_(), V(Ee, { data: e(u) }, {
          default: o(() => [
            n(se, {
              prop: "name",
              label: e(a)("tags.colName"),
              "min-width": "140",
              sortable: ""
            }, {
              default: o(({ row: M }) => [
                n(Se, null, {
                  default: o(() => [
                    k(c(M.name), 1)
                  ]),
                  _: 2
                }, 1024)
              ]),
              _: 1
            }, 8, ["label"]),
            n(se, {
              prop: "description",
              label: e(a)("tags.colDescription"),
              "min-width": "150",
              "show-overflow-tooltip": ""
            }, {
              default: o(({ row: M }) => [
                k(c(M.description || "—"), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            n(se, {
              prop: "memory_count",
              label: e(a)("tags.colMemoryCount"),
              width: "90",
              sortable: ""
            }, null, 8, ["label"]),
            n(se, {
              prop: "last_used_at",
              label: e(a)("tags.colLastUsed"),
              width: "170",
              sortable: "",
              "sort-method": O("last_used_at")
            }, {
              default: o(({ row: M }) => [
                k(c(e(ve)(M.last_used_at)), 1)
              ]),
              _: 1
            }, 8, ["label", "sort-method"]),
            n(se, {
              prop: "created_at",
              label: e(a)("tags.colCreatedAt"),
              width: "170",
              sortable: "",
              "sort-method": O("created_at")
            }, {
              default: o(({ row: M }) => [
                k(c(e(ve)(M.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label", "sort-method"]),
            n(se, {
              label: e(a)("memories.colActions"),
              width: "150",
              fixed: "right"
            }, {
              default: o(({ row: M }) => [
                n(G, {
                  link: "",
                  type: "primary",
                  onClick: (oe) => R(M)
                }, {
                  default: o(() => [
                    k(c(e(a)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                n(G, {
                  link: "",
                  type: "danger",
                  onClick: (oe) => U(M)
                }, {
                  default: o(() => [
                    k(c(e(a)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [Pe, e(p)]
        ]),
        n(ue, {
          modelValue: S.value,
          "onUpdate:modelValue": z[4] || (z[4] = (M) => S.value = M),
          title: w.oldName ? e(a)("tags.editTitle") : e(a)("tags.createTitle"),
          width: e(A) ? "96%" : "480px"
        }, {
          footer: o(() => [
            n(G, {
              onClick: z[3] || (z[3] = (M) => S.value = !1)
            }, {
              default: o(() => [
                k(c(e(a)("common.cancel")), 1)
              ]),
              _: 1
            }),
            n(G, {
              type: "primary",
              loading: I.value,
              onClick: K
            }, {
              default: o(() => [
                k(c(e(a)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: o(() => [
            n(C, { "label-position": "top" }, {
              default: o(() => [
                n(H, {
                  label: e(a)("tags.nameLabel")
                }, {
                  default: o(() => [
                    n(Y, {
                      modelValue: w.name,
                      "onUpdate:modelValue": z[1] || (z[1] = (M) => w.name = M),
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(a)("tags.namePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                n(H, {
                  label: e(a)("tags.descLabel")
                }, {
                  default: o(() => [
                    n(Y, {
                      modelValue: w.description,
                      "onUpdate:modelValue": z[2] || (z[2] = (M) => w.description = M),
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
        n(ue, {
          modelValue: x.value,
          "onUpdate:modelValue": z[7] || (z[7] = (M) => x.value = M),
          title: e(a)("tags.deleteTitle"),
          width: e(A) ? "96%" : "480px"
        }, {
          footer: o(() => [
            n(G, {
              onClick: z[6] || (z[6] = (M) => x.value = !1)
            }, {
              default: o(() => [
                k(c(e(a)("common.cancel")), 1)
              ]),
              _: 1
            }),
            n(G, {
              type: "danger",
              loading: I.value,
              onClick: v
            }, {
              default: o(() => [
                k(c(e(a)("common.delete")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: o(() => {
            var M;
            return [
              E("p", Ua, [
                k(c(e(a)("tags.deleteBefore")) + " ", 1),
                n(Se, null, {
                  default: o(() => {
                    var oe;
                    return [
                      k(c((oe = D.value) == null ? void 0 : oe.name), 1)
                    ];
                  }),
                  _: 1
                }),
                k(" " + c(e(a)("tags.deleteMiddle")) + " ", 1),
                E("b", null, c((M = D.value) == null ? void 0 : M.memory_count), 1),
                k(" " + c(e(a)("tags.deleteAfter")), 1)
              ]),
              n(ne, {
                modelValue: B.value,
                "onUpdate:modelValue": z[5] || (z[5] = (oe) => B.value = oe)
              }, {
                default: o(() => [
                  n($e, { value: "detach" }, {
                    default: o(() => [
                      k(c(e(a)("tags.detach")), 1)
                    ]),
                    _: 1
                  }),
                  n($e, { value: "purge" }, {
                    default: o(() => [
                      k(c(e(a)("tags.purge")), 1)
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
}), Ma = /* @__PURE__ */ W(Da, [["__scopeId", "data-v-c8aeea57"]]);
function Pa(t) {
  return t.get("/api/stats");
}
function Ba(t) {
  return t.get("/health");
}
function La(t) {
  return t.get("/api/doctor");
}
function za(t) {
  return t.getBlob("/api/export");
}
function Ra(t, s) {
  return t.post("/api/import", s);
}
function Oa(t) {
  return t.post("/api/embeddings/backfill", {});
}
function Na() {
  const t = te(), s = f({}), l = f(""), u = le(() => Xt(s.value.file_size));
  async function p() {
    s.value = await Pa(t), s.value.version = l.value;
  }
  return he(async () => {
    try {
      l.value = (await Ba(t)).version ?? "";
    } catch {
    }
    await p().catch(() => {
    });
  }), {
    stats: s,
    version: l,
    sizeText: u,
    reload: p
  };
}
const Fa = {
  key: 0,
  class: "am-panel-header"
}, Ha = { class: "am-heading" }, qa = { class: "am-panel-title" }, Ka = /* @__PURE__ */ q({
  __name: "OpsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: s }) {
    const l = t, { stats: u, version: p, sizeText: m, reload: g } = Na(), d = f(null), { narrow: r } = Me(d);
    function y(h) {
      return h().catch((i) => J(i instanceof Error ? i.message : String(i)));
    }
    return s({ refresh: () => y(g) }), (h, i) => {
      const b = be, A = ge, I = X, S = Vt, w = de, x = At, B = $t, D = nt, O = at;
      return _(), P("div", {
        ref_key: "rootRef",
        ref: d,
        class: "am-panel"
      }, [
        t.showHeader ? (_(), P("div", Fa, [
          E("div", Ha, [
            E("h2", qa, c(l.title ?? e(a)("ops.title")), 1),
            n(A, {
              content: l.subtitle ?? e(a)("ops.subtitle"),
              placement: "top"
            }, {
              default: o(() => [
                n(b, { class: "am-info" }, {
                  default: o(() => [
                    n(e(Te))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          n(I, {
            icon: e(st),
            onClick: i[0] || (i[0] = (L) => y(e(g)))
          }, {
            default: o(() => [
              k(c(e(a)("ops.refresh")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : F("", !0),
        n(B, { gutter: 14 }, {
          default: o(() => [
            n(x, {
              span: e(r) ? 12 : 8
            }, {
              default: o(() => [
                n(w, { shadow: "never" }, {
                  default: o(() => [
                    n(S, {
                      title: e(a)("ops.statMemories"),
                      value: e(u).memories ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            n(x, {
              span: e(r) ? 12 : 8
            }, {
              default: o(() => [
                n(w, { shadow: "never" }, {
                  default: o(() => [
                    n(S, {
                      title: e(a)("ops.statTags"),
                      value: e(u).tags ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            n(x, {
              span: e(r) ? 12 : 8
            }, {
              default: o(() => [
                n(w, { shadow: "never" }, {
                  default: o(() => [
                    n(S, {
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
            k(c(e(a)("ops.dbCard")), 1)
          ]),
          default: o(() => [
            n(O, {
              column: e(r) ? 1 : 2,
              border: ""
            }, {
              default: o(() => [
                n(D, {
                  label: e(a)("ops.version")
                }, {
                  default: o(() => [
                    k(c(e(u).version ?? e(p)), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                n(D, {
                  label: e(a)("ops.schemaVersion")
                }, {
                  default: o(() => [
                    k(c(e(u).schema_version ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                n(D, {
                  label: e(a)("ops.path"),
                  span: e(r) ? 1 : 2
                }, {
                  default: o(() => [
                    k(c(e(u).path ?? "—"), 1)
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
}), ja = { class: "memory-ui" }, Ja = { class: "brand" }, Wa = { class: "brand-mark" }, Ga = { class: "aside-footer" }, Qa = /* @__PURE__ */ q({
  __name: "MemoryAdmin",
  props: {
    layout: { default: "sidebar" },
    title: { default: "Agent Memory" }
  },
  setup(t) {
    const s = f("memories");
    return (l, u) => {
      const p = be, m = Dt, g = Ut, d = xt, r = Bt, y = Pt, h = Mt, i = It;
      return _(), P("div", ja, [
        n(i, { class: "layout" }, {
          default: o(() => [
            t.layout === "sidebar" ? (_(), V(d, {
              key: 0,
              width: "200px",
              class: "aside"
            }, {
              default: o(() => [
                E("div", Ja, [
                  E("span", Wa, [
                    n(p, { size: 16 }, {
                      default: o(() => [
                        n(e(zt))
                      ]),
                      _: 1
                    })
                  ]),
                  E("span", null, c(t.title), 1)
                ]),
                n(g, {
                  "default-active": s.value,
                  class: "menu",
                  onSelect: u[0] || (u[0] = (b) => s.value = b)
                }, {
                  default: o(() => [
                    n(m, { index: "memories" }, {
                      default: o(() => [
                        n(p, null, {
                          default: o(() => [
                            n(e(Rt))
                          ]),
                          _: 1
                        }),
                        E("span", null, c(e(a)("nav.memories")), 1)
                      ]),
                      _: 1
                    }),
                    n(m, { index: "tags" }, {
                      default: o(() => [
                        n(p, null, {
                          default: o(() => [
                            n(e(Ot))
                          ]),
                          _: 1
                        }),
                        E("span", null, c(e(a)("nav.tags")), 1)
                      ]),
                      _: 1
                    }),
                    n(m, { index: "ops" }, {
                      default: o(() => [
                        n(p, null, {
                          default: o(() => [
                            n(e(Nt))
                          ]),
                          _: 1
                        }),
                        E("span", null, c(e(a)("nav.ops")), 1)
                      ]),
                      _: 1
                    })
                  ]),
                  _: 1
                }, 8, ["default-active"]),
                E("div", Ga, [
                  kt(l.$slots, "footer", {}, void 0, !0)
                ])
              ]),
              _: 3
            })) : F("", !0),
            n(h, { class: "main" }, {
              default: o(() => [
                t.layout === "tabs" ? (_(), V(y, {
                  key: 0,
                  modelValue: s.value,
                  "onUpdate:modelValue": u[1] || (u[1] = (b) => s.value = b),
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
                }, 8, ["modelValue"])) : F("", !0),
                fe(n($a, null, null, 512), [
                  [Be, s.value === "memories"]
                ]),
                fe(n(Ma, null, null, 512), [
                  [Be, s.value === "tags"]
                ]),
                fe(n(Ka, null, null, 512), [
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
}), Yn = /* @__PURE__ */ W(Qa, [["__scopeId", "data-v-38c91753"]]), me = [
  { key: "read", labelKey: "capRead" },
  { key: "create", labelKey: "capCreate" },
  { key: "update", labelKey: "capUpdate" },
  { key: "delete", labelKey: "capDelete" },
  { key: "tag_manage", labelKey: "capTagManage" },
  { key: "admin", labelKey: "capAdmin" }
];
function Qe() {
  return Object.fromEntries(me.map((t) => [t.key, !1]));
}
const Xa = {
  admin: me.map((t) => t.key),
  member: ["read", "create", "update", "tag_manage"],
  viewer: ["read"]
};
function vt(t) {
  const s = me.find((l) => l.key === t);
  return s ? a(`access.${s.labelKey}`) : t;
}
function Xe(t) {
  return me.map((s) => s.key).filter((s) => {
    var l;
    return ((l = t.permissions) == null ? void 0 : l[s]) === !0;
  });
}
function Ya(t) {
  return t ? `…${t}` : "—";
}
async function Q(t, s) {
  try {
    const l = await t();
    return s && j(s), l;
  } catch (l) {
    J(l instanceof Error ? l.message : String(l));
    return;
  }
}
const Za = { class: "auth-row" }, en = { class: "auth-text" }, tn = { class: "auth-label" }, an = { class: "auth-hint" }, nn = /* @__PURE__ */ q({
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
    const u = f(!1), p = te(), m = De();
    async function g() {
      var r;
      const d = !l.value;
      if (d && !s.hasAdminIdentity)
        return J(a("access.enableBlocked")), !1;
      try {
        await ye.confirm(
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
      u.value = !0;
      try {
        return await Q(() => p.put("/api/settings", { auth_required: d })) !== void 0 && ((r = m.onAuthChanged) == null || r.call(m, d)), !0;
      } finally {
        u.value = !1;
      }
    }
    return (d, r) => {
      const y = ot, h = ge, i = de;
      return _(), V(i, { shadow: "never" }, {
        default: o(() => [
          E("div", Za, [
            E("div", en, [
              E("span", tn, c(e(a)("access.authTitle")), 1),
              E("span", an, c(e(a)("access.authHint")), 1)
            ]),
            n(h, {
              disabled: t.hasAdminIdentity,
              content: e(a)("access.enableBlocked"),
              placement: "top"
            }, {
              default: o(() => [
                n(y, {
                  modelValue: l.value,
                  "onUpdate:modelValue": r[0] || (r[0] = (b) => l.value = b),
                  "before-change": g,
                  loading: u.value,
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
}), on = /* @__PURE__ */ W(nn, [["__scopeId", "data-v-ca2a1dcb"]]), ln = { class: "token-cell" }, sn = { class: "token-text" }, rn = {
  key: 0,
  class: "muted"
}, dn = /* @__PURE__ */ q({
  __name: "IdentityTableCard",
  props: {
    identities: {},
    compact: { type: Boolean }
  },
  emits: ["edit", "delete", "reset-token"],
  setup(t, { emit: s }) {
    const l = s;
    return (u, p) => {
      const m = lt, g = Ne, d = X, r = xe, y = Oe, h = de;
      return _(), V(h, { shadow: "never" }, {
        default: o(() => [
          t.identities.length === 0 ? (_(), V(m, {
            key: 0,
            description: e(a)("access.empty")
          }, null, 8, ["description"])) : (_(), V(y, {
            key: 1,
            data: t.identities
          }, {
            default: o(() => [
              n(g, {
                prop: "name",
                label: e(a)("access.colName"),
                "min-width": "120"
              }, null, 8, ["label"]),
              n(g, {
                label: e(a)("access.colToken"),
                "min-width": "200"
              }, {
                default: o(({ row: i }) => [
                  E("div", ln, [
                    E("code", sn, c(e(Ya)(i.token_hint)), 1),
                    n(d, {
                      link: "",
                      type: "primary",
                      onClick: (b) => l("reset-token", i)
                    }, {
                      default: o(() => [
                        k(c(e(a)("access.resetToken")), 1)
                      ]),
                      _: 1
                    }, 8, ["onClick"])
                  ])
                ]),
                _: 1
              }, 8, ["label"]),
              n(g, {
                label: e(a)("access.colPermissions"),
                "min-width": "240"
              }, {
                default: o(({ row: i }) => [
                  (_(!0), P(ee, null, re(e(Xe)(i), (b) => (_(), V(r, {
                    key: b,
                    size: "small",
                    class: "cap-tag",
                    type: b === "admin" ? "danger" : "info"
                  }, {
                    default: o(() => [
                      k(c(e(vt)(b)), 1)
                    ]),
                    _: 2
                  }, 1032, ["type"]))), 128)),
                  e(Xe)(i).length === 0 ? (_(), P("span", rn, "—")) : F("", !0)
                ]),
                _: 1
              }, 8, ["label"]),
              t.compact ? F("", !0) : (_(), V(g, {
                key: 0,
                label: e(a)("access.colCreatedAt"),
                width: "170"
              }, {
                default: o(({ row: i }) => [
                  k(c(e(ve)(i.created_at)), 1)
                ]),
                _: 1
              }, 8, ["label"])),
              n(g, {
                label: e(a)("access.colActions"),
                width: "140",
                fixed: "right"
              }, {
                default: o(({ row: i }) => [
                  n(d, {
                    link: "",
                    type: "primary",
                    onClick: (b) => l("edit", i)
                  }, {
                    default: o(() => [
                      k(c(e(a)("common.edit")), 1)
                    ]),
                    _: 1
                  }, 8, ["onClick"]),
                  n(d, {
                    link: "",
                    type: "danger",
                    onClick: (b) => l("delete", i)
                  }, {
                    default: o(() => [
                      k(c(e(a)("common.delete")), 1)
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
}), cn = /* @__PURE__ */ W(dn, [["__scopeId", "data-v-26d3aaad"]]), un = { class: "presets" }, mn = { class: "presets-label" }, pn = { class: "caps" }, fn = /* @__PURE__ */ q({
  __name: "IdentityFormDialog",
  props: {
    compact: { type: Boolean }
  },
  emits: ["saved", "created"],
  setup(t, { expose: s, emit: l }) {
    const u = l, p = te(), m = f(!1), g = f(!1), d = f(null), r = f("custom"), y = Ye({
      name: "",
      caps: Qe()
    });
    function h() {
      if (r.value === "custom") return;
      const S = new Set(Xa[r.value] ?? []);
      for (const w of me) y.caps[w.key] = S.has(w.key);
    }
    function i() {
      r.value = "custom";
    }
    function b() {
      d.value = null, y.name = "", Object.assign(y.caps, Qe()), r.value = "member", h(), m.value = !0;
    }
    function A(S) {
      var w;
      d.value = S, y.name = S.name;
      for (const x of me) y.caps[x.key] = ((w = S.permissions) == null ? void 0 : w[x.key]) === !0;
      r.value = "custom", m.value = !0;
    }
    s({ openCreate: b, openEdit: A });
    async function I() {
      g.value = !0;
      try {
        if (d.value) {
          const S = d.value.name;
          await Q(() => p.put(`/api/identities/${encodeURIComponent(S)}`, { permissions: { ...y.caps } })), j(a("access.saved")), u("saved");
        } else {
          const S = await Q(
            () => p.post("/api/identities", {
              name: y.name,
              permissions: { ...y.caps }
            })
          );
          S != null && S.token && u("created", { name: y.name.trim(), token: S.token });
        }
        m.value = !1;
      } finally {
        g.value = !1;
      }
    }
    return (S, w) => {
      const x = _e, B = we, D = Re, O = Ie, L = Lt, R = ke, K = X, U = Ve;
      return _(), V(U, {
        modelValue: m.value,
        "onUpdate:modelValue": w[4] || (w[4] = (v) => m.value = v),
        title: d.value ? e(a)("access.editTitle", { name: d.value.name }) : e(a)("access.createTitle"),
        width: t.compact ? "96%" : "480px"
      }, {
        footer: o(() => [
          n(K, {
            onClick: w[3] || (w[3] = (v) => m.value = !1)
          }, {
            default: o(() => [
              k(c(e(a)("common.cancel")), 1)
            ]),
            _: 1
          }),
          n(K, {
            type: "primary",
            loading: g.value,
            onClick: I
          }, {
            default: o(() => [
              k(c(e(a)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: o(() => [
          n(R, {
            "label-position": "top",
            onSubmit: w[2] || (w[2] = ze(() => {
            }, ["prevent"]))
          }, {
            default: o(() => [
              d.value ? F("", !0) : (_(), V(B, {
                key: 0,
                label: e(a)("access.nameLabel")
              }, {
                default: o(() => [
                  n(x, {
                    modelValue: y.name,
                    "onUpdate:modelValue": w[0] || (w[0] = (v) => y.name = v),
                    placeholder: e(a)("access.namePlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"])),
              n(B, {
                label: e(a)("access.permsLabel")
              }, {
                default: o(() => [
                  E("div", un, [
                    E("span", mn, c(e(a)("access.presets")), 1),
                    n(O, {
                      modelValue: r.value,
                      "onUpdate:modelValue": w[1] || (w[1] = (v) => r.value = v),
                      size: "small",
                      onChange: h
                    }, {
                      default: o(() => [
                        n(D, { value: "admin" }, {
                          default: o(() => [
                            k(c(e(a)("access.presetAdmin")), 1)
                          ]),
                          _: 1
                        }),
                        n(D, { value: "member" }, {
                          default: o(() => [
                            k(c(e(a)("access.presetMember")), 1)
                          ]),
                          _: 1
                        }),
                        n(D, { value: "viewer" }, {
                          default: o(() => [
                            k(c(e(a)("access.presetViewer")), 1)
                          ]),
                          _: 1
                        }),
                        n(D, { value: "custom" }, {
                          default: o(() => [
                            k(c(e(a)("access.presetCustom")), 1)
                          ]),
                          _: 1
                        })
                      ]),
                      _: 1
                    }, 8, ["modelValue"])
                  ]),
                  E("div", pn, [
                    (_(!0), P(ee, null, re(e(me), (v) => (_(), V(L, {
                      key: v.key,
                      modelValue: y.caps[v.key],
                      "onUpdate:modelValue": ($) => y.caps[v.key] = $,
                      onChange: i
                    }, {
                      default: o(() => [
                        k(c(e(vt)(v.key)), 1)
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
}), vn = /* @__PURE__ */ W(fn, [["__scopeId", "data-v-0e04bfb3"]]), _n = { class: "new-token" }, gn = /* @__PURE__ */ q({
  __name: "TokenOnceDialog",
  props: {
    compact: { type: Boolean }
  },
  setup(t, { expose: s }) {
    const l = De(), u = f(!1), p = f(""), m = f(""), g = f("");
    function d(h, i, b) {
      p.value = h, m.value = i, g.value = b, u.value = !0;
    }
    s({ show: d });
    async function r() {
      if (m.value)
        try {
          await l.onIdentityToken(m.value, g.value), u.value = !1, j(a("access.savedToBrowser", { name: m.value }));
        } catch (h) {
          J(h instanceof Error ? h.message : String(h));
        }
    }
    async function y(h) {
      try {
        await navigator.clipboard.writeText(h), j(a("access.copied"));
      } catch {
        J(h);
      }
    }
    return (h, i) => {
      const b = X, A = Ve;
      return _(), V(A, {
        modelValue: u.value,
        "onUpdate:modelValue": i[1] || (i[1] = (I) => u.value = I),
        title: p.value,
        width: t.compact ? "96%" : "560px"
      }, {
        footer: o(() => [
          e(l).onIdentityToken ? (_(), V(b, {
            key: 0,
            type: "primary",
            onClick: r
          }, {
            default: o(() => [
              k(c(e(a)("access.saveToBrowser")), 1)
            ]),
            _: 1
          })) : F("", !0),
          n(b, {
            type: "primary",
            onClick: i[0] || (i[0] = () => {
              y(g.value), u.value = !1;
            })
          }, {
            default: o(() => [
              k(c(e(a)("access.copyToken")), 1)
            ]),
            _: 1
          })
        ]),
        default: o(() => [
          E("p", null, c(e(a)("access.created")), 1),
          E("code", _n, c(g.value), 1)
        ]),
        _: 1
      }, 8, ["modelValue", "title", "width"]);
    };
  }
}), bn = /* @__PURE__ */ W(gn, [["__scopeId", "data-v-cb1c8349"]]), yn = { class: "save-row" }, hn = /* @__PURE__ */ q({
  __name: "EmbeddingSettingsCard",
  props: {
    embeddingEnabled: { type: Boolean },
    embeddingBaseUrl: {},
    embeddingModel: {},
    embeddingApiKey: {}
  },
  setup(t) {
    const s = t, l = f(s.embeddingEnabled), u = f(s.embeddingBaseUrl), p = f(s.embeddingModel), m = f(s.embeddingApiKey);
    pe(
      () => [s.embeddingEnabled, s.embeddingBaseUrl, s.embeddingModel, s.embeddingApiKey],
      ([h, i, b, A]) => {
        l.value = h, u.value = i, p.value = b, m.value = A;
      }
    );
    const g = te(), d = f(!1), r = f(null);
    async function y() {
      d.value = !0, r.value = null;
      try {
        if (await Q(
          () => g.put("/api/settings", {
            embedding_enabled: l.value,
            embedding_base_url: u.value,
            embedding_model: p.value,
            embedding_api_key: m.value
          })
        ) === void 0 || (j(a("access.saved")), !l.value)) return;
        r.value = await Q(() => g.post("/api/embeddings/test", {}));
      } finally {
        d.value = !1;
      }
    }
    return (h, i) => {
      const b = Ce, A = ot, I = we, S = _e, w = ke, x = X, B = de;
      return _(), V(B, { shadow: "never" }, {
        header: o(() => [
          k(c(e(a)("access.embeddingTitle")), 1)
        ]),
        default: o(() => {
          var D;
          return [
            n(b, {
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
                    n(A, {
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
                    n(S, {
                      modelValue: u.value,
                      "onUpdate:modelValue": i[1] || (i[1] = (O) => u.value = O),
                      placeholder: "https://api.siliconflow.cn/v1 或 http://127.0.0.1:11434/v1"
                    }, null, 8, ["modelValue"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                n(I, {
                  label: e(a)("access.embeddingModelLabel")
                }, {
                  default: o(() => [
                    n(S, {
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
                    n(S, {
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
            E("div", yn, [
              n(x, {
                type: "primary",
                loading: d.value,
                onClick: y
              }, {
                default: o(() => [
                  k(c(e(a)("access.embeddingSaveTest")), 1)
                ]),
                _: 1
              }, 8, ["loading"])
            ]),
            (D = r.value) != null && D.ok ? (_(), V(b, {
              key: 0,
              title: e(a)("access.embeddingTestOk", { dim: r.value.dim ?? 0, ms: r.value.elapsed_ms ?? 0 }),
              type: "success",
              "show-icon": "",
              closable: !1,
              class: "settings-hint"
            }, null, 8, ["title"])) : r.value && !r.value.ok ? (_(), V(b, {
              key: 1,
              title: e(a)("access.embeddingTestFail", { error: r.value.error ?? "" }),
              type: "error",
              "show-icon": "",
              closable: !1,
              class: "settings-hint"
            }, null, 8, ["title"])) : F("", !0)
          ];
        }),
        _: 1
      });
    };
  }
}), kn = /* @__PURE__ */ W(hn, [["__scopeId", "data-v-03caf794"]]), wn = { class: "field-hint" }, Cn = {
  key: 0,
  class: "default-view"
}, Tn = { class: "default-text" }, Sn = { class: "field-hint" }, En = { class: "save-row" }, $n = /* @__PURE__ */ q({
  __name: "PromptSettingsCard",
  props: {
    instructions: {},
    conventions: {},
    defaultInstructions: {}
  },
  setup(t) {
    const s = t, l = f(s.instructions), u = f(s.conventions);
    pe(
      () => [s.instructions, s.conventions],
      ([d, r]) => {
        l.value = d, u.value = r;
      }
    );
    const p = te(), m = f(!1);
    async function g() {
      m.value = !0;
      try {
        await Q(
          () => p.put("/api/settings", {
            instructions: l.value,
            conventions: u.value
          })
        ), j(a("access.saved"));
      } finally {
        m.value = !1;
      }
    }
    return (d, r) => {
      const y = _e, h = we, i = ke, b = X, A = de;
      return _(), V(A, { shadow: "never" }, {
        header: o(() => [
          k(c(e(a)("access.settingsTitle")), 1)
        ]),
        default: o(() => [
          n(i, {
            "label-position": "top",
            onSubmit: r[2] || (r[2] = ze(() => {
            }, ["prevent"]))
          }, {
            default: o(() => [
              n(h, {
                label: e(a)("access.instructionsLabel")
              }, {
                default: o(() => [
                  n(y, {
                    modelValue: l.value,
                    "onUpdate:modelValue": r[0] || (r[0] = (I) => l.value = I),
                    type: "textarea",
                    rows: 5,
                    placeholder: e(a)("access.instructionsPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"]),
                  E("div", wn, c(e(a)("access.instructionsHint")), 1),
                  l.value ? (_(), P("details", Cn, [
                    E("summary", null, c(e(a)("access.viewDefault")), 1),
                    E("pre", Tn, c(t.defaultInstructions), 1)
                  ])) : F("", !0)
                ]),
                _: 1
              }, 8, ["label"]),
              n(h, {
                label: e(a)("access.conventionsLabel")
              }, {
                default: o(() => [
                  n(y, {
                    modelValue: u.value,
                    "onUpdate:modelValue": r[1] || (r[1] = (I) => u.value = I),
                    type: "textarea",
                    rows: 4,
                    placeholder: e(a)("access.conventionsPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"]),
                  E("div", Sn, c(e(a)("access.conventionsHint")), 1)
                ]),
                _: 1
              }, 8, ["label"])
            ]),
            _: 1
          }),
          E("div", En, [
            n(b, {
              type: "primary",
              loading: m.value,
              onClick: g
            }, {
              default: o(() => [
                k(c(e(a)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ])
        ]),
        _: 1
      });
    };
  }
}), An = /* @__PURE__ */ W($n, [["__scopeId", "data-v-3bcdc16f"]]);
function Ke() {
  const t = te(), s = f({ ok: !0, issues: [] }), l = f(!1), u = f(!1), p = f(!1), m = f(!1), g = f(!1);
  async function d() {
    u.value = !0;
    try {
      s.value = await La(t), l.value = !0;
    } finally {
      u.value = !1;
    }
  }
  async function r() {
    p.value = !0;
    try {
      const i = await za(t), b = URL.createObjectURL(i), A = document.createElement("a");
      A.href = b, A.download = "agent-memory-export.json", A.click(), URL.revokeObjectURL(b);
    } finally {
      p.value = !1;
    }
  }
  async function y(i) {
    m.value = !0;
    try {
      const b = await i.text();
      let A;
      try {
        A = JSON.parse(b);
      } catch {
        throw new Error(a("errors.invalidBackup"));
      }
      return await Ra(t, A);
    } finally {
      m.value = !1;
    }
  }
  async function h() {
    g.value = !0;
    try {
      let i = 0;
      for (; ; ) {
        const b = await Oa(t);
        if (!b.configured) throw new Error(a("ops.embeddingNotConfigured"));
        if (b.error) throw new Error(b.error);
        if (i += b.processed ?? 0, (b.processed ?? 0) === 0) break;
      }
      return i;
    } finally {
      g.value = !1;
    }
  }
  return {
    doctor: s,
    doctorRan: l,
    doctorLoading: u,
    exporting: p,
    importing: m,
    backfilling: g,
    runDoctor: d,
    exportData: r,
    importFile: y,
    backfill: h
  };
}
const Vn = { class: "actions" }, In = /* @__PURE__ */ q({
  __name: "BackupCard",
  setup(t) {
    const { exporting: s, importing: l, exportData: u, importFile: p } = Ke(), m = f(null);
    function g(d) {
      var h;
      const r = d.target, y = (h = r.files) == null ? void 0 : h[0];
      r.value = "", y && p(y).then((i) => {
        j(a("access.imported", { memories: i.imported_memories, tags: i.imported_tags }));
      }).catch((i) => {
        J(i instanceof Error ? i.message : String(i));
      });
    }
    return (d, r) => {
      const y = X, h = be, i = ge, b = de;
      return _(), V(b, { shadow: "never" }, {
        header: o(() => [
          k(c(e(a)("access.backupCard")), 1)
        ]),
        default: o(() => [
          E("div", Vn, [
            n(y, {
              icon: e(Ft),
              loading: e(s),
              onClick: r[0] || (r[0] = (A) => e(Q)(e(u)))
            }, {
              default: o(() => [
                k(c(e(a)("access.export")), 1)
              ]),
              _: 1
            }, 8, ["icon", "loading"]),
            n(y, {
              icon: e(Ht),
              loading: e(l),
              onClick: r[1] || (r[1] = (A) => {
                var I;
                return (I = m.value) == null ? void 0 : I.click();
              })
            }, {
              default: o(() => [
                k(c(e(a)("access.import")), 1)
              ]),
              _: 1
            }, 8, ["icon", "loading"]),
            n(i, {
              content: e(a)("access.importHint"),
              placement: "top"
            }, {
              default: o(() => [
                n(h, { class: "am-info" }, {
                  default: o(() => [
                    n(e(Te))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"]),
            E("input", {
              ref_key: "importInput",
              ref: m,
              type: "file",
              accept: "application/json,.json",
              style: { display: "none" },
              onChange: g
            }, null, 544)
          ])
        ]),
        _: 1
      });
    };
  }
}), xn = /* @__PURE__ */ W(In, [["__scopeId", "data-v-106154fe"]]), Un = { class: "card-header" }, Dn = {
  key: 2,
  class: "issues"
}, Mn = /* @__PURE__ */ q({
  __name: "DoctorCard",
  setup(t) {
    const { doctor: s, doctorRan: l, doctorLoading: u, runDoctor: p } = Ke();
    return (m, g) => {
      const d = X, r = Ce, y = lt, h = de;
      return _(), V(h, { shadow: "never" }, {
        header: o(() => [
          E("div", Un, [
            E("span", null, c(e(a)("access.doctorCard")), 1),
            n(d, {
              size: "small",
              icon: e(He),
              loading: e(u),
              onClick: g[0] || (g[0] = (i) => e(Q)(e(p)))
            }, {
              default: o(() => [
                k(c(e(a)("access.runDoctor")), 1)
              ]),
              _: 1
            }, 8, ["icon", "loading"])
          ])
        ]),
        default: o(() => [
          e(l) ? (_(), P(ee, { key: 0 }, [
            e(s).ok ? (_(), V(r, {
              key: 0,
              title: e(a)("access.doctorOk"),
              type: "success",
              "show-icon": "",
              closable: !1
            }, null, 8, ["title"])) : (_(), V(r, {
              key: 1,
              title: e(a)("access.doctorFail", { count: e(s).issues.length }),
              type: "error",
              "show-icon": "",
              closable: !1
            }, null, 8, ["title"])),
            e(s).ok ? F("", !0) : (_(), P("ul", Dn, [
              (_(!0), P(ee, null, re(e(s).issues, (i, b) => (_(), P("li", { key: b }, c(i), 1))), 128))
            ]))
          ], 64)) : (_(), V(y, {
            key: 1,
            description: e(a)("access.doctorEmpty"),
            "image-size": 60
          }, null, 8, ["description"]))
        ]),
        _: 1
      });
    };
  }
}), Pn = /* @__PURE__ */ W(Mn, [["__scopeId", "data-v-ae41a618"]]), Bn = { class: "card-header" }, Ln = /* @__PURE__ */ q({
  __name: "EmbeddingCoverageCard",
  props: {
    stats: {},
    compact: { type: Boolean }
  },
  emits: ["changed"],
  setup(t, { emit: s }) {
    const l = t;
    te();
    const { backfilling: u, backfill: p } = Ke(), m = s, g = le(() => {
      var r, y, h, i;
      return (((y = (r = l.stats) == null ? void 0 : r.embedding) == null ? void 0 : y.embedded) ?? 0) + (((i = (h = l.stats) == null ? void 0 : h.embedding) == null ? void 0 : i.pending) ?? 0);
    });
    async function d() {
      const r = await Q(p);
      r !== void 0 && (m("changed"), j(r === 0 ? a("access.embeddingUpToDate") : a("access.embeddingDone", { count: r })));
    }
    return (r, y) => {
      const h = X, i = nt, b = at, A = Ce, I = de;
      return _(), V(I, { shadow: "never" }, {
        header: o(() => [
          E("div", Bn, [
            E("span", null, c(e(a)("access.embeddingCard")), 1),
            n(h, {
              size: "small",
              icon: e(st),
              loading: e(u),
              onClick: d
            }, {
              default: o(() => [
                k(c(e(a)("access.runBackfill")), 1)
              ]),
              _: 1
            }, 8, ["icon", "loading"])
          ])
        ]),
        default: o(() => {
          var S, w;
          return [
            (w = (S = t.stats) == null ? void 0 : S.embedding) != null && w.enabled ? (_(), P(ee, { key: 0 }, [
              n(b, {
                column: t.compact ? 1 : 2,
                border: ""
              }, {
                default: o(() => [
                  n(i, {
                    label: e(a)("access.embeddingModel")
                  }, {
                    default: o(() => [
                      k(c(t.stats.embedding.model ?? "—"), 1)
                    ]),
                    _: 1
                  }, 8, ["label"]),
                  n(i, {
                    label: e(a)("access.embeddingCoverage")
                  }, {
                    default: o(() => [
                      k(c(t.stats.embedding.embedded ?? 0) + " / " + c(g.value), 1)
                    ]),
                    _: 1
                  }, 8, ["label"])
                ]),
                _: 1
              }, 8, ["column"]),
              (t.stats.embedding.pending ?? 0) > 0 ? (_(), V(A, {
                key: 0,
                title: e(a)("access.embeddingPending", { count: t.stats.embedding.pending }),
                type: "warning",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])) : F("", !0)
            ], 64)) : (_(), V(A, {
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
}), zn = /* @__PURE__ */ W(Ln, [["__scopeId", "data-v-ea4ac165"]]), Rn = {
  key: 0,
  class: "am-panel-header"
}, On = { class: "am-heading" }, Nn = { class: "am-panel-title" }, Fn = /* @__PURE__ */ q({
  __name: "AdminPanel",
  props: {
    who: { default: null },
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: s }) {
    const l = t, u = te(), p = f([]), m = f(""), g = f(""), d = f(""), r = f(!1), y = f(!1), h = f(""), i = f(""), b = f(""), A = le(() => {
      var U;
      return !!l.who && ((U = l.who.permissions) == null ? void 0 : U.admin) === !0;
    }), I = f(null), S = f(null), { compact: w } = Me(S);
    async function x() {
      await Q(async () => {
        const [U, v, $] = await Promise.all([
          u.get("/api/identities"),
          u.get("/api/settings"),
          u.get("/api/stats")
        ]);
        p.value = Array.isArray(U == null ? void 0 : U.identities) ? U.identities : [], m.value = (v == null ? void 0 : v.instructions) ?? "", g.value = (v == null ? void 0 : v.conventions) ?? "", d.value = (v == null ? void 0 : v.default_instructions) ?? "", r.value = (v == null ? void 0 : v.auth_required) === !0, y.value = (v == null ? void 0 : v.embedding_enabled) === !0, h.value = (v == null ? void 0 : v.embedding_base_url) ?? "", i.value = (v == null ? void 0 : v.embedding_model) ?? "", b.value = (v == null ? void 0 : v.embedding_api_key) ?? "", I.value = $ ?? null;
      });
    }
    he(() => {
      A.value && x();
    }), pe(A, (U) => {
      U && x();
    }), s({
      refresh: () => {
        A.value && Q(x);
      }
    });
    const B = f(null), D = f(null), O = le(() => p.value.some((U) => {
      var v;
      return ((v = U.permissions) == null ? void 0 : v.admin) === !0;
    }));
    function L(U) {
      var v;
      (v = D.value) == null || v.show(a("access.createTitle"), U.name, U.token), x();
    }
    async function R(U) {
      try {
        await ye.confirm(a("access.deleteConfirm", { name: U.name }), a("access.deleteTitle"), {
          type: "warning",
          confirmButtonText: a("common.delete"),
          cancelButtonText: a("common.cancel")
        });
      } catch {
        return;
      }
      await Q(() => u.del(`/api/identities/${encodeURIComponent(U.name)}`)) !== void 0 && (j(a("access.deleted")), await x());
    }
    async function K(U) {
      var $;
      try {
        await ye.confirm(a("access.resetConfirm", { name: U.name }), a("access.resetTitle"), {
          type: "warning",
          confirmButtonText: a("access.resetToken"),
          cancelButtonText: a("common.cancel")
        });
      } catch {
        return;
      }
      const v = await Q(
        () => u.post(`/api/identities/${encodeURIComponent(U.name)}/token-reset`, {})
      );
      v !== void 0 && (v != null && v.token && (($ = D.value) == null || $.show(a("access.resetTitle"), U.name, v.token)), await x());
    }
    return (U, v) => {
      const $ = be, z = ge, N = X, ce = Ce;
      return _(), P("div", {
        ref_key: "rootRef",
        ref: S,
        class: "am-panel admin-panel"
      }, [
        t.showHeader ? (_(), P("div", Rn, [
          E("div", On, [
            E("h2", Nn, c(l.title ?? e(a)("access.title")), 1),
            n(z, {
              content: l.subtitle ?? e(a)("access.subtitle"),
              placement: "top"
            }, {
              default: o(() => [
                n($, { class: "am-info" }, {
                  default: o(() => [
                    n(e(Te))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          A.value ? (_(), V(N, {
            key: 0,
            type: "primary",
            icon: e(Fe),
            onClick: v[0] || (v[0] = (G) => {
              var Y;
              return (Y = B.value) == null ? void 0 : Y.openCreate();
            })
          }, {
            default: o(() => [
              k(c(e(a)("access.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])) : F("", !0)
        ])) : F("", !0),
        t.who ? t.who.mode === "open" ? (_(), V(ce, {
          key: 2,
          type: "warning",
          title: e(a)("access.openMode"),
          closable: !1
        }, null, 8, ["title"])) : A.value ? F("", !0) : (_(), V(ce, {
          key: 3,
          type: "info",
          title: e(a)("access.needAdmin"),
          closable: !1
        }, null, 8, ["title"])) : (_(), V(ce, {
          key: 1,
          type: "info",
          title: e(a)("access.needAdmin"),
          closable: !1
        }, null, 8, ["title"])),
        t.who && A.value ? (_(), P(ee, { key: 4 }, [
          n(on, {
            "auth-required": r.value,
            "has-admin-identity": O.value
          }, null, 8, ["auth-required", "has-admin-identity"]),
          n(cn, {
            identities: p.value,
            compact: e(w),
            onEdit: v[1] || (v[1] = (G) => {
              var Y;
              return (Y = B.value) == null ? void 0 : Y.openEdit(G);
            }),
            onDelete: R,
            onResetToken: K
          }, null, 8, ["identities", "compact"]),
          n(kn, {
            "embedding-enabled": y.value,
            "embedding-base-url": h.value,
            "embedding-model": i.value,
            "embedding-api-key": b.value
          }, null, 8, ["embedding-enabled", "embedding-base-url", "embedding-model", "embedding-api-key"]),
          n(An, {
            instructions: m.value,
            conventions: g.value,
            "default-instructions": d.value
          }, null, 8, ["instructions", "conventions", "default-instructions"]),
          n(xn),
          n(Pn),
          n(zn, {
            stats: I.value,
            compact: e(w),
            onChanged: x
          }, null, 8, ["stats", "compact"])
        ], 64)) : F("", !0),
        n(vn, {
          ref_key: "formDialog",
          ref: B,
          compact: e(w),
          onSaved: x,
          onCreated: L
        }, null, 8, ["compact"]),
        n(bn, {
          ref_key: "tokenDialog",
          ref: D,
          compact: e(w)
        }, null, 8, ["compact"])
      ], 512);
    };
  }
}), Zn = /* @__PURE__ */ W(Fn, [["__scopeId", "data-v-604ebc45"]]);
export {
  Zn as AdminPanel,
  ft as MarkdownView,
  $a as MemoriesPanel,
  Yn as MemoryAdmin,
  ga as MemoryDetailDrawer,
  ua as MemoryEditorDialog,
  dt as MemoryUIConfigKey,
  Ka as OpsPanel,
  Ma as TagsPanel,
  Gt as applyMemoryUILocalePreference,
  Zt as buildMemoriesQuery,
  Yt as createApiClient,
  Qn as currentMemoryUILocale,
  Xt as formatSize,
  ve as formatTime,
  Le as isSearchMode,
  Ue as memoryUIi18n,
  Xn as provideMemoryUI,
  ta as renderMarkdown,
  aa as sanitizeHtml,
  Wt as setMemoryUILocale,
  a as t,
  J as toastError,
  j as toastSuccess,
  Ke as useAdmin,
  te as useApiClient,
  ia as useMemories,
  De as useMemoryConfig,
  Na as useOps,
  Aa as useTags
};
