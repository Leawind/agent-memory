INSERT INTO settings (key, value) VALUES ('search_revision', '1')
ON CONFLICT(key) DO UPDATE SET value = CAST(CAST(settings.value AS INTEGER) + 1 AS TEXT);
