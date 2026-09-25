UPDATE tags SET name = ?1, description = COALESCE(?2, description) WHERE name = ?3
