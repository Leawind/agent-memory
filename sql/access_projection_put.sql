INSERT INTO memory_access_projection(memory_id, score, as_of, reads, uses) VALUES (?1, ?2, ?3, ?4, ?5)
ON CONFLICT(memory_id) DO UPDATE SET score=excluded.score, as_of=excluded.as_of, reads=excluded.reads, uses=excluded.uses
