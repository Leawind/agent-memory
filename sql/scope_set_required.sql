-- Toggle the requirement; the timestamp is when it started applying (the cutoff that keeps
-- earlier memories out of the new write contract). Reports whether it actually changed.
UPDATE tag_scopes SET required_since = CASE WHEN ?2 THEN ?3 ELSE NULL END
WHERE name = ?1 AND ((required_since IS NOT NULL) != ?2)
