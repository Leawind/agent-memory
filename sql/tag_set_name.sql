-- A rename may move a tag to another scope, so the scope_id column follows the name.
UPDATE tags SET name = ?1, scope_id = ?2 WHERE name = ?3
