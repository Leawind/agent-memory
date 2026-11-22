-- 标签可以被配置为写入必填：required_since 非空表示该标签在每次记忆写入时都必须显式声明
-- 取值（一个值标签，或 NULL 表示这条记忆在该标签上不取任何值）。时间戳即生效点：早于它
-- 创建的存量记忆是未分类的，不被回溯要求；此后的写入必须声明。NULL 表示普通标签。
--
-- 声明本身不落库：NULL 声明不产生行，与"存量未分类"在数据上无法区分，二者的界线由
-- created_at 与 required_since 的比较给出，因此不占任何行内状态。

ALTER TABLE tags ADD COLUMN required_since INTEGER;
