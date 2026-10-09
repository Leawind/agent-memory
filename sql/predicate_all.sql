SELECT id, namespace, name, predicate, description FROM tag_predicates
WHERE ?1 IS NULL OR namespace = 'global' OR namespace = ?1 ORDER BY id;
