# Changelog

## Unreleased

- **生命周期（破坏性）**：归档机制整体移除——"把记忆藏出默认视图"由到期时间承担（`expires_at` 设为过去即等价于旧归档，清除即恢复），记忆状态只剩 active / expired 两种；`memory_lifecycle` 工具的 `archived` 参数删除，`memory_list` / `memory_search` 的 `state` 枚举去掉 `archived`，管理界面状态页签与生命周期对话框同步精简；迁移 4 重建 `memory_lifecycle` 表删除 `archived_at` 列，已归档记忆恢复为 active
- **工具/schema（破坏性，补记）**：标签统一为"可以没有值的键值对"——键在一条记忆中唯一、值是标识词或规范十进制数，复用标签管道的全部机制（`memory_attributes` 表、`tag_keys` 键注册表、string/bool/enum/number 键类型、多值键全部删除；key_set / key_list / key_delete 工具与 /api/keys 路由移除，添加标签只有显式创建与写记忆时顺带创建两种方式）。存储随之规范化：记忆-标签关系表直接携带可空的标签值列（`memory_tags(memory_id, tag_id, value)`），`original/derived` 列删除，迁移 3 原地升级旧库，读取时组装 `key:value` 展示名。MCP 写入统一为一个 tags JSON 对象 `{键: 值|null}`（null 即裸标签），`attributes` 参数移除；`memory_update` 改增量维护——`add_tags` 对象逐键 upsert、`remove_tags` 按键名删除，不再整组替换；`key=value` 查询与数值规范化（1 与 1.0 同值）改为无注册表实现；派生的类型化 attributes 视图直接从存储的 (键, 值) 关系推导
- **前端**：界面统一到同一套标签词汇——记忆编辑器一个标签选择器同时收裸标签与 `key:value`（网页端直接写 key:value，保存映射为 tags 对象；编辑按键做增量：值变化走 add_tags、删掉的键走 remove_tags；输入在下拉框源头拦截——合法 token 才出现“添加”项，非法 token（格式错误、同键已有值、重复标签）以红色原因说明替代、无法上屏），记忆卡片上带值标签与普通标签同为徽章；属性行编辑器与"属性键"管理页（上一版引入）连同其 API 封装一并移除；侧栏对组合标签隐藏改名/删除入口（服务端标签改名/删除不接受冒号名）；早已失效的"标签规则"管理卡（后端无对应路由）与编辑器读取已删除字段 original_tags / derived_tags 导致的打开报错一并修复；mock API 同步为对象/增量语义
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
