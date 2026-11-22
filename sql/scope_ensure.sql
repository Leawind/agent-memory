-- Implicit scope creation: the first tag of a scope, or a write that is allowed to extend the
-- vocabulary, creates the scope with no description and no requirement on it.
INSERT OR IGNORE INTO tag_scopes(name, description, created_at) VALUES (?1, ?2, ?3)
