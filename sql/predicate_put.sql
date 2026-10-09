INSERT INTO tag_predicates(namespace, owner_id, name, predicate, description)
VALUES (?1, ?2, ?3, ?4, ?5)
ON CONFLICT(namespace, name) DO UPDATE SET predicate = excluded.predicate, description = excluded.description;
