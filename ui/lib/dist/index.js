import { provide as Xe, inject as Ze, ref as w, computed as re, onMounted as de, watch as _e, onUnmounted as et, defineComponent as X, openBlock as b, createElementBlock as V, createBlock as L, unref as e, withCtx as l, createVNode as a, createElementVNode as E, toDisplayString as d, createTextVNode as v, Fragment as Y, renderList as te, createCommentVNode as J, withKeys as tt, isRef as ee, withDirectives as ae, reactive as at, renderSlot as ot, vShow as ge } from "vue";
import { createI18n as lt } from "vue-i18n";
import { ElDialog as Ee, ElForm as Ue, ElFormItem as De, ElInput as ye, ElRadioGroup as be, ElRadioButton as $e, ElSelect as xe, ElOption as ze, ElButton as ce, ElDrawer as nt, ElTag as he, ElDivider as st, ElTooltip as we, ElIcon as ue, ElAlert as Ve, ElTable as Le, ElTableColumn as Ie, ElLoadingDirective as Ne, ElPagination as it, ElRadio as rt, ElRow as dt, ElCol as ct, ElCard as ut, ElStatistic as mt, ElDescriptions as pt, ElDescriptionsItem as ft, ElEmpty as gt, ElContainer as vt, ElAside as _t, ElMenu as yt, ElMenuItem as bt, ElMain as ht, ElTabs as wt, ElTabPane as kt } from "element-plus/es";
import { InfoFilled as le, Plus as Ae, Search as Re, Refresh as Ct, Download as Tt, UploadFilled as St, Collection as Pt, Notebook as Mt, PriceTag as Et, Odometer as Ut } from "@element-plus/icons-vue";
import { ElMessage as R, ElMessageBox as Oe } from "element-plus";
import { Marked as Dt } from "marked";
import Be from "dompurify";
const $t = {
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
    statNextId: "下一个记忆 ID",
    dbCard: "数据库",
    path: "文件路径",
    lastUpdate: "最近更新",
    lastUpdateValue: "{id}（{time}）",
    crossPlatform: "跨平台迁移",
    crossPlatformYes: "支持",
    crossPlatformNote: "SQLite 文件格式平台无关，停服后可直接复制 .db 文件到其他机器；运行中请改用「导出备份」。",
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
    openMode: "开放模式：当前未配置任何身份，所有请求免鉴权（适合个人本地部署）。创建第一个身份后，所有 /mcp 与 /api 请求必须携带 Authorization: Bearer <token>。第一个身份建议设为管理员。",
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
    created: "身份已创建，token 可随时在列表中复制：",
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
    empty: "尚无身份。创建第一个身份以启用 token 鉴权。"
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
    statNextId: "Next memory ID",
    dbCard: "Database",
    path: "File path",
    lastUpdate: "Last update",
    lastUpdateValue: "{id} ({time})",
    crossPlatform: "Cross-platform migration",
    crossPlatformYes: "Supported",
    crossPlatformNote: 'The SQLite file format is platform-independent: copy the .db file to another machine while the server is stopped; use "Export backup" while running.',
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
    openMode: "Open mode: no identities configured, all requests are unauthenticated (fine for personal local deployments). Once the first identity exists, every /mcp and /api request must carry Authorization: Bearer <token>. Make the first identity an admin.",
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
    created: "Identity created; its token can be copied from the list anytime:",
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
    empty: "No identities yet. Create the first one to enable token auth."
  }
};
function Fe() {
  var t;
  return typeof navigator > "u" || (t = navigator.language) != null && t.toLowerCase().startsWith("zh") ? "zh" : "en";
}
const me = lt({
  legacy: !1,
  locale: Fe(),
  fallbackLocale: "zh",
  messages: { zh: $t, en: xt },
  // 面向宿主组件库，缺 key 时静默回退即可，不刷控制台
  missingWarn: !1,
  fallbackWarn: !1
}), { t: o } = me.global;
function zt(t) {
  me.global.locale.value = t;
}
function Vt(t) {
  zt(t === "auto" ? Fe() : t);
}
function Ja() {
  return me.global.locale.value;
}
const He = Symbol("memory-ui-config"), ie = {
  baseUrl: "",
  fetch: (...t) => globalThis.fetch(...t),
  defaultPageSize: 20,
  locale: "auto"
};
function Ka(t) {
  t.locale && Vt(t.locale), Xe(He, t);
}
function qe() {
  const t = Ze(He);
  return {
    baseUrl: ((t == null ? void 0 : t.baseUrl) ?? ie.baseUrl).replace(/\/+$/, ""),
    fetch: (t == null ? void 0 : t.fetch) ?? ie.fetch,
    defaultPageSize: (t == null ? void 0 : t.defaultPageSize) ?? ie.defaultPageSize,
    locale: (t == null ? void 0 : t.locale) ?? ie.locale
  };
}
function ne(t) {
  if (!t) return "—";
  const s = me.global.locale.value === "en" ? "en-US" : "zh-CN";
  return new Date(t * 1e3).toLocaleString(s, { hour12: !1 });
}
function Lt(t) {
  return t == null ? "—" : t < 1024 ? `${t} B` : t < 1024 * 1024 ? `${(t / 1024).toFixed(1)} KB` : `${(t / 1024 / 1024).toFixed(2)} MB`;
}
function It(t) {
  async function s(r, u = {}) {
    const g = await t.fetch(t.baseUrl + r, {
      headers: { "Content-Type": "application/json" },
      ...u
    }), h = await g.text();
    let i = null;
    try {
      i = h ? JSON.parse(h) : null;
    } catch {
      i = null;
    }
    if (!g.ok) {
      const C = (i == null ? void 0 : i.error) ?? o("errors.http", { status: g.status });
      throw new Error(C);
    }
    return i;
  }
  async function n(r) {
    var g;
    const u = await t.fetch(t.baseUrl + r);
    if (!u.ok) {
      const h = await u.text().catch(() => "");
      let i = o("errors.http", { status: u.status });
      try {
        i = ((g = JSON.parse(h)) == null ? void 0 : g.error) ?? i;
      } catch {
      }
      throw new Error(i);
    }
    return u.blob();
  }
  return {
    get: (r) => s(r),
    post: (r, u) => s(r, { method: "POST", body: JSON.stringify(u ?? {}) }),
    put: (r, u) => s(r, { method: "PUT", body: JSON.stringify(u ?? {}) }),
    del: (r) => s(r, { method: "DELETE" }),
    getBlob: n
  };
}
function se() {
  return It(qe());
}
function Nt(t) {
  const s = ve(t.query), n = new URLSearchParams();
  return s ? (n.set("query", t.query.trim()), t.tagFilter && n.set("tags", t.tagFilter)) : (t.tagFilter && n.set("tag", t.tagFilter), n.set("sort", t.sort), n.set("order", t.order)), n.set("offset", String((t.page - 1) * t.pageSize)), n.set("limit", String(t.pageSize)), n.toString();
}
function ve(t) {
  return t.trim().length > 0;
}
const At = new Dt();
function Rt(t) {
  const s = At.parse(t, { async: !1 });
  return Be.sanitize(s);
}
function Ot(t) {
  return Be.sanitize(t);
}
function ke(t, s) {
  return t.get(`/api/memories/${encodeURIComponent(s)}`);
}
function je(t, s) {
  return t.post("/api/memories", s);
}
function Je(t, s, n) {
  return t.put(`/api/memories/${encodeURIComponent(s)}`, n);
}
function Bt(t, s) {
  return t.del(`/api/memories/${encodeURIComponent(s)}`);
}
function Ke(t) {
  return t.get("/api/tags");
}
function Ft(t, s, n) {
  return t.post("/api/tags", { name: s, description: n });
}
function Ht(t, s, n, r) {
  const u = { description: r }, g = n.trim();
  return g && g !== s && (u.new_name = g), t.put(`/api/tags/${encodeURIComponent(s)}`, u);
}
function qt(t, s, n) {
  return t.del(`/api/tags/${encodeURIComponent(s)}?mode=${n}`);
}
function jt() {
  const t = se(), { defaultPageSize: s } = qe(), n = w(""), r = w(""), u = w("updated_at"), g = w("desc"), h = w(1), i = w(s), C = w([]), z = w([]), P = w(0), m = w(""), _ = w(!1), $ = w([]), D = re(() => ve(n.value));
  function T() {
    return h.value = 1, F();
  }
  function N() {
    return Nt({
      query: n.value,
      tagFilter: r.value,
      sort: u.value,
      order: g.value,
      page: h.value,
      pageSize: i.value
    });
  }
  let B = 0;
  async function F() {
    var M;
    const p = ++B;
    _.value = !0;
    try {
      const O = N();
      if (ve(n.value)) {
        const k = await t.get(`/api/memories?${O}`);
        if (p !== B) return;
        z.value = (k.results ?? []).map((I) => ({ ...I, snippet: Ot(I.snippet) })), P.value = k.total_matches ?? 0, m.value = "";
      } else {
        const k = await t.get(`/api/memories?${O}`);
        if (p !== B) return;
        const I = Math.max(1, Math.ceil(k.total / i.value));
        if (((M = k.memories) == null ? void 0 : M.length) === 0 && k.total > 0 && h.value > I)
          return h.value = I, _.value = !1, F();
        C.value = k.memories ?? [], P.value = k.total ?? 0, m.value = k.note ?? "";
      }
    } finally {
      p === B && (_.value = !1);
    }
  }
  async function A() {
    try {
      const p = await Ke(t);
      $.value = (p.tags ?? []).map((M) => M.name);
    } catch {
    }
  }
  async function U(p) {
    if (p.id) {
      const M = await ke(t, p.id), O = new Set(M.tags), k = new Set(p.tags);
      await Je(t, p.id, {
        summary: p.summary,
        content: p.content,
        add_tags: [...k].filter((I) => !O.has(I)),
        remove_tags: [...O].filter((I) => !k.has(I))
      });
    } else
      await je(t, { summary: p.summary, content: p.content, tags: p.tags });
    await Promise.all([F(), A()]);
  }
  async function y(p) {
    await Bt(t, p), await F();
  }
  return de(() => {
    F().catch(() => {
    }), A();
  }), {
    query: n,
    tagFilter: r,
    sort: u,
    order: g,
    page: h,
    pageSize: i,
    rows: C,
    searchResults: z,
    total: P,
    note: m,
    loading: _,
    tagOptions: $,
    searching: D,
    onSearch: T,
    reload: F,
    loadTagOptions: A,
    saveMemory: U,
    removeMemory: y
  };
}
function Ce(t, s = 720) {
  const n = w(0);
  let r = null;
  function u(g) {
    r == null || r.disconnect(), r = null, !(!g || typeof ResizeObserver > "u") && (r = new ResizeObserver((h) => {
      var i;
      n.value = ((i = h[0]) == null ? void 0 : i.contentRect.width) ?? 0;
    }), r.observe(g));
  }
  return de(() => u(t.value)), _e(t, (g) => u(g)), et(() => r == null ? void 0 : r.disconnect()), { width: n, compact: re(() => n.value > 0 && n.value < s) };
}
const Jt = ["innerHTML"], We = /* @__PURE__ */ X({
  __name: "MarkdownView",
  props: {
    source: {}
  },
  setup(t) {
    const s = t, n = re(() => Rt(s.source));
    return (r, u) => (b(), V("div", {
      class: "md-body",
      innerHTML: n.value
    }, null, 8, Jt));
  }
}), Kt = { class: "content-label" }, Wt = /* @__PURE__ */ X({
  __name: "MemoryEditorDialog",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    tagOptions: {},
    width: {}
  },
  emits: ["update:visible", "saved"],
  setup(t, { emit: s }) {
    const n = t, r = s, u = se(), g = w(!1), h = w("edit"), i = w({ id: null, summary: "", content: "", tags: [] });
    let C = [];
    _e(
      () => n.visible,
      async (P) => {
        if (P)
          if (h.value = "edit", n.memoryId)
            try {
              const m = await ke(u, n.memoryId);
              i.value = { id: m.id, summary: m.summary, content: m.content, tags: [...m.tags] }, C = [...m.tags];
            } catch (m) {
              R.error(m instanceof Error ? m.message : String(m)), r("update:visible", !1);
            }
          else
            i.value = { id: null, summary: "", content: "", tags: [] }, C = [];
      }
    );
    async function z() {
      g.value = !0;
      try {
        if (i.value.id) {
          const P = new Set(C), m = new Set(i.value.tags);
          await Je(u, i.value.id, {
            summary: i.value.summary,
            content: i.value.content,
            add_tags: [...m].filter((_) => !P.has(_)),
            remove_tags: [...P].filter((_) => !m.has(_))
          }), R.success(o("editor.updated"));
        } else
          await je(u, {
            summary: i.value.summary,
            content: i.value.content,
            tags: i.value.tags
          }), R.success(o("editor.created"));
        r("update:visible", !1), r("saved");
      } catch (P) {
        R.error(P instanceof Error ? P.message : String(P));
      } finally {
        g.value = !1;
      }
    }
    return (P, m) => {
      const _ = ye, $ = De, D = $e, T = be, N = ze, B = xe, F = Ue, A = ce, U = Ee;
      return b(), L(U, {
        "model-value": t.visible,
        title: i.value.id ? e(o)("editor.editTitle") : e(o)("editor.createTitle"),
        width: t.width,
        "onUpdate:modelValue": m[5] || (m[5] = (y) => r("update:visible", y))
      }, {
        footer: l(() => [
          a(A, {
            onClick: m[4] || (m[4] = (y) => r("update:visible", !1))
          }, {
            default: l(() => [
              v(d(e(o)("common.cancel")), 1)
            ]),
            _: 1
          }),
          a(A, {
            type: "primary",
            loading: g.value,
            onClick: z
          }, {
            default: l(() => [
              v(d(e(o)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: l(() => [
          a(F, { "label-position": "top" }, {
            default: l(() => [
              a($, {
                label: e(o)("editor.summaryLabel")
              }, {
                default: l(() => [
                  a(_, {
                    modelValue: i.value.summary,
                    "onUpdate:modelValue": m[0] || (m[0] = (y) => i.value.summary = y),
                    maxlength: "512",
                    "show-word-limit": "",
                    placeholder: e(o)("editor.summaryPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"]),
              a($, null, {
                label: l(() => [
                  E("div", Kt, [
                    E("span", null, d(e(o)("editor.contentLabel")), 1),
                    a(T, {
                      modelValue: h.value,
                      "onUpdate:modelValue": m[1] || (m[1] = (y) => h.value = y),
                      size: "small"
                    }, {
                      default: l(() => [
                        a(D, { value: "edit" }, {
                          default: l(() => [
                            v(d(e(o)("editor.tabEdit")), 1)
                          ]),
                          _: 1
                        }),
                        a(D, { value: "preview" }, {
                          default: l(() => [
                            v(d(e(o)("editor.tabPreview")), 1)
                          ]),
                          _: 1
                        })
                      ]),
                      _: 1
                    }, 8, ["modelValue"])
                  ])
                ]),
                default: l(() => [
                  h.value === "edit" ? (b(), L(_, {
                    key: 0,
                    modelValue: i.value.content,
                    "onUpdate:modelValue": m[2] || (m[2] = (y) => i.value.content = y),
                    type: "textarea",
                    rows: 12,
                    maxlength: "200000",
                    "show-word-limit": "",
                    placeholder: e(o)("editor.contentPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])) : (b(), L(We, {
                    key: 1,
                    class: "content-preview",
                    source: i.value.content
                  }, null, 8, ["source"]))
                ]),
                _: 1
              }),
              a($, {
                label: e(o)("editor.tagsLabel")
              }, {
                default: l(() => [
                  a(B, {
                    modelValue: i.value.tags,
                    "onUpdate:modelValue": m[3] || (m[3] = (y) => i.value.tags = y),
                    multiple: "",
                    filterable: "",
                    "allow-create": "",
                    "default-first-option": "",
                    placeholder: e(o)("editor.tagsPlaceholder"),
                    class: "tags-select"
                  }, {
                    default: l(() => [
                      (b(!0), V(Y, null, te(t.tagOptions, (y) => (b(), L(N, {
                        key: y,
                        label: y,
                        value: y
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
}), oe = (t, s) => {
  const n = t.__vccOpts || t;
  for (const [r, u] of s)
    n[r] = u;
  return n;
}, Qt = /* @__PURE__ */ oe(Wt, [["__scopeId", "data-v-602bada0"]]), Yt = { class: "detail-summary" }, Gt = { class: "detail-tags" }, Xt = { class: "detail-toolbar" }, Zt = {
  key: 1,
  class: "detail-content"
}, ea = /* @__PURE__ */ X({
  __name: "MemoryDetailDrawer",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    size: {}
  },
  emits: ["update:visible"],
  setup(t, { emit: s }) {
    const n = t, r = s, u = se(), g = w(null), h = w("rendered");
    return _e(
      () => [n.visible, n.memoryId],
      async ([i]) => {
        if (!(!i || !n.memoryId)) {
          h.value = "rendered";
          try {
            g.value = await ke(u, n.memoryId);
          } catch (C) {
            R.error(C instanceof Error ? C.message : String(C)), r("update:visible", !1);
          }
        }
      }
    ), (i, C) => {
      var D;
      const z = he, P = st, m = $e, _ = be, $ = nt;
      return b(), L($, {
        "model-value": t.visible,
        title: e(o)("drawer.title", { id: ((D = g.value) == null ? void 0 : D.id) ?? t.memoryId ?? "" }),
        size: t.size,
        "onUpdate:modelValue": C[1] || (C[1] = (T) => r("update:visible", T))
      }, {
        default: l(() => [
          g.value ? (b(), V(Y, { key: 0 }, [
            E("h3", Yt, d(g.value.summary), 1),
            E("div", Gt, [
              (b(!0), V(Y, null, te(g.value.tags, (T) => (b(), L(z, {
                key: T,
                size: "small",
                class: "am-tag"
              }, {
                default: l(() => [
                  v(d(T), 1)
                ]),
                _: 2
              }, 1024))), 128))
            ]),
            a(P),
            E("div", Xt, [
              a(_, {
                modelValue: h.value,
                "onUpdate:modelValue": C[0] || (C[0] = (T) => h.value = T),
                size: "small"
              }, {
                default: l(() => [
                  a(m, { value: "rendered" }, {
                    default: l(() => [
                      v(d(e(o)("drawer.rendered")), 1)
                    ]),
                    _: 1
                  }),
                  a(m, { value: "source" }, {
                    default: l(() => [
                      v(d(e(o)("drawer.source")), 1)
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["modelValue"])
            ]),
            h.value === "rendered" ? (b(), L(We, {
              key: 0,
              source: g.value.content
            }, null, 8, ["source"])) : (b(), V("pre", Zt, d(g.value.content), 1))
          ], 64)) : J("", !0)
        ]),
        _: 1
      }, 8, ["model-value", "title", "size"]);
    };
  }
}), ta = /* @__PURE__ */ oe(ea, [["__scopeId", "data-v-b634d116"]]), aa = {
  key: 0,
  class: "am-panel-header"
}, oa = { class: "am-heading" }, la = { class: "am-panel-title" }, na = { class: "am-toolbar" }, sa = { class: "am-summary" }, ia = ["innerHTML"], ra = {
  key: 4,
  class: "am-pager"
}, da = {
  key: 5,
  class: "am-pager"
}, ca = /* @__PURE__ */ X({
  __name: "MemoriesPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t) {
    const s = t, {
      query: n,
      tagFilter: r,
      sort: u,
      order: g,
      page: h,
      pageSize: i,
      rows: C,
      searchResults: z,
      total: P,
      note: m,
      loading: _,
      tagOptions: $,
      searching: D,
      onSearch: T,
      reload: N,
      loadTagOptions: B,
      removeMemory: F
    } = jt(), A = w(null), { compact: U } = Ce(A), y = w(!1), p = w(null), M = w(!1), O = w(null);
    function k(x) {
      return x().catch((c) => R.error(c instanceof Error ? c.message : String(c)));
    }
    function I() {
      p.value = null, y.value = !0;
    }
    function j(x) {
      p.value = x, y.value = !0;
    }
    function K(x) {
      O.value = x, M.value = !0;
    }
    function G() {
      k(N), B();
    }
    function W({ prop: x, order: c }) {
      c && (x === "updated_at" || x === "created_at") ? (u.value = x, c.value = c === "ascending" ? "asc" : "desc") : (u.value = "updated_at", c.value = "desc"), k(N);
    }
    async function Z(x) {
      try {
        await Oe.confirm(o("memories.deleteConfirm", { id: x.id }), o("memories.deleteTitle"), {
          type: "warning"
        });
      } catch {
        return;
      }
      try {
        await F(x.id), R.success(o("memories.deleted"));
      } catch (c) {
        R.error(c instanceof Error ? c.message : String(c));
      }
    }
    return (x, c) => {
      const pe = ue, fe = we, S = ce, Q = ye, Qe = ze, Ye = xe, Ge = Ve, H = Ie, Te = he, Se = Le, Pe = it, Me = Ne;
      return b(), V("div", {
        ref_key: "rootRef",
        ref: A,
        class: "am-panel"
      }, [
        t.showHeader ? (b(), V("div", aa, [
          E("div", oa, [
            E("h2", la, d(s.title ?? e(o)("memories.title")), 1),
            a(fe, {
              content: s.subtitle ?? e(o)("memories.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(pe, { class: "am-info" }, {
                  default: l(() => [
                    a(e(le))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(S, {
            type: "primary",
            icon: e(Ae),
            onClick: I
          }, {
            default: l(() => [
              v(d(e(o)("memories.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : J("", !0),
        E("div", na, [
          a(Q, {
            modelValue: e(n),
            "onUpdate:modelValue": c[1] || (c[1] = (f) => ee(n) ? n.value = f : null),
            placeholder: e(o)("memories.searchPlaceholder"),
            clearable: "",
            class: "search",
            onKeyup: c[2] || (c[2] = tt((f) => k(e(T)), ["enter"])),
            onClear: c[3] || (c[3] = (f) => k(e(T)))
          }, {
            append: l(() => [
              a(S, {
                icon: e(Re),
                onClick: c[0] || (c[0] = (f) => k(e(T)))
              }, null, 8, ["icon"])
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(Ye, {
            modelValue: e(r),
            "onUpdate:modelValue": c[4] || (c[4] = (f) => ee(r) ? r.value = f : null),
            placeholder: e(o)("memories.tagFilter"),
            clearable: "",
            filterable: "",
            class: "tag-filter",
            onChange: c[5] || (c[5] = (f) => k(e(T)))
          }, {
            default: l(() => [
              (b(!0), V(Y, null, te(e($), (f) => (b(), L(Qe, {
                key: f,
                label: f,
                value: f
              }, null, 8, ["label", "value"]))), 128))
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"])
        ]),
        e(m) ? (b(), L(Ge, {
          key: 1,
          title: e(m),
          type: "info",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : J("", !0),
        e(D) ? ae((b(), L(Se, {
          key: 2,
          data: e(z)
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
                E("div", sa, d(f.summary), 1),
                E("div", {
                  class: "am-snippet",
                  innerHTML: f.snippet
                }, null, 8, ia)
              ]),
              _: 1
            }, 8, ["label"]),
            a(H, {
              label: e(o)("memories.colTags"),
              width: "220"
            }, {
              default: l(({ row: f }) => [
                (b(!0), V(Y, null, te(f.tags, (q) => (b(), L(Te, {
                  key: q,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: l(() => [
                    v(d(q), 1)
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
                v(d(e(ne)(f.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(H, {
              label: e(o)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: l(({ row: f }) => [
                a(S, {
                  link: "",
                  type: "primary",
                  onClick: (q) => K(f.id)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(S, {
                  link: "",
                  type: "primary",
                  onClick: (q) => j(f.id)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(S, {
                  link: "",
                  type: "danger",
                  onClick: (q) => Z(f)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [Me, e(_)]
        ]) : ae((b(), L(Se, {
          key: 3,
          data: e(C),
          "default-sort": { prop: e(u), order: e(g) === "asc" ? "ascending" : "descending" },
          onSortChange: W
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
                (b(!0), V(Y, null, te(f.tags, (q) => (b(), L(Te, {
                  key: q,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: l(() => [
                    v(d(q), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            e(U) ? J("", !0) : (b(), L(H, {
              key: 0,
              prop: "created_at",
              label: e(o)("memories.colCreatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: l(({ row: f }) => [
                v(d(e(ne)(f.created_at)), 1)
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
                v(d(e(ne)(f.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(H, {
              label: e(o)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: l(({ row: f }) => [
                a(S, {
                  link: "",
                  type: "primary",
                  onClick: (q) => K(f.id)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(S, {
                  link: "",
                  type: "primary",
                  onClick: (q) => j(f.id)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(S, {
                  link: "",
                  type: "danger",
                  onClick: (q) => Z(f)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data", "default-sort"])), [
          [Me, e(_)]
        ]),
        e(D) ? (b(), V("div", da, [
          a(Pe, {
            "current-page": e(h),
            "onUpdate:currentPage": c[10] || (c[10] = (f) => ee(h) ? h.value = f : null),
            "page-size": e(i),
            "onUpdate:pageSize": c[11] || (c[11] = (f) => ee(i) ? i.value = f : null),
            total: e(P),
            "page-sizes": [10, 20, 50],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: c[12] || (c[12] = (f) => k(e(N))),
            onSizeChange: c[13] || (c[13] = (f) => k(e(N)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])) : (b(), V("div", ra, [
          a(Pe, {
            "current-page": e(h),
            "onUpdate:currentPage": c[6] || (c[6] = (f) => ee(h) ? h.value = f : null),
            "page-size": e(i),
            "onUpdate:pageSize": c[7] || (c[7] = (f) => ee(i) ? i.value = f : null),
            total: e(P),
            "page-sizes": [20, 50, 100, 200],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: c[8] || (c[8] = (f) => k(e(N))),
            onSizeChange: c[9] || (c[9] = (f) => k(e(N)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])),
        a(Qt, {
          visible: y.value,
          "onUpdate:visible": c[14] || (c[14] = (f) => y.value = f),
          "memory-id": p.value,
          "tag-options": e($),
          width: e(U) ? "96%" : "640px",
          onSaved: G
        }, null, 8, ["visible", "memory-id", "tag-options", "width"]),
        a(ta, {
          visible: M.value,
          "onUpdate:visible": c[15] || (c[15] = (f) => M.value = f),
          "memory-id": O.value,
          size: e(U) ? "100%" : "45%"
        }, null, 8, ["visible", "memory-id", "size"])
      ], 512);
    };
  }
}), ua = /* @__PURE__ */ oe(ca, [["__scopeId", "data-v-2d2fb0ab"]]);
function ma() {
  const t = se(), s = w([]), n = w(!1);
  async function r() {
    n.value = !0;
    try {
      const i = await Ke(t);
      s.value = i.tags ?? [];
    } finally {
      n.value = !1;
    }
  }
  async function u(i, C) {
    await Ft(t, i, C), await r();
  }
  async function g(i, C, z) {
    await Ht(t, i, C, z), await r();
  }
  async function h(i, C) {
    await qt(t, i, C), await r();
  }
  return de(() => {
    r().catch(() => {
    });
  }), { rows: s, loading: n, reload: r, create: u, rename: g, remove: h };
}
const pa = {
  key: 0,
  class: "am-panel-header"
}, fa = { class: "am-heading" }, ga = { class: "am-panel-title" }, va = { class: "delete-body" }, _a = /* @__PURE__ */ X({
  __name: "TagsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t) {
    const s = t, { rows: n, loading: r, reload: u, create: g, rename: h, remove: i } = ma(), C = w(null), { compact: z } = Ce(C), P = w(!1), m = w(!1), _ = at({ oldName: null, name: "", newName: "", description: "" }), $ = w(!1), D = w("detach"), T = w(null);
    function N() {
      Object.assign(_, { oldName: null, name: "", newName: "", description: "" }), m.value = !0;
    }
    function B(y) {
      Object.assign(_, { oldName: y.name, name: y.name, newName: "", description: y.description ?? "" }), m.value = !0;
    }
    async function F() {
      P.value = !0;
      try {
        _.oldName ? (await h(_.oldName, _.newName, _.description), R.success(o("tags.saved"))) : (await g(_.name, _.description), R.success(o("tags.created"))), m.value = !1;
      } catch (y) {
        R.error(y instanceof Error ? y.message : String(y));
      } finally {
        P.value = !1;
      }
    }
    function A(y) {
      T.value = y, D.value = "detach", $.value = !0;
    }
    async function U() {
      var y, p;
      if (D.value === "purge")
        try {
          await Oe.confirm(
            o("tags.purgeConfirm", { name: (y = T.value) == null ? void 0 : y.name, count: ((p = T.value) == null ? void 0 : p.memory_count) ?? 0 }),
            o("tags.purgeConfirmTitle"),
            { type: "error", confirmButtonText: o("tags.purgeButton") }
          );
        } catch {
          return;
        }
      if (T.value) {
        P.value = !0;
        try {
          await i(T.value.name, D.value), R.success(o("tags.deleted")), $.value = !1;
        } catch (M) {
          R.error(M instanceof Error ? M.message : String(M));
        } finally {
          P.value = !1;
        }
      }
    }
    return (y, p) => {
      const M = ue, O = we, k = ce, I = he, j = Ie, K = Le, G = ye, W = De, Z = Ue, x = Ee, c = rt, pe = be, fe = Ne;
      return b(), V("div", {
        ref_key: "rootRef",
        ref: C,
        class: "am-panel"
      }, [
        t.showHeader ? (b(), V("div", pa, [
          E("div", fa, [
            E("h2", ga, d(s.title ?? e(o)("tags.title")), 1),
            a(O, {
              content: s.subtitle ?? e(o)("tags.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(M, { class: "am-info" }, {
                  default: l(() => [
                    a(e(le))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(k, {
            type: "primary",
            icon: e(Ae),
            onClick: N
          }, {
            default: l(() => [
              v(d(e(o)("tags.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : J("", !0),
        ae((b(), L(K, { data: e(n) }, {
          default: l(() => [
            a(j, {
              prop: "name",
              label: e(o)("tags.colName"),
              "min-width": "160"
            }, {
              default: l(({ row: S }) => [
                a(I, null, {
                  default: l(() => [
                    v(d(S.name), 1)
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
                v(d(S.description || "—"), 1)
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
                v(d(e(ne)(S.last_used_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(j, {
              label: e(o)("memories.colActions"),
              width: "150",
              fixed: "right"
            }, {
              default: l(({ row: S }) => [
                a(k, {
                  link: "",
                  type: "primary",
                  onClick: (Q) => B(S)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(k, {
                  link: "",
                  type: "danger",
                  onClick: (Q) => A(S)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.delete")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            }, 8, ["label"])
          ]),
          _: 1
        }, 8, ["data"])), [
          [fe, e(r)]
        ]),
        a(x, {
          modelValue: m.value,
          "onUpdate:modelValue": p[4] || (p[4] = (S) => m.value = S),
          title: _.oldName ? e(o)("tags.editTitle") : e(o)("tags.createTitle"),
          width: e(z) ? "96%" : "480px"
        }, {
          footer: l(() => [
            a(k, {
              onClick: p[3] || (p[3] = (S) => m.value = !1)
            }, {
              default: l(() => [
                v(d(e(o)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(k, {
              type: "primary",
              loading: P.value,
              onClick: F
            }, {
              default: l(() => [
                v(d(e(o)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: l(() => [
            a(Z, { "label-position": "top" }, {
              default: l(() => [
                a(W, {
                  label: e(o)("tags.nameLabel")
                }, {
                  default: l(() => [
                    a(G, {
                      modelValue: _.name,
                      "onUpdate:modelValue": p[0] || (p[0] = (S) => _.name = S),
                      disabled: !!_.oldName,
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(o)("tags.namePlaceholder")
                    }, null, 8, ["modelValue", "disabled", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                _.oldName ? (b(), L(W, {
                  key: 0,
                  label: e(o)("tags.renameLabel")
                }, {
                  default: l(() => [
                    a(G, {
                      modelValue: _.newName,
                      "onUpdate:modelValue": p[1] || (p[1] = (S) => _.newName = S),
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(o)("tags.renamePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"])) : J("", !0),
                a(W, {
                  label: e(o)("tags.descLabel")
                }, {
                  default: l(() => [
                    a(G, {
                      modelValue: _.description,
                      "onUpdate:modelValue": p[2] || (p[2] = (S) => _.description = S),
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
        a(x, {
          modelValue: $.value,
          "onUpdate:modelValue": p[7] || (p[7] = (S) => $.value = S),
          title: e(o)("tags.deleteTitle"),
          width: e(z) ? "96%" : "480px"
        }, {
          footer: l(() => [
            a(k, {
              onClick: p[6] || (p[6] = (S) => $.value = !1)
            }, {
              default: l(() => [
                v(d(e(o)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(k, {
              type: "danger",
              loading: P.value,
              onClick: U
            }, {
              default: l(() => [
                v(d(e(o)("common.delete")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: l(() => {
            var S;
            return [
              E("p", va, [
                v(d(e(o)("tags.deleteBefore")) + " ", 1),
                a(I, null, {
                  default: l(() => {
                    var Q;
                    return [
                      v(d((Q = T.value) == null ? void 0 : Q.name), 1)
                    ];
                  }),
                  _: 1
                }),
                v(" " + d(e(o)("tags.deleteMiddle")) + " ", 1),
                E("b", null, d((S = T.value) == null ? void 0 : S.memory_count), 1),
                v(" " + d(e(o)("tags.deleteAfter")), 1)
              ]),
              a(pe, {
                modelValue: D.value,
                "onUpdate:modelValue": p[5] || (p[5] = (Q) => D.value = Q)
              }, {
                default: l(() => [
                  a(c, { value: "detach" }, {
                    default: l(() => [
                      v(d(e(o)("tags.detach")), 1)
                    ]),
                    _: 1
                  }),
                  a(c, { value: "purge" }, {
                    default: l(() => [
                      v(d(e(o)("tags.purge")), 1)
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
}), ya = /* @__PURE__ */ oe(_a, [["__scopeId", "data-v-db74fa31"]]);
function ba(t) {
  return t.get("/api/stats");
}
function ha(t) {
  return t.get("/health");
}
function wa(t) {
  return t.get("/api/doctor");
}
function ka(t) {
  return t.getBlob("/api/export");
}
function Ca(t, s) {
  return t.post("/api/import", s);
}
function Ta() {
  const t = se(), s = w({}), n = w(""), r = w({ ok: !0, issues: [] }), u = w(!1), g = w(!1), h = w(!1), i = w(!1), C = re(() => Lt(s.value.file_size));
  async function z() {
    s.value = await ba(t), s.value.version = n.value;
  }
  async function P() {
    g.value = !0;
    try {
      r.value = await wa(t), u.value = !0;
    } finally {
      g.value = !1;
    }
  }
  async function m() {
    h.value = !0;
    try {
      const $ = await ka(t), D = URL.createObjectURL($), T = document.createElement("a");
      T.href = D, T.download = "agent-memory-export.json", T.click(), URL.revokeObjectURL(D);
    } finally {
      h.value = !1;
    }
  }
  async function _($) {
    i.value = !0;
    try {
      const D = await $.text();
      let T;
      try {
        T = JSON.parse(D);
      } catch {
        throw new Error(o("errors.invalidBackup"));
      }
      const N = await Ca(t, T);
      return await z(), N;
    } finally {
      i.value = !1;
    }
  }
  return de(async () => {
    try {
      n.value = (await ha(t)).version ?? "";
    } catch {
    }
    await z().catch(() => {
    });
  }), {
    stats: s,
    version: n,
    doctor: r,
    doctorRan: u,
    doctorLoading: g,
    exporting: h,
    importing: i,
    sizeText: C,
    reload: z,
    runDoctor: P,
    exportData: m,
    importFile: _
  };
}
const Sa = {
  key: 0,
  class: "am-panel-header"
}, Pa = { class: "am-heading" }, Ma = { class: "am-panel-title" }, Ea = { class: "label-help" }, Ua = { class: "actions" }, Da = { class: "card-header" }, $a = {
  key: 2,
  class: "issues"
}, xa = /* @__PURE__ */ X({
  __name: "OpsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t) {
    const s = t, {
      stats: n,
      version: r,
      doctor: u,
      doctorRan: g,
      doctorLoading: h,
      exporting: i,
      importing: C,
      sizeText: z,
      reload: P,
      runDoctor: m,
      exportData: _,
      importFile: $
    } = Ta(), D = w(null), { compact: T } = Ce(D), N = w(null);
    function B(A) {
      return A().catch((U) => R.error(U instanceof Error ? U.message : String(U)));
    }
    function F(A) {
      var p;
      const U = A.target, y = (p = U.files) == null ? void 0 : p[0];
      U.value = "", y && $(y).then((M) => {
        R.success(o("ops.imported", { memories: M.imported_memories, tags: M.imported_tags }));
      }).catch((M) => {
        R.error(M instanceof Error ? M.message : String(M));
      });
    }
    return (A, U) => {
      const y = ue, p = we, M = ce, O = mt, k = ut, I = ct, j = dt, K = ft, G = pt, W = Ve, Z = gt;
      return b(), V("div", {
        ref_key: "rootRef",
        ref: D,
        class: "am-panel"
      }, [
        t.showHeader ? (b(), V("div", Sa, [
          E("div", Pa, [
            E("h2", Ma, d(s.title ?? e(o)("ops.title")), 1),
            a(p, {
              content: s.subtitle ?? e(o)("ops.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(y, { class: "am-info" }, {
                  default: l(() => [
                    a(e(le))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(M, {
            icon: e(Ct),
            onClick: U[0] || (U[0] = (x) => B(e(P)))
          }, {
            default: l(() => [
              v(d(e(o)("ops.refresh")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : J("", !0),
        a(j, { gutter: 14 }, {
          default: l(() => [
            a(I, {
              span: e(T) ? 12 : 6
            }, {
              default: l(() => [
                a(k, { shadow: "never" }, {
                  default: l(() => [
                    a(O, {
                      title: e(o)("ops.statMemories"),
                      value: e(n).memories ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(I, {
              span: e(T) ? 12 : 6
            }, {
              default: l(() => [
                a(k, { shadow: "never" }, {
                  default: l(() => [
                    a(O, {
                      title: e(o)("ops.statTags"),
                      value: e(n).tags ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(I, {
              span: e(T) ? 12 : 6
            }, {
              default: l(() => [
                a(k, { shadow: "never" }, {
                  default: l(() => [
                    a(O, {
                      title: e(o)("ops.statSize"),
                      value: e(z)
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(I, {
              span: e(T) ? 12 : 6
            }, {
              default: l(() => [
                a(k, { shadow: "never" }, {
                  default: l(() => [
                    a(O, {
                      title: e(o)("ops.statNextId"),
                      value: e(n).next_id ?? "—"
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
        a(k, { shadow: "never" }, {
          header: l(() => [
            v(d(e(o)("ops.dbCard")), 1)
          ]),
          default: l(() => [
            a(G, {
              column: e(T) ? 1 : 2,
              border: ""
            }, {
              default: l(() => [
                a(K, {
                  label: e(o)("ops.path")
                }, {
                  default: l(() => [
                    v(d(e(n).path ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(K, {
                  label: e(o)("ops.lastUpdate")
                }, {
                  default: l(() => [
                    v(d(e(n).newest_update ? e(o)("ops.lastUpdateValue", {
                      id: e(n).newest_update.id,
                      time: e(ne)(e(n).newest_update.updated_at)
                    }) : "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(K, null, {
                  label: l(() => [
                    E("span", Ea, [
                      v(d(e(o)("ops.crossPlatform")) + " ", 1),
                      a(p, {
                        content: e(o)("ops.crossPlatformNote"),
                        placement: "top"
                      }, {
                        default: l(() => [
                          a(y, { class: "am-info" }, {
                            default: l(() => [
                              a(e(le))
                            ]),
                            _: 1
                          })
                        ]),
                        _: 1
                      }, 8, ["content"])
                    ])
                  ]),
                  default: l(() => [
                    v(" " + d(e(o)("ops.crossPlatformYes")), 1)
                  ]),
                  _: 1
                }),
                a(K, {
                  label: e(o)("ops.version")
                }, {
                  default: l(() => [
                    v(d(e(n).version ?? e(r)), 1)
                  ]),
                  _: 1
                }, 8, ["label"])
              ]),
              _: 1
            }, 8, ["column"]),
            E("div", Ua, [
              a(M, {
                icon: e(Tt),
                loading: e(i),
                onClick: U[1] || (U[1] = (x) => B(e(_)))
              }, {
                default: l(() => [
                  v(d(e(o)("ops.export")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(M, {
                icon: e(St),
                loading: e(C),
                onClick: U[2] || (U[2] = (x) => {
                  var c;
                  return (c = N.value) == null ? void 0 : c.click();
                })
              }, {
                default: l(() => [
                  v(d(e(o)("ops.import")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(p, {
                content: e(o)("ops.importHint"),
                placement: "top"
              }, {
                default: l(() => [
                  a(y, { class: "am-info" }, {
                    default: l(() => [
                      a(e(le))
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["content"]),
              E("input", {
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
        a(k, { shadow: "never" }, {
          header: l(() => [
            E("div", Da, [
              E("span", null, d(e(o)("ops.doctorCard")), 1),
              a(M, {
                size: "small",
                icon: e(Re),
                loading: e(h),
                onClick: U[3] || (U[3] = (x) => B(e(m)))
              }, {
                default: l(() => [
                  v(d(e(o)("ops.runDoctor")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"])
            ])
          ]),
          default: l(() => [
            e(g) ? (b(), V(Y, { key: 0 }, [
              e(u).ok ? (b(), L(W, {
                key: 0,
                title: e(o)("ops.doctorOk"),
                type: "success",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])) : (b(), L(W, {
                key: 1,
                title: e(o)("ops.doctorFail", { count: e(u).issues.length }),
                type: "error",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])),
              e(u).ok ? J("", !0) : (b(), V("ul", $a, [
                (b(!0), V(Y, null, te(e(u).issues, (x, c) => (b(), V("li", { key: c }, d(x), 1))), 128))
              ]))
            ], 64)) : (b(), L(Z, {
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
}), za = /* @__PURE__ */ oe(xa, [["__scopeId", "data-v-a8956eae"]]), Va = { class: "memory-ui" }, La = { class: "brand" }, Ia = { class: "brand-mark" }, Na = { class: "aside-footer" }, Aa = /* @__PURE__ */ X({
  __name: "MemoryAdmin",
  props: {
    layout: { default: "sidebar" },
    title: { default: "agent-memory" }
  },
  setup(t) {
    const s = w("memories");
    return (n, r) => {
      const u = ue, g = bt, h = yt, i = _t, C = kt, z = wt, P = ht, m = vt;
      return b(), V("div", Va, [
        a(m, { class: "layout" }, {
          default: l(() => [
            t.layout === "sidebar" ? (b(), L(i, {
              key: 0,
              width: "200px",
              class: "aside"
            }, {
              default: l(() => [
                E("div", La, [
                  E("span", Ia, [
                    a(u, { size: 16 }, {
                      default: l(() => [
                        a(e(Pt))
                      ]),
                      _: 1
                    })
                  ]),
                  E("span", null, d(t.title), 1)
                ]),
                a(h, {
                  "default-active": s.value,
                  class: "menu",
                  onSelect: r[0] || (r[0] = (_) => s.value = _)
                }, {
                  default: l(() => [
                    a(g, { index: "memories" }, {
                      default: l(() => [
                        a(u, null, {
                          default: l(() => [
                            a(e(Mt))
                          ]),
                          _: 1
                        }),
                        E("span", null, d(e(o)("nav.memories")), 1)
                      ]),
                      _: 1
                    }),
                    a(g, { index: "tags" }, {
                      default: l(() => [
                        a(u, null, {
                          default: l(() => [
                            a(e(Et))
                          ]),
                          _: 1
                        }),
                        E("span", null, d(e(o)("nav.tags")), 1)
                      ]),
                      _: 1
                    }),
                    a(g, { index: "ops" }, {
                      default: l(() => [
                        a(u, null, {
                          default: l(() => [
                            a(e(Ut))
                          ]),
                          _: 1
                        }),
                        E("span", null, d(e(o)("nav.ops")), 1)
                      ]),
                      _: 1
                    })
                  ]),
                  _: 1
                }, 8, ["default-active"]),
                E("div", Na, [
                  ot(n.$slots, "footer", {}, void 0, !0)
                ])
              ]),
              _: 3
            })) : J("", !0),
            a(P, { class: "main" }, {
              default: l(() => [
                t.layout === "tabs" ? (b(), L(z, {
                  key: 0,
                  modelValue: s.value,
                  "onUpdate:modelValue": r[1] || (r[1] = (_) => s.value = _),
                  class: "tabs-bar"
                }, {
                  default: l(() => [
                    a(C, {
                      label: e(o)("nav.memories"),
                      name: "memories"
                    }, null, 8, ["label"]),
                    a(C, {
                      label: e(o)("nav.tags"),
                      name: "tags"
                    }, null, 8, ["label"]),
                    a(C, {
                      label: e(o)("nav.ops"),
                      name: "ops"
                    }, null, 8, ["label"])
                  ]),
                  _: 1
                }, 8, ["modelValue"])) : J("", !0),
                ae(a(ua, null, null, 512), [
                  [ge, s.value === "memories"]
                ]),
                ae(a(ya, null, null, 512), [
                  [ge, s.value === "tags"]
                ]),
                ae(a(za, null, null, 512), [
                  [ge, s.value === "ops"]
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
}), Wa = /* @__PURE__ */ oe(Aa, [["__scopeId", "data-v-6f56cb0b"]]);
export {
  We as MarkdownView,
  ua as MemoriesPanel,
  Wa as MemoryAdmin,
  ta as MemoryDetailDrawer,
  Qt as MemoryEditorDialog,
  He as MemoryUIConfigKey,
  za as OpsPanel,
  ya as TagsPanel,
  Vt as applyMemoryUILocalePreference,
  Nt as buildMemoriesQuery,
  It as createApiClient,
  Ja as currentMemoryUILocale,
  Lt as formatSize,
  ne as formatTime,
  ve as isSearchMode,
  me as memoryUIi18n,
  Ka as provideMemoryUI,
  Rt as renderMarkdown,
  Ot as sanitizeHtml,
  zt as setMemoryUILocale,
  o as t,
  se as useApiClient,
  jt as useMemories,
  qe as useMemoryConfig,
  Ta as useOps,
  ma as useTags
};
