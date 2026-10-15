INSERT OR IGNORE INTO memory_attributes(memory_id, key, value)
SELECT ?1, key, value FROM memory_attributes WHERE memory_id = ?2
