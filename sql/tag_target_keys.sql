-- Distinct (target tag name, source tag name) pairs over every edge: which keys point at
-- each tag. Navigation faces use it to group attribute values (`key:value`) under their key.
SELECT tt.name, ts.name
FROM memory_tags mt
JOIN tags tt ON tt.id = mt.target_tag_id
JOIN tags ts ON ts.id = mt.tag_id
GROUP BY tt.name, ts.name
ORDER BY tt.name, ts.name
