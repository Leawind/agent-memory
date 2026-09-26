import { provide as Ze, inject as et, ref as y, computed as de, onMounted as ue, watch as ye, onUnmounted as tt, defineComponent as Z, openBlock as _, createElementBlock as L, createBlock as I, unref as e, withCtx as l, createVNode as a, createElementVNode as z, toDisplayString as c, createTextVNode as v, Fragment as X, renderList as oe, createCommentVNode as J, withKeys as at, isRef as ae, withDirectives as le, reactive as ot, renderSlot as lt, vShow as ve } from "vue";
import { createI18n as nt } from "vue-i18n";
import { ElDialog as ze, ElForm as De, ElFormItem as $e, ElInput as he, ElRadioGroup as be, ElRadioButton as Ve, ElSelect as xe, ElOption as Ue, ElButton as me, ElDrawer as st, ElTag as ke, ElDivider as it, ElTooltip as we, ElIcon as pe, ElAlert as Le, ElTable as Ie, ElTableColumn as Ae, ElLoadingDirective as Ne, ElPagination as rt, ElRadio as ct, ElRow as dt, ElCol as ut, ElCard as mt, ElStatistic as pt, ElDescriptions as ft, ElDescriptionsItem as gt, ElEmpty as vt, ElContainer as _t, ElAside as yt, ElMenu as ht, ElMenuItem as bt, ElMain as kt, ElTabs as wt, ElTabPane as Ct } from "element-plus/es";
import { InfoFilled as ce, Plus as Re, Search as Oe, Refresh as Tt, Download as St, UploadFilled as Mt, Collection as Et, Notebook as Pt, PriceTag as zt, Odometer as Dt } from "@element-plus/icons-vue";
import { ElMessage as O, ElMessageBox as Be } from "element-plus";
import { Marked as $t } from "marked";
import Fe from "dompurify";
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
  messages: { zh: Vt, en: xt },
  // 面向宿主组件库，缺 key 时静默回退即可，不刷控制台
  missingWarn: !1,
  fallbackWarn: !1
}), { t: o } = fe.global;
function Ut(t) {
  fe.global.locale.value = t;
}
function Lt(t) {
  Ut(t === "auto" ? He() : t);
}
function Ja() {
  return fe.global.locale.value;
}
const qe = Symbol("memory-ui-config"), ie = {
  baseUrl: "",
  fetch: (...t) => globalThis.fetch(...t),
  defaultPageSize: 20,
  locale: "auto"
};
function Ka(t) {
  t.locale && Lt(t.locale), Ze(qe, t);
}
function je() {
  const t = et(qe);
  return {
    baseUrl: ((t == null ? void 0 : t.baseUrl) ?? ie.baseUrl).replace(/\/+$/, ""),
    fetch: (t == null ? void 0 : t.fetch) ?? ie.fetch,
    defaultPageSize: (t == null ? void 0 : t.defaultPageSize) ?? ie.defaultPageSize,
    locale: (t == null ? void 0 : t.locale) ?? ie.locale
  };
}
function re(t) {
  if (!t) return "—";
  const i = fe.global.locale.value === "en" ? "en-US" : "zh-CN";
  return new Date(t * 1e3).toLocaleString(i, { hour12: !1 });
}
function It(t) {
  return t == null ? "—" : t < 1024 ? `${t} B` : t < 1024 * 1024 ? `${(t / 1024).toFixed(1)} KB` : `${(t / 1024 / 1024).toFixed(2)} MB`;
}
function At(t) {
  async function i(n, f = {}) {
    const u = await t.fetch(t.baseUrl + n, {
      headers: { "Content-Type": "application/json" },
      ...f
    }), b = await u.text();
    let s = null;
    try {
      s = b ? JSON.parse(b) : null;
    } catch {
      s = null;
    }
    if (!u.ok) {
      const h = (s == null ? void 0 : s.error) ?? o("errors.http", { status: u.status });
      throw new Error(h);
    }
    return s;
  }
  async function r(n) {
    var u;
    const f = await t.fetch(t.baseUrl + n);
    if (!f.ok) {
      const b = await f.text().catch(() => "");
      let s = o("errors.http", { status: f.status });
      try {
        s = ((u = JSON.parse(b)) == null ? void 0 : u.error) ?? s;
      } catch {
      }
      throw new Error(s);
    }
    return f.blob();
  }
  return {
    get: (n) => i(n),
    post: (n, f) => i(n, { method: "POST", body: JSON.stringify(f ?? {}) }),
    put: (n, f) => i(n, { method: "PUT", body: JSON.stringify(f ?? {}) }),
    del: (n) => i(n, { method: "DELETE" }),
    getBlob: r
  };
}
function se() {
  return At(je());
}
function Nt(t) {
  const i = _e(t.query), r = new URLSearchParams();
  return i ? (r.set("query", t.query.trim()), t.tagFilter && r.set("tags", t.tagFilter)) : (t.tagFilter && r.set("tag", t.tagFilter), r.set("sort", t.sort), r.set("order", t.order)), r.set("offset", String((t.page - 1) * t.pageSize)), r.set("limit", String(t.pageSize)), r.toString();
}
function _e(t) {
  return t.trim().length > 0;
}
const Rt = new $t();
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
  const f = { description: n }, u = r.trim();
  return u && u !== i && (f.new_name = u), t.put(`/api/tags/${encodeURIComponent(i)}`, f);
}
function jt(t, i, r) {
  return t.del(`/api/tags/${encodeURIComponent(i)}?mode=${r}`);
}
function Jt() {
  const t = se(), { defaultPageSize: i } = je(), r = y(""), n = y(""), f = y("updated_at"), u = y("desc"), b = y(1), s = y(i), h = y([]), x = y([]), D = y(0), m = y(""), M = y(!1), C = y([]), $ = de(() => _e(r.value));
  function E() {
    return b.value = 1, R();
  }
  function V() {
    return Nt({
      query: r.value,
      tagFilter: n.value,
      sort: f.value,
      order: u.value,
      page: b.value,
      pageSize: s.value
    });
  }
  let N = 0;
  async function R() {
    var w;
    const g = ++N;
    M.value = !0;
    try {
      const P = V();
      if (_e(r.value)) {
        const U = await t.get(`/api/memories?${P}`);
        if (g !== N) return;
        x.value = (U.results ?? []).map((T) => ({ ...T, snippet: Bt(T.snippet) })), D.value = U.total_matches ?? 0, m.value = "";
      } else {
        const U = await t.get(`/api/memories?${P}`);
        if (g !== N) return;
        const T = Math.max(1, Math.ceil(U.total / s.value));
        if (((w = U.memories) == null ? void 0 : w.length) === 0 && U.total > 0 && b.value > T)
          return b.value = T, M.value = !1, R();
        h.value = U.memories ?? [], D.value = U.total ?? 0, m.value = U.note ?? "";
      }
    } finally {
      g === N && (M.value = !1);
    }
  }
  async function F() {
    try {
      const g = await We(t);
      C.value = (g.tags ?? []).map((w) => w.name);
    } catch {
    }
  }
  async function B(g) {
    if (g.id) {
      const w = await Ce(t, g.id), P = new Set(w.tags), U = new Set(g.tags);
      await Ke(t, g.id, {
        summary: g.summary,
        content: g.content,
        add_tags: [...U].filter((T) => !P.has(T)),
        remove_tags: [...P].filter((T) => !U.has(T))
      });
    } else
      await Je(t, { summary: g.summary, content: g.content, tags: g.tags });
    await Promise.all([R(), F()]);
  }
  async function k(g) {
    await Ft(t, g), await R();
  }
  return ue(() => {
    R().catch(() => {
    }), F();
  }), {
    query: r,
    tagFilter: n,
    sort: f,
    order: u,
    page: b,
    pageSize: s,
    rows: h,
    searchResults: x,
    total: D,
    note: m,
    loading: M,
    tagOptions: C,
    searching: $,
    onSearch: E,
    reload: R,
    loadTagOptions: F,
    saveMemory: B,
    removeMemory: k
  };
}
function Te(t, i = 720) {
  const r = y(0);
  let n = null;
  function f(u) {
    n == null || n.disconnect(), n = null, !(!u || typeof ResizeObserver > "u") && (n = new ResizeObserver((b) => {
      var s;
      r.value = ((s = b[0]) == null ? void 0 : s.contentRect.width) ?? 0;
    }), n.observe(u));
  }
  return ue(() => f(t.value)), ye(t, (u) => f(u)), tt(() => n == null ? void 0 : n.disconnect()), { width: r, compact: de(() => r.value > 0 && r.value < i) };
}
const Kt = ["innerHTML"], Qe = /* @__PURE__ */ Z({
  __name: "MarkdownView",
  props: {
    source: {}
  },
  setup(t) {
    const i = t, r = de(() => Ot(i.source));
    return (n, f) => (_(), L("div", {
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
    const r = t, n = i, f = se(), u = y(!1), b = y("edit"), s = y({ id: null, summary: "", content: "", tags: [] });
    let h = [];
    ye(
      () => r.visible,
      async (D) => {
        if (D)
          if (b.value = "edit", r.memoryId)
            try {
              const m = await Ce(f, r.memoryId);
              s.value = { id: m.id, summary: m.summary, content: m.content, tags: [...m.tags] }, h = [...m.tags];
            } catch (m) {
              O.error(m instanceof Error ? m.message : String(m)), n("update:visible", !1);
            }
          else
            s.value = { id: null, summary: "", content: "", tags: [] }, h = [];
      }
    );
    async function x() {
      u.value = !0;
      try {
        if (s.value.id) {
          const D = new Set(h), m = new Set(s.value.tags);
          await Ke(f, s.value.id, {
            summary: s.value.summary,
            content: s.value.content,
            add_tags: [...m].filter((M) => !D.has(M)),
            remove_tags: [...D].filter((M) => !m.has(M))
          }), O.success(o("editor.updated"));
        } else
          await Je(f, {
            summary: s.value.summary,
            content: s.value.content,
            tags: s.value.tags
          }), O.success(o("editor.created"));
        n("update:visible", !1), n("saved");
      } catch (D) {
        O.error(D instanceof Error ? D.message : String(D));
      } finally {
        u.value = !1;
      }
    }
    return (D, m) => {
      const M = he, C = $e, $ = Ve, E = be, V = Ue, N = xe, R = De, F = me, B = ze;
      return _(), I(B, {
        "model-value": t.visible,
        title: s.value.id ? e(o)("editor.editTitle") : e(o)("editor.createTitle"),
        width: t.width,
        "onUpdate:modelValue": m[5] || (m[5] = (k) => n("update:visible", k))
      }, {
        footer: l(() => [
          a(F, {
            onClick: m[4] || (m[4] = (k) => n("update:visible", !1))
          }, {
            default: l(() => [
              v(c(e(o)("common.cancel")), 1)
            ]),
            _: 1
          }),
          a(F, {
            type: "primary",
            loading: u.value,
            onClick: x
          }, {
            default: l(() => [
              v(c(e(o)("common.save")), 1)
            ]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: l(() => [
          a(R, { "label-position": "top" }, {
            default: l(() => [
              a(C, {
                label: e(o)("editor.summaryLabel")
              }, {
                default: l(() => [
                  a(M, {
                    modelValue: s.value.summary,
                    "onUpdate:modelValue": m[0] || (m[0] = (k) => s.value.summary = k),
                    maxlength: "512",
                    "show-word-limit": "",
                    placeholder: e(o)("editor.summaryPlaceholder")
                  }, null, 8, ["modelValue", "placeholder"])
                ]),
                _: 1
              }, 8, ["label"]),
              a(C, null, {
                label: l(() => [
                  z("div", Wt, [
                    z("span", null, c(e(o)("editor.contentLabel")), 1),
                    a(E, {
                      modelValue: b.value,
                      "onUpdate:modelValue": m[1] || (m[1] = (k) => b.value = k),
                      size: "small"
                    }, {
                      default: l(() => [
                        a($, { value: "edit" }, {
                          default: l(() => [
                            v(c(e(o)("editor.tabEdit")), 1)
                          ]),
                          _: 1
                        }),
                        a($, { value: "preview" }, {
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
                  b.value === "edit" ? (_(), I(M, {
                    key: 0,
                    modelValue: s.value.content,
                    "onUpdate:modelValue": m[2] || (m[2] = (k) => s.value.content = k),
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
              a(C, {
                label: e(o)("editor.tagsLabel")
              }, {
                default: l(() => [
                  a(N, {
                    modelValue: s.value.tags,
                    "onUpdate:modelValue": m[3] || (m[3] = (k) => s.value.tags = k),
                    multiple: "",
                    filterable: "",
                    "allow-create": "",
                    "default-first-option": "",
                    placeholder: e(o)("editor.tagsPlaceholder"),
                    class: "tags-select"
                  }, {
                    default: l(() => [
                      (_(!0), L(X, null, oe(t.tagOptions, (k) => (_(), I(V, {
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
}), ne = (t, i) => {
  const r = t.__vccOpts || t;
  for (const [n, f] of i)
    r[n] = f;
  return r;
}, Gt = /* @__PURE__ */ ne(Qt, [["__scopeId", "data-v-602bada0"]]), Xt = { class: "detail-summary" }, Yt = { class: "detail-tags" }, Zt = { class: "detail-toolbar" }, ea = {
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
    const r = t, n = i, f = se(), u = y(null), b = y("rendered");
    return ye(
      () => [r.visible, r.memoryId],
      async ([s]) => {
        if (!(!s || !r.memoryId)) {
          b.value = "rendered";
          try {
            u.value = await Ce(f, r.memoryId);
          } catch (h) {
            O.error(h instanceof Error ? h.message : String(h)), n("update:visible", !1);
          }
        }
      }
    ), (s, h) => {
      var $;
      const x = ke, D = it, m = Ve, M = be, C = st;
      return _(), I(C, {
        "model-value": t.visible,
        title: e(o)("drawer.title", { id: (($ = u.value) == null ? void 0 : $.id) ?? t.memoryId ?? "" }),
        size: t.size,
        "onUpdate:modelValue": h[1] || (h[1] = (E) => n("update:visible", E))
      }, {
        default: l(() => [
          u.value ? (_(), L(X, { key: 0 }, [
            z("h3", Xt, c(u.value.summary), 1),
            z("div", Yt, [
              (_(!0), L(X, null, oe(u.value.tags, (E) => (_(), I(x, {
                key: E,
                size: "small",
                class: "am-tag"
              }, {
                default: l(() => [
                  v(c(E), 1)
                ]),
                _: 2
              }, 1024))), 128))
            ]),
            a(D),
            z("div", Zt, [
              a(M, {
                modelValue: b.value,
                "onUpdate:modelValue": h[0] || (h[0] = (E) => b.value = E),
                size: "small"
              }, {
                default: l(() => [
                  a(m, { value: "rendered" }, {
                    default: l(() => [
                      v(c(e(o)("drawer.rendered")), 1)
                    ]),
                    _: 1
                  }),
                  a(m, { value: "source" }, {
                    default: l(() => [
                      v(c(e(o)("drawer.source")), 1)
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["modelValue"])
            ]),
            b.value === "rendered" ? (_(), I(Qe, {
              key: 0,
              source: u.value.content
            }, null, 8, ["source"])) : (_(), L("pre", ea, c(u.value.content), 1))
          ], 64)) : J("", !0)
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
      tagFilter: f,
      sort: u,
      order: b,
      page: s,
      pageSize: h,
      rows: x,
      searchResults: D,
      total: m,
      note: M,
      loading: C,
      tagOptions: $,
      searching: E,
      onSearch: V,
      reload: N,
      loadTagOptions: R,
      removeMemory: F
    } = Jt(), B = y(null), { compact: k } = Te(B), g = y(!1), w = y(null), P = y(!1), U = y(null);
    function T(A) {
      return A().catch((d) => O.error(d instanceof Error ? d.message : String(d)));
    }
    function K() {
      w.value = null, g.value = !0;
    }
    function j(A) {
      w.value = A, g.value = !0;
    }
    function W(A) {
      U.value = A, P.value = !0;
    }
    function Y() {
      T(N), R();
    }
    function Q(A) {
      const { prop: d, order: te } = A;
      te && (d === "updated_at" || d === "created_at") ? (u.value = d, b.value = te === "ascending" ? "asc" : "desc") : (u.value = "updated_at", b.value = "desc"), T(N);
    }
    async function ee(A) {
      try {
        await Be.confirm(o("memories.deleteConfirm", { id: A.id }), o("memories.deleteTitle"), {
          type: "warning"
        });
      } catch {
        return;
      }
      try {
        await F(A.id), O.success(o("memories.deleted"));
      } catch (d) {
        O.error(d instanceof Error ? d.message : String(d));
      }
    }
    return i({ refresh: () => T(N) }), (A, d) => {
      const te = pe, ge = we, S = me, G = he, Ge = Ue, Xe = xe, Ye = Le, H = Ae, Se = ke, Me = Ie, Ee = rt, Pe = Ne;
      return _(), L("div", {
        ref_key: "rootRef",
        ref: B,
        class: "am-panel"
      }, [
        t.showHeader ? (_(), L("div", oa, [
          z("div", la, [
            z("h2", na, c(r.title ?? e(o)("memories.title")), 1),
            a(ge, {
              content: r.subtitle ?? e(o)("memories.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(te, { class: "am-info" }, {
                  default: l(() => [
                    a(e(ce))
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
              v(c(e(o)("memories.create")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : J("", !0),
        z("div", sa, [
          a(G, {
            modelValue: e(n),
            "onUpdate:modelValue": d[1] || (d[1] = (p) => ae(n) ? n.value = p : null),
            placeholder: e(o)("memories.searchPlaceholder"),
            clearable: "",
            class: "search",
            onKeyup: d[2] || (d[2] = at((p) => T(e(V)), ["enter"])),
            onClear: d[3] || (d[3] = (p) => T(e(V)))
          }, {
            append: l(() => [
              a(S, {
                icon: e(Oe),
                onClick: d[0] || (d[0] = (p) => T(e(V)))
              }, null, 8, ["icon"])
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"]),
          a(Xe, {
            modelValue: e(f),
            "onUpdate:modelValue": d[4] || (d[4] = (p) => ae(f) ? f.value = p : null),
            placeholder: e(o)("memories.tagFilter"),
            clearable: "",
            filterable: "",
            class: "tag-filter",
            onChange: d[5] || (d[5] = (p) => T(e(V)))
          }, {
            default: l(() => [
              (_(!0), L(X, null, oe(e($), (p) => (_(), I(Ge, {
                key: p,
                label: p,
                value: p
              }, null, 8, ["label", "value"]))), 128))
            ]),
            _: 1
          }, 8, ["modelValue", "placeholder"])
        ]),
        e(M) ? (_(), I(Ye, {
          key: 1,
          title: e(M),
          type: "info",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : J("", !0),
        e(E) ? le((_(), I(Me, {
          key: 2,
          data: e(D)
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
                z("div", ia, c(p.summary), 1),
                z("div", {
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
                (_(!0), L(X, null, oe(p.tags, (q) => (_(), I(Se, {
                  key: q,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: l(() => [
                    v(c(q), 1)
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
                v(c(e(re)(p.updated_at)), 1)
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
                    v(c(e(o)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(S, {
                  link: "",
                  type: "primary",
                  onClick: (q) => j(p.id)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(S, {
                  link: "",
                  type: "danger",
                  onClick: (q) => ee(p)
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
          [Pe, e(C)]
        ]) : le((_(), I(Me, {
          key: 3,
          data: e(x),
          "default-sort": { prop: e(u), order: e(b) === "asc" ? "ascending" : "descending" },
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
                (_(!0), L(X, null, oe(p.tags, (q) => (_(), I(Se, {
                  key: q,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: l(() => [
                    v(c(q), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }, 8, ["label"]),
            e(k) ? J("", !0) : (_(), I(H, {
              key: 0,
              prop: "created_at",
              label: e(o)("memories.colCreatedAt"),
              width: "170",
              sortable: "custom"
            }, {
              default: l(({ row: p }) => [
                v(c(e(re)(p.created_at)), 1)
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
                v(c(e(re)(p.updated_at)), 1)
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
                    v(c(e(o)("common.detail")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(S, {
                  link: "",
                  type: "primary",
                  onClick: (q) => j(p.id)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(S, {
                  link: "",
                  type: "danger",
                  onClick: (q) => ee(p)
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
          [Pe, e(C)]
        ]),
        e(E) ? (_(), L("div", da, [
          a(Ee, {
            "current-page": e(s),
            "onUpdate:currentPage": d[10] || (d[10] = (p) => ae(s) ? s.value = p : null),
            "page-size": e(h),
            "onUpdate:pageSize": d[11] || (d[11] = (p) => ae(h) ? h.value = p : null),
            total: e(m),
            "page-sizes": [10, 20, 50],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: d[12] || (d[12] = (p) => T(e(N))),
            onSizeChange: d[13] || (d[13] = (p) => T(e(N)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])) : (_(), L("div", ca, [
          a(Ee, {
            "current-page": e(s),
            "onUpdate:currentPage": d[6] || (d[6] = (p) => ae(s) ? s.value = p : null),
            "page-size": e(h),
            "onUpdate:pageSize": d[7] || (d[7] = (p) => ae(h) ? h.value = p : null),
            total: e(m),
            "page-sizes": [20, 50, 100, 200],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: d[8] || (d[8] = (p) => T(e(N))),
            onSizeChange: d[9] || (d[9] = (p) => T(e(N)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])),
        a(Gt, {
          visible: g.value,
          "onUpdate:visible": d[14] || (d[14] = (p) => g.value = p),
          "memory-id": w.value,
          "tag-options": e($),
          width: e(k) ? "96%" : "640px",
          onSaved: Y
        }, null, 8, ["visible", "memory-id", "tag-options", "width"]),
        a(aa, {
          visible: P.value,
          "onUpdate:visible": d[15] || (d[15] = (p) => P.value = p),
          "memory-id": U.value,
          size: e(k) ? "100%" : "45%"
        }, null, 8, ["visible", "memory-id", "size"])
      ], 512);
    };
  }
}), ma = /* @__PURE__ */ ne(ua, [["__scopeId", "data-v-1ab68c72"]]);
function pa() {
  const t = se(), i = y([]), r = y(!1);
  async function n() {
    r.value = !0;
    try {
      const s = await We(t);
      i.value = s.tags ?? [];
    } finally {
      r.value = !1;
    }
  }
  async function f(s, h) {
    await Ht(t, s, h), await n();
  }
  async function u(s, h, x) {
    await qt(t, s, h, x), await n();
  }
  async function b(s, h) {
    await jt(t, s, h), await n();
  }
  return ue(() => {
    n().catch(() => {
    });
  }), { rows: i, loading: r, reload: n, create: f, rename: u, remove: b };
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
    const r = t, { rows: n, loading: f, reload: u, create: b, rename: s, remove: h } = pa(), x = y(null), { compact: D } = Te(x), m = y(!1), M = y(!1), C = ot({ oldName: null, name: "", newName: "", description: "" }), $ = y(!1), E = y("detach"), V = y(null);
    function N() {
      Object.assign(C, { oldName: null, name: "", newName: "", description: "" }), M.value = !0;
    }
    function R(g) {
      Object.assign(C, { oldName: g.name, name: g.name, newName: "", description: g.description ?? "" }), M.value = !0;
    }
    async function F() {
      m.value = !0;
      try {
        C.oldName ? (await s(C.oldName, C.newName, C.description), O.success(o("tags.saved"))) : (await b(C.name, C.description), O.success(o("tags.created"))), M.value = !1;
      } catch (g) {
        O.error(g instanceof Error ? g.message : String(g));
      } finally {
        m.value = !1;
      }
    }
    function B(g) {
      V.value = g, E.value = "detach", $.value = !0;
    }
    async function k() {
      var g, w;
      if (E.value === "purge")
        try {
          await Be.confirm(
            o("tags.purgeConfirm", { name: (g = V.value) == null ? void 0 : g.name, count: ((w = V.value) == null ? void 0 : w.memory_count) ?? 0 }),
            o("tags.purgeConfirmTitle"),
            { type: "error", confirmButtonText: o("tags.purgeButton") }
          );
        } catch {
          return;
        }
      if (V.value) {
        m.value = !0;
        try {
          await h(V.value.name, E.value), O.success(o("tags.deleted")), $.value = !1;
        } catch (P) {
          O.error(P instanceof Error ? P.message : String(P));
        } finally {
          m.value = !1;
        }
      }
    }
    return i({
      refresh: () => u().catch((g) => O.error(g instanceof Error ? g.message : String(g)))
    }), (g, w) => {
      const P = pe, U = we, T = me, K = ke, j = Ae, W = Ie, Y = he, Q = $e, ee = De, A = ze, d = ct, te = be, ge = Ne;
      return _(), L("div", {
        ref_key: "rootRef",
        ref: x,
        class: "am-panel"
      }, [
        t.showHeader ? (_(), L("div", fa, [
          z("div", ga, [
            z("h2", va, c(r.title ?? e(o)("tags.title")), 1),
            a(U, {
              content: r.subtitle ?? e(o)("tags.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(P, { class: "am-info" }, {
                  default: l(() => [
                    a(e(ce))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(T, {
            type: "primary",
            icon: e(Re),
            onClick: N
          }, {
            default: l(() => [
              v(c(e(o)("tags.create")), 1)
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
                    v(c(S.name), 1)
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
                v(c(S.description || "—"), 1)
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
                v(c(e(re)(S.last_used_at)), 1)
              ]),
              _: 1
            }, 8, ["label"]),
            a(j, {
              label: e(o)("memories.colActions"),
              width: "150",
              fixed: "right"
            }, {
              default: l(({ row: S }) => [
                a(T, {
                  link: "",
                  type: "primary",
                  onClick: (G) => R(S)
                }, {
                  default: l(() => [
                    v(c(e(o)("common.edit")), 1)
                  ]),
                  _: 1
                }, 8, ["onClick"]),
                a(T, {
                  link: "",
                  type: "danger",
                  onClick: (G) => B(S)
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
          [ge, e(f)]
        ]),
        a(A, {
          modelValue: M.value,
          "onUpdate:modelValue": w[4] || (w[4] = (S) => M.value = S),
          title: C.oldName ? e(o)("tags.editTitle") : e(o)("tags.createTitle"),
          width: e(D) ? "96%" : "480px"
        }, {
          footer: l(() => [
            a(T, {
              onClick: w[3] || (w[3] = (S) => M.value = !1)
            }, {
              default: l(() => [
                v(c(e(o)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(T, {
              type: "primary",
              loading: m.value,
              onClick: F
            }, {
              default: l(() => [
                v(c(e(o)("common.save")), 1)
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
                    a(Y, {
                      modelValue: C.name,
                      "onUpdate:modelValue": w[0] || (w[0] = (S) => C.name = S),
                      disabled: !!C.oldName,
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: e(o)("tags.namePlaceholder")
                    }, null, 8, ["modelValue", "disabled", "placeholder"])
                  ]),
                  _: 1
                }, 8, ["label"]),
                C.oldName ? (_(), I(Q, {
                  key: 0,
                  label: e(o)("tags.renameLabel")
                }, {
                  default: l(() => [
                    a(Y, {
                      modelValue: C.newName,
                      "onUpdate:modelValue": w[1] || (w[1] = (S) => C.newName = S),
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
                    a(Y, {
                      modelValue: C.description,
                      "onUpdate:modelValue": w[2] || (w[2] = (S) => C.description = S),
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
        a(A, {
          modelValue: $.value,
          "onUpdate:modelValue": w[7] || (w[7] = (S) => $.value = S),
          title: e(o)("tags.deleteTitle"),
          width: e(D) ? "96%" : "480px"
        }, {
          footer: l(() => [
            a(T, {
              onClick: w[6] || (w[6] = (S) => $.value = !1)
            }, {
              default: l(() => [
                v(c(e(o)("common.cancel")), 1)
              ]),
              _: 1
            }),
            a(T, {
              type: "danger",
              loading: m.value,
              onClick: k
            }, {
              default: l(() => [
                v(c(e(o)("common.delete")), 1)
              ]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: l(() => {
            var S;
            return [
              z("p", _a, [
                v(c(e(o)("tags.deleteBefore")) + " ", 1),
                a(K, null, {
                  default: l(() => {
                    var G;
                    return [
                      v(c((G = V.value) == null ? void 0 : G.name), 1)
                    ];
                  }),
                  _: 1
                }),
                v(" " + c(e(o)("tags.deleteMiddle")) + " ", 1),
                z("b", null, c((S = V.value) == null ? void 0 : S.memory_count), 1),
                v(" " + c(e(o)("tags.deleteAfter")), 1)
              ]),
              a(te, {
                modelValue: E.value,
                "onUpdate:modelValue": w[5] || (w[5] = (G) => E.value = G)
              }, {
                default: l(() => [
                  a(d, { value: "detach" }, {
                    default: l(() => [
                      v(c(e(o)("tags.detach")), 1)
                    ]),
                    _: 1
                  }),
                  a(d, { value: "purge" }, {
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
}), ha = /* @__PURE__ */ ne(ya, [["__scopeId", "data-v-9847d5f4"]]);
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
function Ta(t, i) {
  return t.post("/api/import", i);
}
function Sa() {
  const t = se(), i = y({}), r = y(""), n = y({ ok: !0, issues: [] }), f = y(!1), u = y(!1), b = y(!1), s = y(!1), h = de(() => It(i.value.file_size));
  async function x() {
    i.value = await ba(t), i.value.version = r.value;
  }
  async function D() {
    u.value = !0;
    try {
      n.value = await wa(t), f.value = !0;
    } finally {
      u.value = !1;
    }
  }
  async function m() {
    b.value = !0;
    try {
      const C = await Ca(t), $ = URL.createObjectURL(C), E = document.createElement("a");
      E.href = $, E.download = "agent-memory-export.json", E.click(), URL.revokeObjectURL($);
    } finally {
      b.value = !1;
    }
  }
  async function M(C) {
    s.value = !0;
    try {
      const $ = await C.text();
      let E;
      try {
        E = JSON.parse($);
      } catch {
        throw new Error(o("errors.invalidBackup"));
      }
      const V = await Ta(t, E);
      return await x(), V;
    } finally {
      s.value = !1;
    }
  }
  return ue(async () => {
    try {
      r.value = (await ka(t)).version ?? "";
    } catch {
    }
    await x().catch(() => {
    });
  }), {
    stats: i,
    version: r,
    doctor: n,
    doctorRan: f,
    doctorLoading: u,
    exporting: b,
    importing: s,
    sizeText: h,
    reload: x,
    runDoctor: D,
    exportData: m,
    importFile: M
  };
}
const Ma = {
  key: 0,
  class: "am-panel-header"
}, Ea = { class: "am-heading" }, Pa = { class: "am-panel-title" }, za = { class: "actions" }, Da = { class: "card-header" }, $a = {
  key: 2,
  class: "issues"
}, Va = /* @__PURE__ */ Z({
  __name: "OpsPanel",
  props: {
    showHeader: { type: Boolean, default: !0 },
    title: {},
    subtitle: {}
  },
  setup(t, { expose: i }) {
    const r = t, {
      stats: n,
      version: f,
      doctor: u,
      doctorRan: b,
      doctorLoading: s,
      exporting: h,
      importing: x,
      sizeText: D,
      reload: m,
      runDoctor: M,
      exportData: C,
      importFile: $
    } = Sa(), E = y(null), { compact: V } = Te(E), N = y(null);
    function R(B) {
      return B().catch((k) => O.error(k instanceof Error ? k.message : String(k)));
    }
    function F(B) {
      var w;
      const k = B.target, g = (w = k.files) == null ? void 0 : w[0];
      k.value = "", g && $(g).then((P) => {
        O.success(o("ops.imported", { memories: P.imported_memories, tags: P.imported_tags }));
      }).catch((P) => {
        O.error(P instanceof Error ? P.message : String(P));
      });
    }
    return i({ refresh: () => R(m) }), (B, k) => {
      const g = pe, w = we, P = me, U = pt, T = mt, K = ut, j = dt, W = gt, Y = ft, Q = Le, ee = vt;
      return _(), L("div", {
        ref_key: "rootRef",
        ref: E,
        class: "am-panel"
      }, [
        t.showHeader ? (_(), L("div", Ma, [
          z("div", Ea, [
            z("h2", Pa, c(r.title ?? e(o)("ops.title")), 1),
            a(w, {
              content: r.subtitle ?? e(o)("ops.subtitle"),
              placement: "top"
            }, {
              default: l(() => [
                a(g, { class: "am-info" }, {
                  default: l(() => [
                    a(e(ce))
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["content"])
          ]),
          a(P, {
            icon: e(Tt),
            onClick: k[0] || (k[0] = (A) => R(e(m)))
          }, {
            default: l(() => [
              v(c(e(o)("ops.refresh")), 1)
            ]),
            _: 1
          }, 8, ["icon"])
        ])) : J("", !0),
        a(j, { gutter: 14 }, {
          default: l(() => [
            a(K, {
              span: e(V) ? 12 : 8
            }, {
              default: l(() => [
                a(T, { shadow: "never" }, {
                  default: l(() => [
                    a(U, {
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
              span: e(V) ? 12 : 8
            }, {
              default: l(() => [
                a(T, { shadow: "never" }, {
                  default: l(() => [
                    a(U, {
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
              span: e(V) ? 12 : 8
            }, {
              default: l(() => [
                a(T, { shadow: "never" }, {
                  default: l(() => [
                    a(U, {
                      title: e(o)("ops.statSize"),
                      value: e(D)
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
        a(T, { shadow: "never" }, {
          header: l(() => [
            v(c(e(o)("ops.dbCard")), 1)
          ]),
          default: l(() => [
            a(Y, {
              column: e(V) ? 1 : 2,
              border: ""
            }, {
              default: l(() => [
                a(W, {
                  label: e(o)("ops.path")
                }, {
                  default: l(() => [
                    v(c(e(n).path ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(W, {
                  label: e(o)("ops.schemaVersion")
                }, {
                  default: l(() => [
                    v(c(e(n).schema_version ?? "—"), 1)
                  ]),
                  _: 1
                }, 8, ["label"]),
                a(W, {
                  label: e(o)("ops.version")
                }, {
                  default: l(() => [
                    v(c(e(n).version ?? e(f)), 1)
                  ]),
                  _: 1
                }, 8, ["label"])
              ]),
              _: 1
            }, 8, ["column"]),
            z("div", za, [
              a(P, {
                icon: e(St),
                loading: e(h),
                onClick: k[1] || (k[1] = (A) => R(e(C)))
              }, {
                default: l(() => [
                  v(c(e(o)("ops.export")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(P, {
                icon: e(Mt),
                loading: e(x),
                onClick: k[2] || (k[2] = (A) => {
                  var d;
                  return (d = N.value) == null ? void 0 : d.click();
                })
              }, {
                default: l(() => [
                  v(c(e(o)("ops.import")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"]),
              a(w, {
                content: e(o)("ops.importHint"),
                placement: "top"
              }, {
                default: l(() => [
                  a(g, { class: "am-info" }, {
                    default: l(() => [
                      a(e(ce))
                    ]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["content"]),
              z("input", {
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
        a(T, { shadow: "never" }, {
          header: l(() => [
            z("div", Da, [
              z("span", null, c(e(o)("ops.doctorCard")), 1),
              a(P, {
                size: "small",
                icon: e(Oe),
                loading: e(s),
                onClick: k[3] || (k[3] = (A) => R(e(M)))
              }, {
                default: l(() => [
                  v(c(e(o)("ops.runDoctor")), 1)
                ]),
                _: 1
              }, 8, ["icon", "loading"])
            ])
          ]),
          default: l(() => [
            e(b) ? (_(), L(X, { key: 0 }, [
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
                (_(!0), L(X, null, oe(e(u).issues, (A, d) => (_(), L("li", { key: d }, c(A), 1))), 128))
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
}), xa = /* @__PURE__ */ ne(Va, [["__scopeId", "data-v-27c1e600"]]), Ua = { class: "memory-ui" }, La = { class: "brand" }, Ia = { class: "brand-mark" }, Aa = { class: "aside-footer" }, Na = /* @__PURE__ */ Z({
  __name: "MemoryAdmin",
  props: {
    layout: { default: "sidebar" },
    title: { default: "agent-memory" }
  },
  setup(t) {
    const i = y("memories");
    return (r, n) => {
      const f = pe, u = bt, b = ht, s = yt, h = Ct, x = wt, D = kt, m = _t;
      return _(), L("div", Ua, [
        a(m, { class: "layout" }, {
          default: l(() => [
            t.layout === "sidebar" ? (_(), I(s, {
              key: 0,
              width: "200px",
              class: "aside"
            }, {
              default: l(() => [
                z("div", La, [
                  z("span", Ia, [
                    a(f, { size: 16 }, {
                      default: l(() => [
                        a(e(Et))
                      ]),
                      _: 1
                    })
                  ]),
                  z("span", null, c(t.title), 1)
                ]),
                a(b, {
                  "default-active": i.value,
                  class: "menu",
                  onSelect: n[0] || (n[0] = (M) => i.value = M)
                }, {
                  default: l(() => [
                    a(u, { index: "memories" }, {
                      default: l(() => [
                        a(f, null, {
                          default: l(() => [
                            a(e(Pt))
                          ]),
                          _: 1
                        }),
                        z("span", null, c(e(o)("nav.memories")), 1)
                      ]),
                      _: 1
                    }),
                    a(u, { index: "tags" }, {
                      default: l(() => [
                        a(f, null, {
                          default: l(() => [
                            a(e(zt))
                          ]),
                          _: 1
                        }),
                        z("span", null, c(e(o)("nav.tags")), 1)
                      ]),
                      _: 1
                    }),
                    a(u, { index: "ops" }, {
                      default: l(() => [
                        a(f, null, {
                          default: l(() => [
                            a(e(Dt))
                          ]),
                          _: 1
                        }),
                        z("span", null, c(e(o)("nav.ops")), 1)
                      ]),
                      _: 1
                    })
                  ]),
                  _: 1
                }, 8, ["default-active"]),
                z("div", Aa, [
                  lt(r.$slots, "footer", {}, void 0, !0)
                ])
              ]),
              _: 3
            })) : J("", !0),
            a(D, { class: "main" }, {
              default: l(() => [
                t.layout === "tabs" ? (_(), I(x, {
                  key: 0,
                  modelValue: i.value,
                  "onUpdate:modelValue": n[1] || (n[1] = (M) => i.value = M),
                  class: "tabs-bar"
                }, {
                  default: l(() => [
                    a(h, {
                      label: e(o)("nav.memories"),
                      name: "memories"
                    }, null, 8, ["label"]),
                    a(h, {
                      label: e(o)("nav.tags"),
                      name: "tags"
                    }, null, 8, ["label"]),
                    a(h, {
                      label: e(o)("nav.ops"),
                      name: "ops"
                    }, null, 8, ["label"])
                  ]),
                  _: 1
                }, 8, ["modelValue"])) : J("", !0),
                le(a(ma, null, null, 512), [
                  [ve, i.value === "memories"]
                ]),
                le(a(ha, null, null, 512), [
                  [ve, i.value === "tags"]
                ]),
                le(a(xa, null, null, 512), [
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
}), Wa = /* @__PURE__ */ ne(Na, [["__scopeId", "data-v-6f56cb0b"]]);
export {
  Qe as MarkdownView,
  ma as MemoriesPanel,
  Wa as MemoryAdmin,
  aa as MemoryDetailDrawer,
  Gt as MemoryEditorDialog,
  qe as MemoryUIConfigKey,
  xa as OpsPanel,
  ha as TagsPanel,
  Lt as applyMemoryUILocalePreference,
  Nt as buildMemoriesQuery,
  At as createApiClient,
  Ja as currentMemoryUILocale,
  It as formatSize,
  re as formatTime,
  _e as isSearchMode,
  fe as memoryUIi18n,
  Ka as provideMemoryUI,
  Ot as renderMarkdown,
  Bt as sanitizeHtml,
  Ut as setMemoryUILocale,
  o as t,
  se as useApiClient,
  Jt as useMemories,
  je as useMemoryConfig,
  Sa as useOps,
  pa as useTags
};
