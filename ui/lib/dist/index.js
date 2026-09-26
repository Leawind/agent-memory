import { inject as Yt, provide as Zt, ref as m, computed as pe, onMounted as xe, watch as Fe, onUnmounted as ea, defineComponent as _e, openBlock as f, createElementBlock as x, createBlock as A, unref as e, withCtx as o, createVNode as a, createElementVNode as k, toDisplayString as s, createTextVNode as u, Fragment as se, renderList as be, createCommentVNode as W, withKeys as ta, isRef as ge, withDirectives as Ce, reactive as ht, renderSlot as aa, vShow as Xe, withModifiers as Ye } from "vue";
import { createI18n as la } from "vue-i18n";
import { ElDialog as et, ElForm as tt, ElFormItem as at, ElInput as He, ElRadioGroup as Ke, ElRadioButton as lt, ElSelect as kt, ElOption as wt, ElButton as $e, ElDrawer as oa, ElTag as qe, ElDivider as Tt, ElTooltip as je, ElIcon as De, ElAlert as Ct, ElTable as ot, ElTableColumn as nt, ElLoadingDirective as St, ElPagination as na, ElRadio as sa, ElRow as ia, ElCol as ra, ElCard as Et, ElStatistic as da, ElDescriptions as Vt, ElDescriptionsItem as At, ElContainer as ca, ElAside as ua, ElMenu as ma, ElMenuItem as pa, ElMain as fa, ElTabs as va, ElTabPane as ga, ElSwitch as ba, ElEmpty as _a, ElCheckbox as ya } from "element-plus/es";
import { InfoFilled as Ue, Plus as st, Search as it, Refresh as Mt, Collection as ha, Notebook as ka, PriceTag as wa, Odometer as Ta, Download as Ca, UploadFilled as Sa } from "@element-plus/icons-vue";
import { ElMessage as Ea, ElMessageBox as Pe } from "element-plus";
import { Marked as Va } from "marked";
import Pt from "dompurify";
const Aa = {
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
}, Ma = {
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
function Ut() {
  var l;
  return typeof navigator > "u" || (l = navigator.language) != null && l.toLowerCase().startsWith("zh") ? "zh" : "en";
}
const Je = la({
  legacy: !1,
  locale: Ut(),
  fallbackLocale: "zh",
  messages: { zh: Aa, en: Ma },
  // 面向宿主组件库，缺 key 时静默回退即可，不刷控制台
  missingWarn: !1,
  fallbackWarn: !1
}), { t } = Je.global;
function Pa(l) {
  Je.global.locale.value = l;
}
function Ua(l) {
  Pa(l === "auto" ? Ut() : l);
}
function fo() {
  return Je.global.locale.value;
}
const xt = Symbol("memory-ui-config"), Ne = {
  baseUrl: "",
  fetch: (...l) => globalThis.fetch(...l),
  defaultPageSize: 20,
  locale: "auto"
};
function vo(l) {
  l.locale && Ua(l.locale), Zt(xt, l);
}
function rt() {
  const l = Yt(xt);
  return {
    baseUrl: ((l == null ? void 0 : l.baseUrl) ?? Ne.baseUrl).replace(/\/+$/, ""),
    fetch: (l == null ? void 0 : l.fetch) ?? Ne.fetch,
    defaultPageSize: (l == null ? void 0 : l.defaultPageSize) ?? Ne.defaultPageSize,
    locale: (l == null ? void 0 : l.locale) ?? Ne.locale,
    onIdentityToken: l == null ? void 0 : l.onIdentityToken,
    onAuthChanged: l == null ? void 0 : l.onAuthChanged
  };
}
const xa = 72;
function $t(l, r) {
  let i;
  return i = Ea({
    type: l,
    message: r,
    offset: xa,
    showClose: !0,
    grouping: !0,
    onClick: () => i.close()
  }), i;
}
function X(l) {
  return $t("success", l);
}
function Y(l) {
  return $t("error", l);
}
function Se(l) {
  if (!l) return "—";
  const r = Je.global.locale.value === "en" ? "en-US" : "zh-CN";
  return new Date(l * 1e3).toLocaleString(r, { hour12: !1 });
}
function $a(l) {
  return l == null ? "—" : l < 1024 ? `${l} B` : l < 1024 * 1024 ? `${(l / 1024).toFixed(1)} KB` : `${(l / 1024 / 1024).toFixed(2)} MB`;
}
function Da(l) {
  async function r(d, g = {}) {
    const _ = await l.fetch(l.baseUrl + d, {
      headers: { "Content-Type": "application/json" },
      ...g
    }), w = await _.text();
    let v = null;
    try {
      v = w ? JSON.parse(w) : null;
    } catch {
      v = null;
    }
    if (!_.ok) {
      const y = (v == null ? void 0 : v.error) ?? t("errors.http", { status: _.status });
      throw new Error(y);
    }
    return v;
  }
  async function i(d) {
    var _;
    const g = await l.fetch(l.baseUrl + d);
    if (!g.ok) {
      const w = await g.text().catch(() => "");
      let v = t("errors.http", { status: g.status });
      try {
        v = ((_ = JSON.parse(w)) == null ? void 0 : _.error) ?? v;
      } catch {
      }
      throw new Error(v);
    }
    return g.blob();
  }
  return {
    get: (d) => r(d),
    post: (d, g) => r(d, { method: "POST", body: JSON.stringify(g ?? {}) }),
    put: (d, g) => r(d, { method: "PUT", body: JSON.stringify(g ?? {}) }),
    del: (d) => r(d, { method: "DELETE" }),
    getBlob: i
  };
}
function he() {
  return Da(rt());
}
function Ia(l) {
  const r = Ze(l.query), i = new URLSearchParams();
  return r ? (i.set("query", l.query.trim()), l.tagFilter && i.set("tags", l.tagFilter), l.mode && l.mode !== "auto" && i.set("mode", l.mode)) : (l.tagFilter && i.set("tag", l.tagFilter), i.set("sort", l.sort), i.set("order", l.order)), i.set("offset", String((l.page - 1) * l.pageSize)), i.set("limit", String(l.pageSize)), i.toString();
}
function Ze(l) {
  return l.trim().length > 0;
}
const Ba = new Va();
function La(l) {
  const r = Ba.parse(l, { async: !1 });
  return Pt.sanitize(r);
}
function za(l) {
  return Pt.sanitize(l);
}
function dt(l, r) {
  return l.get(`/api/memories/${encodeURIComponent(r)}`);
}
function Dt(l, r) {
  return l.post("/api/memories", r);
}
function It(l, r, i) {
  return l.put(`/api/memories/${encodeURIComponent(r)}`, i);
}
function Ra(l, r) {
  return l.del(`/api/memories/${encodeURIComponent(r)}`);
}
function Bt(l, r) {
  const i = r ? `?filter=${encodeURIComponent(r)}` : "";
  return l.get(`/api/tags${i}`);
}
function Oa(l, r, i) {
  return l.post("/api/tags", { name: r, description: i });
}
function Na(l, r, i, d) {
  const g = { description: d }, _ = i.trim();
  return _ && _ !== r && (g.new_name = _), l.put(`/api/tags/${encodeURIComponent(r)}`, g);
}
function Fa(l, r, i) {
  return l.del(`/api/tags/${encodeURIComponent(r)}?mode=${i}`);
}
function Ha() {
  const l = he(), { defaultPageSize: r } = rt(), i = m(""), d = m(""), g = m("auto"), _ = m("updated_at"), w = m("desc"), v = m(1), y = m(r), M = m([]), V = m([]), b = m(0), T = m(!1), P = m([]), R = pe(() => Ze(i.value));
  function $() {
    return v.value = 1, j();
  }
  function U() {
    return Ia({
      query: i.value,
      tagFilter: d.value,
      mode: g.value,
      sort: _.value,
      order: w.value,
      page: v.value,
      pageSize: y.value
    });
  }
  let I = 0;
  async function j() {
    var le;
    const N = ++I;
    T.value = !0;
    try {
      const K = U();
      if (Ze(i.value)) {
        const J = await l.get(`/api/memories?${K}`);
        if (N !== I) return;
        V.value = (J.results ?? []).map((C) => ({ ...C, snippet: za(C.snippet) })), b.value = J.total_matches ?? 0, note.value = J.semantic_fallback ? t("memories.semanticFallback") : "";
      } else {
        const J = await l.get(`/api/memories?${K}`);
        if (N !== I) return;
        const C = Math.max(1, Math.ceil(J.total / y.value));
        if (((le = J.memories) == null ? void 0 : le.length) === 0 && J.total > 0 && v.value > C)
          return v.value = C, T.value = !1, j();
        M.value = J.memories ?? [], b.value = J.total ?? 0;
      }
    } finally {
      N === I && (T.value = !1);
    }
  }
  async function H() {
    try {
      const N = await Bt(l);
      P.value = (N.tags ?? []).map((le) => le.name);
    } catch {
    }
  }
  async function ae(N) {
    if (N.id) {
      const le = await dt(l, N.id), K = new Set(le.tags), J = new Set(N.tags);
      await It(l, N.id, {
        summary: N.summary,
        content: N.content,
        add_tags: [...J].filter((C) => !K.has(C)),
        remove_tags: [...K].filter((C) => !J.has(C))
      });
    } else
      await Dt(l, { summary: N.summary, content: N.content, tags: N.tags });
    await Promise.all([j(), H()]);
  }
  async function O(N) {
    await Ra(l, N), await j();
  }
  return xe(() => {
    j().catch(() => {
    }), H();
  }), {
    query: i,
    tagFilter: d,
    mode: g,
    sort: _,
    order: w,
    page: v,
    pageSize: y,
    rows: M,
    searchResults: V,
    total: b,
    loading: T,
    tagOptions: P,
    searching: R,
    onSearch: $,
    reload: j,
    loadTagOptions: H,
    saveMemory: ae,
    removeMemory: O
  };
}
function We(l) {
  const r = m(0);
  let i = null;
  function d(w) {
    i == null || i.disconnect(), i = null, !(!w || typeof ResizeObserver > "u") && (i = new ResizeObserver((v) => {
      var y;
      r.value = ((y = v[0]) == null ? void 0 : y.contentRect.width) ?? 0;
    }), i.observe(w));
  }
  xe(() => d(l.value)), Fe(l, (w) => d(w)), ea(() => i == null ? void 0 : i.disconnect());
  const g = pe(() => r.value > 0 && r.value < 960), _ = pe(() => r.value > 0 && r.value < 720);
  return { width: r, compact: g, narrow: _ };
}
const Ka = ["innerHTML"], Lt = /* @__PURE__ */ _e({
  __name: "MarkdownView",
  props: {
    source: {}
  },
  setup(l) {
    const r = l, i = pe(() => La(r.source));
    return (d, g) => (f(), x("div", {
      class: "md-body",
      innerHTML: i.value
    }, null, 8, Ka));
  }
}), qa = { class: "content-label" }, ja = /* @__PURE__ */ _e({
  __name: "MemoryEditorDialog",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    tagOptions: {},
    width: {}
  },
  emits: ["update:visible", "saved"],
  setup(l, { emit: r }) {
    const i = l, d = r, g = he(), _ = m(!1), w = m("edit"), v = m({ id: null, summary: "", content: "", tags: [] });
    let y = [];
    Fe(
      () => i.visible,
      async (V) => {
        if (V)
          if (w.value = "edit", i.memoryId)
            try {
              const b = await dt(g, i.memoryId);
              v.value = { id: b.id, summary: b.summary, content: b.content, tags: [...b.tags] }, y = [...b.tags];
            } catch (b) {
              Y(b instanceof Error ? b.message : String(b)), d("update:visible", !1);
            }
          else
            v.value = { id: null, summary: "", content: "", tags: [] }, y = [];
      }
    );
    async function M() {
      _.value = !0;
      try {
        if (v.value.id) {
          const V = new Set(y), b = new Set(v.value.tags);
          await It(g, v.value.id, {
            summary: v.value.summary,
            content: v.value.content,
            add_tags: [...b].filter((T) => !V.has(T)),
            remove_tags: [...V].filter((T) => !b.has(T))
          }), X(t("editor.updated"));
        } else
          await Dt(g, {
            summary: v.value.summary,
            content: v.value.content,
            tags: v.value.tags
          }), X(t("editor.created"));
        d("update:visible", !1), d("saved");
      } catch (V) {
        Y(V instanceof Error ? V.message : String(V));
      } finally {
        _.value = !1;
      }
    }
    return (V, b) => {
      const T = He, P = at, R = lt, $ = Ke, U = wt, I = kt, j = tt, H = $e, ae = et;
      return f(), A(ae, {
        "model-value": l.visible,
        title: v.value.id ? e(t)("editor.editTitle") : e(t)("editor.createTitle"),
        width: l.width,
        "onUpdate:modelValue": b[5] || (b[5] = (O) => d("update:visible", O))
      }, {
        footer: o(() => [
          a(H, {
            onClick: b[4] || (b[4] = (O) => d("update:visible", !1))
          }, {
            default: o(() => [
              u(s(e(t)("common.cancel")), 1)
            ]),
            _: 1
          }),
          a(H, {
            type: "primary",
            loading: _.value,
            onClick: M
          }, {
            default: o(() => [
              u(s(e(t)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: o(() => [
          a(j, { "label-position": "top" }, {
            default: o(() => [
              a(P, {
                label: e(t)("editor.summaryLabel")
              }, {
                default: o(() => [
                  a(T, {
                    modelValue: v.value.summary,
                    "onUpdate:modelValue": b[0] || (b[0] = (O) => v.value.summary = O),
                    maxlength: "512",
                    "show-word-limit": "",
                    placeholder: e(t)("editor.summaryPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"]),
              a(P, null, {
                label: o(() => [
                  k("div", qa, [
                    k("span", null, s(e(t)("editor.contentLabel")), 1),
                    a($, {
                      modelValue: w.value,
                      "onUpdate:modelValue": b[1] || (b[1] = (O) => w.value = O),
                      size: "small"
                    }, {
                      default: o(() => [
                        a(R, { value: "edit" }, {
                          default: o(() => [
                            u(s(e(t)("editor.tabEdit")), 1)
                          ]),
                          _: 1
                        }),
                        a(R, { value: "preview" }, {
                          default: o(() => [
                            u(s(e(t)("editor.tabPreview")), 1)
                          ]),
                          _: 1
                        })
                      ]),
                      _: 1
                    }, 8, ["modelValue"])
                  ])
                ]),
                default: o(() => [
                  w.value === "edit" ? (f(), A(T, {
                    key: 0,
                    modelValue: v.value.content,
                    "onUpdate:modelValue": b[2] || (b[2] = (O) => v.value.content = O),
                    type: "textarea",
                    rows: 12,
                    maxlength: "262144",
                    "show-word-limit": "",
                    placeholder: e(t)("editor.contentPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])) : (f(), A(Lt, {
                    key: 1,
                    class: "content-preview",
                    source: v.value.content
                  }, null, 8, ["source"]))
                ]),
                _: 1
              }),
              a(P, {
                label: e(t)("editor.tagsLabel")
              }, {
                default: o(() => [
                  a(I, {
                    modelValue: v.value.tags,
                    "onUpdate:modelValue": b[3] || (b[3] = (O) => v.value.tags = O),
                    multiple: "",
                    filterable: "",
                    "allow-create": "",
                    "default-first-option": "",
                    placeholder: e(t)("editor.tagsPlaceholder"),
                    class: "tags-select"
                  }, {
                    default: o(() => [
                      (f(!0), x(se, null, be(l.tagOptions, (O) => (f(), A(U, {
                        key: O,
                        label: O,
                        value: O
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
}), Ee = (l, r) => {
  const i = l.__vccOpts || l;
  for (const [d, g] of r)
    i[d] = g;
  return i;
}, Ja = /* @__PURE__ */ Ee(ja, [["__scopeId", "data-v-a0fec0dd"]]), Wa = { class: "detail-summary" }, Ga = { class: "detail-tags" }, Qa = { class: "detail-toolbar" }, Xa = {
  key: 1,
  class: "detail-content"
}, Ya = /* @__PURE__ */ _e({
  __name: "MemoryDetailDrawer",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    size: {}
  },
  emits: ["update:visible"],
  setup(l, { emit: r }) {
    const i = l, d = r, g = he(), _ = m(null), w = m("rendered");
    return Fe(
      () => [i.visible, i.memoryId],
      async ([v]) => {
        if (!(!v || !i.memoryId)) {
          w.value = "rendered";
          try {
            _.value = await dt(g, i.memoryId);
          } catch (y) {
            Y(y instanceof Error ? y.message : String(y)), d("update:visible", !1);
          }
        }
      }
    ), (v, y) => {
      var R;
      const M = qe, V = Tt, b = lt, T = Ke, P = oa;
      return f(), A(P, {
        "model-value": l.visible,
        title: e(t)("drawer.title", { id: ((R = _.value) == null ? void 0 : R.id) ?? l.memoryId ?? "" }),
        size: l.size,
        "onUpdate:modelValue": y[1] || (y[1] = ($) => d("update:visible", $))
      }, {
        default: o(() => [
          _.value ? (f(), x(se, { key: 0 }, [
            k("h3", Wa, s(_.value.summary), 1),
            k("div", Ga, [
              (f(!0), x(se, null, be(_.value.tags, ($) => (f(), A(M, {
                key: $,
                size: "small",
                class: "am-tag"
              }, {
                default: o(() => [
                  u(s($), 1)
                ]),
                _: 2
              }, 1024))), 128))
            ]),
            a(V),
            k("div", Qa, [
              a(T, {
                modelValue: w.value,
                "onUpdate:modelValue": y[0] || (y[0] = ($) => w.value = $),
                size: "small"
              }, {
                default: o(() => [
                  a(b, { value: "rendered" }, {
                    default: o(() => [
                      u(s(e(t)("drawer.rendered")), 1)
                    ]),
                    _: 1
                  }),
                  a(b, { value: "source" }, {
                    default: o(() => [
                      u(s(e(t)("drawer.source")), 1)
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["modelValue"])
            ]),
            w.value === "rendered" ? (f(), A(Lt, {
              key: 0,
              source: _.value.content
            }, null, 8, ["source"])) : (f(), x("pre", Xa, s(_.value.content), 1))
          ], 64)) : W("", !0)
        ]),
        _: 1
      }, 8, ["model-value", "title", "size"]);
    };
  }
}), Za = /* @__PURE__ */ Ee(Ya, [["__scopeId", "data-v-613d4260"]]), el = {
  key: 0,
  class: "am-panel-header"
}, tl = { class: "am-heading" }, al = { class: "am-panel-title" }, ll = { class: "am-toolbar" }, ol = { class: "am-summary" }, nl = ["innerHTML"], sl = {
  key: 4,
  class: "am-pager"
}, il = {
  key: 5,
  class: "am-pager"
}, rl = /* @__PURE__ */ _e({
  __name: "MemoriesPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(l, { expose: r }) {
    const i = l, {
      query: d,
      tagFilter: g,
      mode: _,
      sort: w,
      order: v,
      page: y,
      pageSize: M,
      rows: V,
      searchResults: b,
      total: T,
      loading: P,
      tagOptions: R,
      searching: $,
      onSearch: U,
      reload: I,
      loadTagOptions: j,
      removeMemory: H
    } = Ha(), ae = pe(() => {
      if (P.value || T.value !== 0) return "";
      if ($.value) return t("memories.searchEmpty");
      const B = g.value.trim();
      return B ? t("memories.tagEmpty", { tag: B }) : "";
    }), O = m(null), { compact: N, narrow: le } = We(O), K = m(!1), J = m(null), C = m(!1), D = m(null);
    function z(B) {
      return B().catch((p) => Y(p instanceof Error ? p.message : String(p)));
    }
    function ke() {
      J.value = null, K.value = !0;
    }
    function G(B) {
      J.value = B, K.value = !0;
    }
    function ie(B) {
      D.value = B, C.value = !0;
    }
    function ue() {
      z(I), j();
    }
    function oe(B) {
      const { prop: p, order: Z } = B;
      Z && (p === "updated_at" || p === "created_at" || p === "id") ? (w.value = p, v.value = Z === "ascending" ? "asc" : "desc") : (w.value = "updated_at", v.value = "desc"), z(I);
    }
    async function fe(B) {
      try {
        await Pe.confirm(t("memories.deleteConfirm", { id: B.id }), t("memories.deleteTitle"), {
          type: "warning"
        });
      } catch {
        return;
      }
      try {
        await H(B.id), X(t("memories.deleted"));
      } catch (p) {
        Y(p instanceof Error ? p.message : String(p));
      }
    }
    return r({ refresh: () => z(I) }), (B, p) => {
      const Z = De, re = je, ee = $e, de = He, S = wt, F = kt, Ie = Ct, te = nt, Ve = qe, Be = ot, Le = na, ze = St;
      return f(), x("div", {
        ref_key: "rootRef",
        ref: O,
        class: "am-panel"
      }, [
        l.showHeader ? (f(), x("div", el, [
          k("div", tl, [
            k("h2", al, s(i.title ?? e(t)("memories.title")), 1),
            a(re, {
              content: i.subtitle ?? e(t)("memories.subtitle"),
              placement: "top"
            }, {
              default: o(() => [
                a(Z, { class: "am-info" }, {
                  default: o(() => [
                    a(e(Ue))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(ee, {
            type: "primary",
            icon: e(st),
            onClick: ke
          }, {
            default: o(() => [
              u(s(e(t)("memories.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : W("", !0),
        k("div", ll, [
          a(de, {
            modelValue: e(d),
            "onUpdate:modelValue": p[1] || (p[1] = (h) => ge(d) ? d.value = h : null),
            placeholder: e(t)("memories.searchPlaceholder"),
            clearable: "",
            class: "search",
            onKeyup: p[2] || (p[2] = ta((h) => z(e(U)), ["enter"])),
            onClear: p[3] || (p[3] = (h) => z(e(U)))
          }, {
            append: o(() => [
              a(ee, {
                icon: e(it),
                onClick: p[0] || (p[0] = (h) => z(e(U)))
              }, null, 8, ["icon"])
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(F, {
            modelValue: e(g),
            "onUpdate:modelValue": p[4] || (p[4] = (h) => ge(g) ? g.value = h : null),
            placeholder: e(t)("memories.tagFilter"),
            clearable: "",
            filterable: "",
            class: "tag-filter",
            onChange: p[5] || (p[5] = (h) => z(e(U)))
          }, {
            default: o(() => [
              (f(!0), x(se, null, be(e(R), (h) => (f(), A(S, {
                key: h,
                label: h,
                value: h
              }, null, 8, ["label", "value"]))), 128))
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(F, {
            modelValue: e(_),
            "onUpdate:modelValue": p[6] || (p[6] = (h) => ge(_) ? _.value = h : null),
            class: "mode-select",
            onChange: p[7] || (p[7] = (h) => z(e(U)))
          }, {
            default: o(() => [
              a(S, {
                label: e(t)("memories.modeAuto"),
                value: "auto"
              }, null, 8, ["label"]),
              a(S, {
                label: e(t)("memories.modeKeyword"),
                value: "keyword"
              }, null, 8, ["label"]),
              a(S, {
                label: e(t)("memories.modeHybrid"),
                value: "hybrid"
              }, null, 8, ["label"])
            ]),
            _: 1
          }, 8, ["modelValue"])
        ]),
        ae.value ? (f(), A(Ie, {
          key: 1,
          title: ae.value,
          type: "info",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : W("", !0),
        e($) ? Ce((f(), A(Be, {
          key: 2,
          data: e(b)
        }, {
          default: o(() => [
            a(te, {
              prop: "id",
              label: e(t)("memories.colId"),
              width: "80"
            }, null, 8, ["label"]),
            a(te, {
              label: e(t)("memories.colSummary")
            }, {
              default: o(({ row: h }) => [
                k("div", ol, s(h.summary), 1),
                k("div", {
                  class: "am-snippet",
                  innerHTML: h.snippet
                }, null, 8, nl)
              ]),
              _: 1
            }, 8, ["label"]),
            a(te, {
              label: e(t)("memories.colTags"),
              "min-width": "150"
            }, {
              default: o(({ row: h }) => [
                (f(!0), x(se, null, be(h.tags, (ne) => (f(), A(Ve, {
                  key: ne,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: o(() => [
                    u(s(ne), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            a(te, {
              prop: "score",
              label: e(t)("memories.colScore"),
              width: "80",
              sortable: ""
            }, null, 8, ["label"]),
            a(te, {
              label: e(t)("memories.colUpdatedAt"),
              width: "170"
            }, {
              default: o(({ row: h }) => [
                u(s(e(Se)(h.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(te, {
              label: e(t)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: o(({ row: h }) => [
                a(ee, {
                  link: "",
                  type: "primary",
                  onClick: (ne) => ie(h.id)
                }, {
                  default: o(() => [
                    u(s(e(t)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(ee, {
                  link: "",
                  type: "primary",
                  onClick: (ne) => G(h.id)
                }, {
                  default: o(() => [
                    u(s(e(t)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(ee, {
                  link: "",
                  type: "danger",
                  onClick: (ne) => fe(h)
                }, {
                  default: o(() => [
                    u(s(e(t)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [ze, e(P)]
        ]) : Ce((f(), A(Be, {
          key: 3,
          data: e(V),
          "default-sort": { prop: e(w), order: e(v) === "asc" ? "ascending" : "descending" },
          onSortChange: oe
        }, {
          default: o(() => [
            a(te, {
              prop: "id",
              label: e(t)("memories.colId"),
              width: "80",
              sortable: "custom"
            }, null, 8, ["label"]),
            a(te, {
              prop: "summary",
              label: e(t)("memories.colSummary"),
              "min-width": "180",
              "show-overflow-tooltip": ""
            }, null, 8, ["label"]),
            a(te, {
              label: e(t)("memories.colTags"),
              "min-width": "150"
            }, {
              default: o(({ row: h }) => [
                (f(!0), x(se, null, be(h.tags, (ne) => (f(), A(Ve, {
                  key: ne,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: o(() => [
                    u(s(ne), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            e(N) ? W("", !0) : (f(), A(te, {
              key: 0,
              prop: "created_at",
              label: e(t)("memories.colCreatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: o(({ row: h }) => [
                u(s(e(Se)(h.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label"])),
            a(te, {
              prop: "updated_at",
              label: e(t)("memories.colUpdatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: o(({ row: h }) => [
                u(s(e(Se)(h.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(te, {
              label: e(t)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: o(({ row: h }) => [
                a(ee, {
                  link: "",
                  type: "primary",
                  onClick: (ne) => ie(h.id)
                }, {
                  default: o(() => [
                    u(s(e(t)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(ee, {
                  link: "",
                  type: "primary",
                  onClick: (ne) => G(h.id)
                }, {
                  default: o(() => [
                    u(s(e(t)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(ee, {
                  link: "",
                  type: "danger",
                  onClick: (ne) => fe(h)
                }, {
                  default: o(() => [
                    u(s(e(t)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data", "default-sort"])), [
          [ze, e(P)]
        ]),
        e($) ? (f(), x("div", il, [
          a(Le, {
            "current-page": e(y),
            "onUpdate:currentPage": p[12] || (p[12] = (h) => ge(y) ? y.value = h : null),
            "page-size": e(M),
            "onUpdate:pageSize": p[13] || (p[13] = (h) => ge(M) ? M.value = h : null),
            total: e(T),
            "page-sizes": [10, 20, 50],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: p[14] || (p[14] = (h) => z(e(I))),
            onSizeChange: p[15] || (p[15] = (h) => z(e(I)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])) : (f(), x("div", sl, [
          a(Le, {
            "current-page": e(y),
            "onUpdate:currentPage": p[8] || (p[8] = (h) => ge(y) ? y.value = h : null),
            "page-size": e(M),
            "onUpdate:pageSize": p[9] || (p[9] = (h) => ge(M) ? M.value = h : null),
            total: e(T),
            "page-sizes": [20, 50, 100, 200],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: p[10] || (p[10] = (h) => z(e(I))),
            onSizeChange: p[11] || (p[11] = (h) => z(e(I)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])),
        a(Ja, {
          visible: K.value,
          "onUpdate:visible": p[16] || (p[16] = (h) => K.value = h),
          "memory-id": J.value,
          "tag-options": e(R),
          width: e(le) ? "96%" : "640px",
          onSaved: ue
        }, null, 8, ["visible", "memory-id", "tag-options", "width"]),
        a(Za, {
          visible: C.value,
          "onUpdate:visible": p[17] || (p[17] = (h) => C.value = h),
          "memory-id": D.value,
          size: e(le) ? "100%" : "45%"
        }, null, 8, ["visible", "memory-id", "size"])
      ], 512);
    };
  }
}), dl = /* @__PURE__ */ Ee(rl, [["__scopeId", "data-v-a269dc91"]]);
function cl() {
  const l = he(), r = m([]), i = m(!1), d = m("");
  async function g() {
    i.value = !0;
    try {
      const y = await Bt(l, d.value.trim() || void 0);
      r.value = y.tags ?? [];
    } finally {
      i.value = !1;
    }
  }
  async function _(y, M) {
    await Oa(l, y, M), await g();
  }
  async function w(y, M, V) {
    await Na(l, y, M, V), await g();
  }
  async function v(y, M) {
    await Fa(l, y, M), await g();
  }
  return xe(() => {
    g().catch(() => {
    });
  }), { rows: r, loading: i, filter: d, reload: g, create: _, rename: w, remove: v };
}
const ul = {
  key: 0,
  class: "am-panel-header"
}, ml = { class: "am-heading" }, pl = { class: "am-panel-title" }, fl = { class: "delete-body" }, vl = /* @__PURE__ */ _e({
  __name: "TagsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(l, { expose: r }) {
    const i = l, { rows: d, loading: g, filter: _, reload: w, create: v, rename: y, remove: M } = cl();
    let V = null;
    function b() {
      V && clearTimeout(V), V = setTimeout(() => {
        V = null, w().catch((C) => Y(C instanceof Error ? C.message : String(C)));
      }, 300);
    }
    const T = m(null), { narrow: P } = We(T), R = m(!1), $ = m(!1), U = ht({ oldName: null, name: "", description: "" }), I = m(!1), j = m("detach"), H = m(null);
    function ae(C) {
      return (D, z) => (D[C] ?? 0) - (z[C] ?? 0);
    }
    function O() {
      Object.assign(U, { oldName: null, name: "", description: "" }), $.value = !0;
    }
    function N(C) {
      Object.assign(U, { oldName: C.name, name: C.name, description: C.description ?? "" }), $.value = !0;
    }
    async function le() {
      R.value = !0;
      try {
        U.oldName ? (await y(U.oldName, U.name, U.description), X(t("tags.saved"))) : (await v(U.name, U.description), X(t("tags.created"))), $.value = !1;
      } catch (C) {
        Y(C instanceof Error ? C.message : String(C));
      } finally {
        R.value = !1;
      }
    }
    function K(C) {
      H.value = C, j.value = "detach", I.value = !0;
    }
    async function J() {
      var C, D;
      if (j.value === "purge")
        try {
          await Pe.confirm(
            t("tags.purgeConfirm", { name: (C = H.value) == null ? void 0 : C.name, count: ((D = H.value) == null ? void 0 : D.memory_count) ?? 0 }),
            t("tags.purgeConfirmTitle"),
            { type: "error", confirmButtonText: t("tags.purgeButton") }
          );
        } catch {
          return;
        }
      if (H.value) {
        R.value = !0;
        try {
          await M(H.value.name, j.value), X(t("tags.deleted")), I.value = !1;
        } catch (z) {
          Y(z instanceof Error ? z.message : String(z));
        } finally {
          R.value = !1;
        }
      }
    }
    return r({
      refresh: () => w().catch((C) => Y(C instanceof Error ? C.message : String(C)))
    }), (C, D) => {
      const z = De, ke = je, G = $e, ie = He, ue = qe, oe = nt, fe = ot, B = at, p = tt, Z = et, re = sa, ee = Ke, de = St;
      return f(), x("div", {
        ref_key: "rootRef",
        ref: T,
        class: "am-panel"
      }, [
        l.showHeader ? (f(), x("div", ul, [
          k("div", ml, [
            k("h2", pl, s(i.title ?? e(t)("tags.title")), 1),
            a(ke, {
              content: i.subtitle ?? e(t)("tags.subtitle"),
              placement: "top"
            }, {
              default: o(() => [
                a(z, { class: "am-info" }, {
                  default: o(() => [
                    a(e(Ue))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(G, {
            type: "primary",
            icon: e(st),
            onClick: O
          }, {
            default: o(() => [
              u(s(e(t)("tags.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : W("", !0),
        a(ie, {
          modelValue: e(_),
          "onUpdate:modelValue": D[0] || (D[0] = (S) => ge(_) ? _.value = S : null),
          class: "am-tag-filter",
          placeholder: e(t)("tags.filterPlaceholder"),
          clearable: "",
          "prefix-icon": e(it),
          onInput: b,
          onClear: b
        }, null, 8, ["modelValue", "placeholder", "prefix-icon"]),
        Ce((f(), A(fe, { data: e(d) }, {
          default: o(() => [
            a(oe, {
              prop: "name",
              label: e(t)("tags.colName"),
              "min-width": "140",
              sortable: ""
            }, {
              default: o(({ row: S }) => [
                a(ue, null, {
                  default: o(() => [
                    u(s(S.name), 1)
                  ]),
                  _: 2
                }, 1024)
              ]),
              _: 1
            }, 8, ["label"]),
            a(oe, {
              prop: "description",
              label: e(t)("tags.colDescription"),
              "min-width": "150",
              "show-overflow-tooltip": ""
            }, {
              default: o(({ row: S }) => [
                u(s(S.description || "—"), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(oe, {
              prop: "memory_count",
              label: e(t)("tags.colMemoryCount"),
              width: "90",
              sortable: ""
            }, null, 8, ["label"]),
            a(oe, {
              prop: "last_used_at",
              label: e(t)("tags.colLastUsed"),
              width: "170",
              sortable: "",
              "sort-method": ae("last_used_at")
            }, {
              default: o(({ row: S }) => [
                u(s(e(Se)(S.last_used_at)), 1)
              ]),
              _: 1
            }, 8, ["label", "sort-method"]),
            a(oe, {
              prop: "created_at",
              label: e(t)("tags.colCreatedAt"),
              width: "170",
              sortable: "",
              "sort-method": ae("created_at")
            }, {
              default: o(({ row: S }) => [
                u(s(e(Se)(S.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label", "sort-method"]),
            a(oe, {
              label: e(t)("memories.colActions"),
              width: "150",
              fixed: "right"
            }, {
              default: o(({ row: S }) => [
                a(G, {
                  link: "",
                  type: "primary",
                  onClick: (F) => N(S)
                }, {
                  default: o(() => [
                    u(s(e(t)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(G, {
                  link: "",
                  type: "danger",
                  onClick: (F) => K(S)
                }, {
                  default: o(() => [
                    u(s(e(t)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [de, e(g)]
        ]),
        a(Z, {
          modelValue: $.value,
          "onUpdate:modelValue": D[4] || (D[4] = (S) => $.value = S),
          title: U.oldName ? e(t)("tags.editTitle") : e(t)("tags.createTitle"),
          width: e(P) ? "96%" : "480px"
        }, {
          footer: o(() => [
            a(G, {
              onClick: D[3] || (D[3] = (S) => $.value = !1)
            }, {
              default: o(() => [
                u(s(e(t)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(G, {
              type: "primary",
              loading: R.value,
              onClick: le
            }, {
              default: o(() => [
                u(s(e(t)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: o(() => [
            a(p, { "label-position": "top" }, {
              default: o(() => [
                a(B, {
                  label: e(t)("tags.nameLabel")
                }, {
                  default: o(() => [
                    a(ie, {
                      modelValue: U.name,
                      "onUpdate:modelValue": D[1] || (D[1] = (S) => U.name = S),
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(t)("tags.namePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(B, {
                  label: e(t)("tags.descLabel")
                }, {
                  default: o(() => [
                    a(ie, {
                      modelValue: U.description,
                      "onUpdate:modelValue": D[2] || (D[2] = (S) => U.description = S),
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
        a(Z, {
          modelValue: I.value,
          "onUpdate:modelValue": D[7] || (D[7] = (S) => I.value = S),
          title: e(t)("tags.deleteTitle"),
          width: e(P) ? "96%" : "480px"
        }, {
          footer: o(() => [
            a(G, {
              onClick: D[6] || (D[6] = (S) => I.value = !1)
            }, {
              default: o(() => [
                u(s(e(t)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(G, {
              type: "danger",
              loading: R.value,
              onClick: J
            }, {
              default: o(() => [
                u(s(e(t)("common.delete")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: o(() => {
            var S;
            return [
              k("p", fl, [
                u(s(e(t)("tags.deleteBefore")) + " ", 1),
                a(ue, null, {
                  default: o(() => {
                    var F;
                    return [
                      u(s((F = H.value) == null ? void 0 : F.name), 1)
                    ];
                  }),
                  _: 1
                }),
                u(" " + s(e(t)("tags.deleteMiddle")) + " ", 1),
                k("b", null, s((S = H.value) == null ? void 0 : S.memory_count), 1),
                u(" " + s(e(t)("tags.deleteAfter")), 1)
              ]),
              a(ee, {
                modelValue: j.value,
                "onUpdate:modelValue": D[5] || (D[5] = (F) => j.value = F)
              }, {
                default: o(() => [
                  a(re, { value: "detach" }, {
                    default: o(() => [
                      u(s(e(t)("tags.detach")), 1)
                    ]),
                    _: 1
                  }),
                  a(re, { value: "purge" }, {
                    default: o(() => [
                      u(s(e(t)("tags.purge")), 1)
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
}), gl = /* @__PURE__ */ Ee(vl, [["__scopeId", "data-v-c8aeea57"]]);
function bl(l) {
  return l.get("/api/stats");
}
function _l(l) {
  return l.get("/health");
}
function yl(l) {
  return l.get("/api/doctor");
}
function hl(l) {
  return l.getBlob("/api/export");
}
function kl(l, r) {
  return l.post("/api/import", r);
}
function wl(l) {
  return l.post("/api/embeddings/backfill", {});
}
function Tl() {
  const l = he(), r = m({}), i = m(""), d = pe(() => $a(r.value.file_size));
  async function g() {
    r.value = await bl(l), r.value.version = i.value;
  }
  return xe(async () => {
    try {
      i.value = (await _l(l)).version ?? "";
    } catch {
    }
    await g().catch(() => {
    });
  }), {
    stats: r,
    version: i,
    sizeText: d,
    reload: g
  };
}
const Cl = {
  key: 0,
  class: "am-panel-header"
}, Sl = { class: "am-heading" }, El = { class: "am-panel-title" }, Vl = /* @__PURE__ */ _e({
  __name: "OpsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(l, { expose: r }) {
    const i = l, { stats: d, version: g, sizeText: _, reload: w } = Tl(), v = m(null), { narrow: y } = We(v);
    function M(V) {
      return V().catch((b) => Y(b instanceof Error ? b.message : String(b)));
    }
    return r({ refresh: () => M(w) }), (V, b) => {
      const T = De, P = je, R = $e, $ = da, U = Et, I = ra, j = ia, H = At, ae = Vt;
      return f(), x("div", {
        ref_key: "rootRef",
        ref: v,
        class: "am-panel"
      }, [
        l.showHeader ? (f(), x("div", Cl, [
          k("div", Sl, [
            k("h2", El, s(i.title ?? e(t)("ops.title")), 1),
            a(P, {
              content: i.subtitle ?? e(t)("ops.subtitle"),
              placement: "top"
            }, {
              default: o(() => [
                a(T, { class: "am-info" }, {
                  default: o(() => [
                    a(e(Ue))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(R, {
            icon: e(Mt),
            onClick: b[0] || (b[0] = (O) => M(e(w)))
          }, {
            default: o(() => [
              u(s(e(t)("ops.refresh")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : W("", !0),
        a(j, { gutter: 14 }, {
          default: o(() => [
            a(I, {
              span: e(y) ? 12 : 8
            }, {
              default: o(() => [
                a(U, { shadow: "never" }, {
                  default: o(() => [
                    a($, {
                      title: e(t)("ops.statMemories"),
                      value: e(d).memories ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(I, {
              span: e(y) ? 12 : 8
            }, {
              default: o(() => [
                a(U, { shadow: "never" }, {
                  default: o(() => [
                    a($, {
                      title: e(t)("ops.statTags"),
                      value: e(d).tags ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(I, {
              span: e(y) ? 12 : 8
            }, {
              default: o(() => [
                a(U, { shadow: "never" }, {
                  default: o(() => [
                    a($, {
                      title: e(t)("ops.statSize"),
                      value: e(_)
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
        a(U, { shadow: "never" }, {
          header: o(() => [
            u(s(e(t)("ops.dbCard")), 1)
          ]),
          default: o(() => [
            a(ae, {
              column: e(y) ? 1 : 2,
              border: ""
            }, {
              default: o(() => [
                a(H, {
                  label: e(t)("ops.version")
                }, {
                  default: o(() => [
                    u(s(e(d).version ?? e(g)), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(H, {
                  label: e(t)("ops.schemaVersion")
                }, {
                  default: o(() => [
                    u(s(e(d).schema_version ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(H, {
                  label: e(t)("ops.path"),
                  span: e(y) ? 1 : 2
                }, {
                  default: o(() => [
                    u(s(e(d).path ?? "—"), 1)
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
}), Al = { class: "memory-ui" }, Ml = { class: "brand" }, Pl = { class: "brand-mark" }, Ul = { class: "aside-footer" }, xl = /* @__PURE__ */ _e({
  __name: "MemoryAdmin",
  props: {
    layout: { default: "sidebar" },
    title: { default: "Agent Memory" }
  },
  setup(l) {
    const r = m("memories");
    return (i, d) => {
      const g = De, _ = pa, w = ma, v = ua, y = ga, M = va, V = fa, b = ca;
      return f(), x("div", Al, [
        a(b, { class: "layout" }, {
          default: o(() => [
            l.layout === "sidebar" ? (f(), A(v, {
              key: 0,
              width: "200px",
              class: "aside"
            }, {
              default: o(() => [
                k("div", Ml, [
                  k("span", Pl, [
                    a(g, { size: 16 }, {
                      default: o(() => [
                        a(e(ha))
                      ]),
                      _: 1
                    })
                  ]),
                  k("span", null, s(l.title), 1)
                ]),
                a(w, {
                  "default-active": r.value,
                  class: "menu",
                  onSelect: d[0] || (d[0] = (T) => r.value = T)
                }, {
                  default: o(() => [
                    a(_, { index: "memories" }, {
                      default: o(() => [
                        a(g, null, {
                          default: o(() => [
                            a(e(ka))
                          ]),
                          _: 1
                        }),
                        k("span", null, s(e(t)("nav.memories")), 1)
                      ]),
                      _: 1
                    }),
                    a(_, { index: "tags" }, {
                      default: o(() => [
                        a(g, null, {
                          default: o(() => [
                            a(e(wa))
                          ]),
                          _: 1
                        }),
                        k("span", null, s(e(t)("nav.tags")), 1)
                      ]),
                      _: 1
                    }),
                    a(_, { index: "ops" }, {
                      default: o(() => [
                        a(g, null, {
                          default: o(() => [
                            a(e(Ta))
                          ]),
                          _: 1
                        }),
                        k("span", null, s(e(t)("nav.ops")), 1)
                      ]),
                      _: 1
                    })
                  ]),
                  _: 1
                }, 8, ["default-active"]),
                k("div", Ul, [
                  aa(i.$slots, "footer", {}, void 0, !0)
                ])
              ]),
              _: 3
            })) : W("", !0),
            a(V, { class: "main" }, {
              default: o(() => [
                l.layout === "tabs" ? (f(), A(M, {
                  key: 0,
                  modelValue: r.value,
                  "onUpdate:modelValue": d[1] || (d[1] = (T) => r.value = T),
                  class: "tabs-bar"
                }, {
                  default: o(() => [
                    a(y, {
                      label: e(t)("nav.memories"),
                      name: "memories"
                    }, null, 8, ["label"]),
                    a(y, {
                      label: e(t)("nav.tags"),
                      name: "tags"
                    }, null, 8, ["label"]),
                    a(y, {
                      label: e(t)("nav.ops"),
                      name: "ops"
                    }, null, 8, ["label"])
                  ]),
                  _: 1
                }, 8, ["modelValue"])) : W("", !0),
                Ce(a(dl, null, null, 512), [
                  [Xe, r.value === "memories"]
                ]),
                Ce(a(gl, null, null, 512), [
                  [Xe, r.value === "tags"]
                ]),
                Ce(a(Vl, null, null, 512), [
                  [Xe, r.value === "ops"]
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
}), go = /* @__PURE__ */ Ee(xl, [["__scopeId", "data-v-38c91753"]]);
function $l() {
  const l = he(), r = m({ ok: !0, issues: [] }), i = m(!1), d = m(!1), g = m(!1), _ = m(!1), w = m(!1);
  async function v() {
    d.value = !0;
    try {
      r.value = await yl(l), i.value = !0;
    } finally {
      d.value = !1;
    }
  }
  async function y() {
    g.value = !0;
    try {
      const b = await hl(l), T = URL.createObjectURL(b), P = document.createElement("a");
      P.href = T, P.download = "agent-memory-export.json", P.click(), URL.revokeObjectURL(T);
    } finally {
      g.value = !1;
    }
  }
  async function M(b) {
    _.value = !0;
    try {
      const T = await b.text();
      let P;
      try {
        P = JSON.parse(T);
      } catch {
        throw new Error(t("errors.invalidBackup"));
      }
      return await kl(l, P);
    } finally {
      _.value = !1;
    }
  }
  async function V() {
    w.value = !0;
    try {
      let b = 0;
      for (; ; ) {
        const T = await wl(l);
        if (!T.configured) throw new Error(t("ops.embeddingNotConfigured"));
        if (T.error) throw new Error(T.error);
        if (b += T.processed ?? 0, (T.processed ?? 0) === 0) break;
      }
      return b;
    } finally {
      w.value = !1;
    }
  }
  return {
    doctor: r,
    doctorRan: i,
    doctorLoading: d,
    exporting: g,
    importing: _,
    backfilling: w,
    runDoctor: v,
    exportData: y,
    importFile: M,
    backfill: V
  };
}
const Dl = {
  key: 0,
  class: "am-panel-header"
}, Il = { class: "am-heading" }, Bl = { class: "am-panel-title" }, Ll = { class: "auth-row" }, zl = { class: "auth-text" }, Rl = { class: "auth-label" }, Ol = { class: "auth-hint" }, Nl = { class: "token-cell" }, Fl = { class: "token-text" }, Hl = {
  key: 0,
  class: "muted"
}, Kl = { class: "save-row" }, ql = { class: "card-header" }, jl = { class: "section-title" }, Jl = { class: "field-hint" }, Wl = {
  key: 0,
  class: "default-view"
}, Gl = { class: "default-text" }, Ql = { class: "field-hint" }, Xl = { class: "save-row" }, Yl = { class: "actions" }, Zl = { class: "card-header" }, eo = {
  key: 2,
  class: "issues"
}, to = { class: "presets" }, ao = { class: "presets-label" }, lo = { class: "caps" }, oo = { class: "new-token" }, no = /* @__PURE__ */ _e({
  __name: "AdminPanel",
  props: {
    who: { default: null },
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(l, { expose: r }) {
    const i = l, d = he(), g = rt(), _ = [
      { key: "read", labelKey: "capRead" },
      { key: "create", labelKey: "capCreate" },
      { key: "update", labelKey: "capUpdate" },
      { key: "delete", labelKey: "capDelete" },
      { key: "tag_manage", labelKey: "capTagManage" },
      { key: "admin", labelKey: "capAdmin" }
    ], w = m([]), v = m(""), y = m(""), M = m(""), V = m(!1), b = m(!1), T = m(!1), P = pe(() => {
      var c;
      return !!i.who && ((c = i.who.permissions) == null ? void 0 : c.admin) === !0;
    }), {
      doctor: R,
      doctorRan: $,
      doctorLoading: U,
      exporting: I,
      importing: j,
      backfilling: H,
      runDoctor: ae,
      exportData: O,
      importFile: N,
      backfill: le
    } = $l(), K = m(null), J = pe(() => {
      var c, n, L, me;
      return (((n = (c = K.value) == null ? void 0 : c.embedding) == null ? void 0 : n.embedded) ?? 0) + (((me = (L = K.value) == null ? void 0 : L.embedding) == null ? void 0 : me.pending) ?? 0);
    });
    async function C() {
      const c = await p(le);
      if (c !== void 0) {
        try {
          K.value = await d.get("/api/stats");
        } catch {
        }
        X(c === 0 ? t("access.embeddingUpToDate") : t("access.embeddingDone", { count: c }));
      }
    }
    const D = m(null), { compact: z } = We(D), ke = m(null), G = m(!1), ie = m(""), ue = m(""), oe = m(""), fe = m(!1), B = m(null);
    async function p(c, n) {
      try {
        return await c();
      } catch (L) {
        Y(L instanceof Error ? L.message : String(L));
        return;
      }
    }
    async function Z() {
      await p(async () => {
        const [c, n, L] = await Promise.all([
          d.get("/api/identities"),
          d.get("/api/settings"),
          d.get("/api/stats")
        ]);
        w.value = Array.isArray(c == null ? void 0 : c.identities) ? c.identities : [], v.value = (n == null ? void 0 : n.instructions) ?? "", y.value = (n == null ? void 0 : n.conventions) ?? "", M.value = (n == null ? void 0 : n.default_instructions) ?? "", b.value = (n == null ? void 0 : n.auth_required) === !0, G.value = (n == null ? void 0 : n.embedding_enabled) === !0, ie.value = (n == null ? void 0 : n.embedding_base_url) ?? "", ue.value = (n == null ? void 0 : n.embedding_model) ?? "", oe.value = (n == null ? void 0 : n.embedding_api_key) ?? "", K.value = L ?? null;
      });
    }
    xe(() => {
      P.value && Z();
    }), Fe(P, (c) => {
      c && Z();
    }), r({
      refresh: () => {
        P.value && p(Z);
      }
    });
    const re = m(!1), ee = m(!1), de = m(null), S = m("custom"), F = ht({
      name: "",
      caps: Ie()
    });
    function Ie() {
      return Object.fromEntries(_.map((c) => [c.key, !1]));
    }
    const te = {
      admin: _.map((c) => c.key),
      member: ["read", "create", "update", "tag_manage"],
      viewer: ["read"]
    };
    function Ve() {
      if (S.value === "custom") return;
      const c = new Set(te[S.value] ?? []);
      for (const n of _) F.caps[n.key] = c.has(n.key);
    }
    function Be() {
      S.value = "custom";
    }
    function Le() {
      de.value = null, F.name = "", Object.assign(F.caps, Ie()), S.value = "member", Ve(), re.value = !0;
    }
    function ze(c) {
      var n;
      de.value = c, F.name = c.name;
      for (const L of _) F.caps[L.key] = ((n = c.permissions) == null ? void 0 : n[L.key]) === !0;
      S.value = "custom", re.value = !0;
    }
    async function h() {
      ee.value = !0;
      try {
        if (de.value)
          await p(
            () => d.put(`/api/identities/${encodeURIComponent(de.value.name)}`, {
              permissions: { ...F.caps }
            })
          ), X(t("access.saved"));
        else {
          const c = await p(
            () => d.post("/api/identities", {
              name: F.name,
              permissions: { ...F.caps }
            })
          );
          c != null && c.token && mt(t("access.createTitle"), F.name.trim(), c.token);
        }
        re.value = !1, await Z();
      } finally {
        ee.value = !1;
      }
    }
    async function ne(c) {
      try {
        await Pe.confirm(t("access.deleteConfirm", { name: c.name }), t("access.deleteTitle"), {
          type: "warning",
          confirmButtonText: t("common.delete"),
          cancelButtonText: t("common.cancel")
        });
      } catch {
        return;
      }
      await p(() => d.del(`/api/identities/${encodeURIComponent(c.name)}`)) !== void 0 && (X(t("access.deleted")), await Z());
    }
    const Ge = pe(() => w.value.some((c) => {
      var n;
      return ((n = c.permissions) == null ? void 0 : n.admin) === !0;
    }));
    async function zt() {
      var n;
      const c = !b.value;
      if (c && !Ge.value)
        return Y(t("access.enableBlocked")), !1;
      try {
        await Pe.confirm(
          t(c ? "access.authEnableConfirm" : "access.authDisableConfirm"),
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
      T.value = !0;
      try {
        return await p(() => d.put("/api/settings", { auth_required: c })) !== void 0 && ((n = g.onAuthChanged) == null || n.call(g, c)), !0;
      } finally {
        T.value = !1;
      }
    }
    async function Rt() {
      V.value = !0;
      try {
        await p(
          () => d.put("/api/settings", {
            instructions: v.value,
            conventions: y.value
          })
        ), X(t("access.saved"));
      } finally {
        V.value = !1;
      }
    }
    function Ot(c) {
      var me;
      const n = c.target, L = (me = n.files) == null ? void 0 : me[0];
      n.value = "", L && N(L).then((q) => {
        X(t("access.imported", { memories: q.imported_memories, tags: q.imported_tags }));
      }).catch((q) => {
        Y(q instanceof Error ? q.message : String(q));
      });
    }
    async function Nt() {
      fe.value = !0, B.value = null;
      try {
        if (await p(
          () => d.put("/api/settings", {
            embedding_enabled: G.value,
            embedding_base_url: ie.value,
            embedding_model: ue.value,
            embedding_api_key: oe.value
          })
        ) === void 0 || (X(t("access.saved")), !G.value)) return;
        B.value = await p(() => d.post("/api/embeddings/test", {}));
      } finally {
        fe.value = !1;
      }
    }
    const Ae = m(!1), ct = m(""), ut = m(""), Re = m("");
    function Ft(c) {
      return c ? `…${c}` : "—";
    }
    function mt(c, n, L) {
      ct.value = c, ut.value = n, Re.value = L, Ae.value = !0;
    }
    async function Ht() {
      const c = ut.value;
      if (c)
        try {
          await g.onIdentityToken(c, Re.value), Ae.value = !1, X(t("access.savedToBrowser", { name: c }));
        } catch (n) {
          Y(n instanceof Error ? n.message : String(n));
        }
    }
    async function Kt(c) {
      try {
        await Pe.confirm(t("access.resetConfirm", { name: c.name }), t("access.resetTitle"), {
          type: "warning",
          confirmButtonText: t("access.resetToken"),
          cancelButtonText: t("common.cancel")
        });
      } catch {
        return;
      }
      const n = await p(
        () => d.post(`/api/identities/${encodeURIComponent(c.name)}/token-reset`, {})
      );
      n !== void 0 && (n != null && n.token && mt(t("access.resetTitle"), c.name, n.token), await Z());
    }
    function pt(c) {
      return _.map((n) => n.key).filter((n) => {
        var L;
        return ((L = c.permissions) == null ? void 0 : L[n]) === !0;
      });
    }
    function ft(c) {
      const n = _.find((L) => L.key === c);
      return n ? t(`access.${n.labelKey}`) : c;
    }
    async function qt(c) {
      try {
        await navigator.clipboard.writeText(c), X(t("access.copied"));
      } catch {
        Y(c);
      }
    }
    return (c, n) => {
      const L = De, me = je, q = $e, ce = Ct, vt = ba, we = Et, gt = _a, Me = nt, jt = qe, Jt = ot, ve = at, Te = He, Qe = tt, Wt = Tt, bt = At, Gt = Vt, Oe = lt, Qt = Ke, Xt = ya, _t = et;
      return f(), x("div", {
        ref_key: "rootRef",
        ref: D,
        class: "am-panel admin-panel"
      }, [
        l.showHeader ? (f(), x("div", Dl, [
          k("div", Il, [
            k("h2", Bl, s(i.title ?? e(t)("access.title")), 1),
            a(me, {
              content: i.subtitle ?? e(t)("access.subtitle"),
              placement: "top"
            }, {
              default: o(() => [
                a(L, { class: "am-info" }, {
                  default: o(() => [
                    a(e(Ue))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          P.value ? (f(), A(q, {
            key: 0,
            type: "primary",
            icon: e(st),
            onClick: Le
          }, {
            default: o(() => [
              u(s(e(t)("access.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])) : W("", !0)
        ])) : W("", !0),
        l.who ? l.who.mode === "open" ? (f(), A(ce, {
          key: 2,
          type: "warning",
          title: e(t)("access.openMode"),
          closable: !1
        }, null, 8, ["title"])) : P.value ? W("", !0) : (f(), A(ce, {
          key: 3,
          type: "info",
          title: e(t)("access.needAdmin"),
          closable: !1
        }, null, 8, ["title"])) : (f(), A(ce, {
          key: 1,
          type: "info",
          title: e(t)("access.needAdmin"),
          closable: !1
        }, null, 8, ["title"])),
        l.who && P.value ? (f(), x(se, { key: 4 }, [
          a(we, { shadow: "never" }, {
            default: o(() => [
              k("div", Ll, [
                k("div", zl, [
                  k("span", Rl, s(e(t)("access.authTitle")), 1),
                  k("span", Ol, s(e(t)("access.authHint")), 1)
                ]),
                a(me, {
                  disabled: Ge.value,
                  content: e(t)("access.enableBlocked"),
                  placement: "top"
                }, {
                  default: o(() => [
                    a(vt, {
                      modelValue: b.value,
                      "onUpdate:modelValue": n[0] || (n[0] = (E) => b.value = E),
                      "before-change": zt,
                      loading: T.value,
                      disabled: !Ge.value
                    }, null, 8, ["modelValue", "loading", "disabled"])
                  ]),
                  _: 1
                }, 8, ["disabled", "content"])
              ])
            ]),
            _: 1
          }),
          a(we, { shadow: "never" }, {
            default: o(() => [
              w.value.length === 0 ? (f(), A(gt, {
                key: 0,
                description: e(t)("access.empty")
              }, null, 8, ["description"])) : (f(), A(Jt, {
                key: 1,
                data: w.value
              }, {
                default: o(() => [
                  a(Me, {
                    prop: "name",
                    label: e(t)("access.colName"),
                    "min-width": "120"
                  }, null, 8, ["label"]),
                  a(Me, {
                    label: e(t)("access.colToken"),
                    "min-width": "200"
                  }, {
                    default: o(({ row: E }) => [
                      k("div", Nl, [
                        k("code", Fl, s(Ft(E.token_hint)), 1),
                        a(q, {
                          link: "",
                          type: "primary",
                          onClick: (Q) => Kt(E)
                        }, {
                          default: o(() => [
                            u(s(e(t)("access.resetToken")), 1)
                          ]),
                          _: 1
                        }, 8, ["onClick"])
                      ])
                    ]),
                    _: 1
                  }, 8, ["label"]),
                  a(Me, {
                    label: e(t)("access.colPermissions"),
                    "min-width": "240"
                  }, {
                    default: o(({ row: E }) => [
                      (f(!0), x(se, null, be(pt(E), (Q) => (f(), A(jt, {
                        key: Q,
                        size: "small",
                        class: "cap-tag",
                        type: Q === "admin" ? "danger" : "info"
                      }, {
                        default: o(() => [
                          u(s(ft(Q)), 1)
                        ]),
                        _: 2
                      }, 1032, ["type"]))), 128)),
                      pt(E).length === 0 ? (f(), x("span", Hl, "—")) : W("", !0)
                    ]),
                    _: 1
                  }, 8, ["label"]),
                  e(z) ? W("", !0) : (f(), A(Me, {
                    key: 0,
                    label: e(t)("access.colCreatedAt"),
                    width: "170"
                  }, {
                    default: o(({ row: E }) => [
                      u(s(e(Se)(E.created_at)), 1)
                    ]),
                    _: 1
                  }, 8, ["label"])),
                  a(Me, {
                    label: e(t)("access.colActions"),
                    width: "140",
                    fixed: "right"
                  }, {
                    default: o(({ row: E }) => [
                      a(q, {
                        link: "",
                        type: "primary",
                        onClick: (Q) => ze(E)
                      }, {
                        default: o(() => [
                          u(s(e(t)("common.edit")), 1)
                        ]),
                        _: 1
                      }, 8, ["onClick"]),
                      a(q, {
                        link: "",
                        type: "danger",
                        onClick: (Q) => ne(E)
                      }, {
                        default: o(() => [
                          u(s(e(t)("common.delete")), 1)
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
          a(we, { shadow: "never" }, {
            header: o(() => [
              u(s(e(t)("access.embeddingTitle")), 1)
            ]),
            default: o(() => {
              var E, Q, yt;
              return [
                a(ce, {
                  title: e(t)("access.embeddingHint"),
                  type: "info",
                  "show-icon": "",
                  closable: !1,
                  class: "settings-hint"
                }, null, 8, ["title"]),
                a(Qe, {
                  "label-position": "top",
                  onSubmit: n[5] || (n[5] = Ye(() => {
                  }, ["prevent"]))
                }, {
                  default: o(() => [
                    a(ve, {
                      label: e(t)("access.embeddingEnabledLabel")
                    }, {
                      default: o(() => [
                        a(vt, {
                          modelValue: G.value,
                          "onUpdate:modelValue": n[1] || (n[1] = (ye) => G.value = ye)
                        }, null, 8, ["modelValue"])
                      ]),
                      _: 1
                    }, 8, ["label"]),
                    a(ve, {
                      label: e(t)("access.embeddingBaseUrl")
                    }, {
                      default: o(() => [
                        a(Te, {
                          modelValue: ie.value,
                          "onUpdate:modelValue": n[2] || (n[2] = (ye) => ie.value = ye),
                          placeholder: "https://api.siliconflow.cn/v1 或 http://127.0.0.1:11434/v1"
                        }, null, 8, ["modelValue"])
                      ]),
                      _: 1
                    }, 8, ["label"]),
                    a(ve, {
                      label: e(t)("access.embeddingModelLabel")
                    }, {
                      default: o(() => [
                        a(Te, {
                          modelValue: ue.value,
                          "onUpdate:modelValue": n[3] || (n[3] = (ye) => ue.value = ye),
                          placeholder: "BAAI/bge-m3 / bge-m3 / nomic-embed-text"
                        }, null, 8, ["modelValue"])
                      ]),
                      _: 1
                    }, 8, ["label"]),
                    a(ve, {
                      label: e(t)("access.embeddingApiKeyLabel")
                    }, {
                      default: o(() => [
                        a(Te, {
                          modelValue: oe.value,
                          "onUpdate:modelValue": n[4] || (n[4] = (ye) => oe.value = ye),
                          "show-password": "",
                          placeholder: e(t)("access.embeddingApiKeyPlaceholder")
                        }, null, 8, ["modelValue", "placeholder"])
                      ]),
                      _: 1
                    }, 8, ["label"])
                  ]),
                  _: 1
                }),
                k("div", Kl, [
                  a(q, {
                    type: "primary",
                    loading: fe.value,
                    onClick: Nt
                  }, {
                    default: o(() => [
                      u(s(e(t)("access.embeddingSaveTest")), 1)
                    ]),
                    _: 1
                  }, 8, ["loading"])
                ]),
                (E = B.value) != null && E.ok ? (f(), A(ce, {
                  key: 0,
                  title: e(t)("access.embeddingTestOk", { dim: B.value.dim ?? 0, ms: B.value.elapsed_ms ?? 0 }),
                  type: "success",
                  "show-icon": "",
                  closable: !1,
                  class: "settings-hint"
                }, null, 8, ["title"])) : B.value && !B.value.ok ? (f(), A(ce, {
                  key: 1,
                  title: e(t)("access.embeddingTestFail", { error: B.value.error ?? "" }),
                  type: "error",
                  "show-icon": "",
                  closable: !1,
                  class: "settings-hint"
                }, null, 8, ["title"])) : W("", !0),
                a(Wt),
                k("div", ql, [
                  k("span", jl, s(e(t)("access.embeddingCoverageTitle")), 1),
                  a(q, {
                    size: "small",
                    icon: e(Mt),
                    loading: e(H),
                    onClick: C
                  }, {
                    default: o(() => [
                      u(s(e(t)("access.runBackfill")), 1)
                    ]),
                    _: 1
                  }, 8, ["icon", "loading"])
                ]),
                (yt = (Q = K.value) == null ? void 0 : Q.embedding) != null && yt.enabled ? (f(), x(se, { key: 2 }, [
                  a(Gt, {
                    column: e(z) ? 1 : 2,
                    border: ""
                  }, {
                    default: o(() => [
                      a(bt, {
                        label: e(t)("access.embeddingModel")
                      }, {
                        default: o(() => [
                          u(s(K.value.embedding.model ?? "—"), 1)
                        ]),
                        _: 1
                      }, 8, ["label"]),
                      a(bt, {
                        label: e(t)("access.embeddingCoverage")
                      }, {
                        default: o(() => [
                          u(s(K.value.embedding.embedded ?? 0) + " / " + s(J.value), 1)
                        ]),
                        _: 1
                      }, 8, ["label"])
                    ]),
                    _: 1
                  }, 8, ["column"]),
                  (K.value.embedding.pending ?? 0) > 0 ? (f(), A(ce, {
                    key: 0,
                    title: e(t)("access.embeddingPending", { count: K.value.embedding.pending }),
                    type: "warning",
                    "show-icon": "",
                    closable: !1,
                    class: "settings-hint"
                  }, null, 8, ["title"])) : W("", !0)
                ], 64)) : (f(), A(ce, {
                  key: 3,
                  title: e(t)("access.embeddingDisabled"),
                  type: "info",
                  "show-icon": "",
                  closable: !1
                }, null, 8, ["title"]))
              ];
            }),
            _: 1
          }),
          a(we, { shadow: "never" }, {
            header: o(() => [
              u(s(e(t)("access.settingsTitle")), 1)
            ]),
            default: o(() => [
              a(Qe, {
                "label-position": "top",
                onSubmit: n[8] || (n[8] = Ye(() => {
                }, ["prevent"]))
              }, {
                default: o(() => [
                  a(ve, {
                    label: e(t)("access.instructionsLabel")
                  }, {
                    default: o(() => [
                      a(Te, {
                        modelValue: v.value,
                        "onUpdate:modelValue": n[6] || (n[6] = (E) => v.value = E),
                        type: "textarea",
                        rows: 5,
                        placeholder: e(t)("access.instructionsPlaceholder")
                      }, null, 8, ["modelValue", "placeholder"]),
                      k("div", Jl, s(e(t)("access.instructionsHint")), 1),
                      v.value ? (f(), x("details", Wl, [
                        k("summary", null, s(e(t)("access.viewDefault")), 1),
                        k("pre", Gl, s(M.value), 1)
                      ])) : W("", !0)
                    ]),
                    _: 1
                  }, 8, ["label"]),
                  a(ve, {
                    label: e(t)("access.conventionsLabel")
                  }, {
                    default: o(() => [
                      a(Te, {
                        modelValue: y.value,
                        "onUpdate:modelValue": n[7] || (n[7] = (E) => y.value = E),
                        type: "textarea",
                        rows: 4,
                        placeholder: e(t)("access.conventionsPlaceholder")
                      }, null, 8, ["modelValue", "placeholder"]),
                      k("div", Ql, s(e(t)("access.conventionsHint")), 1)
                    ]),
                    _: 1
                  }, 8, ["label"])
                ]),
                _: 1
              }),
              k("div", Xl, [
                a(q, {
                  type: "primary",
                  loading: V.value,
                  onClick: Rt
                }, {
                  default: o(() => [
                    u(s(e(t)("common.save")), 1)
                  ]),
                  _: 1
                }, 8, ["loading"])
              ])
            ]),
            _: 1
          }),
          a(we, { shadow: "never" }, {
            header: o(() => [
              u(s(e(t)("access.backupCard")), 1)
            ]),
            default: o(() => [
              k("div", Yl, [
                a(q, {
                  icon: e(Ca),
                  loading: e(I),
                  onClick: n[9] || (n[9] = (E) => p(e(O)))
                }, {
                  default: o(() => [
                    u(s(e(t)("access.export")), 1)
                  ]),
                  _: 1
                }, 8, ["icon", "loading"]),
                a(q, {
                  icon: e(Sa),
                  loading: e(j),
                  onClick: n[10] || (n[10] = (E) => {
                    var Q;
                    return (Q = ke.value) == null ? void 0 : Q.click();
                  })
                }, {
                  default: o(() => [
                    u(s(e(t)("access.import")), 1)
                  ]),
                  _: 1
                }, 8, ["icon", "loading"]),
                a(me, {
                  content: e(t)("access.importHint"),
                  placement: "top"
                }, {
                  default: o(() => [
                    a(L, { class: "am-info" }, {
                      default: o(() => [
                        a(e(Ue))
                      ]),
                      _: 1
                    })
                  ]),
                  _: 1
                }, 8, ["content"]),
                k("input", {
                  ref_key: "importInput",
                  ref: ke,
                  type: "file",
                  accept: "application/json,.json",
                  style: { display: "none" },
                  onChange: Ot
                }, null, 544)
              ])
            ]),
            _: 1
          }),
          a(we, { shadow: "never" }, {
            header: o(() => [
              k("div", Zl, [
                k("span", null, s(e(t)("access.doctorCard")), 1),
                a(q, {
                  size: "small",
                  icon: e(it),
                  loading: e(U),
                  onClick: n[11] || (n[11] = (E) => p(e(ae)))
                }, {
                  default: o(() => [
                    u(s(e(t)("access.runDoctor")), 1)
                  ]),
                  _: 1
                }, 8, ["icon", "loading"])
              ])
            ]),
            default: o(() => [
              e($) ? (f(), x(se, { key: 0 }, [
                e(R).ok ? (f(), A(ce, {
                  key: 0,
                  title: e(t)("access.doctorOk"),
                  type: "success",
                  "show-icon": "",
                  closable: !1
                }, null, 8, ["title"])) : (f(), A(ce, {
                  key: 1,
                  title: e(t)("access.doctorFail", { count: e(R).issues.length }),
                  type: "error",
                  "show-icon": "",
                  closable: !1
                }, null, 8, ["title"])),
                e(R).ok ? W("", !0) : (f(), x("ul", eo, [
                  (f(!0), x(se, null, be(e(R).issues, (E, Q) => (f(), x("li", { key: Q }, s(E), 1))), 128))
                ]))
              ], 64)) : (f(), A(gt, {
                key: 1,
                description: e(t)("access.doctorEmpty"),
                "image-size": 60
              }, null, 8, ["description"]))
            ]),
            _: 1
          })
        ], 64)) : W("", !0),
        a(_t, {
          modelValue: re.value,
          "onUpdate:modelValue": n[16] || (n[16] = (E) => re.value = E),
          title: de.value ? e(t)("access.editTitle", { name: de.value.name }) : e(t)("access.createTitle"),
          width: e(z) ? "96%" : "480px"
        }, {
          footer: o(() => [
            a(q, {
              onClick: n[15] || (n[15] = (E) => re.value = !1)
            }, {
              default: o(() => [
                u(s(e(t)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(q, {
              type: "primary",
              loading: ee.value,
              onClick: h
            }, {
              default: o(() => [
                u(s(e(t)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: o(() => [
            a(Qe, {
              "label-position": "top",
              onSubmit: n[14] || (n[14] = Ye(() => {
              }, ["prevent"]))
            }, {
              default: o(() => [
                de.value ? W("", !0) : (f(), A(ve, {
                  key: 0,
                  label: e(t)("access.nameLabel")
                }, {
                  default: o(() => [
                    a(Te, {
                      modelValue: F.name,
                      "onUpdate:modelValue": n[12] || (n[12] = (E) => F.name = E),
                      placeholder: e(t)("access.namePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"])),
                a(ve, {
                  label: e(t)("access.permsLabel")
                }, {
                  default: o(() => [
                    k("div", to, [
                      k("span", ao, s(e(t)("access.presets")), 1),
                      a(Qt, {
                        modelValue: S.value,
                        "onUpdate:modelValue": n[13] || (n[13] = (E) => S.value = E),
                        size: "small",
                        onChange: Ve
                      }, {
                        default: o(() => [
                          a(Oe, { value: "admin" }, {
                            default: o(() => [
                              u(s(e(t)("access.presetAdmin")), 1)
                            ]),
                            _: 1
                          }),
                          a(Oe, { value: "member" }, {
                            default: o(() => [
                              u(s(e(t)("access.presetMember")), 1)
                            ]),
                            _: 1
                          }),
                          a(Oe, { value: "viewer" }, {
                            default: o(() => [
                              u(s(e(t)("access.presetViewer")), 1)
                            ]),
                            _: 1
                          }),
                          a(Oe, { value: "custom" }, {
                            default: o(() => [
                              u(s(e(t)("access.presetCustom")), 1)
                            ]),
                            _: 1
                          })
                        ]),
                        _: 1
                      }, 8, ["modelValue"])
                    ]),
                    k("div", lo, [
                      (f(), x(se, null, be(_, (E) => a(Xt, {
                        key: E.key,
                        modelValue: F.caps[E.key],
                        "onUpdate:modelValue": (Q) => F.caps[E.key] = Q,
                        onChange: Be
                      }, {
                        default: o(() => [
                          u(s(ft(E.key)), 1)
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
        a(_t, {
          modelValue: Ae.value,
          "onUpdate:modelValue": n[18] || (n[18] = (E) => Ae.value = E),
          title: ct.value,
          width: e(z) ? "96%" : "560px"
        }, {
          footer: o(() => [
            e(g).onIdentityToken ? (f(), A(q, {
              key: 0,
              type: "primary",
              onClick: Ht
            }, {
              default: o(() => [
                u(s(e(t)("access.saveToBrowser")), 1)
              ]),
              _: 1
            })) : W("", !0),
            a(q, {
              type: "primary",
              onClick: n[17] || (n[17] = () => {
                qt(Re.value), Ae.value = !1;
              })
            }, {
              default: o(() => [
                u(s(e(t)("access.copyToken")), 1)
              ]),
              _: 1
            })
          ]),
          default: o(() => [
            k("p", null, s(e(t)("access.created")), 1),
            k("code", oo, s(Re.value), 1)
          ]),
          _: 1
        }, 8, ["modelValue", "title", "width"])
      ], 512);
    };
  }
}), bo = /* @__PURE__ */ Ee(no, [["__scopeId", "data-v-9420b7ad"]]);
export {
  bo as AdminPanel,
  Lt as MarkdownView,
  dl as MemoriesPanel,
  go as MemoryAdmin,
  Za as MemoryDetailDrawer,
  Ja as MemoryEditorDialog,
  xt as MemoryUIConfigKey,
  Vl as OpsPanel,
  gl as TagsPanel,
  Ua as applyMemoryUILocalePreference,
  Ia as buildMemoriesQuery,
  Da as createApiClient,
  fo as currentMemoryUILocale,
  $a as formatSize,
  Se as formatTime,
  Ze as isSearchMode,
  Je as memoryUIi18n,
  vo as provideMemoryUI,
  La as renderMarkdown,
  za as sanitizeHtml,
  Pa as setMemoryUILocale,
  t,
  Y as toastError,
  X as toastSuccess,
  $l as useAdmin,
  he as useApiClient,
  Ha as useMemories,
  rt as useMemoryConfig,
  Tl as useOps,
  cl as useTags
};
