import { inject as Wt, provide as Gt, ref as m, computed as pe, onMounted as Ue, watch as Ne, onUnmounted as Qt, defineComponent as ye, openBlock as f, createElementBlock as x, createBlock as A, unref as e, withCtx as n, createVNode as a, createElementVNode as k, toDisplayString as s, createTextVNode as u, Fragment as se, renderList as be, createCommentVNode as G, withKeys as Xt, isRef as ge, withDirectives as Ce, reactive as yt, renderSlot as Yt, vShow as Qe, withModifiers as Xe } from "vue";
import { createI18n as Zt } from "vue-i18n";
import { ElDialog as Ze, ElForm as et, ElFormItem as tt, ElInput as Fe, ElRadioGroup as He, ElRadioButton as at, ElSelect as _t, ElOption as ht, ElButton as xe, ElDrawer as ea, ElTag as Ke, ElDivider as ta, ElTooltip as qe, ElIcon as $e, ElAlert as kt, ElTable as lt, ElTableColumn as nt, ElLoadingDirective as wt, ElPagination as aa, ElRadio as la, ElRow as na, ElCol as oa, ElCard as Ct, ElStatistic as sa, ElDescriptions as Tt, ElDescriptionsItem as St, ElContainer as ia, ElAside as ra, ElMenu as da, ElMenuItem as ca, ElMain as ua, ElTabs as ma, ElTabPane as pa, ElSwitch as fa, ElEmpty as va, ElCheckbox as ga } from "element-plus/es";
import { InfoFilled as Pe, Plus as ot, Search as st, Refresh as Et, Collection as ba, Notebook as ya, PriceTag as _a, Odometer as ha, Download as ka, UploadFilled as wa } from "@element-plus/icons-vue";
import { ElMessage as Ca, ElMessageBox as Me } from "element-plus";
import { Marked as Ta } from "marked";
import Vt from "dompurify";
const Sa = {
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
    embeddingCard: "语义搜索（embedding）",
    embeddingModel: "模型",
    embeddingCoverage: "已向量化 / 记忆总数",
    embeddingPending: "{count} 条记忆缺最新向量，点击「补跑向量化」重建",
    runBackfill: "补跑向量化",
    embeddingDisabled: "未启用：先在上方「语义搜索」卡配置 embedding 服务并开启",
    embeddingDone: "已补跑 {count} 条记忆的向量",
    embeddingUpToDate: "全部记忆已是最新向量，无需补跑",
    embeddingNotConfigured: "尚未配置 embedding 服务",
    embeddingTitle: "语义搜索（embedding）",
    embeddingHint: "配置 OpenAI 兼容的 /embeddings 服务：云端（如 SiliconFlow：https://api.siliconflow.cn/v1 + BAAI/bge-m3）或本地 Ollama（http://127.0.0.1:11434/v1 + bge-m3）。服务不可用时搜索自动回退关键词，写入不中断；缺向量的记忆可在下方「语义搜索」卡补跑。",
    embeddingEnabledLabel: "启用语义搜索",
    embeddingBaseUrl: "服务地址（base_url，OpenAI 兼容）",
    embeddingModelLabel: "模型名",
    embeddingApiKeyLabel: "API Key",
    embeddingApiKeyPlaceholder: "本地服务可留空",
    embeddingSaveTest: "保存并测试连接",
    embeddingTestOk: "连接成功：向量维度 {dim}，耗时 {ms} ms",
    embeddingTestFail: "连接失败：{error}"
  }
}, Ea = {
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
    embeddingCard: "Semantic search (embedding)",
    embeddingModel: "Model",
    embeddingCoverage: "Embedded / total memories",
    embeddingPending: '{count} memories lack up-to-date vectors; click "Backfill embeddings" to rebuild',
    runBackfill: "Backfill embeddings",
    embeddingDisabled: 'Not enabled: configure an embedding service in the "Semantic search" card above, then enable it',
    embeddingDone: "Backfilled embeddings for {count} memories",
    embeddingUpToDate: "All memories already have up-to-date embeddings",
    embeddingNotConfigured: "Embedding service is not configured",
    embeddingTitle: "Semantic search (embedding)",
    embeddingHint: 'Configure an OpenAI-compatible /embeddings service: cloud (e.g. SiliconFlow: https://api.siliconflow.cn/v1 + BAAI/bge-m3) or local Ollama (http://127.0.0.1:11434/v1 + bge-m3). If the service is unavailable, search falls back to keywords and writes are never blocked; missing vectors can be backfilled from the "Semantic search" card below.',
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
function At() {
  var l;
  return typeof navigator > "u" || (l = navigator.language) != null && l.toLowerCase().startsWith("zh") ? "zh" : "en";
}
const je = Zt({
  legacy: !1,
  locale: At(),
  fallbackLocale: "zh",
  messages: { zh: Sa, en: Ea },
  // 面向宿主组件库，缺 key 时静默回退即可，不刷控制台
  missingWarn: !1,
  fallbackWarn: !1
}), { t } = je.global;
function Va(l) {
  je.global.locale.value = l;
}
function Aa(l) {
  Va(l === "auto" ? At() : l);
}
function cn() {
  return je.global.locale.value;
}
const Mt = Symbol("memory-ui-config"), Oe = {
  baseUrl: "",
  fetch: (...l) => globalThis.fetch(...l),
  defaultPageSize: 20,
  locale: "auto"
};
function un(l) {
  l.locale && Aa(l.locale), Gt(Mt, l);
}
function it() {
  const l = Wt(Mt);
  return {
    baseUrl: ((l == null ? void 0 : l.baseUrl) ?? Oe.baseUrl).replace(/\/+$/, ""),
    fetch: (l == null ? void 0 : l.fetch) ?? Oe.fetch,
    defaultPageSize: (l == null ? void 0 : l.defaultPageSize) ?? Oe.defaultPageSize,
    locale: (l == null ? void 0 : l.locale) ?? Oe.locale,
    onIdentityToken: l == null ? void 0 : l.onIdentityToken,
    onAuthChanged: l == null ? void 0 : l.onAuthChanged
  };
}
const Ma = 72;
function Pt(l, r) {
  let i;
  return i = Ca({
    type: l,
    message: r,
    offset: Ma,
    showClose: !0,
    grouping: !0,
    onClick: () => i.close()
  }), i;
}
function X(l) {
  return Pt("success", l);
}
function Y(l) {
  return Pt("error", l);
}
function Te(l) {
  if (!l) return "—";
  const r = je.global.locale.value === "en" ? "en-US" : "zh-CN";
  return new Date(l * 1e3).toLocaleString(r, { hour12: !1 });
}
function Pa(l) {
  return l == null ? "—" : l < 1024 ? `${l} B` : l < 1024 * 1024 ? `${(l / 1024).toFixed(1)} KB` : `${(l / 1024 / 1024).toFixed(2)} MB`;
}
function Ua(l) {
  async function r(d, g = {}) {
    const y = await l.fetch(l.baseUrl + d, {
      headers: { "Content-Type": "application/json" },
      ...g
    }), w = await y.text();
    let v = null;
    try {
      v = w ? JSON.parse(w) : null;
    } catch {
      v = null;
    }
    if (!y.ok) {
      const _ = (v == null ? void 0 : v.error) ?? t("errors.http", { status: y.status });
      throw new Error(_);
    }
    return v;
  }
  async function i(d) {
    var y;
    const g = await l.fetch(l.baseUrl + d);
    if (!g.ok) {
      const w = await g.text().catch(() => "");
      let v = t("errors.http", { status: g.status });
      try {
        v = ((y = JSON.parse(w)) == null ? void 0 : y.error) ?? v;
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
  return Ua(it());
}
function xa(l) {
  const r = Ye(l.query), i = new URLSearchParams();
  return r ? (i.set("query", l.query.trim()), l.tagFilter && i.set("tags", l.tagFilter), l.mode && l.mode !== "auto" && i.set("mode", l.mode)) : (l.tagFilter && i.set("tag", l.tagFilter), i.set("sort", l.sort), i.set("order", l.order)), i.set("offset", String((l.page - 1) * l.pageSize)), i.set("limit", String(l.pageSize)), i.toString();
}
function Ye(l) {
  return l.trim().length > 0;
}
const $a = new Ta();
function Da(l) {
  const r = $a.parse(l, { async: !1 });
  return Vt.sanitize(r);
}
function Ia(l) {
  return Vt.sanitize(l);
}
function rt(l, r) {
  return l.get(`/api/memories/${encodeURIComponent(r)}`);
}
function Ut(l, r) {
  return l.post("/api/memories", r);
}
function xt(l, r, i) {
  return l.put(`/api/memories/${encodeURIComponent(r)}`, i);
}
function Ba(l, r) {
  return l.del(`/api/memories/${encodeURIComponent(r)}`);
}
function $t(l, r) {
  const i = r ? `?filter=${encodeURIComponent(r)}` : "";
  return l.get(`/api/tags${i}`);
}
function La(l, r, i) {
  return l.post("/api/tags", { name: r, description: i });
}
function za(l, r, i, d) {
  const g = { description: d }, y = i.trim();
  return y && y !== r && (g.new_name = y), l.put(`/api/tags/${encodeURIComponent(r)}`, g);
}
function Ra(l, r, i) {
  return l.del(`/api/tags/${encodeURIComponent(r)}?mode=${i}`);
}
function Oa() {
  const l = he(), { defaultPageSize: r } = it(), i = m(""), d = m(""), g = m("auto"), y = m("updated_at"), w = m("desc"), v = m(1), _ = m(r), M = m([]), V = m([]), b = m(0), C = m(!1), P = m([]), R = pe(() => Ye(i.value));
  function $() {
    return v.value = 1, J();
  }
  function U() {
    return xa({
      query: i.value,
      tagFilter: d.value,
      mode: g.value,
      sort: y.value,
      order: w.value,
      page: v.value,
      pageSize: _.value
    });
  }
  let I = 0;
  async function J() {
    var le;
    const F = ++I;
    C.value = !0;
    try {
      const q = U();
      if (Ye(i.value)) {
        const W = await l.get(`/api/memories?${q}`);
        if (F !== I) return;
        V.value = (W.results ?? []).map((S) => ({ ...S, snippet: Ia(S.snippet) })), b.value = W.total_matches ?? 0, note.value = W.semantic_fallback ? t("memories.semanticFallback") : "";
      } else {
        const W = await l.get(`/api/memories?${q}`);
        if (F !== I) return;
        const S = Math.max(1, Math.ceil(W.total / _.value));
        if (((le = W.memories) == null ? void 0 : le.length) === 0 && W.total > 0 && v.value > S)
          return v.value = S, C.value = !1, J();
        M.value = W.memories ?? [], b.value = W.total ?? 0;
      }
    } finally {
      F === I && (C.value = !1);
    }
  }
  async function K() {
    try {
      const F = await $t(l);
      P.value = (F.tags ?? []).map((le) => le.name);
    } catch {
    }
  }
  async function ae(F) {
    if (F.id) {
      const le = await rt(l, F.id), q = new Set(le.tags), W = new Set(F.tags);
      await xt(l, F.id, {
        summary: F.summary,
        content: F.content,
        add_tags: [...W].filter((S) => !q.has(S)),
        remove_tags: [...q].filter((S) => !W.has(S))
      });
    } else
      await Ut(l, { summary: F.summary, content: F.content, tags: F.tags });
    await Promise.all([J(), K()]);
  }
  async function O(F) {
    await Ba(l, F), await J();
  }
  return Ue(() => {
    J().catch(() => {
    }), K();
  }), {
    query: i,
    tagFilter: d,
    mode: g,
    sort: y,
    order: w,
    page: v,
    pageSize: _,
    rows: M,
    searchResults: V,
    total: b,
    loading: C,
    tagOptions: P,
    searching: R,
    onSearch: $,
    reload: J,
    loadTagOptions: K,
    saveMemory: ae,
    removeMemory: O
  };
}
function Je(l) {
  const r = m(0);
  let i = null;
  function d(w) {
    i == null || i.disconnect(), i = null, !(!w || typeof ResizeObserver > "u") && (i = new ResizeObserver((v) => {
      var _;
      r.value = ((_ = v[0]) == null ? void 0 : _.contentRect.width) ?? 0;
    }), i.observe(w));
  }
  Ue(() => d(l.value)), Ne(l, (w) => d(w)), Qt(() => i == null ? void 0 : i.disconnect());
  const g = pe(() => r.value > 0 && r.value < 960), y = pe(() => r.value > 0 && r.value < 720);
  return { width: r, compact: g, narrow: y };
}
const Na = ["innerHTML"], Dt = /* @__PURE__ */ ye({
  __name: "MarkdownView",
  props: {
    source: {}
  },
  setup(l) {
    const r = l, i = pe(() => Da(r.source));
    return (d, g) => (f(), x("div", {
      class: "md-body",
      innerHTML: i.value
    }, null, 8, Na));
  }
}), Fa = { class: "content-label" }, Ha = /* @__PURE__ */ ye({
  __name: "MemoryEditorDialog",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    tagOptions: {},
    width: {}
  },
  emits: ["update:visible", "saved"],
  setup(l, { emit: r }) {
    const i = l, d = r, g = he(), y = m(!1), w = m("edit"), v = m({ id: null, summary: "", content: "", tags: [] });
    let _ = [];
    Ne(
      () => i.visible,
      async (V) => {
        if (V)
          if (w.value = "edit", i.memoryId)
            try {
              const b = await rt(g, i.memoryId);
              v.value = { id: b.id, summary: b.summary, content: b.content, tags: [...b.tags] }, _ = [...b.tags];
            } catch (b) {
              Y(b instanceof Error ? b.message : String(b)), d("update:visible", !1);
            }
          else
            v.value = { id: null, summary: "", content: "", tags: [] }, _ = [];
      }
    );
    async function M() {
      y.value = !0;
      try {
        if (v.value.id) {
          const V = new Set(_), b = new Set(v.value.tags);
          await xt(g, v.value.id, {
            summary: v.value.summary,
            content: v.value.content,
            add_tags: [...b].filter((C) => !V.has(C)),
            remove_tags: [...V].filter((C) => !b.has(C))
          }), X(t("editor.updated"));
        } else
          await Ut(g, {
            summary: v.value.summary,
            content: v.value.content,
            tags: v.value.tags
          }), X(t("editor.created"));
        d("update:visible", !1), d("saved");
      } catch (V) {
        Y(V instanceof Error ? V.message : String(V));
      } finally {
        y.value = !1;
      }
    }
    return (V, b) => {
      const C = Fe, P = tt, R = at, $ = He, U = ht, I = _t, J = et, K = xe, ae = Ze;
      return f(), A(ae, {
        "model-value": l.visible,
        title: v.value.id ? e(t)("editor.editTitle") : e(t)("editor.createTitle"),
        width: l.width,
        "onUpdate:modelValue": b[5] || (b[5] = (O) => d("update:visible", O))
      }, {
        footer: n(() => [
          a(K, {
            onClick: b[4] || (b[4] = (O) => d("update:visible", !1))
          }, {
            default: n(() => [
              u(s(e(t)("common.cancel")), 1)
            ]),
            _: 1
          }),
          a(K, {
            type: "primary",
            loading: y.value,
            onClick: M
          }, {
            default: n(() => [
              u(s(e(t)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: n(() => [
          a(J, { "label-position": "top" }, {
            default: n(() => [
              a(P, {
                label: e(t)("editor.summaryLabel")
              }, {
                default: n(() => [
                  a(C, {
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
                label: n(() => [
                  k("div", Fa, [
                    k("span", null, s(e(t)("editor.contentLabel")), 1),
                    a($, {
                      modelValue: w.value,
                      "onUpdate:modelValue": b[1] || (b[1] = (O) => w.value = O),
                      size: "small"
                    }, {
                      default: n(() => [
                        a(R, { value: "edit" }, {
                          default: n(() => [
                            u(s(e(t)("editor.tabEdit")), 1)
                          ]),
                          _: 1
                        }),
                        a(R, { value: "preview" }, {
                          default: n(() => [
                            u(s(e(t)("editor.tabPreview")), 1)
                          ]),
                          _: 1
                        })
                      ]),
                      _: 1
                    }, 8, ["modelValue"])
                  ])
                ]),
                default: n(() => [
                  w.value === "edit" ? (f(), A(C, {
                    key: 0,
                    modelValue: v.value.content,
                    "onUpdate:modelValue": b[2] || (b[2] = (O) => v.value.content = O),
                    type: "textarea",
                    rows: 12,
                    maxlength: "262144",
                    "show-word-limit": "",
                    placeholder: e(t)("editor.contentPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])) : (f(), A(Dt, {
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
                default: n(() => [
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
                    default: n(() => [
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
}), Se = (l, r) => {
  const i = l.__vccOpts || l;
  for (const [d, g] of r)
    i[d] = g;
  return i;
}, Ka = /* @__PURE__ */ Se(Ha, [["__scopeId", "data-v-a0fec0dd"]]), qa = { class: "detail-summary" }, ja = { class: "detail-tags" }, Ja = { class: "detail-toolbar" }, Wa = {
  key: 1,
  class: "detail-content"
}, Ga = /* @__PURE__ */ ye({
  __name: "MemoryDetailDrawer",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    size: {}
  },
  emits: ["update:visible"],
  setup(l, { emit: r }) {
    const i = l, d = r, g = he(), y = m(null), w = m("rendered");
    return Ne(
      () => [i.visible, i.memoryId],
      async ([v]) => {
        if (!(!v || !i.memoryId)) {
          w.value = "rendered";
          try {
            y.value = await rt(g, i.memoryId);
          } catch (_) {
            Y(_ instanceof Error ? _.message : String(_)), d("update:visible", !1);
          }
        }
      }
    ), (v, _) => {
      var R;
      const M = Ke, V = ta, b = at, C = He, P = ea;
      return f(), A(P, {
        "model-value": l.visible,
        title: e(t)("drawer.title", { id: ((R = y.value) == null ? void 0 : R.id) ?? l.memoryId ?? "" }),
        size: l.size,
        "onUpdate:modelValue": _[1] || (_[1] = ($) => d("update:visible", $))
      }, {
        default: n(() => [
          y.value ? (f(), x(se, { key: 0 }, [
            k("h3", qa, s(y.value.summary), 1),
            k("div", ja, [
              (f(!0), x(se, null, be(y.value.tags, ($) => (f(), A(M, {
                key: $,
                size: "small",
                class: "am-tag"
              }, {
                default: n(() => [
                  u(s($), 1)
                ]),
                _: 2
              }, 1024))), 128))
            ]),
            a(V),
            k("div", Ja, [
              a(C, {
                modelValue: w.value,
                "onUpdate:modelValue": _[0] || (_[0] = ($) => w.value = $),
                size: "small"
              }, {
                default: n(() => [
                  a(b, { value: "rendered" }, {
                    default: n(() => [
                      u(s(e(t)("drawer.rendered")), 1)
                    ]),
                    _: 1
                  }),
                  a(b, { value: "source" }, {
                    default: n(() => [
                      u(s(e(t)("drawer.source")), 1)
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["modelValue"])
            ]),
            w.value === "rendered" ? (f(), A(Dt, {
              key: 0,
              source: y.value.content
            }, null, 8, ["source"])) : (f(), x("pre", Wa, s(y.value.content), 1))
          ], 64)) : G("", !0)
        ]),
        _: 1
      }, 8, ["model-value", "title", "size"]);
    };
  }
}), Qa = /* @__PURE__ */ Se(Ga, [["__scopeId", "data-v-613d4260"]]), Xa = {
  key: 0,
  class: "am-panel-header"
}, Ya = { class: "am-heading" }, Za = { class: "am-panel-title" }, el = { class: "am-toolbar" }, tl = { class: "am-summary" }, al = ["innerHTML"], ll = {
  key: 4,
  class: "am-pager"
}, nl = {
  key: 5,
  class: "am-pager"
}, ol = /* @__PURE__ */ ye({
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
      mode: y,
      sort: w,
      order: v,
      page: _,
      pageSize: M,
      rows: V,
      searchResults: b,
      total: C,
      loading: P,
      tagOptions: R,
      searching: $,
      onSearch: U,
      reload: I,
      loadTagOptions: J,
      removeMemory: K
    } = Oa(), ae = pe(() => {
      if (P.value || C.value !== 0) return "";
      if ($.value) return t("memories.searchEmpty");
      const B = g.value.trim();
      return B ? t("memories.tagEmpty", { tag: B }) : "";
    }), O = m(null), { compact: F, narrow: le } = Je(O), q = m(!1), W = m(null), S = m(!1), D = m(null);
    function z(B) {
      return B().catch((p) => Y(p instanceof Error ? p.message : String(p)));
    }
    function ke() {
      W.value = null, q.value = !0;
    }
    function Q(B) {
      W.value = B, q.value = !0;
    }
    function ie(B) {
      D.value = B, S.value = !0;
    }
    function ue() {
      z(I), J();
    }
    function ne(B) {
      const { prop: p, order: Z } = B;
      Z && (p === "updated_at" || p === "created_at" || p === "id") ? (w.value = p, v.value = Z === "ascending" ? "asc" : "desc") : (w.value = "updated_at", v.value = "desc"), z(I);
    }
    async function fe(B) {
      try {
        await Me.confirm(t("memories.deleteConfirm", { id: B.id }), t("memories.deleteTitle"), {
          type: "warning"
        });
      } catch {
        return;
      }
      try {
        await K(B.id), X(t("memories.deleted"));
      } catch (p) {
        Y(p instanceof Error ? p.message : String(p));
      }
    }
    return r({ refresh: () => z(I) }), (B, p) => {
      const Z = $e, re = qe, ee = xe, de = Fe, E = ht, H = _t, De = kt, te = nt, Ee = Ke, Ie = lt, Be = aa, Le = wt;
      return f(), x("div", {
        ref_key: "rootRef",
        ref: O,
        class: "am-panel"
      }, [
        l.showHeader ? (f(), x("div", Xa, [
          k("div", Ya, [
            k("h2", Za, s(i.title ?? e(t)("memories.title")), 1),
            a(re, {
              content: i.subtitle ?? e(t)("memories.subtitle"),
              placement: "top"
            }, {
              default: n(() => [
                a(Z, { class: "am-info" }, {
                  default: n(() => [
                    a(e(Pe))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(ee, {
            type: "primary",
            icon: e(ot),
            onClick: ke
          }, {
            default: n(() => [
              u(s(e(t)("memories.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : G("", !0),
        k("div", el, [
          a(de, {
            modelValue: e(d),
            "onUpdate:modelValue": p[1] || (p[1] = (h) => ge(d) ? d.value = h : null),
            placeholder: e(t)("memories.searchPlaceholder"),
            clearable: "",
            class: "search",
            onKeyup: p[2] || (p[2] = Xt((h) => z(e(U)), ["enter"])),
            onClear: p[3] || (p[3] = (h) => z(e(U)))
          }, {
            append: n(() => [
              a(ee, {
                icon: e(st),
                onClick: p[0] || (p[0] = (h) => z(e(U)))
              }, null, 8, ["icon"])
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(H, {
            modelValue: e(g),
            "onUpdate:modelValue": p[4] || (p[4] = (h) => ge(g) ? g.value = h : null),
            placeholder: e(t)("memories.tagFilter"),
            clearable: "",
            filterable: "",
            class: "tag-filter",
            onChange: p[5] || (p[5] = (h) => z(e(U)))
          }, {
            default: n(() => [
              (f(!0), x(se, null, be(e(R), (h) => (f(), A(E, {
                key: h,
                label: h,
                value: h
              }, null, 8, ["label", "value"]))), 128))
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(H, {
            modelValue: e(y),
            "onUpdate:modelValue": p[6] || (p[6] = (h) => ge(y) ? y.value = h : null),
            class: "mode-select",
            onChange: p[7] || (p[7] = (h) => z(e(U)))
          }, {
            default: n(() => [
              a(E, {
                label: e(t)("memories.modeAuto"),
                value: "auto"
              }, null, 8, ["label"]),
              a(E, {
                label: e(t)("memories.modeKeyword"),
                value: "keyword"
              }, null, 8, ["label"]),
              a(E, {
                label: e(t)("memories.modeHybrid"),
                value: "hybrid"
              }, null, 8, ["label"])
            ]),
            _: 1
          }, 8, ["modelValue"])
        ]),
        ae.value ? (f(), A(De, {
          key: 1,
          title: ae.value,
          type: "info",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : G("", !0),
        e($) ? Ce((f(), A(Ie, {
          key: 2,
          data: e(b)
        }, {
          default: n(() => [
            a(te, {
              prop: "id",
              label: e(t)("memories.colId"),
              width: "80"
            }, null, 8, ["label"]),
            a(te, {
              label: e(t)("memories.colSummary")
            }, {
              default: n(({ row: h }) => [
                k("div", tl, s(h.summary), 1),
                k("div", {
                  class: "am-snippet",
                  innerHTML: h.snippet
                }, null, 8, al)
              ]),
              _: 1
            }, 8, ["label"]),
            a(te, {
              label: e(t)("memories.colTags"),
              "min-width": "150"
            }, {
              default: n(({ row: h }) => [
                (f(!0), x(se, null, be(h.tags, (oe) => (f(), A(Ee, {
                  key: oe,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: n(() => [
                    u(s(oe), 1)
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
              default: n(({ row: h }) => [
                u(s(e(Te)(h.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(te, {
              label: e(t)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: n(({ row: h }) => [
                a(ee, {
                  link: "",
                  type: "primary",
                  onClick: (oe) => ie(h.id)
                }, {
                  default: n(() => [
                    u(s(e(t)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(ee, {
                  link: "",
                  type: "primary",
                  onClick: (oe) => Q(h.id)
                }, {
                  default: n(() => [
                    u(s(e(t)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(ee, {
                  link: "",
                  type: "danger",
                  onClick: (oe) => fe(h)
                }, {
                  default: n(() => [
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
          [Le, e(P)]
        ]) : Ce((f(), A(Ie, {
          key: 3,
          data: e(V),
          "default-sort": { prop: e(w), order: e(v) === "asc" ? "ascending" : "descending" },
          onSortChange: ne
        }, {
          default: n(() => [
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
              default: n(({ row: h }) => [
                (f(!0), x(se, null, be(h.tags, (oe) => (f(), A(Ee, {
                  key: oe,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: n(() => [
                    u(s(oe), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            e(F) ? G("", !0) : (f(), A(te, {
              key: 0,
              prop: "created_at",
              label: e(t)("memories.colCreatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: n(({ row: h }) => [
                u(s(e(Te)(h.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label"])),
            a(te, {
              prop: "updated_at",
              label: e(t)("memories.colUpdatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: n(({ row: h }) => [
                u(s(e(Te)(h.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(te, {
              label: e(t)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: n(({ row: h }) => [
                a(ee, {
                  link: "",
                  type: "primary",
                  onClick: (oe) => ie(h.id)
                }, {
                  default: n(() => [
                    u(s(e(t)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(ee, {
                  link: "",
                  type: "primary",
                  onClick: (oe) => Q(h.id)
                }, {
                  default: n(() => [
                    u(s(e(t)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(ee, {
                  link: "",
                  type: "danger",
                  onClick: (oe) => fe(h)
                }, {
                  default: n(() => [
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
          [Le, e(P)]
        ]),
        e($) ? (f(), x("div", nl, [
          a(Be, {
            "current-page": e(_),
            "onUpdate:currentPage": p[12] || (p[12] = (h) => ge(_) ? _.value = h : null),
            "page-size": e(M),
            "onUpdate:pageSize": p[13] || (p[13] = (h) => ge(M) ? M.value = h : null),
            total: e(C),
            "page-sizes": [10, 20, 50],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: p[14] || (p[14] = (h) => z(e(I))),
            onSizeChange: p[15] || (p[15] = (h) => z(e(I)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])) : (f(), x("div", ll, [
          a(Be, {
            "current-page": e(_),
            "onUpdate:currentPage": p[8] || (p[8] = (h) => ge(_) ? _.value = h : null),
            "page-size": e(M),
            "onUpdate:pageSize": p[9] || (p[9] = (h) => ge(M) ? M.value = h : null),
            total: e(C),
            "page-sizes": [20, 50, 100, 200],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: p[10] || (p[10] = (h) => z(e(I))),
            onSizeChange: p[11] || (p[11] = (h) => z(e(I)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])),
        a(Ka, {
          visible: q.value,
          "onUpdate:visible": p[16] || (p[16] = (h) => q.value = h),
          "memory-id": W.value,
          "tag-options": e(R),
          width: e(le) ? "96%" : "640px",
          onSaved: ue
        }, null, 8, ["visible", "memory-id", "tag-options", "width"]),
        a(Qa, {
          visible: S.value,
          "onUpdate:visible": p[17] || (p[17] = (h) => S.value = h),
          "memory-id": D.value,
          size: e(le) ? "100%" : "45%"
        }, null, 8, ["visible", "memory-id", "size"])
      ], 512);
    };
  }
}), sl = /* @__PURE__ */ Se(ol, [["__scopeId", "data-v-a269dc91"]]);
function il() {
  const l = he(), r = m([]), i = m(!1), d = m("");
  async function g() {
    i.value = !0;
    try {
      const _ = await $t(l, d.value.trim() || void 0);
      r.value = _.tags ?? [];
    } finally {
      i.value = !1;
    }
  }
  async function y(_, M) {
    await La(l, _, M), await g();
  }
  async function w(_, M, V) {
    await za(l, _, M, V), await g();
  }
  async function v(_, M) {
    await Ra(l, _, M), await g();
  }
  return Ue(() => {
    g().catch(() => {
    });
  }), { rows: r, loading: i, filter: d, reload: g, create: y, rename: w, remove: v };
}
const rl = {
  key: 0,
  class: "am-panel-header"
}, dl = { class: "am-heading" }, cl = { class: "am-panel-title" }, ul = { class: "delete-body" }, ml = /* @__PURE__ */ ye({
  __name: "TagsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(l, { expose: r }) {
    const i = l, { rows: d, loading: g, filter: y, reload: w, create: v, rename: _, remove: M } = il();
    let V = null;
    function b() {
      V && clearTimeout(V), V = setTimeout(() => {
        V = null, w().catch((S) => Y(S instanceof Error ? S.message : String(S)));
      }, 300);
    }
    const C = m(null), { narrow: P } = Je(C), R = m(!1), $ = m(!1), U = yt({ oldName: null, name: "", description: "" }), I = m(!1), J = m("detach"), K = m(null);
    function ae(S) {
      return (D, z) => (D[S] ?? 0) - (z[S] ?? 0);
    }
    function O() {
      Object.assign(U, { oldName: null, name: "", description: "" }), $.value = !0;
    }
    function F(S) {
      Object.assign(U, { oldName: S.name, name: S.name, description: S.description ?? "" }), $.value = !0;
    }
    async function le() {
      R.value = !0;
      try {
        U.oldName ? (await _(U.oldName, U.name, U.description), X(t("tags.saved"))) : (await v(U.name, U.description), X(t("tags.created"))), $.value = !1;
      } catch (S) {
        Y(S instanceof Error ? S.message : String(S));
      } finally {
        R.value = !1;
      }
    }
    function q(S) {
      K.value = S, J.value = "detach", I.value = !0;
    }
    async function W() {
      var S, D;
      if (J.value === "purge")
        try {
          await Me.confirm(
            t("tags.purgeConfirm", { name: (S = K.value) == null ? void 0 : S.name, count: ((D = K.value) == null ? void 0 : D.memory_count) ?? 0 }),
            t("tags.purgeConfirmTitle"),
            { type: "error", confirmButtonText: t("tags.purgeButton") }
          );
        } catch {
          return;
        }
      if (K.value) {
        R.value = !0;
        try {
          await M(K.value.name, J.value), X(t("tags.deleted")), I.value = !1;
        } catch (z) {
          Y(z instanceof Error ? z.message : String(z));
        } finally {
          R.value = !1;
        }
      }
    }
    return r({
      refresh: () => w().catch((S) => Y(S instanceof Error ? S.message : String(S)))
    }), (S, D) => {
      const z = $e, ke = qe, Q = xe, ie = Fe, ue = Ke, ne = nt, fe = lt, B = tt, p = et, Z = Ze, re = la, ee = He, de = wt;
      return f(), x("div", {
        ref_key: "rootRef",
        ref: C,
        class: "am-panel"
      }, [
        l.showHeader ? (f(), x("div", rl, [
          k("div", dl, [
            k("h2", cl, s(i.title ?? e(t)("tags.title")), 1),
            a(ke, {
              content: i.subtitle ?? e(t)("tags.subtitle"),
              placement: "top"
            }, {
              default: n(() => [
                a(z, { class: "am-info" }, {
                  default: n(() => [
                    a(e(Pe))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(Q, {
            type: "primary",
            icon: e(ot),
            onClick: O
          }, {
            default: n(() => [
              u(s(e(t)("tags.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : G("", !0),
        a(ie, {
          modelValue: e(y),
          "onUpdate:modelValue": D[0] || (D[0] = (E) => ge(y) ? y.value = E : null),
          class: "am-tag-filter",
          placeholder: e(t)("tags.filterPlaceholder"),
          clearable: "",
          "prefix-icon": e(st),
          onInput: b,
          onClear: b
        }, null, 8, ["modelValue", "placeholder", "prefix-icon"]),
        Ce((f(), A(fe, { data: e(d) }, {
          default: n(() => [
            a(ne, {
              prop: "name",
              label: e(t)("tags.colName"),
              "min-width": "140",
              sortable: ""
            }, {
              default: n(({ row: E }) => [
                a(ue, null, {
                  default: n(() => [
                    u(s(E.name), 1)
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
              default: n(({ row: E }) => [
                u(s(E.description || "—"), 1)
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
              "sort-method": ae("last_used_at")
            }, {
              default: n(({ row: E }) => [
                u(s(e(Te)(E.last_used_at)), 1)
              ]),
              _: 1
            }, 8, ["label", "sort-method"]),
            a(ne, {
              prop: "created_at",
              label: e(t)("tags.colCreatedAt"),
              width: "170",
              sortable: "",
              "sort-method": ae("created_at")
            }, {
              default: n(({ row: E }) => [
                u(s(e(Te)(E.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label", "sort-method"]),
            a(ne, {
              label: e(t)("memories.colActions"),
              width: "150",
              fixed: "right"
            }, {
              default: n(({ row: E }) => [
                a(Q, {
                  link: "",
                  type: "primary",
                  onClick: (H) => F(E)
                }, {
                  default: n(() => [
                    u(s(e(t)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(Q, {
                  link: "",
                  type: "danger",
                  onClick: (H) => q(E)
                }, {
                  default: n(() => [
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
          "onUpdate:modelValue": D[4] || (D[4] = (E) => $.value = E),
          title: U.oldName ? e(t)("tags.editTitle") : e(t)("tags.createTitle"),
          width: e(P) ? "96%" : "480px"
        }, {
          footer: n(() => [
            a(Q, {
              onClick: D[3] || (D[3] = (E) => $.value = !1)
            }, {
              default: n(() => [
                u(s(e(t)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(Q, {
              type: "primary",
              loading: R.value,
              onClick: le
            }, {
              default: n(() => [
                u(s(e(t)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: n(() => [
            a(p, { "label-position": "top" }, {
              default: n(() => [
                a(B, {
                  label: e(t)("tags.nameLabel")
                }, {
                  default: n(() => [
                    a(ie, {
                      modelValue: U.name,
                      "onUpdate:modelValue": D[1] || (D[1] = (E) => U.name = E),
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
                  default: n(() => [
                    a(ie, {
                      modelValue: U.description,
                      "onUpdate:modelValue": D[2] || (D[2] = (E) => U.description = E),
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
          "onUpdate:modelValue": D[7] || (D[7] = (E) => I.value = E),
          title: e(t)("tags.deleteTitle"),
          width: e(P) ? "96%" : "480px"
        }, {
          footer: n(() => [
            a(Q, {
              onClick: D[6] || (D[6] = (E) => I.value = !1)
            }, {
              default: n(() => [
                u(s(e(t)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(Q, {
              type: "danger",
              loading: R.value,
              onClick: W
            }, {
              default: n(() => [
                u(s(e(t)("common.delete")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: n(() => {
            var E;
            return [
              k("p", ul, [
                u(s(e(t)("tags.deleteBefore")) + " ", 1),
                a(ue, null, {
                  default: n(() => {
                    var H;
                    return [
                      u(s((H = K.value) == null ? void 0 : H.name), 1)
                    ];
                  }),
                  _: 1
                }),
                u(" " + s(e(t)("tags.deleteMiddle")) + " ", 1),
                k("b", null, s((E = K.value) == null ? void 0 : E.memory_count), 1),
                u(" " + s(e(t)("tags.deleteAfter")), 1)
              ]),
              a(ee, {
                modelValue: J.value,
                "onUpdate:modelValue": D[5] || (D[5] = (H) => J.value = H)
              }, {
                default: n(() => [
                  a(re, { value: "detach" }, {
                    default: n(() => [
                      u(s(e(t)("tags.detach")), 1)
                    ]),
                    _: 1
                  }),
                  a(re, { value: "purge" }, {
                    default: n(() => [
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
}), pl = /* @__PURE__ */ Se(ml, [["__scopeId", "data-v-c8aeea57"]]);
function fl(l) {
  return l.get("/api/stats");
}
function vl(l) {
  return l.get("/health");
}
function gl(l) {
  return l.get("/api/doctor");
}
function bl(l) {
  return l.getBlob("/api/export");
}
function yl(l, r) {
  return l.post("/api/import", r);
}
function _l(l) {
  return l.post("/api/embeddings/backfill", {});
}
function hl() {
  const l = he(), r = m({}), i = m(""), d = pe(() => Pa(r.value.file_size));
  async function g() {
    r.value = await fl(l), r.value.version = i.value;
  }
  return Ue(async () => {
    try {
      i.value = (await vl(l)).version ?? "";
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
const kl = {
  key: 0,
  class: "am-panel-header"
}, wl = { class: "am-heading" }, Cl = { class: "am-panel-title" }, Tl = /* @__PURE__ */ ye({
  __name: "OpsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(l, { expose: r }) {
    const i = l, { stats: d, version: g, sizeText: y, reload: w } = hl(), v = m(null), { narrow: _ } = Je(v);
    function M(V) {
      return V().catch((b) => Y(b instanceof Error ? b.message : String(b)));
    }
    return r({ refresh: () => M(w) }), (V, b) => {
      const C = $e, P = qe, R = xe, $ = sa, U = Ct, I = oa, J = na, K = St, ae = Tt;
      return f(), x("div", {
        ref_key: "rootRef",
        ref: v,
        class: "am-panel"
      }, [
        l.showHeader ? (f(), x("div", kl, [
          k("div", wl, [
            k("h2", Cl, s(i.title ?? e(t)("ops.title")), 1),
            a(P, {
              content: i.subtitle ?? e(t)("ops.subtitle"),
              placement: "top"
            }, {
              default: n(() => [
                a(C, { class: "am-info" }, {
                  default: n(() => [
                    a(e(Pe))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(R, {
            icon: e(Et),
            onClick: b[0] || (b[0] = (O) => M(e(w)))
          }, {
            default: n(() => [
              u(s(e(t)("ops.refresh")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : G("", !0),
        a(J, { gutter: 14 }, {
          default: n(() => [
            a(I, {
              span: e(_) ? 12 : 8
            }, {
              default: n(() => [
                a(U, { shadow: "never" }, {
                  default: n(() => [
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
              span: e(_) ? 12 : 8
            }, {
              default: n(() => [
                a(U, { shadow: "never" }, {
                  default: n(() => [
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
              span: e(_) ? 12 : 8
            }, {
              default: n(() => [
                a(U, { shadow: "never" }, {
                  default: n(() => [
                    a($, {
                      title: e(t)("ops.statSize"),
                      value: e(y)
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
          header: n(() => [
            u(s(e(t)("ops.dbCard")), 1)
          ]),
          default: n(() => [
            a(ae, {
              column: e(_) ? 1 : 2,
              border: ""
            }, {
              default: n(() => [
                a(K, {
                  label: e(t)("ops.version")
                }, {
                  default: n(() => [
                    u(s(e(d).version ?? e(g)), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(K, {
                  label: e(t)("ops.schemaVersion")
                }, {
                  default: n(() => [
                    u(s(e(d).schema_version ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(K, {
                  label: e(t)("ops.path"),
                  span: e(_) ? 1 : 2
                }, {
                  default: n(() => [
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
}), Sl = { class: "memory-ui" }, El = { class: "brand" }, Vl = { class: "brand-mark" }, Al = { class: "aside-footer" }, Ml = /* @__PURE__ */ ye({
  __name: "MemoryAdmin",
  props: {
    layout: { default: "sidebar" },
    title: { default: "Agent Memory" }
  },
  setup(l) {
    const r = m("memories");
    return (i, d) => {
      const g = $e, y = ca, w = da, v = ra, _ = pa, M = ma, V = ua, b = ia;
      return f(), x("div", Sl, [
        a(b, { class: "layout" }, {
          default: n(() => [
            l.layout === "sidebar" ? (f(), A(v, {
              key: 0,
              width: "200px",
              class: "aside"
            }, {
              default: n(() => [
                k("div", El, [
                  k("span", Vl, [
                    a(g, { size: 16 }, {
                      default: n(() => [
                        a(e(ba))
                      ]),
                      _: 1
                    })
                  ]),
                  k("span", null, s(l.title), 1)
                ]),
                a(w, {
                  "default-active": r.value,
                  class: "menu",
                  onSelect: d[0] || (d[0] = (C) => r.value = C)
                }, {
                  default: n(() => [
                    a(y, { index: "memories" }, {
                      default: n(() => [
                        a(g, null, {
                          default: n(() => [
                            a(e(ya))
                          ]),
                          _: 1
                        }),
                        k("span", null, s(e(t)("nav.memories")), 1)
                      ]),
                      _: 1
                    }),
                    a(y, { index: "tags" }, {
                      default: n(() => [
                        a(g, null, {
                          default: n(() => [
                            a(e(_a))
                          ]),
                          _: 1
                        }),
                        k("span", null, s(e(t)("nav.tags")), 1)
                      ]),
                      _: 1
                    }),
                    a(y, { index: "ops" }, {
                      default: n(() => [
                        a(g, null, {
                          default: n(() => [
                            a(e(ha))
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
                k("div", Al, [
                  Yt(i.$slots, "footer", {}, void 0, !0)
                ])
              ]),
              _: 3
            })) : G("", !0),
            a(V, { class: "main" }, {
              default: n(() => [
                l.layout === "tabs" ? (f(), A(M, {
                  key: 0,
                  modelValue: r.value,
                  "onUpdate:modelValue": d[1] || (d[1] = (C) => r.value = C),
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
                }, 8, ["modelValue"])) : G("", !0),
                Ce(a(sl, null, null, 512), [
                  [Qe, r.value === "memories"]
                ]),
                Ce(a(pl, null, null, 512), [
                  [Qe, r.value === "tags"]
                ]),
                Ce(a(Tl, null, null, 512), [
                  [Qe, r.value === "ops"]
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
}), mn = /* @__PURE__ */ Se(Ml, [["__scopeId", "data-v-38c91753"]]);
function Pl() {
  const l = he(), r = m({ ok: !0, issues: [] }), i = m(!1), d = m(!1), g = m(!1), y = m(!1), w = m(!1);
  async function v() {
    d.value = !0;
    try {
      r.value = await gl(l), i.value = !0;
    } finally {
      d.value = !1;
    }
  }
  async function _() {
    g.value = !0;
    try {
      const b = await bl(l), C = URL.createObjectURL(b), P = document.createElement("a");
      P.href = C, P.download = "agent-memory-export.json", P.click(), URL.revokeObjectURL(C);
    } finally {
      g.value = !1;
    }
  }
  async function M(b) {
    y.value = !0;
    try {
      const C = await b.text();
      let P;
      try {
        P = JSON.parse(C);
      } catch {
        throw new Error(t("errors.invalidBackup"));
      }
      return await yl(l, P);
    } finally {
      y.value = !1;
    }
  }
  async function V() {
    w.value = !0;
    try {
      let b = 0;
      for (; ; ) {
        const C = await _l(l);
        if (!C.configured) throw new Error(t("ops.embeddingNotConfigured"));
        if (C.error) throw new Error(C.error);
        if (b += C.processed ?? 0, (C.processed ?? 0) === 0) break;
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
    importing: y,
    backfilling: w,
    runDoctor: v,
    exportData: _,
    importFile: M,
    backfill: V
  };
}
const Ul = {
  key: 0,
  class: "am-panel-header"
}, xl = { class: "am-heading" }, $l = { class: "am-panel-title" }, Dl = { class: "auth-row" }, Il = { class: "auth-text" }, Bl = { class: "auth-label" }, Ll = { class: "auth-hint" }, zl = { class: "token-cell" }, Rl = { class: "token-text" }, Ol = {
  key: 0,
  class: "muted"
}, Nl = { class: "save-row" }, Fl = { class: "field-hint" }, Hl = {
  key: 0,
  class: "default-view"
}, Kl = { class: "default-text" }, ql = { class: "field-hint" }, jl = { class: "save-row" }, Jl = { class: "actions" }, Wl = { class: "card-header" }, Gl = {
  key: 2,
  class: "issues"
}, Ql = { class: "card-header" }, Xl = { class: "presets" }, Yl = { class: "presets-label" }, Zl = { class: "caps" }, en = { class: "new-token" }, tn = /* @__PURE__ */ ye({
  __name: "AdminPanel",
  props: {
    who: { default: null },
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(l, { expose: r }) {
    const i = l, d = he(), g = it(), y = [
      { key: "read", labelKey: "capRead" },
      { key: "create", labelKey: "capCreate" },
      { key: "update", labelKey: "capUpdate" },
      { key: "delete", labelKey: "capDelete" },
      { key: "tag_manage", labelKey: "capTagManage" },
      { key: "admin", labelKey: "capAdmin" }
    ], w = m([]), v = m(""), _ = m(""), M = m(""), V = m(!1), b = m(!1), C = m(!1), P = pe(() => {
      var c;
      return !!i.who && ((c = i.who.permissions) == null ? void 0 : c.admin) === !0;
    }), {
      doctor: R,
      doctorRan: $,
      doctorLoading: U,
      exporting: I,
      importing: J,
      backfilling: K,
      runDoctor: ae,
      exportData: O,
      importFile: F,
      backfill: le
    } = Pl(), q = m(null), W = pe(() => {
      var c, o, L, me;
      return (((o = (c = q.value) == null ? void 0 : c.embedding) == null ? void 0 : o.embedded) ?? 0) + (((me = (L = q.value) == null ? void 0 : L.embedding) == null ? void 0 : me.pending) ?? 0);
    });
    async function S() {
      const c = await p(le);
      if (c !== void 0) {
        try {
          q.value = await d.get("/api/stats");
        } catch {
        }
        X(c === 0 ? t("access.embeddingUpToDate") : t("access.embeddingDone", { count: c }));
      }
    }
    const D = m(null), { compact: z } = Je(D), ke = m(null), Q = m(!1), ie = m(""), ue = m(""), ne = m(""), fe = m(!1), B = m(null);
    async function p(c, o) {
      try {
        return await c();
      } catch (L) {
        Y(L instanceof Error ? L.message : String(L));
        return;
      }
    }
    async function Z() {
      await p(async () => {
        const [c, o, L] = await Promise.all([
          d.get("/api/identities"),
          d.get("/api/settings"),
          d.get("/api/stats")
        ]);
        w.value = Array.isArray(c == null ? void 0 : c.identities) ? c.identities : [], v.value = (o == null ? void 0 : o.instructions) ?? "", _.value = (o == null ? void 0 : o.conventions) ?? "", M.value = (o == null ? void 0 : o.default_instructions) ?? "", b.value = (o == null ? void 0 : o.auth_required) === !0, Q.value = (o == null ? void 0 : o.embedding_enabled) === !0, ie.value = (o == null ? void 0 : o.embedding_base_url) ?? "", ue.value = (o == null ? void 0 : o.embedding_model) ?? "", ne.value = (o == null ? void 0 : o.embedding_api_key) ?? "", q.value = L ?? null;
      });
    }
    Ue(() => {
      P.value && Z();
    }), Ne(P, (c) => {
      c && Z();
    }), r({
      refresh: () => {
        P.value && p(Z);
      }
    });
    const re = m(!1), ee = m(!1), de = m(null), E = m("custom"), H = yt({
      name: "",
      caps: De()
    });
    function De() {
      return Object.fromEntries(y.map((c) => [c.key, !1]));
    }
    const te = {
      admin: y.map((c) => c.key),
      member: ["read", "create", "update", "tag_manage"],
      viewer: ["read"]
    };
    function Ee() {
      if (E.value === "custom") return;
      const c = new Set(te[E.value] ?? []);
      for (const o of y) H.caps[o.key] = c.has(o.key);
    }
    function Ie() {
      E.value = "custom";
    }
    function Be() {
      de.value = null, H.name = "", Object.assign(H.caps, De()), E.value = "member", Ee(), re.value = !0;
    }
    function Le(c) {
      var o;
      de.value = c, H.name = c.name;
      for (const L of y) H.caps[L.key] = ((o = c.permissions) == null ? void 0 : o[L.key]) === !0;
      E.value = "custom", re.value = !0;
    }
    async function h() {
      ee.value = !0;
      try {
        if (de.value)
          await p(
            () => d.put(`/api/identities/${encodeURIComponent(de.value.name)}`, {
              permissions: { ...H.caps }
            })
          ), X(t("access.saved"));
        else {
          const c = await p(
            () => d.post("/api/identities", {
              name: H.name,
              permissions: { ...H.caps }
            })
          );
          c != null && c.token && ut(t("access.createTitle"), H.name.trim(), c.token);
        }
        re.value = !1, await Z();
      } finally {
        ee.value = !1;
      }
    }
    async function oe(c) {
      try {
        await Me.confirm(t("access.deleteConfirm", { name: c.name }), t("access.deleteTitle"), {
          type: "warning",
          confirmButtonText: t("common.delete"),
          cancelButtonText: t("common.cancel")
        });
      } catch {
        return;
      }
      await p(() => d.del(`/api/identities/${encodeURIComponent(c.name)}`)) !== void 0 && (X(t("access.deleted")), await Z());
    }
    const We = pe(() => w.value.some((c) => {
      var o;
      return ((o = c.permissions) == null ? void 0 : o.admin) === !0;
    }));
    async function It() {
      var o;
      const c = !b.value;
      if (c && !We.value)
        return Y(t("access.enableBlocked")), !1;
      try {
        await Me.confirm(
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
      C.value = !0;
      try {
        return await p(() => d.put("/api/settings", { auth_required: c })) !== void 0 && ((o = g.onAuthChanged) == null || o.call(g, c)), !0;
      } finally {
        C.value = !1;
      }
    }
    async function Bt() {
      V.value = !0;
      try {
        await p(
          () => d.put("/api/settings", {
            instructions: v.value,
            conventions: _.value
          })
        ), X(t("access.saved"));
      } finally {
        V.value = !1;
      }
    }
    function Lt(c) {
      var me;
      const o = c.target, L = (me = o.files) == null ? void 0 : me[0];
      o.value = "", L && F(L).then((j) => {
        X(t("access.imported", { memories: j.imported_memories, tags: j.imported_tags }));
      }).catch((j) => {
        Y(j instanceof Error ? j.message : String(j));
      });
    }
    async function zt() {
      fe.value = !0, B.value = null;
      try {
        if (await p(
          () => d.put("/api/settings", {
            embedding_enabled: Q.value,
            embedding_base_url: ie.value,
            embedding_model: ue.value,
            embedding_api_key: ne.value
          })
        ) === void 0 || (X(t("access.saved")), !Q.value)) return;
        B.value = await p(() => d.post("/api/embeddings/test", {}));
      } finally {
        fe.value = !1;
      }
    }
    const Ve = m(!1), dt = m(""), ct = m(""), ze = m("");
    function Rt(c) {
      return c ? `…${c}` : "—";
    }
    function ut(c, o, L) {
      dt.value = c, ct.value = o, ze.value = L, Ve.value = !0;
    }
    async function Ot() {
      const c = ct.value;
      if (c)
        try {
          await g.onIdentityToken(c, ze.value), Ve.value = !1, X(t("access.savedToBrowser", { name: c }));
        } catch (o) {
          Y(o instanceof Error ? o.message : String(o));
        }
    }
    async function Nt(c) {
      try {
        await Me.confirm(t("access.resetConfirm", { name: c.name }), t("access.resetTitle"), {
          type: "warning",
          confirmButtonText: t("access.resetToken"),
          cancelButtonText: t("common.cancel")
        });
      } catch {
        return;
      }
      const o = await p(
        () => d.post(`/api/identities/${encodeURIComponent(c.name)}/token-reset`, {})
      );
      o !== void 0 && (o != null && o.token && ut(t("access.resetTitle"), c.name, o.token), await Z());
    }
    function mt(c) {
      return y.map((o) => o.key).filter((o) => {
        var L;
        return ((L = c.permissions) == null ? void 0 : L[o]) === !0;
      });
    }
    function pt(c) {
      const o = y.find((L) => L.key === c);
      return o ? t(`access.${o.labelKey}`) : c;
    }
    async function Ft(c) {
      try {
        await navigator.clipboard.writeText(c), X(t("access.copied"));
      } catch {
        Y(c);
      }
    }
    return (c, o) => {
      const L = $e, me = qe, j = xe, ce = kt, ft = fa, _e = Ct, vt = va, Ae = nt, Ht = Ke, Kt = lt, ve = tt, we = Fe, Ge = et, gt = St, qt = Tt, Re = at, jt = He, Jt = ga, bt = Ze;
      return f(), x("div", {
        ref_key: "rootRef",
        ref: D,
        class: "am-panel admin-panel"
      }, [
        l.showHeader ? (f(), x("div", Ul, [
          k("div", xl, [
            k("h2", $l, s(i.title ?? e(t)("access.title")), 1),
            a(me, {
              content: i.subtitle ?? e(t)("access.subtitle"),
              placement: "top"
            }, {
              default: n(() => [
                a(L, { class: "am-info" }, {
                  default: n(() => [
                    a(e(Pe))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          P.value ? (f(), A(j, {
            key: 0,
            type: "primary",
            icon: e(ot),
            onClick: Be
          }, {
            default: n(() => [
              u(s(e(t)("access.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])) : G("", !0)
        ])) : G("", !0),
        l.who ? l.who.mode === "open" ? (f(), A(ce, {
          key: 2,
          type: "warning",
          title: e(t)("access.openMode"),
          closable: !1
        }, null, 8, ["title"])) : P.value ? G("", !0) : (f(), A(ce, {
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
          a(_e, { shadow: "never" }, {
            default: n(() => [
              k("div", Dl, [
                k("div", Il, [
                  k("span", Bl, s(e(t)("access.authTitle")), 1),
                  k("span", Ll, s(e(t)("access.authHint")), 1)
                ]),
                a(me, {
                  disabled: We.value,
                  content: e(t)("access.enableBlocked"),
                  placement: "top"
                }, {
                  default: n(() => [
                    a(ft, {
                      modelValue: b.value,
                      "onUpdate:modelValue": o[0] || (o[0] = (T) => b.value = T),
                      "before-change": It,
                      loading: C.value,
                      disabled: !We.value
                    }, null, 8, ["modelValue", "loading", "disabled"])
                  ]),
                  _: 1
                }, 8, ["disabled", "content"])
              ])
            ]),
            _: 1
          }),
          a(_e, { shadow: "never" }, {
            default: n(() => [
              w.value.length === 0 ? (f(), A(vt, {
                key: 0,
                description: e(t)("access.empty")
              }, null, 8, ["description"])) : (f(), A(Kt, {
                key: 1,
                data: w.value
              }, {
                default: n(() => [
                  a(Ae, {
                    prop: "name",
                    label: e(t)("access.colName"),
                    "min-width": "120"
                  }, null, 8, ["label"]),
                  a(Ae, {
                    label: e(t)("access.colToken"),
                    "min-width": "200"
                  }, {
                    default: n(({ row: T }) => [
                      k("div", zl, [
                        k("code", Rl, s(Rt(T.token_hint)), 1),
                        a(j, {
                          link: "",
                          type: "primary",
                          onClick: (N) => Nt(T)
                        }, {
                          default: n(() => [
                            u(s(e(t)("access.resetToken")), 1)
                          ]),
                          _: 1
                        }, 8, ["onClick"])
                      ])
                    ]),
                    _: 1
                  }, 8, ["label"]),
                  a(Ae, {
                    label: e(t)("access.colPermissions"),
                    "min-width": "240"
                  }, {
                    default: n(({ row: T }) => [
                      (f(!0), x(se, null, be(mt(T), (N) => (f(), A(Ht, {
                        key: N,
                        size: "small",
                        class: "cap-tag",
                        type: N === "admin" ? "danger" : "info"
                      }, {
                        default: n(() => [
                          u(s(pt(N)), 1)
                        ]),
                        _: 2
                      }, 1032, ["type"]))), 128)),
                      mt(T).length === 0 ? (f(), x("span", Ol, "—")) : G("", !0)
                    ]),
                    _: 1
                  }, 8, ["label"]),
                  e(z) ? G("", !0) : (f(), A(Ae, {
                    key: 0,
                    label: e(t)("access.colCreatedAt"),
                    width: "170"
                  }, {
                    default: n(({ row: T }) => [
                      u(s(e(Te)(T.created_at)), 1)
                    ]),
                    _: 1
                  }, 8, ["label"])),
                  a(Ae, {
                    label: e(t)("access.colActions"),
                    width: "140",
                    fixed: "right"
                  }, {
                    default: n(({ row: T }) => [
                      a(j, {
                        link: "",
                        type: "primary",
                        onClick: (N) => Le(T)
                      }, {
                        default: n(() => [
                          u(s(e(t)("common.edit")), 1)
                        ]),
                        _: 1
                      }, 8, ["onClick"]),
                      a(j, {
                        link: "",
                        type: "danger",
                        onClick: (N) => oe(T)
                      }, {
                        default: n(() => [
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
          a(_e, { shadow: "never" }, {
            header: n(() => [
              u(s(e(t)("access.embeddingTitle")), 1)
            ]),
            default: n(() => {
              var T;
              return [
                a(ce, {
                  title: e(t)("access.embeddingHint"),
                  type: "info",
                  "show-icon": "",
                  closable: !1,
                  class: "settings-hint"
                }, null, 8, ["title"]),
                a(Ge, {
                  "label-position": "top",
                  onSubmit: o[5] || (o[5] = Xe(() => {
                  }, ["prevent"]))
                }, {
                  default: n(() => [
                    a(ve, {
                      label: e(t)("access.embeddingEnabledLabel")
                    }, {
                      default: n(() => [
                        a(ft, {
                          modelValue: Q.value,
                          "onUpdate:modelValue": o[1] || (o[1] = (N) => Q.value = N)
                        }, null, 8, ["modelValue"])
                      ]),
                      _: 1
                    }, 8, ["label"]),
                    a(ve, {
                      label: e(t)("access.embeddingBaseUrl")
                    }, {
                      default: n(() => [
                        a(we, {
                          modelValue: ie.value,
                          "onUpdate:modelValue": o[2] || (o[2] = (N) => ie.value = N),
                          placeholder: "https://api.siliconflow.cn/v1 或 http://127.0.0.1:11434/v1"
                        }, null, 8, ["modelValue"])
                      ]),
                      _: 1
                    }, 8, ["label"]),
                    a(ve, {
                      label: e(t)("access.embeddingModelLabel")
                    }, {
                      default: n(() => [
                        a(we, {
                          modelValue: ue.value,
                          "onUpdate:modelValue": o[3] || (o[3] = (N) => ue.value = N),
                          placeholder: "BAAI/bge-m3 / bge-m3 / nomic-embed-text"
                        }, null, 8, ["modelValue"])
                      ]),
                      _: 1
                    }, 8, ["label"]),
                    a(ve, {
                      label: e(t)("access.embeddingApiKeyLabel")
                    }, {
                      default: n(() => [
                        a(we, {
                          modelValue: ne.value,
                          "onUpdate:modelValue": o[4] || (o[4] = (N) => ne.value = N),
                          "show-password": "",
                          placeholder: e(t)("access.embeddingApiKeyPlaceholder")
                        }, null, 8, ["modelValue", "placeholder"])
                      ]),
                      _: 1
                    }, 8, ["label"])
                  ]),
                  _: 1
                }),
                k("div", Nl, [
                  a(j, {
                    type: "primary",
                    loading: fe.value,
                    onClick: zt
                  }, {
                    default: n(() => [
                      u(s(e(t)("access.embeddingSaveTest")), 1)
                    ]),
                    _: 1
                  }, 8, ["loading"])
                ]),
                (T = B.value) != null && T.ok ? (f(), A(ce, {
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
                }, null, 8, ["title"])) : G("", !0)
              ];
            }),
            _: 1
          }),
          a(_e, { shadow: "never" }, {
            header: n(() => [
              u(s(e(t)("access.settingsTitle")), 1)
            ]),
            default: n(() => [
              a(Ge, {
                "label-position": "top",
                onSubmit: o[8] || (o[8] = Xe(() => {
                }, ["prevent"]))
              }, {
                default: n(() => [
                  a(ve, {
                    label: e(t)("access.instructionsLabel")
                  }, {
                    default: n(() => [
                      a(we, {
                        modelValue: v.value,
                        "onUpdate:modelValue": o[6] || (o[6] = (T) => v.value = T),
                        type: "textarea",
                        rows: 5,
                        placeholder: e(t)("access.instructionsPlaceholder")
                      }, null, 8, ["modelValue", "placeholder"]),
                      k("div", Fl, s(e(t)("access.instructionsHint")), 1),
                      v.value ? (f(), x("details", Hl, [
                        k("summary", null, s(e(t)("access.viewDefault")), 1),
                        k("pre", Kl, s(M.value), 1)
                      ])) : G("", !0)
                    ]),
                    _: 1
                  }, 8, ["label"]),
                  a(ve, {
                    label: e(t)("access.conventionsLabel")
                  }, {
                    default: n(() => [
                      a(we, {
                        modelValue: _.value,
                        "onUpdate:modelValue": o[7] || (o[7] = (T) => _.value = T),
                        type: "textarea",
                        rows: 4,
                        placeholder: e(t)("access.conventionsPlaceholder")
                      }, null, 8, ["modelValue", "placeholder"]),
                      k("div", ql, s(e(t)("access.conventionsHint")), 1)
                    ]),
                    _: 1
                  }, 8, ["label"])
                ]),
                _: 1
              }),
              k("div", jl, [
                a(j, {
                  type: "primary",
                  loading: V.value,
                  onClick: Bt
                }, {
                  default: n(() => [
                    u(s(e(t)("common.save")), 1)
                  ]),
                  _: 1
                }, 8, ["loading"])
              ])
            ]),
            _: 1
          }),
          a(_e, { shadow: "never" }, {
            header: n(() => [
              u(s(e(t)("access.backupCard")), 1)
            ]),
            default: n(() => [
              k("div", Jl, [
                a(j, {
                  icon: e(ka),
                  loading: e(I),
                  onClick: o[9] || (o[9] = (T) => p(e(O)))
                }, {
                  default: n(() => [
                    u(s(e(t)("access.export")), 1)
                  ]),
                  _: 1
                }, 8, ["icon", "loading"]),
                a(j, {
                  icon: e(wa),
                  loading: e(J),
                  onClick: o[10] || (o[10] = (T) => {
                    var N;
                    return (N = ke.value) == null ? void 0 : N.click();
                  })
                }, {
                  default: n(() => [
                    u(s(e(t)("access.import")), 1)
                  ]),
                  _: 1
                }, 8, ["icon", "loading"]),
                a(me, {
                  content: e(t)("access.importHint"),
                  placement: "top"
                }, {
                  default: n(() => [
                    a(L, { class: "am-info" }, {
                      default: n(() => [
                        a(e(Pe))
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
                  onChange: Lt
                }, null, 544)
              ])
            ]),
            _: 1
          }),
          a(_e, { shadow: "never" }, {
            header: n(() => [
              k("div", Wl, [
                k("span", null, s(e(t)("access.doctorCard")), 1),
                a(j, {
                  size: "small",
                  icon: e(st),
                  loading: e(U),
                  onClick: o[11] || (o[11] = (T) => p(e(ae)))
                }, {
                  default: n(() => [
                    u(s(e(t)("access.runDoctor")), 1)
                  ]),
                  _: 1
                }, 8, ["icon", "loading"])
              ])
            ]),
            default: n(() => [
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
                e(R).ok ? G("", !0) : (f(), x("ul", Gl, [
                  (f(!0), x(se, null, be(e(R).issues, (T, N) => (f(), x("li", { key: N }, s(T), 1))), 128))
                ]))
              ], 64)) : (f(), A(vt, {
                key: 1,
                description: e(t)("access.doctorEmpty"),
                "image-size": 60
              }, null, 8, ["description"]))
            ]),
            _: 1
          }),
          a(_e, { shadow: "never" }, {
            header: n(() => [
              k("div", Ql, [
                k("span", null, s(e(t)("access.embeddingCard")), 1),
                a(j, {
                  size: "small",
                  icon: e(Et),
                  loading: e(K),
                  onClick: S
                }, {
                  default: n(() => [
                    u(s(e(t)("access.runBackfill")), 1)
                  ]),
                  _: 1
                }, 8, ["icon", "loading"])
              ])
            ]),
            default: n(() => {
              var T, N;
              return [
                (N = (T = q.value) == null ? void 0 : T.embedding) != null && N.enabled ? (f(), x(se, { key: 0 }, [
                  a(qt, {
                    column: e(z) ? 1 : 2,
                    border: ""
                  }, {
                    default: n(() => [
                      a(gt, {
                        label: e(t)("access.embeddingModel")
                      }, {
                        default: n(() => [
                          u(s(q.value.embedding.model ?? "—"), 1)
                        ]),
                        _: 1
                      }, 8, ["label"]),
                      a(gt, {
                        label: e(t)("access.embeddingCoverage")
                      }, {
                        default: n(() => [
                          u(s(q.value.embedding.embedded ?? 0) + " / " + s(W.value), 1)
                        ]),
                        _: 1
                      }, 8, ["label"])
                    ]),
                    _: 1
                  }, 8, ["column"]),
                  (q.value.embedding.pending ?? 0) > 0 ? (f(), A(ce, {
                    key: 0,
                    title: e(t)("access.embeddingPending", { count: q.value.embedding.pending }),
                    type: "warning",
                    "show-icon": "",
                    closable: !1,
                    class: "settings-hint"
                  }, null, 8, ["title"])) : G("", !0)
                ], 64)) : (f(), A(ce, {
                  key: 1,
                  title: e(t)("access.embeddingDisabled"),
                  type: "info",
                  "show-icon": "",
                  closable: !1
                }, null, 8, ["title"]))
              ];
            }),
            _: 1
          })
        ], 64)) : G("", !0),
        a(bt, {
          modelValue: re.value,
          "onUpdate:modelValue": o[16] || (o[16] = (T) => re.value = T),
          title: de.value ? e(t)("access.editTitle", { name: de.value.name }) : e(t)("access.createTitle"),
          width: e(z) ? "96%" : "480px"
        }, {
          footer: n(() => [
            a(j, {
              onClick: o[15] || (o[15] = (T) => re.value = !1)
            }, {
              default: n(() => [
                u(s(e(t)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(j, {
              type: "primary",
              loading: ee.value,
              onClick: h
            }, {
              default: n(() => [
                u(s(e(t)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: n(() => [
            a(Ge, {
              "label-position": "top",
              onSubmit: o[14] || (o[14] = Xe(() => {
              }, ["prevent"]))
            }, {
              default: n(() => [
                de.value ? G("", !0) : (f(), A(ve, {
                  key: 0,
                  label: e(t)("access.nameLabel")
                }, {
                  default: n(() => [
                    a(we, {
                      modelValue: H.name,
                      "onUpdate:modelValue": o[12] || (o[12] = (T) => H.name = T),
                      placeholder: e(t)("access.namePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"])),
                a(ve, {
                  label: e(t)("access.permsLabel")
                }, {
                  default: n(() => [
                    k("div", Xl, [
                      k("span", Yl, s(e(t)("access.presets")), 1),
                      a(jt, {
                        modelValue: E.value,
                        "onUpdate:modelValue": o[13] || (o[13] = (T) => E.value = T),
                        size: "small",
                        onChange: Ee
                      }, {
                        default: n(() => [
                          a(Re, { value: "admin" }, {
                            default: n(() => [
                              u(s(e(t)("access.presetAdmin")), 1)
                            ]),
                            _: 1
                          }),
                          a(Re, { value: "member" }, {
                            default: n(() => [
                              u(s(e(t)("access.presetMember")), 1)
                            ]),
                            _: 1
                          }),
                          a(Re, { value: "viewer" }, {
                            default: n(() => [
                              u(s(e(t)("access.presetViewer")), 1)
                            ]),
                            _: 1
                          }),
                          a(Re, { value: "custom" }, {
                            default: n(() => [
                              u(s(e(t)("access.presetCustom")), 1)
                            ]),
                            _: 1
                          })
                        ]),
                        _: 1
                      }, 8, ["modelValue"])
                    ]),
                    k("div", Zl, [
                      (f(), x(se, null, be(y, (T) => a(Jt, {
                        key: T.key,
                        modelValue: H.caps[T.key],
                        "onUpdate:modelValue": (N) => H.caps[T.key] = N,
                        onChange: Ie
                      }, {
                        default: n(() => [
                          u(s(pt(T.key)), 1)
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
        a(bt, {
          modelValue: Ve.value,
          "onUpdate:modelValue": o[18] || (o[18] = (T) => Ve.value = T),
          title: dt.value,
          width: e(z) ? "96%" : "560px"
        }, {
          footer: n(() => [
            e(g).onIdentityToken ? (f(), A(j, {
              key: 0,
              type: "primary",
              onClick: Ot
            }, {
              default: n(() => [
                u(s(e(t)("access.saveToBrowser")), 1)
              ]),
              _: 1
            })) : G("", !0),
            a(j, {
              type: "primary",
              onClick: o[17] || (o[17] = () => {
                Ft(ze.value), Ve.value = !1;
              })
            }, {
              default: n(() => [
                u(s(e(t)("access.copyToken")), 1)
              ]),
              _: 1
            })
          ]),
          default: n(() => [
            k("p", null, s(e(t)("access.created")), 1),
            k("code", en, s(ze.value), 1)
          ]),
          _: 1
        }, 8, ["modelValue", "title", "width"])
      ], 512);
    };
  }
}), pn = /* @__PURE__ */ Se(tn, [["__scopeId", "data-v-9415ab6d"]]);
export {
  pn as AdminPanel,
  Dt as MarkdownView,
  sl as MemoriesPanel,
  mn as MemoryAdmin,
  Qa as MemoryDetailDrawer,
  Ka as MemoryEditorDialog,
  Mt as MemoryUIConfigKey,
  Tl as OpsPanel,
  pl as TagsPanel,
  Aa as applyMemoryUILocalePreference,
  xa as buildMemoriesQuery,
  Ua as createApiClient,
  cn as currentMemoryUILocale,
  Pa as formatSize,
  Te as formatTime,
  Ye as isSearchMode,
  je as memoryUIi18n,
  un as provideMemoryUI,
  Da as renderMarkdown,
  Ia as sanitizeHtml,
  Va as setMemoryUILocale,
  t,
  Y as toastError,
  X as toastSuccess,
  Pl as useAdmin,
  he as useApiClient,
  Oa as useMemories,
  it as useMemoryConfig,
  hl as useOps,
  il as useTags
};
