INSERT INTO tag_predicates(name, predicate, description)
VALUES (?1, ?2, ?3)
ON CONFLICT(name) DO UPDATE SET predicate = excluded.predicate, description = excluded.description;
