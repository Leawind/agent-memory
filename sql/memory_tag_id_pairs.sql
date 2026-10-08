SELECT mt.memory_id, mt.tag_id FROM memory_tags mt
WHERE mt.original = 1
ORDER BY mt.memory_id, mt.tag_id
