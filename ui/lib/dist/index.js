import { provide as Ze, inject as et, ref as y, computed as ce, onMounted as ue, watch as ye, onUnmounted as tt, defineComponent as Z, openBlock as _, createElementBlock as L, createBlock as I, unref as e, withCtx as l, createVNode as a, createElementVNode as M, toDisplayString as d, createTextVNode as v, Fragment as G, renderList as oe, createCommentVNode as J, withKeys as at, isRef as ae, withDirectives as le, reactive as ot, renderSlot as lt, vShow as ve } from "vue";
import { createI18n as nt } from "vue-i18n";
import { ElDialog as xe, ElForm as Ue, ElFormItem as De, ElInput as be, ElRadioGroup as he, ElRadioButton as $e, ElSelect as ze, ElOption as Ve, ElButton as me, ElDrawer as st, ElTag as we, ElDivider as it, ElTooltip as ke, ElIcon as pe, ElAlert as Le, ElTable as Ie, ElTableColumn as Ne, ElLoadingDirective as Ae, ElPagination as rt, ElRadio as dt, ElRow as ct, ElCol as ut, ElCard as mt, ElStatistic as pt, ElDescriptions as ft, ElDescriptionsItem as gt, ElEmpty as vt, ElContainer as _t, ElAside as yt, ElMenu as bt, ElMenuItem as ht, ElMain as wt, ElTabs as kt, ElTabPane as Ct } from "element-plus/es";
import { InfoFilled as se, Plus as Re, Search as Oe, Refresh as Tt, Download as St, UploadFilled as Pt, Collection as Et, Notebook as Mt, PriceTag as xt, Odometer as Ut } from "@element-plus/icons-vue";
import { ElMessage as O, ElMessageBox as Be } from "element-plus";
import { Marked as Dt } from "marked";
import Fe from "dompurify";
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
function He() {
  var t;
  return typeof navigator > "u" || (t = navigator.language) != null && t.toLowerCase().startsWith("zh") ? "zh" : "en";
}
const fe = nt({
  legacy: !1,
  locale: He(),
  fallbackLocale: "zh",
  messages: { zh: $t, en: zt },
  // 面向宿主组件库，缺 key 时静默回退即可，不刷控制台
  missingWarn: !1,
  fallbackWarn: !1
}), { t: o } = fe.global;
function Vt(t) {
  fe.global.locale.value = t;
}
function Lt(t) {
  Vt(t === "auto" ? He() : t);
}
function Ka() {
  return fe.global.locale.value;
}
const qe = Symbol("memory-ui-config"), de = {
  baseUrl: "",
  fetch: (...t) => globalThis.fetch(...t),
  defaultPageSize: 20,
  locale: "auto"
};
function Wa(t) {
  t.locale && Lt(t.locale), Ze(qe, t);
}
function je() {
  const t = et(qe);
  return {
    baseUrl: ((t == null ? void 0 : t.baseUrl) ?? de.baseUrl).replace(/\/+$/, ""),
    fetch: (t == null ? void 0 : t.fetch) ?? de.fetch,
    defaultPageSize: (t == null ? void 0 : t.defaultPageSize) ?? de.defaultPageSize,
    locale: (t == null ? void 0 : t.locale) ?? de.locale
  };
}
function ie(t) {
  if (!t) return "—";
  const i = fe.global.locale.value === "en" ? "en-US" : "zh-CN";
  return new Date(t * 1e3).toLocaleString(i, { hour12: !1 });
}
function It(t) {
  return t == null ? "—" : t < 1024 ? `${t} B` : t < 1024 * 1024 ? `${(t / 1024).toFixed(1)} KB` : `${(t / 1024 / 1024).toFixed(2)} MB`;
}
function Nt(t) {
  async function i(n, g = {}) {
    const u = await t.fetch(t.baseUrl + n, {
      headers: { "Content-Type": "application/json" },
      ...g
    }), h = await u.text();
    let s = null;
    try {
      s = h ? JSON.parse(h) : null;
    } catch {
      s = null;
    }
    if (!u.ok) {
      const b = (s == null ? void 0 : s.error) ?? o("errors.http", { status: u.status });
      throw new Error(b);
    }
    return s;
  }
  async function r(n) {
    var u;
    const g = await t.fetch(t.baseUrl + n);
    if (!g.ok) {
      const h = await g.text().catch(() => "");
      let s = o("errors.http", { status: g.status });
      try {
        s = ((u = JSON.parse(h)) == null ? void 0 : u.error) ?? s;
      } catch {
      }
      throw new Error(s);
    }
    return g.blob();
  }
  return {
    get: (n) => i(n),
    post: (n, g) => i(n, { method: "POST", body: JSON.stringify(g ?? {}) }),
    put: (n, g) => i(n, { method: "PUT", body: JSON.stringify(g ?? {}) }),
    del: (n) => i(n, { method: "DELETE" }),
    getBlob: r
  };
}
function re() {
  return Nt(je());
}
function At(t) {
  const i = _e(t.query), r = new URLSearchParams();
  return i ? (r.set("query", t.query.trim()), t.tagFilter && r.set("tags", t.tagFilter)) : (t.tagFilter && r.set("tag", t.tagFilter), r.set("sort", t.sort), r.set("order", t.order)), r.set("offset", String((t.page - 1) * t.pageSize)), r.set("limit", String(t.pageSize)), r.toString();
}
function _e(t) {
  return t.trim().length > 0;
}
const Rt = new Dt();
function Ot(t) {
  const i = Rt.parse(t, { async: !1 });
  return Fe.sanitize(i);
}
function Bt(t) {
  return Fe.sanitize(t);
}
function Ce(t, i) {
  return t.get(`/api/memories/${encodeURIComponent(i)}`);
}
function Je(t, i) {
  return t.post("/api/memories", i);
}
function Ke(t, i, r) {
  return t.put(`/api/memories/${encodeURIComponent(i)}`, r);
}
function Ft(t, i) {
  return t.del(`/api/memories/${encodeURIComponent(i)}`);
}
function We(t) {
  return t.get("/api/tags");
}
function Ht(t, i, r) {
  return t.post("/api/tags", { name: i, description: r });
}
function qt(t, i, r, n) {
  const g = { description: n }, u = r.trim();
  return u && u !== i && (g.new_name = u), t.put(`/api/tags/${encodeURIComponent(i)}`, g);
}
function jt(t, i, r) {
  return t.del(`/api/tags/${encodeURIComponent(i)}?mode=${r}`);
}
function Jt() {
  const t = re(), { defaultPageSize: i } = je(), r = y(""), n = y(""), g = y("updated_at"), u = y("desc"), h = y(1), s = y(i), b = y([]), V = y([]), U = y(0), m = y(""), P = y(!1), T = y([]), $ = ce(() => _e(r.value));
  function E() {
    return h.value = 1, R();
  }
  function D() {
    return At({
      query: r.value,
      tagFilter: n.value,
      sort: g.value,
      order: u.value,
      page: h.value,
      pageSize: s.value
    });
  }
  let A = 0;
  async function R() {
    var k;
    const f = ++A;
    P.value = !0;
    try {
      const x = D();
      if (_e(r.value)) {
        const z = await t.get(`/api/memories?${x}`);
        if (f !== A) return;
        V.value = (z.results ?? []).map((C) => ({ ...C, snippet: Bt(C.snippet) })), U.value = z.total_matches ?? 0, m.value = "";
      } else {
        const z = await t.get(`/api/memories?${x}`);
        if (f !== A) return;
        const C = Math.max(1, Math.ceil(z.total / s.value));
        if (((k = z.memories) == null ? void 0 : k.length) === 0 && z.total > 0 && h.value > C)
          return h.value = C, P.value = !1, R();
        b.value = z.memories ?? [], U.value = z.total ?? 0, m.value = z.note ?? "";
      }
    } finally {
      f === A && (P.value = !1);
    }
  }
  async function F() {
    try {
      const f = await We(t);
      T.value = (f.tags ?? []).map((k) => k.name);
    } catch {
    }
  }
  async function B(f) {
    if (f.id) {
      const k = await Ce(t, f.id), x = new Set(k.tags), z = new Set(f.tags);
      await Ke(t, f.id, {
        summary: f.summary,
        content: f.content,
        add_tags: [...z].filter((C) => !x.has(C)),
        remove_tags: [...x].filter((C) => !z.has(C))
      });
    } else
      await Je(t, { summary: f.summary, content: f.content, tags: f.tags });
    await Promise.all([R(), F()]);
  }
  async function w(f) {
    await Ft(t, f), await R();
  }
  return ue(() => {
    R().catch(() => {
    }), F();
  }), {
    query: r,
    tagFilter: n,
    sort: g,
    order: u,
    page: h,
    pageSize: s,
    rows: b,
    searchResults: V,
    total: U,
    note: m,
    loading: P,
    tagOptions: T,
    searching: $,
    onSearch: E,
    reload: R,
    loadTagOptions: F,
    saveMemory: B,
    removeMemory: w
  };
}
function Te(t, i = 720) {
  const r = y(0);
  let n = null;
  function g(u) {
    n == null || n.disconnect(), n = null, !(!u || typeof ResizeObserver > "u") && (n = new ResizeObserver((h) => {
      var s;
      r.value = ((s = h[0]) == null ? void 0 : s.contentRect.width) ?? 0;
    }), n.observe(u));
  }
  return ue(() => g(t.value)), ye(t, (u) => g(u)), tt(() => n == null ? void 0 : n.disconnect()), { width: r, compact: ce(() => r.value > 0 && r.value < i) };
}
const Kt = ["innerHTML"], Qe = /* @__PURE__ */ Z({
  __name: "MarkdownView",
  props: {
    source: {}
  },
  setup(t) {
    const i = t, r = ce(() => Ot(i.source));
    return (n, g) => (_(), L("div", {
      class: "md-body",
      innerHTML: r.value
    }, null, 8, Kt));
  }
}), Wt = { class: "content-label" }, Qt = /* @__PURE__ */ Z({
  __name: "MemoryEditorDialog",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    tagOptions: {},
    width: {}
  },
  emits: ["update:visible", "saved"],
  setup(t, { emit: i }) {
    const r = t, n = i, g = re(), u = y(!1), h = y("edit"), s = y({ id: null, summary: "", content: "", tags: [] });
    let b = [];
    ye(
      () => r.visible,
      async (U) => {
        if (U)
          if (h.value = "edit", r.memoryId)
            try {
              const m = await Ce(g, r.memoryId);
              s.value = { id: m.id, summary: m.summary, content: m.content, tags: [...m.tags] }, b = [...m.tags];
            } catch (m) {
              O.error(m instanceof Error ? m.message : String(m)), n("update:visible", !1);
            }
          else
            s.value = { id: null, summary: "", content: "", tags: [] }, b = [];
      }
    );
    async function V() {
      u.value = !0;
      try {
        if (s.value.id) {
          const U = new Set(b), m = new Set(s.value.tags);
          await Ke(g, s.value.id, {
            summary: s.value.summary,
            content: s.value.content,
            add_tags: [...m].filter((P) => !U.has(P)),
            remove_tags: [...U].filter((P) => !m.has(P))
          }), O.success(o("editor.updated"));
        } else
          await Je(g, {
            summary: s.value.summary,
            content: s.value.content,
            tags: s.value.tags
          }), O.success(o("editor.created"));
        n("update:visible", !1), n("saved");
      } catch (U) {
        O.error(U instanceof Error ? U.message : String(U));
      } finally {
        u.value = !1;
      }
    }
    return (U, m) => {
      const P = be, T = De, $ = $e, E = he, D = Ve, A = ze, R = Ue, F = me, B = xe;
      return _(), I(B, {
        "model-value": t.visible,
        title: s.value.id ? e(o)("editor.editTitle") : e(o)("editor.createTitle"),
        width: t.width,
        "onUpdate:modelValue": m[5] || (m[5] = (w) => n("update:visible", w))
      }, {
        footer: l(() => [
          a(F, {
            onClick: m[4] || (m[4] = (w) => n("update:visible", !1))
          }, {
            default: l(() => [
              v(d(e(o)("common.cancel")), 1)
            ]),
            _: 1
          }),
          a(F, {
            type: "primary",
            loading: u.value,
            onClick: V
          }, {
            default: l(() => [
              v(d(e(o)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: l(() => [
          a(R, { "label-position": "top" }, {
            default: l(() => [
              a(T, {
                label: e(o)("editor.summaryLabel")
              }, {
                default: l(() => [
                  a(P, {
                    modelValue: s.value.summary,
                    "onUpdate:modelValue": m[0] || (m[0] = (w) => s.value.summary = w),
                    maxlength: "512",
                    "show-word-limit": "",
                    placeholder: e(o)("editor.summaryPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"]),
              a(T, null, {
                label: l(() => [
                  M("div", Wt, [
                    M("span", null, d(e(o)("editor.contentLabel")), 1),
                    a(E, {
                      modelValue: h.value,
                      "onUpdate:modelValue": m[1] || (m[1] = (w) => h.value = w),
                      size: "small"
                    }, {
                      default: l(() => [
                        a($, { value: "edit" }, {
                          default: l(() => [
                            v(d(e(o)("editor.tabEdit")), 1)
                          ]),
                          _: 1
                        }),
                        a($, { value: "preview" }, {
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
                  h.value === "edit" ? (_(), I(P, {
                    key: 0,
                    modelValue: s.value.content,
                    "onUpdate:modelValue": m[2] || (m[2] = (w) => s.value.content = w),
                    type: "textarea",
                    rows: 12,
                    maxlength: "200000",
                    "show-word-limit": "",
                    placeholder: e(o)("editor.contentPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])) : (_(), I(Qe, {
                    key: 1,
                    class: "content-preview",
                    source: s.value.content
                  }, null, 8, ["source"]))
                ]),
                _: 1
              }),
              a(T, {
                label: e(o)("editor.tagsLabel")
              }, {
                default: l(() => [
                  a(A, {
                    modelValue: s.value.tags,
                    "onUpdate:modelValue": m[3] || (m[3] = (w) => s.value.tags = w),
                    multiple: "",
                    filterable: "",
                    "allow-create": "",
                    "default-first-option": "",
                    placeholder: e(o)("editor.tagsPlaceholder"),
                    class: "tags-select"
                  }, {
                    default: l(() => [
                      (_(!0), L(G, null, oe(t.tagOptions, (w) => (_(), I(D, {
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
}), ne = (t, i) => {
  const r = t.__vccOpts || t;
  for (const [n, g] of i)
    r[n] = g;
  return r;
}, Yt = /* @__PURE__ */ ne(Qt, [["__scopeId", "data-v-602bada0"]]), Gt = { class: "detail-summary" }, Xt = { class: "detail-tags" }, Zt = { class: "detail-toolbar" }, ea = {
  key: 1,
  class: "detail-content"
}, ta = /* @__PURE__ */ Z({
  __name: "MemoryDetailDrawer",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    size: {}
  },
  emits: ["update:visible"],
  setup(t, { emit: i }) {
    const r = t, n = i, g = re(), u = y(null), h = y("rendered");
    return ye(
      () => [r.visible, r.memoryId],
      async ([s]) => {
        if (!(!s || !r.memoryId)) {
          h.value = "rendered";
          try {
            u.value = await Ce(g, r.memoryId);
          } catch (b) {
            O.error(b instanceof Error ? b.message : String(b)), n("update:visible", !1);
          }
        }
      }
    ), (s, b) => {
      var $;
      const V = we, U = it, m = $e, P = he, T = st;
      return _(), I(T, {
        "model-value": t.visible,
        title: e(o)("drawer.title", { id: (($ = u.value) == null ? void 0 : $.id) ?? t.memoryId ?? "" }),
        size: t.size,
        "onUpdate:modelValue": b[1] || (b[1] = (E) => n("update:visible", E))
      }, {
        default: l(() => [
          u.value ? (_(), L(G, { key: 0 }, [
            M("h3", Gt, d(u.value.summary), 1),
            M("div", Xt, [
              (_(!0), L(G, null, oe(u.value.tags, (E) => (_(), I(V, {
                key: E,
                size: "small",
                class: "am-tag"
              }, {
                default: l(() => [
                  v(d(E), 1)
                ]),
                _: 2
              }, 1024))), 128))
            ]),
            a(U),
            M("div", Zt, [
              a(P, {
                modelValue: h.value,
                "onUpdate:modelValue": b[0] || (b[0] = (E) => h.value = E),
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
            h.value === "rendered" ? (_(), I(Qe, {
              key: 0,
              source: u.value.content
            }, null, 8, ["source"])) : (_(), L("pre", ea, d(u.value.content), 1))
          ], 64)) : J("", !0)
        ]),
        _: 1
      }, 8, ["model-value", "title", "size"]);
    };
  }
}), aa = /* @__PURE__ */ ne(ta, [["__scopeId", "data-v-b634d116"]]), oa = {
  key: 0,
  class: "am-panel-header"
}, la = { class: "am-heading" }, na = { class: "am-panel-title" }, sa = { class: "am-toolbar" }, ia = { class: "am-summary" }, ra = ["innerHTML"], da = {
  key: 4,
  class: "am-pager"
}, ca = {
  key: 5,
  class: "am-pager"
}, ua = /* @__PURE__ */ Z({
  __name: "MemoriesPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: i }) {
    const r = t, {
      query: n,
      tagFilter: g,
      sort: u,
      order: h,
      page: s,
      pageSize: b,
      rows: V,
      searchResults: U,
      total: m,
      note: P,
      loading: T,
      tagOptions: $,
      searching: E,
      onSearch: D,
      reload: A,
      loadTagOptions: R,
      removeMemory: F
    } = Jt(), B = y(null), { compact: w } = Te(B), f = y(!1), k = y(null), x = y(!1), z = y(null);
    function C(N) {
      return N().catch((c) => O.error(c instanceof Error ? c.message : String(c)));
    }
    function K() {
      k.value = null, f.value = !0;
    }
    function j(N) {
      k.value = N, f.value = !0;
    }
    function W(N) {
      z.value = N, x.value = !0;
    }
    function X() {
      C(A), R();
    }
    function Q(N) {
      const { prop: c, order: te } = N;
      te && (c === "updated_at" || c === "created_at") ? (u.value = c, h.value = te === "ascending" ? "asc" : "desc") : (u.value = "updated_at", h.value = "desc"), C(A);
    }
    async function ee(N) {
      try {
        await Be.confirm(o("memories.deleteConfirm", { id: N.id }), o("memories.deleteTitle"), {
          type: "warning"
        });
      } catch {
        return;
      }
      try {
        await F(N.id), O.success(o("memories.deleted"));
      } catch (c) {
        O.error(c instanceof Error ? c.message : String(c));
      }
    }
    return i({ refresh: () => C(A) }), (N, c) => {
      const te = pe, ge = ke, S = me, Y = be, Ye = Ve, Ge = ze, Xe = Le, H = Ne, Se = we, Pe = Ie, Ee = rt, Me = Ae;
      return _(), L("div", {
        ref_key: "rootRef",
        ref: B,
        class: "am-panel"
      }, [
        t.showHeader ? (_(), L("div", oa, [
          M("div", la, [
            M("h2", na, d(r.title ?? e(o)("memories.title")), 1),
            a(ge, {
              content: r.subtitle ?? e(o)("memories.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(te, { class: "am-info" }, {
                  default: l(() => [
                    a(e(se))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(S, {
            type: "primary",
            icon: e(Re),
            onClick: K
          }, {
            default: l(() => [
              v(d(e(o)("memories.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : J("", !0),
        M("div", sa, [
          a(Y, {
            modelValue: e(n),
            "onUpdate:modelValue": c[1] || (c[1] = (p) => ae(n) ? n.value = p : null),
            placeholder: e(o)("memories.searchPlaceholder"),
            clearable: "",
            class: "search",
            onKeyup: c[2] || (c[2] = at((p) => C(e(D)), ["enter"])),
            onClear: c[3] || (c[3] = (p) => C(e(D)))
          }, {
            append: l(() => [
              a(S, {
                icon: e(Oe),
                onClick: c[0] || (c[0] = (p) => C(e(D)))
              }, null, 8, ["icon"])
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(Ge, {
            modelValue: e(g),
            "onUpdate:modelValue": c[4] || (c[4] = (p) => ae(g) ? g.value = p : null),
            placeholder: e(o)("memories.tagFilter"),
            clearable: "",
            filterable: "",
            class: "tag-filter",
            onChange: c[5] || (c[5] = (p) => C(e(D)))
          }, {
            default: l(() => [
              (_(!0), L(G, null, oe(e($), (p) => (_(), I(Ye, {
                key: p,
                label: p,
                value: p
              }, null, 8, ["label", "value"]))), 128))
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"])
        ]),
        e(P) ? (_(), I(Xe, {
          key: 1,
          title: e(P),
          type: "info",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : J("", !0),
        e(E) ? le((_(), I(Pe, {
          key: 2,
          data: e(U)
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
              default: l(({ row: p }) => [
                M("div", ia, d(p.summary), 1),
                M("div", {
                  class: "am-snippet",
                  innerHTML: p.snippet
                }, null, 8, ra)
              ]),
              _: 1
            }, 8, ["label"]),
            a(H, {
              label: e(o)("memories.colTags"),
              width: "220"
            }, {
              default: l(({ row: p }) => [
                (_(!0), L(G, null, oe(p.tags, (q) => (_(), I(Se, {
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
              default: l(({ row: p }) => [
                v(d(e(ie)(p.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(H, {
              label: e(o)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: l(({ row: p }) => [
                a(S, {
                  link: "",
                  type: "primary",
                  onClick: (q) => W(p.id)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(S, {
                  link: "",
                  type: "primary",
                  onClick: (q) => j(p.id)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(S, {
                  link: "",
                  type: "danger",
                  onClick: (q) => ee(p)
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
          [Me, e(T)]
        ]) : le((_(), I(Pe, {
          key: 3,
          data: e(V),
          "default-sort": { prop: e(u), order: e(h) === "asc" ? "ascending" : "descending" },
          onSortChange: Q
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
              default: l(({ row: p }) => [
                (_(!0), L(G, null, oe(p.tags, (q) => (_(), I(Se, {
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
            e(w) ? J("", !0) : (_(), I(H, {
              key: 0,
              prop: "created_at",
              label: e(o)("memories.colCreatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: l(({ row: p }) => [
                v(d(e(ie)(p.created_at)), 1)
              ]),
              _: 1
            }, 8, ["label"])),
            a(H, {
              prop: "updated_at",
              label: e(o)("memories.colUpdatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: l(({ row: p }) => [
                v(d(e(ie)(p.updated_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(H, {
              label: e(o)("memories.colActions"),
              width: "190",
              fixed: "right"
            }, {
              default: l(({ row: p }) => [
                a(S, {
                  link: "",
                  type: "primary",
                  onClick: (q) => W(p.id)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(S, {
                  link: "",
                  type: "primary",
                  onClick: (q) => j(p.id)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(S, {
                  link: "",
                  type: "danger",
                  onClick: (q) => ee(p)
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
          [Me, e(T)]
        ]),
        e(E) ? (_(), L("div", ca, [
          a(Ee, {
            "current-page": e(s),
            "onUpdate:currentPage": c[10] || (c[10] = (p) => ae(s) ? s.value = p : null),
            "page-size": e(b),
            "onUpdate:pageSize": c[11] || (c[11] = (p) => ae(b) ? b.value = p : null),
            total: e(m),
            "page-sizes": [10, 20, 50],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: c[12] || (c[12] = (p) => C(e(A))),
            onSizeChange: c[13] || (c[13] = (p) => C(e(A)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])) : (_(), L("div", da, [
          a(Ee, {
            "current-page": e(s),
            "onUpdate:currentPage": c[6] || (c[6] = (p) => ae(s) ? s.value = p : null),
            "page-size": e(b),
            "onUpdate:pageSize": c[7] || (c[7] = (p) => ae(b) ? b.value = p : null),
            total: e(m),
            "page-sizes": [20, 50, 100, 200],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: c[8] || (c[8] = (p) => C(e(A))),
            onSizeChange: c[9] || (c[9] = (p) => C(e(A)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])),
        a(Yt, {
          visible: f.value,
          "onUpdate:visible": c[14] || (c[14] = (p) => f.value = p),
          "memory-id": k.value,
          "tag-options": e($),
          width: e(w) ? "96%" : "640px",
          onSaved: X
        }, null, 8, ["visible", "memory-id", "tag-options", "width"]),
        a(aa, {
          visible: x.value,
          "onUpdate:visible": c[15] || (c[15] = (p) => x.value = p),
          "memory-id": z.value,
          size: e(w) ? "100%" : "45%"
        }, null, 8, ["visible", "memory-id", "size"])
      ], 512);
    };
  }
}), ma = /* @__PURE__ */ ne(ua, [["__scopeId", "data-v-1ab68c72"]]);
function pa() {
  const t = re(), i = y([]), r = y(!1);
  async function n() {
    r.value = !0;
    try {
      const s = await We(t);
      i.value = s.tags ?? [];
    } finally {
      r.value = !1;
    }
  }
  async function g(s, b) {
    await Ht(t, s, b), await n();
  }
  async function u(s, b, V) {
    await qt(t, s, b, V), await n();
  }
  async function h(s, b) {
    await jt(t, s, b), await n();
  }
  return ue(() => {
    n().catch(() => {
    });
  }), { rows: i, loading: r, reload: n, create: g, rename: u, remove: h };
}
const fa = {
  key: 0,
  class: "am-panel-header"
}, ga = { class: "am-heading" }, va = { class: "am-panel-title" }, _a = { class: "delete-body" }, ya = /* @__PURE__ */ Z({
  __name: "TagsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: i }) {
    const r = t, { rows: n, loading: g, reload: u, create: h, rename: s, remove: b } = pa(), V = y(null), { compact: U } = Te(V), m = y(!1), P = y(!1), T = ot({ oldName: null, name: "", newName: "", description: "" }), $ = y(!1), E = y("detach"), D = y(null);
    function A() {
      Object.assign(T, { oldName: null, name: "", newName: "", description: "" }), P.value = !0;
    }
    function R(f) {
      Object.assign(T, { oldName: f.name, name: f.name, newName: "", description: f.description ?? "" }), P.value = !0;
    }
    async function F() {
      m.value = !0;
      try {
        T.oldName ? (await s(T.oldName, T.newName, T.description), O.success(o("tags.saved"))) : (await h(T.name, T.description), O.success(o("tags.created"))), P.value = !1;
      } catch (f) {
        O.error(f instanceof Error ? f.message : String(f));
      } finally {
        m.value = !1;
      }
    }
    function B(f) {
      D.value = f, E.value = "detach", $.value = !0;
    }
    async function w() {
      var f, k;
      if (E.value === "purge")
        try {
          await Be.confirm(
            o("tags.purgeConfirm", { name: (f = D.value) == null ? void 0 : f.name, count: ((k = D.value) == null ? void 0 : k.memory_count) ?? 0 }),
            o("tags.purgeConfirmTitle"),
            { type: "error", confirmButtonText: o("tags.purgeButton") }
          );
        } catch {
          return;
        }
      if (D.value) {
        m.value = !0;
        try {
          await b(D.value.name, E.value), O.success(o("tags.deleted")), $.value = !1;
        } catch (x) {
          O.error(x instanceof Error ? x.message : String(x));
        } finally {
          m.value = !1;
        }
      }
    }
    return i({
      refresh: () => u().catch((f) => O.error(f instanceof Error ? f.message : String(f)))
    }), (f, k) => {
      const x = pe, z = ke, C = me, K = we, j = Ne, W = Ie, X = be, Q = De, ee = Ue, N = xe, c = dt, te = he, ge = Ae;
      return _(), L("div", {
        ref_key: "rootRef",
        ref: V,
        class: "am-panel"
      }, [
        t.showHeader ? (_(), L("div", fa, [
          M("div", ga, [
            M("h2", va, d(r.title ?? e(o)("tags.title")), 1),
            a(z, {
              content: r.subtitle ?? e(o)("tags.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(x, { class: "am-info" }, {
                  default: l(() => [
                    a(e(se))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(C, {
            type: "primary",
            icon: e(Re),
            onClick: A
          }, {
            default: l(() => [
              v(d(e(o)("tags.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : J("", !0),
        le((_(), I(W, { data: e(n) }, {
          default: l(() => [
            a(j, {
              prop: "name",
              label: e(o)("tags.colName"),
              "min-width": "160"
            }, {
              default: l(({ row: S }) => [
                a(K, null, {
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
                v(d(e(ie)(S.last_used_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(j, {
              label: e(o)("memories.colActions"),
              width: "150",
              fixed: "right"
            }, {
              default: l(({ row: S }) => [
                a(C, {
                  link: "",
                  type: "primary",
                  onClick: (Y) => R(S)
                }, {
                  default: l(() => [
                    v(d(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(C, {
                  link: "",
                  type: "danger",
                  onClick: (Y) => B(S)
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
          [ge, e(g)]
        ]),
        a(N, {
          modelValue: P.value,
          "onUpdate:modelValue": k[4] || (k[4] = (S) => P.value = S),
          title: T.oldName ? e(o)("tags.editTitle") : e(o)("tags.createTitle"),
          width: e(U) ? "96%" : "480px"
        }, {
          footer: l(() => [
            a(C, {
              onClick: k[3] || (k[3] = (S) => P.value = !1)
            }, {
              default: l(() => [
                v(d(e(o)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(C, {
              type: "primary",
              loading: m.value,
              onClick: F
            }, {
              default: l(() => [
                v(d(e(o)("common.save")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: l(() => [
            a(ee, { "label-position": "top" }, {
              default: l(() => [
                a(Q, {
                  label: e(o)("tags.nameLabel")
                }, {
                  default: l(() => [
                    a(X, {
                      modelValue: T.name,
                      "onUpdate:modelValue": k[0] || (k[0] = (S) => T.name = S),
                      disabled: !!T.oldName,
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(o)("tags.namePlaceholder")
                    }, null, 8, ["modelValue", "disabled", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                T.oldName ? (_(), I(Q, {
                  key: 0,
                  label: e(o)("tags.renameLabel")
                }, {
                  default: l(() => [
                    a(X, {
                      modelValue: T.newName,
                      "onUpdate:modelValue": k[1] || (k[1] = (S) => T.newName = S),
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(o)("tags.renamePlaceholder")
                    }, null, 8, ["modelValue", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"])) : J("", !0),
                a(Q, {
                  label: e(o)("tags.descLabel")
                }, {
                  default: l(() => [
                    a(X, {
                      modelValue: T.description,
                      "onUpdate:modelValue": k[2] || (k[2] = (S) => T.description = S),
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
        a(N, {
          modelValue: $.value,
          "onUpdate:modelValue": k[7] || (k[7] = (S) => $.value = S),
          title: e(o)("tags.deleteTitle"),
          width: e(U) ? "96%" : "480px"
        }, {
          footer: l(() => [
            a(C, {
              onClick: k[6] || (k[6] = (S) => $.value = !1)
            }, {
              default: l(() => [
                v(d(e(o)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(C, {
              type: "danger",
              loading: m.value,
              onClick: w
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
              M("p", _a, [
                v(d(e(o)("tags.deleteBefore")) + " ", 1),
                a(K, null, {
                  default: l(() => {
                    var Y;
                    return [
                      v(d((Y = D.value) == null ? void 0 : Y.name), 1)
                    ];
                  }),
                  _: 1
                }),
                v(" " + d(e(o)("tags.deleteMiddle")) + " ", 1),
                M("b", null, d((S = D.value) == null ? void 0 : S.memory_count), 1),
                v(" " + d(e(o)("tags.deleteAfter")), 1)
              ]),
              a(te, {
                modelValue: E.value,
                "onUpdate:modelValue": k[5] || (k[5] = (Y) => E.value = Y)
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
}), ba = /* @__PURE__ */ ne(ya, [["__scopeId", "data-v-9847d5f4"]]);
function ha(t) {
  return t.get("/api/stats");
}
function wa(t) {
  return t.get("/health");
}
function ka(t) {
  return t.get("/api/doctor");
}
function Ca(t) {
  return t.getBlob("/api/export");
}
function Ta(t, i) {
  return t.post("/api/import", i);
}
function Sa() {
  const t = re(), i = y({}), r = y(""), n = y({ ok: !0, issues: [] }), g = y(!1), u = y(!1), h = y(!1), s = y(!1), b = ce(() => It(i.value.file_size));
  async function V() {
    i.value = await ha(t), i.value.version = r.value;
  }
  async function U() {
    u.value = !0;
    try {
      n.value = await ka(t), g.value = !0;
    } finally {
      u.value = !1;
    }
  }
  async function m() {
    h.value = !0;
    try {
      const T = await Ca(t), $ = URL.createObjectURL(T), E = document.createElement("a");
      E.href = $, E.download = "agent-memory-export.json", E.click(), URL.revokeObjectURL($);
    } finally {
      h.value = !1;
    }
  }
  async function P(T) {
    s.value = !0;
    try {
      const $ = await T.text();
      let E;
      try {
        E = JSON.parse($);
      } catch {
        throw new Error(o("errors.invalidBackup"));
      }
      const D = await Ta(t, E);
      return await V(), D;
    } finally {
      s.value = !1;
    }
  }
  return ue(async () => {
    try {
      r.value = (await wa(t)).version ?? "";
    } catch {
    }
    await V().catch(() => {
    });
  }), {
    stats: i,
    version: r,
    doctor: n,
    doctorRan: g,
    doctorLoading: u,
    exporting: h,
    importing: s,
    sizeText: b,
    reload: V,
    runDoctor: U,
    exportData: m,
    importFile: P
  };
}
const Pa = {
  key: 0,
  class: "am-panel-header"
}, Ea = { class: "am-heading" }, Ma = { class: "am-panel-title" }, xa = { class: "label-help" }, Ua = { class: "actions" }, Da = { class: "card-header" }, $a = {
  key: 2,
  class: "issues"
}, za = /* @__PURE__ */ Z({
  __name: "OpsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: i }) {
    const r = t, {
      stats: n,
      version: g,
      doctor: u,
      doctorRan: h,
      doctorLoading: s,
      exporting: b,
      importing: V,
      sizeText: U,
      reload: m,
      runDoctor: P,
      exportData: T,
      importFile: $
    } = Sa(), E = y(null), { compact: D } = Te(E), A = y(null);
    function R(B) {
      return B().catch((w) => O.error(w instanceof Error ? w.message : String(w)));
    }
    function F(B) {
      var k;
      const w = B.target, f = (k = w.files) == null ? void 0 : k[0];
      w.value = "", f && $(f).then((x) => {
        O.success(o("ops.imported", { memories: x.imported_memories, tags: x.imported_tags }));
      }).catch((x) => {
        O.error(x instanceof Error ? x.message : String(x));
      });
    }
    return i({ refresh: () => R(m) }), (B, w) => {
      const f = pe, k = ke, x = me, z = pt, C = mt, K = ut, j = ct, W = gt, X = ft, Q = Le, ee = vt;
      return _(), L("div", {
        ref_key: "rootRef",
        ref: E,
        class: "am-panel"
      }, [
        t.showHeader ? (_(), L("div", Pa, [
          M("div", Ea, [
            M("h2", Ma, d(r.title ?? e(o)("ops.title")), 1),
            a(k, {
              content: r.subtitle ?? e(o)("ops.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(f, { class: "am-info" }, {
                  default: l(() => [
                    a(e(se))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(x, {
            icon: e(Tt),
            onClick: w[0] || (w[0] = (N) => R(e(m)))
          }, {
            default: l(() => [
              v(d(e(o)("ops.refresh")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : J("", !0),
        a(j, { gutter: 14 }, {
          default: l(() => [
            a(K, {
              span: e(D) ? 12 : 6
            }, {
              default: l(() => [
                a(C, { shadow: "never" }, {
                  default: l(() => [
                    a(z, {
                      title: e(o)("ops.statMemories"),
                      value: e(n).memories ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(K, {
              span: e(D) ? 12 : 6
            }, {
              default: l(() => [
                a(C, { shadow: "never" }, {
                  default: l(() => [
                    a(z, {
                      title: e(o)("ops.statTags"),
                      value: e(n).tags ?? 0
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(K, {
              span: e(D) ? 12 : 6
            }, {
              default: l(() => [
                a(C, { shadow: "never" }, {
                  default: l(() => [
                    a(z, {
                      title: e(o)("ops.statSize"),
                      value: e(U)
                    }, null, 8, ["title", "value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            a(K, {
              span: e(D) ? 12 : 6
            }, {
              default: l(() => [
                a(C, { shadow: "never" }, {
                  default: l(() => [
                    a(z, {
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
        a(C, { shadow: "never" }, {
          header: l(() => [
            v(d(e(o)("ops.dbCard")), 1)
          ]),
          default: l(() => [
            a(X, {
              column: e(D) ? 1 : 2,
              border: ""
            }, {
              default: l(() => [
                a(W, {
                  label: e(o)("ops.path")
                }, {
                  default: l(() => [
                    v(d(e(n).path ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(W, {
                  label: e(o)("ops.lastUpdate")
                }, {
                  default: l(() => [
                    v(d(e(n).newest_update ? e(o)("ops.lastUpdateValue", {
                      id: e(n).newest_update.id,
                      time: e(ie)(e(n).newest_update.updated_at)
                    }) : "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(W, null, {
                  label: l(() => [
                    M("span", xa, [
                      v(d(e(o)("ops.crossPlatform")) + " ", 1),
                      a(k, {
                        content: e(o)("ops.crossPlatformNote"),
                        placement: "top"
                      }, {
                        default: l(() => [
                          a(f, { class: "am-info" }, {
                            default: l(() => [
                              a(e(se))
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
                a(W, {
                  label: e(o)("ops.version")
                }, {
                  default: l(() => [
                    v(d(e(n).version ?? e(g)), 1)
                  ]),
                  _: 1
                }, 8, ["label"])
              ]),
              _: 1
            }, 8, ["column"]),
            M("div", Ua, [
              a(x, {
                icon: e(St),
                loading: e(b),
                onClick: w[1] || (w[1] = (N) => R(e(T)))
              }, {
                default: l(() => [
                  v(d(e(o)("ops.export")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(x, {
                icon: e(Pt),
                loading: e(V),
                onClick: w[2] || (w[2] = (N) => {
                  var c;
                  return (c = A.value) == null ? void 0 : c.click();
                })
              }, {
                default: l(() => [
                  v(d(e(o)("ops.import")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(k, {
                content: e(o)("ops.importHint"),
                placement: "top"
              }, {
                default: l(() => [
                  a(f, { class: "am-info" }, {
                    default: l(() => [
                      a(e(se))
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["content"]),
              M("input", {
                ref_key: "importInput",
                ref: A,
                type: "file",
                accept: "application/json,.json",
                style: { display: "none" },
                onChange: F
              }, null, 544)
            ])
          ]),
          _: 1
        }),
        a(C, { shadow: "never" }, {
          header: l(() => [
            M("div", Da, [
              M("span", null, d(e(o)("ops.doctorCard")), 1),
              a(x, {
                size: "small",
                icon: e(Oe),
                loading: e(s),
                onClick: w[3] || (w[3] = (N) => R(e(P)))
              }, {
                default: l(() => [
                  v(d(e(o)("ops.runDoctor")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"])
            ])
          ]),
          default: l(() => [
            e(h) ? (_(), L(G, { key: 0 }, [
              e(u).ok ? (_(), I(Q, {
                key: 0,
                title: e(o)("ops.doctorOk"),
                type: "success",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])) : (_(), I(Q, {
                key: 1,
                title: e(o)("ops.doctorFail", { count: e(u).issues.length }),
                type: "error",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])),
              e(u).ok ? J("", !0) : (_(), L("ul", $a, [
                (_(!0), L(G, null, oe(e(u).issues, (N, c) => (_(), L("li", { key: c }, d(N), 1))), 128))
              ]))
            ], 64)) : (_(), I(ee, {
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
}), Va = /* @__PURE__ */ ne(za, [["__scopeId", "data-v-a2662ae1"]]), La = { class: "memory-ui" }, Ia = { class: "brand" }, Na = { class: "brand-mark" }, Aa = { class: "aside-footer" }, Ra = /* @__PURE__ */ Z({
  __name: "MemoryAdmin",
  props: {
    layout: { default: "sidebar" },
    title: { default: "agent-memory" }
  },
  setup(t) {
    const i = y("memories");
    return (r, n) => {
      const g = pe, u = ht, h = bt, s = yt, b = Ct, V = kt, U = wt, m = _t;
      return _(), L("div", La, [
        a(m, { class: "layout" }, {
          default: l(() => [
            t.layout === "sidebar" ? (_(), I(s, {
              key: 0,
              width: "200px",
              class: "aside"
            }, {
              default: l(() => [
                M("div", Ia, [
                  M("span", Na, [
                    a(g, { size: 16 }, {
                      default: l(() => [
                        a(e(Et))
                      ]),
                      _: 1
                    })
                  ]),
                  M("span", null, d(t.title), 1)
                ]),
                a(h, {
                  "default-active": i.value,
                  class: "menu",
                  onSelect: n[0] || (n[0] = (P) => i.value = P)
                }, {
                  default: l(() => [
                    a(u, { index: "memories" }, {
                      default: l(() => [
                        a(g, null, {
                          default: l(() => [
                            a(e(Mt))
                          ]),
                          _: 1
                        }),
                        M("span", null, d(e(o)("nav.memories")), 1)
                      ]),
                      _: 1
                    }),
                    a(u, { index: "tags" }, {
                      default: l(() => [
                        a(g, null, {
                          default: l(() => [
                            a(e(xt))
                          ]),
                          _: 1
                        }),
                        M("span", null, d(e(o)("nav.tags")), 1)
                      ]),
                      _: 1
                    }),
                    a(u, { index: "ops" }, {
                      default: l(() => [
                        a(g, null, {
                          default: l(() => [
                            a(e(Ut))
                          ]),
                          _: 1
                        }),
                        M("span", null, d(e(o)("nav.ops")), 1)
                      ]),
                      _: 1
                    })
                  ]),
                  _: 1
                }, 8, ["default-active"]),
                M("div", Aa, [
                  lt(r.$slots, "footer", {}, void 0, !0)
                ])
              ]),
              _: 3
            })) : J("", !0),
            a(U, { class: "main" }, {
              default: l(() => [
                t.layout === "tabs" ? (_(), I(V, {
                  key: 0,
                  modelValue: i.value,
                  "onUpdate:modelValue": n[1] || (n[1] = (P) => i.value = P),
                  class: "tabs-bar"
                }, {
                  default: l(() => [
                    a(b, {
                      label: e(o)("nav.memories"),
                      name: "memories"
                    }, null, 8, ["label"]),
                    a(b, {
                      label: e(o)("nav.tags"),
                      name: "tags"
                    }, null, 8, ["label"]),
                    a(b, {
                      label: e(o)("nav.ops"),
                      name: "ops"
                    }, null, 8, ["label"])
                  ]),
                  _: 1
                }, 8, ["modelValue"])) : J("", !0),
                le(a(ma, null, null, 512), [
                  [ve, i.value === "memories"]
                ]),
                le(a(ba, null, null, 512), [
                  [ve, i.value === "tags"]
                ]),
                le(a(Va, null, null, 512), [
                  [ve, i.value === "ops"]
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
}), Qa = /* @__PURE__ */ ne(Ra, [["__scopeId", "data-v-6f56cb0b"]]);
export {
  Qe as MarkdownView,
  ma as MemoriesPanel,
  Qa as MemoryAdmin,
  aa as MemoryDetailDrawer,
  Yt as MemoryEditorDialog,
  qe as MemoryUIConfigKey,
  Va as OpsPanel,
  ba as TagsPanel,
  Lt as applyMemoryUILocalePreference,
  At as buildMemoriesQuery,
  Nt as createApiClient,
  Ka as currentMemoryUILocale,
  It as formatSize,
  ie as formatTime,
  _e as isSearchMode,
  fe as memoryUIi18n,
  Wa as provideMemoryUI,
  Ot as renderMarkdown,
  Bt as sanitizeHtml,
  Vt as setMemoryUILocale,
  o as t,
  re as useApiClient,
  Jt as useMemories,
  je as useMemoryConfig,
  Sa as useOps,
  pa as useTags
};
