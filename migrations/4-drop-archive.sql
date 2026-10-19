-- 置顶与到期机制移除：记忆不再有生命周期状态，时效衰减只是排序信号（lifecycle_policy 设置）。
-- 本迁移曾重建本表以删除 archived_at 列；置顶/到期移除后整表无读写方，直接删除。
-- 基线（迁移 1）已不含本表，对全新数据库是等价空操作。

DROP TABLE IF EXISTS memory_lifecycle;
