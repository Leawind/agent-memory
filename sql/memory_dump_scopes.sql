-- Scope rows for the memory dump: the export writes them verbatim, and the import reads the
-- store's scope inventory (required cutoffs included) from here.
SELECT id, name, description, created_at, required_since FROM tag_scopes ORDER BY id
