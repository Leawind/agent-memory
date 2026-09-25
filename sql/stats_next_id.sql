SELECT COALESCE((SELECT seq FROM sqlite_sequence WHERE name = 'memories'), 0) + 1
