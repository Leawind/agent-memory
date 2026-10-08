SELECT bucket_end, score, reads, uses FROM memory_access_buckets WHERE memory_id=?1 ORDER BY bucket_end
