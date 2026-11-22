-- 身份可以绑定一个带作用域的标签，表示"这个身份的调用者是谁"（如 system::pc1sw0）。
-- 读侧的列表与搜索据此默认只看"共享 + 自己"：绑定标签的持有者，以及与它在该作用域下
-- 没有标签的记忆；调用者显式给出 tag_expr 时以它为准。

ALTER TABLE identities ADD COLUMN tag TEXT;
