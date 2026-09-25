import { provide as qe, inject as je, ref as y, computed as oe, onMounted as ae, watch as me, onUnmounted as He, defineComponent as X, openBlock as b, createElementBlock as O, createBlock as I, withCtx as l, createVNode as e, createElementVNode as k, createTextVNode as v, Fragment as G, renderList as Y, toDisplayString as R, createCommentVNode as W, unref as n, withKeys as Je, isRef as Q, withDirectives as Z, reactive as Ae, renderSlot as Ke, vShow as re } from "vue";
import { ElDialog as ke, ElForm as $e, ElFormItem as Ce, ElInput as pe, ElRadioGroup as ce, ElRadioButton as Ve, ElSelect as Ee, ElOption as Se, ElButton as se, ElDrawer as Qe, ElTag as fe, ElDivider as Ge, ElAlert as xe, ElTable as ze, ElTableColumn as Ue, ElLoadingDirective as Me, ElPagination as We, ElRadio as Xe, ElRow as Ye, ElCol as Ze, ElCard as et, ElStatistic as tt, ElDescriptions as lt, ElDescriptionsItem as nt, ElEmpty as ot, ElContainer as at, ElAside as st, ElIcon as it, ElMenu as rt, ElMenuItem as ut, ElText as dt, ElMain as mt, ElTabs as pt, ElTabPane as ct } from "element-plus/es";
import { Plus as Te, Search as Ie, Refresh as ft, Download as vt, UploadFilled as gt, Collection as _t, Notebook as yt, PriceTag as bt, Odometer as wt } from "@element-plus/icons-vue";
import { ElMessage as B, ElMessageBox as Re } from "element-plus";
import { Marked as kt } from "marked";
import De from "dompurify";
const Ne = Symbol("memory-ui-config"), ue = {
  baseUrl: "",
  fetch: (...t) => globalThis.fetch(...t),
  defaultPageSize: 20
};
function Cl(t) {
  qe(Ne, t);
}
function Oe() {
  const t = je(Ne);
  return {
    baseUrl: ((t == null ? void 0 : t.baseUrl) ?? ue.baseUrl).replace(/\/+$/, ""),
    fetch: (t == null ? void 0 : t.fetch) ?? ue.fetch,
    defaultPageSize: (t == null ? void 0 : t.defaultPageSize) ?? ue.defaultPageSize
  };
}
function te(t) {
  return t ? new Date(t * 1e3).toLocaleString("zh-CN", { hour12: !1 }) : "—";
}
function $t(t) {
  return t == null ? "—" : t < 1024 ? `${t} B` : t < 1024 * 1024 ? `${(t / 1024).toFixed(1)} KB` : `${(t / 1024 / 1024).toFixed(2)} MB`;
}
function Ct(t) {
  async function i(a, f = {}) {
    const p = await t.fetch(t.baseUrl + a, {
      headers: { "Content-Type": "application/json" },
      ...f
    }), _ = await p.text();
    let d = null;
    try {
      d = _ ? JSON.parse(_) : null;
    } catch {
      d = null;
    }
    if (!p.ok) {
      const w = (d == null ? void 0 : d.error) ?? `请求失败 (HTTP ${p.status})`;
      throw new Error(w);
    }
    return d;
  }
  async function u(a) {
    var p;
    const f = await t.fetch(t.baseUrl + a);
    if (!f.ok) {
      const _ = await f.text().catch(() => "");
      let d = `请求失败 (HTTP ${f.status})`;
      try {
        d = ((p = JSON.parse(_)) == null ? void 0 : p.error) ?? d;
      } catch {
      }
      throw new Error(d);
    }
    return f.blob();
  }
  return {
    get: (a) => i(a),
    post: (a, f) => i(a, { method: "POST", body: JSON.stringify(f ?? {}) }),
    put: (a, f) => i(a, { method: "PUT", body: JSON.stringify(f ?? {}) }),
    del: (a) => i(a, { method: "DELETE" }),
    getBlob: u
  };
}
function le() {
  return Ct(Oe());
}
function Vt(t) {
  const i = de(t.query), u = new URLSearchParams();
  return i ? (u.set("query", t.query.trim()), t.tagFilter && u.set("tags", t.tagFilter)) : (t.tagFilter && u.set("tag", t.tagFilter), u.set("sort", t.sort), u.set("order", t.order)), u.set("offset", String((t.page - 1) * t.pageSize)), u.set("limit", String(t.pageSize)), u.toString();
}
function de(t) {
  return t.trim().length > 0;
}
const Et = new kt();
function St(t) {
  const i = Et.parse(t, { async: !1 });
  return De.sanitize(i);
}
function xt(t) {
  return De.sanitize(t);
}
function ve(t, i) {
  return t.get(`/api/memories/${encodeURIComponent(i)}`);
}
function Pe(t, i) {
  return t.post("/api/memories", i);
}
function he(t, i, u) {
  return t.put(`/api/memories/${encodeURIComponent(i)}`, u);
}
function zt(t, i) {
  return t.del(`/api/memories/${encodeURIComponent(i)}`);
}
function Be(t) {
  return t.get("/api/tags");
}
function Ut(t, i, u) {
  return t.post("/api/tags", { name: i, description: u });
}
function Mt(t, i, u, a) {
  const f = { description: a }, p = u.trim();
  return p && p !== i && (f.new_name = p), t.put(`/api/tags/${encodeURIComponent(i)}`, f);
}
function Tt(t, i, u) {
  return t.del(`/api/tags/${encodeURIComponent(i)}?mode=${u}`);
}
function It() {
  const t = le(), { defaultPageSize: i } = Oe(), u = y(""), a = y(""), f = y("updated_at"), p = y("desc"), _ = y(1), d = y(i), w = y([]), S = y([]), $ = y(0), s = y(""), x = y(!1), z = y([]), C = oe(() => de(u.value));
  function V() {
    return _.value = 1, D();
  }
  function F() {
    return Vt({
      query: u.value,
      tagFilter: a.value,
      sort: f.value,
      order: p.value,
      page: _.value,
      pageSize: d.value
    });
  }
  let L = 0;
  async function D() {
    var P;
    const c = ++L;
    x.value = !0;
    try {
      const U = F();
      if (de(u.value)) {
        const T = await t.get(`/api/memories?${U}`);
        if (c !== L) return;
        S.value = (T.results ?? []).map((N) => ({ ...N, snippet: xt(N.snippet) })), $.value = T.total_matches ?? 0, s.value = "";
      } else {
        const T = await t.get(`/api/memories?${U}`);
        if (c !== L) return;
        const N = Math.max(1, Math.ceil(T.total / d.value));
        if (((P = T.memories) == null ? void 0 : P.length) === 0 && T.total > 0 && _.value > N)
          return _.value = N, x.value = !1, D();
        w.value = T.memories ?? [], $.value = T.total ?? 0, s.value = T.note ?? "";
      }
    } finally {
      c === L && (x.value = !1);
    }
  }
  async function g() {
    try {
      const c = await Be(t);
      z.value = (c.tags ?? []).map((P) => P.name);
    } catch {
    }
  }
  async function E(c) {
    if (c.id) {
      const P = await ve(t, c.id), U = new Set(P.tags), T = new Set(c.tags);
      await he(t, c.id, {
        summary: c.summary,
        content: c.content,
        add_tags: [...T].filter((N) => !U.has(N)),
        remove_tags: [...U].filter((N) => !T.has(N))
      });
    } else
      await Pe(t, { summary: c.summary, content: c.content, tags: c.tags });
    await Promise.all([D(), g()]);
  }
  async function o(c) {
    await zt(t, c), await D();
  }
  return ae(() => {
    D().catch(() => {
    }), g();
  }), {
    query: u,
    tagFilter: a,
    sort: f,
    order: p,
    page: _,
    pageSize: d,
    rows: w,
    searchResults: S,
    total: $,
    note: s,
    loading: x,
    tagOptions: z,
    searching: C,
    onSearch: V,
    reload: D,
    loadTagOptions: g,
    saveMemory: E,
    removeMemory: o
  };
}
function ge(t, i = 720) {
  const u = y(0);
  let a = null;
  function f(p) {
    a == null || a.disconnect(), a = null, !(!p || typeof ResizeObserver > "u") && (a = new ResizeObserver((_) => {
      var d;
      u.value = ((d = _[0]) == null ? void 0 : d.contentRect.width) ?? 0;
    }), a.observe(p));
  }
  return ae(() => f(t.value)), me(t, (p) => f(p)), He(() => a == null ? void 0 : a.disconnect()), { width: u, compact: oe(() => u.value > 0 && u.value < i) };
}
const Rt = ["innerHTML"], Fe = /* @__PURE__ */ X({
  __name: "MarkdownView",
  props: {
    source: {}
  },
  setup(t) {
    const i = t, u = oe(() => St(i.source));
    return (a, f) => (b(), O("div", {
      class: "md-body",
      innerHTML: u.value
    }, null, 8, Rt));
  }
}), Dt = { class: "content-label" }, Nt = /* @__PURE__ */ X({
  __name: "MemoryEditorDialog",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    tagOptions: {},
    width: {}
  },
  emits: ["update:visible", "saved"],
  setup(t, { emit: i }) {
    const u = t, a = i, f = le(), p = y(!1), _ = y("edit"), d = y({ id: null, summary: "", content: "", tags: [] });
    let w = [];
    me(
      () => u.visible,
      async ($) => {
        if ($)
          if (_.value = "edit", u.memoryId)
            try {
              const s = await ve(f, u.memoryId);
              d.value = { id: s.id, summary: s.summary, content: s.content, tags: [...s.tags] }, w = [...s.tags];
            } catch (s) {
              B.error(s instanceof Error ? s.message : String(s)), a("update:visible", !1);
            }
          else
            d.value = { id: null, summary: "", content: "", tags: [] }, w = [];
      }
    );
    async function S() {
      p.value = !0;
      try {
        if (d.value.id) {
          const $ = new Set(w), s = new Set(d.value.tags);
          await he(f, d.value.id, {
            summary: d.value.summary,
            content: d.value.content,
            add_tags: [...s].filter((x) => !$.has(x)),
            remove_tags: [...$].filter((x) => !s.has(x))
          }), B.success("已更新");
        } else
          await Pe(f, {
            summary: d.value.summary,
            content: d.value.content,
            tags: d.value.tags
          }), B.success("已创建");
        a("update:visible", !1), a("saved");
      } catch ($) {
        B.error($ instanceof Error ? $.message : String($));
      } finally {
        p.value = !1;
      }
    }
    return ($, s) => {
      const x = pe, z = Ce, C = Ve, V = ce, F = Se, L = Ee, D = $e, g = se, E = ke;
      return b(), I(E, {
        "model-value": t.visible,
        title: d.value.id ? "编辑记忆" : "新建记忆",
        width: t.width,
        "onUpdate:modelValue": s[5] || (s[5] = (o) => a("update:visible", o))
      }, {
        footer: l(() => [
          e(g, {
            onClick: s[4] || (s[4] = (o) => a("update:visible", !1))
          }, {
            default: l(() => [...s[9] || (s[9] = [
              v("取消", -1)
            ])]),
            _: 1
          }),
          e(g, {
            type: "primary",
            loading: p.value,
            onClick: S
          }, {
            default: l(() => [...s[10] || (s[10] = [
              v("保存", -1)
            ])]),
            _: 1
          }, 8, ["loading"])
        ]),
        default: l(() => [
          e(D, { "label-position": "top" }, {
            default: l(() => [
              e(z, { label: "摘要（列表与搜索展示的一行简介）" }, {
                default: l(() => [
                  e(x, {
                    modelValue: d.value.summary,
                    "onUpdate:modelValue": s[0] || (s[0] = (o) => d.value.summary = o),
                    maxlength: "512",
                    "show-word-limit": "",
                    placeholder: "精确、自洽的一句话"
                  }, null, 8, ["modelValue"])
                ]),
                _: 1
              }),
              e(z, null, {
                label: l(() => [
                  k("div", Dt, [
                    s[8] || (s[8] = k("span", null, "正文（Markdown）", -1)),
                    e(V, {
                      modelValue: _.value,
                      "onUpdate:modelValue": s[1] || (s[1] = (o) => _.value = o),
                      size: "small"
                    }, {
                      default: l(() => [
                        e(C, { value: "edit" }, {
                          default: l(() => [...s[6] || (s[6] = [
                            v("编辑", -1)
                          ])]),
                          _: 1
                        }),
                        e(C, { value: "preview" }, {
                          default: l(() => [...s[7] || (s[7] = [
                            v("预览", -1)
                          ])]),
                          _: 1
                        })
                      ]),
                      _: 1
                    }, 8, ["modelValue"])
                  ])
                ]),
                default: l(() => [
                  _.value === "edit" ? (b(), I(x, {
                    key: 0,
                    modelValue: d.value.content,
                    "onUpdate:modelValue": s[2] || (s[2] = (o) => d.value.content = o),
                    type: "textarea",
                    rows: 12,
                    maxlength: "200000",
                    "show-word-limit": "",
                    placeholder: "支持 Markdown：标题、列表、代码块、表格……"
                  }, null, 8, ["modelValue"])) : (b(), I(Fe, {
                    key: 1,
                    class: "content-preview",
                    source: d.value.content
                  }, null, 8, ["source"]))
                ]),
                _: 1
              }),
              e(z, { label: "标签（回车添加，可新建）" }, {
                default: l(() => [
                  e(L, {
                    modelValue: d.value.tags,
                    "onUpdate:modelValue": s[3] || (s[3] = (o) => d.value.tags = o),
                    multiple: "",
                    filterable: "",
                    "allow-create": "",
                    "default-first-option": "",
                    placeholder: "选择或输入标签",
                    class: "tags-select"
                  }, {
                    default: l(() => [
                      (b(!0), O(G, null, Y(t.tagOptions, (o) => (b(), I(F, {
                        key: o,
                        label: o,
                        value: o
                      }, null, 8, ["label", "value"]))), 128))
                    ]),
                    _: 1
                  }, 8, ["modelValue"])
                ]),
                _: 1
              })
            ]),
            _: 1
          })
        ]),
        _: 1
      }, 8, ["model-value", "title", "width"]);
    };
  }
}), ne = (t, i) => {
  const u = t.__vccOpts || t;
  for (const [a, f] of i)
    u[a] = f;
  return u;
}, Ot = /* @__PURE__ */ ne(Nt, [["__scopeId", "data-v-f3ff571d"]]), Pt = { class: "detail-summary" }, ht = { class: "detail-tags" }, Bt = { class: "detail-toolbar" }, Ft = {
  key: 1,
  class: "detail-content"
}, Lt = /* @__PURE__ */ X({
  __name: "MemoryDetailDrawer",
  props: {
    visible: { type: Boolean },
    memoryId: {},
    size: {}
  },
  emits: ["update:visible"],
  setup(t, { emit: i }) {
    const u = t, a = i, f = le(), p = y(null), _ = y("rendered");
    return me(
      () => [u.visible, u.memoryId],
      async ([d]) => {
        if (!(!d || !u.memoryId)) {
          _.value = "rendered";
          try {
            p.value = await ve(f, u.memoryId);
          } catch (w) {
            B.error(w instanceof Error ? w.message : String(w)), a("update:visible", !1);
          }
        }
      }
    ), (d, w) => {
      var C;
      const S = fe, $ = Ge, s = Ve, x = ce, z = Qe;
      return b(), I(z, {
        "model-value": t.visible,
        title: `记忆 ${((C = p.value) == null ? void 0 : C.id) ?? t.memoryId ?? ""}`,
        size: t.size,
        "onUpdate:modelValue": w[1] || (w[1] = (V) => a("update:visible", V))
      }, {
        default: l(() => [
          p.value ? (b(), O(G, { key: 0 }, [
            k("h3", Pt, R(p.value.summary), 1),
            k("div", ht, [
              (b(!0), O(G, null, Y(p.value.tags, (V) => (b(), I(S, {
                key: V,
                size: "small",
                class: "am-tag"
              }, {
                default: l(() => [
                  v(R(V), 1)
                ]),
                _: 2
              }, 1024))), 128))
            ]),
            e($),
            k("div", Bt, [
              e(x, {
                modelValue: _.value,
                "onUpdate:modelValue": w[0] || (w[0] = (V) => _.value = V),
                size: "small"
              }, {
                default: l(() => [
                  e(s, { value: "rendered" }, {
                    default: l(() => [...w[2] || (w[2] = [
                      v("渲染", -1)
                    ])]),
                    _: 1
                  }),
                  e(s, { value: "source" }, {
                    default: l(() => [...w[3] || (w[3] = [
                      v("源码", -1)
                    ])]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["modelValue"])
            ]),
            _.value === "rendered" ? (b(), I(Fe, {
              key: 0,
              source: p.value.content
            }, null, 8, ["source"])) : (b(), O("pre", Ft, R(p.value.content), 1))
          ], 64)) : W("", !0)
        ]),
        _: 1
      }, 8, ["model-value", "title", "size"]);
    };
  }
}), qt = /* @__PURE__ */ ne(Lt, [["__scopeId", "data-v-c800b468"]]), jt = { class: "am-panel-header" }, Ht = { class: "am-toolbar" }, Jt = { class: "am-summary" }, At = ["innerHTML"], Kt = {
  key: 3,
  class: "am-pager"
}, Qt = {
  key: 4,
  class: "am-pager"
}, Gt = /* @__PURE__ */ X({
  __name: "MemoriesPanel",
  setup(t) {
    const {
      query: i,
      tagFilter: u,
      sort: a,
      order: f,
      page: p,
      pageSize: _,
      rows: d,
      searchResults: w,
      total: S,
      note: $,
      loading: s,
      tagOptions: x,
      searching: z,
      onSearch: C,
      reload: V,
      loadTagOptions: F,
      removeMemory: L
    } = It(), D = y(null), { compact: g } = ge(D), E = y(!1), o = y(null), c = y(!1), P = y(null);
    function U(h) {
      return h().catch((r) => B.error(r instanceof Error ? r.message : String(r)));
    }
    function T() {
      o.value = null, E.value = !0;
    }
    function N(h) {
      o.value = h, E.value = !0;
    }
    function A(h) {
      P.value = h, c.value = !0;
    }
    function ee() {
      U(V), F();
    }
    async function H(h) {
      try {
        await Re.confirm(`确定永久删除记忆 ${h.id}？`, "删除确认", { type: "warning" });
      } catch {
        return;
      }
      try {
        await L(h.id), B.success("已删除");
      } catch (r) {
        B.error(r instanceof Error ? r.message : String(r));
      }
    }
    return (h, r) => {
      const K = se, M = pe, q = Se, ie = Ee, Le = xe, j = Ue, _e = fe, ye = ze, be = We, we = Me;
      return b(), O("div", {
        ref_key: "rootRef",
        ref: D,
        class: "am-panel"
      }, [
        k("div", jt, [
          r[21] || (r[21] = k("div", null, [
            k("h2", { class: "am-panel-title" }, "记忆管理"),
            k("p", { class: "am-panel-subtitle" }, "多 agent 共享的记忆库，正文支持 Markdown")
          ], -1)),
          e(K, {
            type: "primary",
            icon: n(Te),
            onClick: T
          }, {
            default: l(() => [...r[20] || (r[20] = [
              v("新建记忆", -1)
            ])]),
            _: 1
          }, 8, ["icon"])
        ]),
        k("div", Ht, [
          e(M, {
            modelValue: n(i),
            "onUpdate:modelValue": r[1] || (r[1] = (m) => Q(i) ? i.value = m : null),
            placeholder: "关键词搜索（空格分隔、全部命中；中文按子串匹配）",
            clearable: "",
            class: "search",
            onKeyup: r[2] || (r[2] = Je((m) => U(n(C)), ["enter"])),
            onClear: r[3] || (r[3] = (m) => U(n(C)))
          }, {
            append: l(() => [
              e(K, {
                icon: n(Ie),
                onClick: r[0] || (r[0] = (m) => U(n(C)))
              }, null, 8, ["icon"])
            ]),
            _: 1
          }, 8, ["modelValue"]),
          e(ie, {
            modelValue: n(u),
            "onUpdate:modelValue": r[4] || (r[4] = (m) => Q(u) ? u.value = m : null),
            placeholder: "按标签过滤",
            clearable: "",
            filterable: "",
            class: "tag-filter",
            onChange: r[5] || (r[5] = (m) => U(n(C)))
          }, {
            default: l(() => [
              (b(!0), O(G, null, Y(n(x), (m) => (b(), I(q, {
                key: m,
                label: m,
                value: m
              }, null, 8, ["label", "value"]))), 128))
            ]),
            _: 1
          }, 8, ["modelValue"]),
          e(ie, {
            modelValue: n(a),
            "onUpdate:modelValue": r[6] || (r[6] = (m) => Q(a) ? a.value = m : null),
            class: "sort",
            onChange: r[7] || (r[7] = (m) => U(n(V)))
          }, {
            default: l(() => [
              e(q, {
                label: "按更新时间",
                value: "updated_at"
              }),
              e(q, {
                label: "按创建时间",
                value: "created_at"
              })
            ]),
            _: 1
          }, 8, ["modelValue"]),
          e(ie, {
            modelValue: n(f),
            "onUpdate:modelValue": r[8] || (r[8] = (m) => Q(f) ? f.value = m : null),
            class: "order",
            onChange: r[9] || (r[9] = (m) => U(n(V)))
          }, {
            default: l(() => [
              e(q, {
                label: "倒序",
                value: "desc"
              }),
              e(q, {
                label: "正序",
                value: "asc"
              })
            ]),
            _: 1
          }, 8, ["modelValue"])
        ]),
        n($) ? (b(), I(Le, {
          key: 0,
          title: n($),
          type: "info",
          "show-icon": "",
          closable: !1
        }, null, 8, ["title"])) : W("", !0),
        n(z) ? Z((b(), I(ye, {
          key: 1,
          data: n(w)
        }, {
          default: l(() => [
            e(j, {
              prop: "id",
              label: "ID",
              width: "80"
            }),
            e(j, { label: "摘要" }, {
              default: l(({ row: m }) => [
                k("div", Jt, R(m.summary), 1),
                k("div", {
                  class: "am-snippet",
                  innerHTML: m.snippet
                }, null, 8, At)
              ]),
              _: 1
            }),
            e(j, {
              label: "标签",
              width: "220"
            }, {
              default: l(({ row: m }) => [
                (b(!0), O(G, null, Y(m.tags, (J) => (b(), I(_e, {
                  key: J,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: l(() => [
                    v(R(J), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }),
            e(j, {
              prop: "score",
              label: "评分",
              width: "80",
              sortable: ""
            }),
            e(j, {
              label: "更新时间",
              width: "170"
            }, {
              default: l(({ row: m }) => [
                v(R(n(te)(m.updated_at)), 1)
              ]),
              _: 1
            }),
            e(j, {
              label: "操作",
              width: "160",
              fixed: "right"
            }, {
              default: l(({ row: m }) => [
                e(K, {
                  link: "",
                  type: "primary",
                  onClick: (J) => A(m.id)
                }, {
                  default: l(() => [...r[22] || (r[22] = [
                    v("详情", -1)
                  ])]),
                  _: 1
                }, 8, ["onClick"]),
                e(K, {
                  link: "",
                  type: "primary",
                  onClick: (J) => N(m.id)
                }, {
                  default: l(() => [...r[23] || (r[23] = [
                    v("编辑", -1)
                  ])]),
                  _: 1
                }, 8, ["onClick"]),
                e(K, {
                  link: "",
                  type: "danger",
                  onClick: (J) => H(m)
                }, {
                  default: l(() => [...r[24] || (r[24] = [
                    v("删除", -1)
                  ])]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            })
          ]),
          _: 1
        }, 8, ["data"])), [
          [we, n(s)]
        ]) : Z((b(), I(ye, {
          key: 2,
          data: n(d)
        }, {
          default: l(() => [
            e(j, {
              prop: "id",
              label: "ID",
              width: "80"
            }),
            e(j, {
              prop: "summary",
              label: "摘要",
              "min-width": "300",
              "show-overflow-tooltip": ""
            }),
            e(j, {
              label: "标签",
              width: "220"
            }, {
              default: l(({ row: m }) => [
                (b(!0), O(G, null, Y(m.tags, (J) => (b(), I(_e, {
                  key: J,
                  size: "small",
                  class: "am-tag"
                }, {
                  default: l(() => [
                    v(R(J), 1)
                  ]),
                  _: 2
                }, 1024))), 128))
              ]),
              _: 1
            }),
            n(g) ? W("", !0) : (b(), I(j, {
              key: 0,
              prop: "created_at",
              label: "创建时间",
              width: "170"
            }, {
              default: l(({ row: m }) => [
                v(R(n(te)(m.created_at)), 1)
              ]),
              _: 1
            })),
            e(j, {
              label: "更新时间",
              width: "170"
            }, {
              default: l(({ row: m }) => [
                v(R(n(te)(m.updated_at)), 1)
              ]),
              _: 1
            }),
            e(j, {
              label: "操作",
              width: "160",
              fixed: "right"
            }, {
              default: l(({ row: m }) => [
                e(K, {
                  link: "",
                  type: "primary",
                  onClick: (J) => A(m.id)
                }, {
                  default: l(() => [...r[25] || (r[25] = [
                    v("详情", -1)
                  ])]),
                  _: 1
                }, 8, ["onClick"]),
                e(K, {
                  link: "",
                  type: "primary",
                  onClick: (J) => N(m.id)
                }, {
                  default: l(() => [...r[26] || (r[26] = [
                    v("编辑", -1)
                  ])]),
                  _: 1
                }, 8, ["onClick"]),
                e(K, {
                  link: "",
                  type: "danger",
                  onClick: (J) => H(m)
                }, {
                  default: l(() => [...r[27] || (r[27] = [
                    v("删除", -1)
                  ])]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            })
          ]),
          _: 1
        }, 8, ["data"])), [
          [we, n(s)]
        ]),
        n(z) ? (b(), O("div", Qt, [
          e(be, {
            "current-page": n(p),
            "onUpdate:currentPage": r[14] || (r[14] = (m) => Q(p) ? p.value = m : null),
            "page-size": n(_),
            "onUpdate:pageSize": r[15] || (r[15] = (m) => Q(_) ? _.value = m : null),
            total: n(S),
            "page-sizes": [10, 20, 50],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: r[16] || (r[16] = (m) => U(n(V))),
            onSizeChange: r[17] || (r[17] = (m) => U(n(V)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])) : (b(), O("div", Kt, [
          e(be, {
            "current-page": n(p),
            "onUpdate:currentPage": r[10] || (r[10] = (m) => Q(p) ? p.value = m : null),
            "page-size": n(_),
            "onUpdate:pageSize": r[11] || (r[11] = (m) => Q(_) ? _.value = m : null),
            total: n(S),
            "page-sizes": [20, 50, 100, 200],
            layout: "total, sizes, prev, pager, next",
            onCurrentChange: r[12] || (r[12] = (m) => U(n(V))),
            onSizeChange: r[13] || (r[13] = (m) => U(n(V)))
          }, null, 8, ["current-page", "page-size", "total"])
        ])),
        e(Ot, {
          visible: E.value,
          "onUpdate:visible": r[18] || (r[18] = (m) => E.value = m),
          "memory-id": o.value,
          "tag-options": n(x),
          width: n(g) ? "96%" : "640px",
          onSaved: ee
        }, null, 8, ["visible", "memory-id", "tag-options", "width"]),
        e(qt, {
          visible: c.value,
          "onUpdate:visible": r[19] || (r[19] = (m) => c.value = m),
          "memory-id": P.value,
          size: n(g) ? "100%" : "45%"
        }, null, 8, ["visible", "memory-id", "size"])
      ], 512);
    };
  }
}), Wt = /* @__PURE__ */ ne(Gt, [["__scopeId", "data-v-7abc1d84"]]);
function Xt() {
  const t = le(), i = y([]), u = y(!1);
  async function a() {
    u.value = !0;
    try {
      const d = await Be(t);
      i.value = d.tags ?? [];
    } finally {
      u.value = !1;
    }
  }
  async function f(d, w) {
    await Ut(t, d, w), await a();
  }
  async function p(d, w, S) {
    await Mt(t, d, w, S), await a();
  }
  async function _(d, w) {
    await Tt(t, d, w), await a();
  }
  return ae(() => {
    a().catch(() => {
    });
  }), { rows: i, loading: u, reload: a, create: f, rename: p, remove: _ };
}
const Yt = { class: "am-panel-header" }, Zt = /* @__PURE__ */ X({
  __name: "TagsPanel",
  setup(t) {
    const { rows: i, loading: u, reload: a, create: f, rename: p, remove: _ } = Xt(), d = y(null), { compact: w } = ge(d), S = y(!1), $ = y(!1), s = Ae({ oldName: null, name: "", newName: "", description: "" }), x = y(!1), z = y("detach"), C = y(null);
    function V() {
      Object.assign(s, { oldName: null, name: "", newName: "", description: "" }), $.value = !0;
    }
    function F(E) {
      Object.assign(s, { oldName: E.name, name: E.name, newName: "", description: E.description ?? "" }), $.value = !0;
    }
    async function L() {
      S.value = !0;
      try {
        s.oldName ? (await p(s.oldName, s.newName, s.description), B.success("已保存")) : (await f(s.name, s.description), B.success("已创建")), $.value = !1;
      } catch (E) {
        B.error(E instanceof Error ? E.message : String(E));
      } finally {
        S.value = !1;
      }
    }
    function D(E) {
      C.value = E, z.value = "detach", x.value = !0;
    }
    async function g() {
      var E, o;
      if (z.value === "purge")
        try {
          await Re.confirm(
            `将永久删除标签「${(E = C.value) == null ? void 0 : E.name}」及其关联的 ${((o = C.value) == null ? void 0 : o.memory_count) ?? 0} 条记忆，且不可恢复！`,
            "高危操作确认",
            { type: "error", confirmButtonText: "永久删除" }
          );
        } catch {
          return;
        }
      if (C.value) {
        S.value = !0;
        try {
          await _(C.value.name, z.value), B.success("已删除"), x.value = !1;
        } catch (c) {
          B.error(c instanceof Error ? c.message : String(c));
        } finally {
          S.value = !1;
        }
      }
    }
    return (E, o) => {
      const c = se, P = fe, U = Ue, T = ze, N = pe, A = Ce, ee = $e, H = ke, h = Xe, r = ce, K = Me;
      return b(), O("div", {
        ref_key: "rootRef",
        ref: d,
        class: "am-panel"
      }, [
        k("div", Yt, [
          o[9] || (o[9] = k("div", null, [
            k("h2", { class: "am-panel-title" }, "标签管理"),
            k("p", { class: "am-panel-subtitle" }, "标签是 agent 自主维护的分类体系；改名会同步更新所有引用它的记忆")
          ], -1)),
          e(c, {
            type: "primary",
            icon: n(Te),
            onClick: V
          }, {
            default: l(() => [...o[8] || (o[8] = [
              v("新建标签", -1)
            ])]),
            _: 1
          }, 8, ["icon"])
        ]),
        Z((b(), I(T, { data: n(i) }, {
          default: l(() => [
            e(U, {
              prop: "name",
              label: "名称",
              "min-width": "160"
            }, {
              default: l(({ row: M }) => [
                e(P, null, {
                  default: l(() => [
                    v(R(M.name), 1)
                  ]),
                  _: 2
                }, 1024)
              ]),
              _: 1
            }),
            e(U, {
              prop: "description",
              label: "描述",
              "min-width": "300",
              "show-overflow-tooltip": ""
            }, {
              default: l(({ row: M }) => [
                v(R(M.description || "—"), 1)
              ]),
              _: 1
            }),
            e(U, {
              prop: "memory_count",
              label: "记忆数",
              width: "100",
              sortable: ""
            }),
            e(U, {
              label: "最近使用",
              width: "170"
            }, {
              default: l(({ row: M }) => [
                v(R(n(te)(M.last_used_at)), 1)
              ]),
              _: 1
            }),
            e(U, {
              label: "操作",
              width: "150",
              fixed: "right"
            }, {
              default: l(({ row: M }) => [
                e(c, {
                  link: "",
                  type: "primary",
                  onClick: (q) => F(M)
                }, {
                  default: l(() => [...o[10] || (o[10] = [
                    v("编辑", -1)
                  ])]),
                  _: 1
                }, 8, ["onClick"]),
                e(c, {
                  link: "",
                  type: "danger",
                  onClick: (q) => D(M)
                }, {
                  default: l(() => [...o[11] || (o[11] = [
                    v("删除", -1)
                  ])]),
                  _: 1
                }, 8, ["onClick"])
              ]),
              _: 1
            })
          ]),
          _: 1
        }, 8, ["data"])), [
          [K, n(u)]
        ]),
        e(H, {
          modelValue: $.value,
          "onUpdate:modelValue": o[4] || (o[4] = (M) => $.value = M),
          title: s.oldName ? "编辑标签" : "新建标签",
          width: n(w) ? "96%" : "480px"
        }, {
          footer: l(() => [
            e(c, {
              onClick: o[3] || (o[3] = (M) => $.value = !1)
            }, {
              default: l(() => [...o[12] || (o[12] = [
                v("取消", -1)
              ])]),
              _: 1
            }),
            e(c, {
              type: "primary",
              loading: S.value,
              onClick: L
            }, {
              default: l(() => [...o[13] || (o[13] = [
                v("保存", -1)
              ])]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: l(() => [
            e(ee, { "label-position": "top" }, {
              default: l(() => [
                e(A, { label: "名称（唯一，≤100 字符）" }, {
                  default: l(() => [
                    e(N, {
                      modelValue: s.name,
                      "onUpdate:modelValue": o[0] || (o[0] = (M) => s.name = M),
                      disabled: !!s.oldName,
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: "如 rust、项目、工作流"
                    }, null, 8, ["modelValue", "disabled"])
                  ]),
                  _: 1
                }),
                s.oldName ? (b(), I(A, {
                  key: 0,
                  label: "改为新名称（留空表示不改名）"
                }, {
                  default: l(() => [
                    e(N, {
                      modelValue: s.newName,
                      "onUpdate:modelValue": o[1] || (o[1] = (M) => s.newName = M),
                      maxlength: "100",
                      "show-word-limit": "",
                      placeholder: "仅大小写改名也可用于合并拼写偏差"
                    }, null, 8, ["modelValue"])
                  ]),
                  _: 1
                })) : W("", !0),
                e(A, { label: "描述（可选，≤500 字符）" }, {
                  default: l(() => [
                    e(N, {
                      modelValue: s.description,
                      "onUpdate:modelValue": o[2] || (o[2] = (M) => s.description = M),
                      type: "textarea",
                      rows: 3,
                      maxlength: "500",
                      "show-word-limit": "",
                      placeholder: "这个标签用来组织什么内容"
                    }, null, 8, ["modelValue"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            })
          ]),
          _: 1
        }, 8, ["modelValue", "title", "width"]),
        e(H, {
          modelValue: x.value,
          "onUpdate:modelValue": o[7] || (o[7] = (M) => x.value = M),
          title: "删除标签",
          width: n(w) ? "96%" : "480px"
        }, {
          footer: l(() => [
            e(c, {
              onClick: o[6] || (o[6] = (M) => x.value = !1)
            }, {
              default: l(() => [...o[19] || (o[19] = [
                v("取消", -1)
              ])]),
              _: 1
            }),
            e(c, {
              type: "danger",
              loading: S.value,
              onClick: g
            }, {
              default: l(() => [...o[20] || (o[20] = [
                v("删除", -1)
              ])]),
              _: 1
            }, 8, ["loading"])
          ]),
          default: l(() => {
            var M;
            return [
              k("p", null, [
                o[14] || (o[14] = v(" 标签 ", -1)),
                e(P, null, {
                  default: l(() => {
                    var q;
                    return [
                      v(R((q = C.value) == null ? void 0 : q.name), 1)
                    ];
                  }),
                  _: 1
                }),
                o[15] || (o[15] = v(" 当前被 ", -1)),
                k("b", null, R((M = C.value) == null ? void 0 : M.memory_count), 1),
                o[16] || (o[16] = v(" 条记忆使用。请选择删除方式： ", -1))
              ]),
              e(r, {
                modelValue: z.value,
                "onUpdate:modelValue": o[5] || (o[5] = (q) => z.value = q)
              }, {
                default: l(() => [
                  e(h, { value: "detach" }, {
                    default: l(() => [...o[17] || (o[17] = [
                      v("仅摘除引用（保留全部记忆）", -1)
                    ])]),
                    _: 1
                  }),
                  e(h, { value: "purge" }, {
                    default: l(() => [...o[18] || (o[18] = [
                      v("连带删除记忆（不可恢复）", -1)
                    ])]),
                    _: 1
                  })
                ]),
                _: 1
              }, 8, ["modelValue"])
            ];
          }),
          _: 1
        }, 8, ["modelValue", "width"])
      ], 512);
    };
  }
});
function el(t) {
  return t.get("/api/stats");
}
function tl(t) {
  return t.get("/health");
}
function ll(t) {
  return t.get("/api/doctor");
}
function nl(t) {
  return t.getBlob("/api/export");
}
function ol(t, i) {
  return t.post("/api/import", i);
}
function al() {
  const t = le(), i = y({}), u = y(""), a = y({ ok: !0, issues: [] }), f = y(!1), p = y(!1), _ = y(!1), d = y(!1), w = oe(() => $t(i.value.file_size));
  async function S() {
    i.value = await el(t), i.value.version = u.value;
  }
  async function $() {
    p.value = !0;
    try {
      a.value = await ll(t), f.value = !0;
    } finally {
      p.value = !1;
    }
  }
  async function s() {
    _.value = !0;
    try {
      const z = await nl(t), C = URL.createObjectURL(z), V = document.createElement("a");
      V.href = C, V.download = "agent-memory-export.json", V.click(), URL.revokeObjectURL(C);
    } finally {
      _.value = !1;
    }
  }
  async function x(z) {
    d.value = !0;
    try {
      const C = await z.text();
      let V;
      try {
        V = JSON.parse(C);
      } catch {
        throw new Error("备份文件不是有效的 JSON");
      }
      const F = await ol(t, V);
      return await S(), F;
    } finally {
      d.value = !1;
    }
  }
  return ae(async () => {
    try {
      u.value = (await tl(t)).version ?? "";
    } catch {
    }
    await S().catch(() => {
    });
  }), {
    stats: i,
    version: u,
    doctor: a,
    doctorRan: f,
    doctorLoading: p,
    exporting: _,
    importing: d,
    sizeText: w,
    reload: S,
    runDoctor: $,
    exportData: s,
    importFile: x
  };
}
const sl = { class: "am-panel-header" }, il = { class: "actions" }, rl = { class: "card-header" }, ul = {
  key: 2,
  class: "issues"
}, dl = /* @__PURE__ */ X({
  __name: "OpsPanel",
  setup(t) {
    const {
      stats: i,
      version: u,
      doctor: a,
      doctorRan: f,
      doctorLoading: p,
      exporting: _,
      importing: d,
      sizeText: w,
      reload: S,
      runDoctor: $,
      exportData: s,
      importFile: x
    } = al(), z = y(null), { compact: C } = ge(z), V = y(null);
    function F(D) {
      return D().catch((g) => B.error(g instanceof Error ? g.message : String(g)));
    }
    function L(D) {
      var o;
      const g = D.target, E = (o = g.files) == null ? void 0 : o[0];
      g.value = "", E && x(E).then((c) => {
        B.success(`已导入 ${c.imported_memories} 条记忆、${c.imported_tags} 个标签`);
      }).catch((c) => {
        B.error(c instanceof Error ? c.message : String(c));
      });
    }
    return (D, g) => {
      const E = se, o = tt, c = et, P = Ze, U = Ye, T = nt, N = lt, A = xe, ee = ot;
      return b(), O("div", {
        ref_key: "rootRef",
        ref: z,
        class: "am-panel"
      }, [
        k("div", sl, [
          g[5] || (g[5] = k("div", null, [
            k("h2", { class: "am-panel-title" }, "运维"),
            k("p", { class: "am-panel-subtitle" }, "数据库概况、备份导入导出与数据体检")
          ], -1)),
          e(E, {
            icon: n(ft),
            onClick: g[0] || (g[0] = (H) => F(n(S)))
          }, {
            default: l(() => [...g[4] || (g[4] = [
              v("刷新概况", -1)
            ])]),
            _: 1
          }, 8, ["icon"])
        ]),
        e(U, { gutter: 14 }, {
          default: l(() => [
            e(P, {
              span: n(C) ? 12 : 6
            }, {
              default: l(() => [
                e(c, { shadow: "never" }, {
                  default: l(() => [
                    e(o, {
                      title: "记忆条数",
                      value: n(i).memories ?? 0
                    }, null, 8, ["value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            e(P, {
              span: n(C) ? 12 : 6
            }, {
              default: l(() => [
                e(c, { shadow: "never" }, {
                  default: l(() => [
                    e(o, {
                      title: "标签数",
                      value: n(i).tags ?? 0
                    }, null, 8, ["value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            e(P, {
              span: n(C) ? 12 : 6
            }, {
              default: l(() => [
                e(c, { shadow: "never" }, {
                  default: l(() => [
                    e(o, {
                      title: "数据库大小",
                      value: n(w)
                    }, null, 8, ["value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"]),
            e(P, {
              span: n(C) ? 12 : 6
            }, {
              default: l(() => [
                e(c, { shadow: "never" }, {
                  default: l(() => [
                    e(o, {
                      title: "下一个记忆 ID",
                      value: n(i).next_id ?? "—"
                    }, null, 8, ["value"])
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["span"])
          ]),
          _: 1
        }),
        e(c, { shadow: "never" }, {
          header: l(() => [...g[6] || (g[6] = [
            v("数据库", -1)
          ])]),
          default: l(() => [
            e(N, {
              column: n(C) ? 1 : 2,
              border: ""
            }, {
              default: l(() => [
                e(T, { label: "文件路径" }, {
                  default: l(() => [
                    v(R(n(i).path ?? "—"), 1)
                  ]),
                  _: 1
                }),
                e(T, { label: "最近更新" }, {
                  default: l(() => [
                    v(R(n(i).newest_update ? `${n(i).newest_update.id}（${n(te)(n(i).newest_update.updated_at)}）` : "—"), 1)
                  ]),
                  _: 1
                }),
                e(T, { label: "跨平台迁移" }, {
                  default: l(() => [...g[7] || (g[7] = [
                    v(" SQLite 文件格式平台无关，停服后可直接复制 .db 文件到其他机器；运行中请改用「导出备份」。 ", -1)
                  ])]),
                  _: 1
                }),
                e(T, { label: "版本" }, {
                  default: l(() => [
                    v(R(n(i).version ?? n(u)), 1)
                  ]),
                  _: 1
                })
              ]),
              _: 1
            }, 8, ["column"]),
            k("div", il, [
              e(E, {
                icon: n(vt),
                loading: n(_),
                onClick: g[1] || (g[1] = (H) => F(n(s)))
              }, {
                default: l(() => [...g[8] || (g[8] = [
                  v("导出备份（JSON）", -1)
                ])]),
                _: 1
              }, 8, ["icon", "loading"]),
              e(E, {
                icon: n(gt),
                loading: n(d),
                onClick: g[2] || (g[2] = (H) => {
                  var h;
                  return (h = V.value) == null ? void 0 : h.click();
                })
              }, {
                default: l(() => [...g[9] || (g[9] = [
                  v("导入备份", -1)
                ])]),
                _: 1
              }, 8, ["icon", "loading"]),
              k("input", {
                ref_key: "importInput",
                ref: V,
                type: "file",
                accept: "application/json,.json",
                style: { display: "none" },
                onChange: L
              }, null, 544)
            ]),
            e(A, {
              title: "导入仅支持空数据库（导入是恢复而非合并）；当前库已有数据时请换一个空的 --db 路径再导入。",
              type: "info",
              "show-icon": "",
              closable: !1,
              class: "import-hint"
            })
          ]),
          _: 1
        }),
        e(c, { shadow: "never" }, {
          header: l(() => [
            k("div", rl, [
              g[11] || (g[11] = k("span", null, "数据体检（doctor）", -1)),
              e(E, {
                size: "small",
                icon: n(Ie),
                loading: n(p),
                onClick: g[3] || (g[3] = (H) => F(n($)))
              }, {
                default: l(() => [...g[10] || (g[10] = [
                  v("运行体检", -1)
                ])]),
                _: 1
              }, 8, ["icon", "loading"])
            ])
          ]),
          default: l(() => [
            n(f) ? (b(), O(G, { key: 0 }, [
              n(a).ok ? (b(), I(A, {
                key: 0,
                title: "体检通过：未发现孤儿引用、大小写冲突或空字段",
                type: "success",
                "show-icon": "",
                closable: !1
              })) : (b(), I(A, {
                key: 1,
                title: `发现 ${n(a).issues.length} 个问题`,
                type: "error",
                "show-icon": "",
                closable: !1
              }, null, 8, ["title"])),
              n(a).ok ? W("", !0) : (b(), O("ul", ul, [
                (b(!0), O(G, null, Y(n(a).issues, (H, h) => (b(), O("li", { key: h }, R(H), 1))), 128))
              ]))
            ], 64)) : (b(), I(ee, {
              key: 1,
              description: "点击「运行体检」检查孤儿标签引用、大小写冲突标签组、空摘要/正文",
              "image-size": 60
            }))
          ]),
          _: 1
        })
      ], 512);
    };
  }
}), ml = /* @__PURE__ */ ne(dl, [["__scopeId", "data-v-8310bd60"]]), pl = { class: "memory-ui" }, cl = { class: "brand" }, fl = { class: "brand-mark" }, vl = { class: "aside-footer" }, gl = /* @__PURE__ */ X({
  __name: "MemoryAdmin",
  props: {
    layout: { default: "sidebar" },
    title: { default: "agent-memory" }
  },
  setup(t) {
    const i = y("memories");
    return (u, a) => {
      const f = it, p = ut, _ = rt, d = dt, w = st, S = ct, $ = pt, s = mt, x = at;
      return b(), O("div", pl, [
        e(x, { class: "layout" }, {
          default: l(() => [
            t.layout === "sidebar" ? (b(), I(w, {
              key: 0,
              width: "200px",
              class: "aside"
            }, {
              default: l(() => [
                k("div", cl, [
                  k("span", fl, [
                    e(f, { size: 16 }, {
                      default: l(() => [
                        e(n(_t))
                      ]),
                      _: 1
                    })
                  ]),
                  k("span", null, R(t.title), 1)
                ]),
                e(_, {
                  "default-active": i.value,
                  class: "menu",
                  onSelect: a[0] || (a[0] = (z) => i.value = z)
                }, {
                  default: l(() => [
                    e(p, { index: "memories" }, {
                      default: l(() => [
                        e(f, null, {
                          default: l(() => [
                            e(n(yt))
                          ]),
                          _: 1
                        }),
                        a[2] || (a[2] = k("span", null, "记忆管理", -1))
                      ]),
                      _: 1
                    }),
                    e(p, { index: "tags" }, {
                      default: l(() => [
                        e(f, null, {
                          default: l(() => [
                            e(n(bt))
                          ]),
                          _: 1
                        }),
                        a[3] || (a[3] = k("span", null, "标签管理", -1))
                      ]),
                      _: 1
                    }),
                    e(p, { index: "ops" }, {
                      default: l(() => [
                        e(f, null, {
                          default: l(() => [
                            e(n(wt))
                          ]),
                          _: 1
                        }),
                        a[4] || (a[4] = k("span", null, "运维", -1))
                      ]),
                      _: 1
                    })
                  ]),
                  _: 1
                }, 8, ["default-active"]),
                k("div", vl, [
                  Ke(u.$slots, "footer", {}, () => [
                    e(d, {
                      size: "small",
                      type: "info"
                    }, {
                      default: l(() => [...a[5] || (a[5] = [
                        v("自托管 · 多 agent 共享", -1)
                      ])]),
                      _: 1
                    })
                  ], !0)
                ])
              ]),
              _: 3
            })) : W("", !0),
            e(s, { class: "main" }, {
              default: l(() => [
                t.layout === "tabs" ? (b(), I($, {
                  key: 0,
                  modelValue: i.value,
                  "onUpdate:modelValue": a[1] || (a[1] = (z) => i.value = z),
                  class: "tabs-bar"
                }, {
                  default: l(() => [
                    e(S, {
                      label: "记忆管理",
                      name: "memories"
                    }),
                    e(S, {
                      label: "标签管理",
                      name: "tags"
                    }),
                    e(S, {
                      label: "运维",
                      name: "ops"
                    })
                  ]),
                  _: 1
                }, 8, ["modelValue"])) : W("", !0),
                Z(e(Wt, null, null, 512), [
                  [re, i.value === "memories"]
                ]),
                Z(e(Zt, null, null, 512), [
                  [re, i.value === "tags"]
                ]),
                Z(e(ml, null, null, 512), [
                  [re, i.value === "ops"]
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
}), Vl = /* @__PURE__ */ ne(gl, [["__scopeId", "data-v-5e17b96e"]]);
export {
  Fe as MarkdownView,
  Wt as MemoriesPanel,
  Vl as MemoryAdmin,
  qt as MemoryDetailDrawer,
  Ot as MemoryEditorDialog,
  Ne as MemoryUIConfigKey,
  ml as OpsPanel,
  Zt as TagsPanel,
  Vt as buildMemoriesQuery,
  Ct as createApiClient,
  $t as formatSize,
  te as formatTime,
  de as isSearchMode,
  Cl as provideMemoryUI,
  St as renderMarkdown,
  xt as sanitizeHtml,
  le as useApiClient,
  It as useMemories,
  Oe as useMemoryConfig,
  al as useOps,
  Xt as useTags
};
