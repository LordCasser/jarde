# CI product verifier v12 review

V12 fixes only the Unicode pre-repair failure capture reader and updates usage/schema/output names. That capture uses `path`, `bytes`, and `sha256`, with separate raw stdout fields; v12 verifies both the stored API bytes and the compressed stable log, then checks decompressed stdout and both stderr sidecars against their captured byte counts and hashes. The earlier fingerprint-failure capture intentionally uses the older `output`/`stored_sha256` schema, and its reader remains unchanged.

I compared these readers and the subsequent frozen JSON consumers with their captured structures: Unicode validation rows, accepted Unicode replay/execution records, fingerprint registration/execution, and prior failure capture. The observed keys match the fields consumed. A direct raw/gzip/hash audit and AST parse passed. V12 was not executed; v11's failure record and all captured evidence remain unchanged.
