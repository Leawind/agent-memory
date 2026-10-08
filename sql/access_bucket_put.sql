INSERT INTO memory_access_buckets(memory_id, bucket_end, score, reads, uses) VALUES (?1, ?2, ?3, ?4, ?5)
ON CONFLICT(memory_id, bucket_end) DO UPDATE SET score=score+excluded.score, reads=reads+excluded.reads, uses=uses+excluded.uses
