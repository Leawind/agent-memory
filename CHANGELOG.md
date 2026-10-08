# Changelog

## Unreleased

- **前端**：语义搜索与重排设置卡片改为紧凑候选列表——每个候选默认一行（名称、检测结论、向量覆盖、右侧启停开关），点击展开带标签的字段与逐项操作（检测/补跑/删除缓存/删除），拖拽手柄调整优先级（拖拽交给 SortableJS：拖动时原处留下落点虚框、相邻行让位带过渡动画，触摸屏同样可用）；检测与保存分作两个按钮，检测既能逐个候选（用行内当前值，未保存也可测）也能整列表一次跑完，结论落在对应行上；补跑带进度条与百分比并可中断；按模型的向量缓存管理并入各候选行，未配置的遗留缓存单列一节（仅可删除）
- **语义搜索**：引入 reranker（重排）阶段——配置有序的 `rerank_models` 候选列表（每项独立启停），搜索时对融合候选池做交叉编码器重打分并按相关性重排（响应携带 `reranked_by` 行，score 切换为重排相关性 ×10000）；显式 keyword 模式保持确定性排序；reranker 不可用时静默保留融合排序；`POST /api/rerank/test` 逐候选探测连通性
- **语义搜索（破坏性）**：embedding 配置改为有序多候选列表（`embedding_models`，每项独立的 enabled 开关、端点凭据、指令前缀与逐模型余弦下限），运行时按优先级发起真实调用、失败自动切换下一候选，搜索响应以 `embedding_model` 标明实际应答的模型；`memory_embeddings` 表改 `(memory_id, model)` 复合主键，多个模型的向量缓存共存，新增按模型补跑/删除（`GET|DELETE /api/embeddings/caches`、`POST /api/embeddings/backfill` 支持指定 `model_key`、`embed-backfill --model`）；连接测试改为逐候选探测（`POST /api/embeddings/test` 的可选 `entries` 直接检测请求里给的候选——未启用、未保存的草稿也能测——不传则探测已保存的启用候选，逐位置返回结论）；旧的单模型扁平设置键移除
- **语义搜索**：语义召回通道新增余弦下限过滤（`embedding_min_similarity`，默认 0.30，0 为关闭；查询时过滤、不影响已存向量）——库小于候选上限时"任何查询都返回全库"的问题就此修复，MCP 与前端同享；管理界面语义搜索卡片新增该配置项
- **前端**：搜索框占位符按语义搜索可用性动态显示；说明文本全面精简（字段标签去括号说明、提示语收敛为一句）
- **REST**：写路径与 MCP 面共用同一段写后钩子（`tools::after_commit`）——REST 创建记忆现在同样返回 `similar_to` 语义查重提示；新增 `POST /api/memories/merge` 路由（复用 MCP `memory_merge` 处理器）；`memory_merge` 纳入回填触发条件，合并后目标向量立即补嵌
- **前端**：编辑器对话框承接查重提示（创建后一键合并/保留两条）、新增"合并另一条"入口；搜索分页行显示关键词命中占比（hybrid 时）
- **工具**：`memory_search` hybrid 模式先按余弦截断语义通道（约 2×limit、下限 20）再做 RRF 融合，`total_matches` 不再恒等于库总量；hybrid 时响应新增 `keyword_matches` 行，标明多少命中来自字面关键词、多少仅由语义通道带入
- **语义搜索**：embedding 配置新增查询侧/文档侧指令前缀（`embedding_query_prefix` / `embedding_passage_prefix`，管理界面可配，逐字应用不 trim）；前缀并入向量身份键，改动前缀与换模型同样使旧向量失效待补跑，避免混合前缀的向量被静默混用
- **schema（破坏性）**：标签改用内部自增 id 关联记忆；基线迁移改写，旧库不兼容，打开时做 schema 指纹校验、不符即拒绝（按提示 export/import 迁移）
- **工具（破坏性）**：`tag_rename` 更名 `tag_update`；响应全面去噪——空字段整个省略、恒真布尔删除；摘要不再携带 `created_at`
- **工具**：对模型暴露的时间戳改为本地墙钟的紧凑人读格式（存储与导出仍是 epoch 秒）；标签视图字段瘦身
- **export / import（破坏性）**：导出重构为紧凑对象结构、只保留主数据；`import` 只接受新格式
- **鉴权**：身份 token 改为 `sk_` 前缀的新格式，已签发旧 token 重置后更换

## 0.1.0 (2026-09-25)

首个版本：多 agent 经 HTTP 共享单个 SQLite 记忆库的 MCP 服务器，内置 Vue3 管理界面；可选 token 鉴权与语义搜索，单二进制交付。详见 [README](README.md)。
