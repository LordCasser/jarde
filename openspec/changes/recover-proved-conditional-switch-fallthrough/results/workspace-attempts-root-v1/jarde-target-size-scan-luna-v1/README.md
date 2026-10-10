# Target size scan adapter v1

This standalone helper replaces only the v9 runner's `target_bytes()` implementation. It performs one explicit `Path.stat()` for each path yielded under `ROOT / "target"`, catches only `FileNotFoundError`, and sums sizes only when `stat.S_ISREG(info.st_mode)` is true. A path deleted before its stat contributes zero; if deletion follows a successful stat, the scan uses that one observed size. Each yielded path is added at most once per scan, so a missing/replaced Cargo build-script path cannot be counted twice. Other errors remain visible and fail the guard rather than looking like a smaller tree. As with the original scan, distinct hard-linked paths are each counted by their directory entry.

The v9 thresholds and process-group behavior remain in the pinned runner: `FREE_LIMIT = 5 * 1024**3`, `TARGET_LIMIT = 1024**3`, `Popen(..., start_new_session=True)`, and the existing one-second polling interval. Do not edit that historical runner; inject this function into its globals from a wrapper/collector.

Example for the workspace wrapper that already loads the v9 module as `m` and has `ROOT`:

```python
from target_size_scan import regular_file_bytes
m.run_command.__globals__["target_bytes"] = lambda: regular_file_bytes(ROOT / "target")
```

Alternatively the wrapper can load this file by path with `importlib.util` and inject the same lambda. This is a scanner adapter only; it does not start commands, change guard limits, or alter process-group cleanup.

## Provenance pins

- Historical v9 runner (read-only, unchanged): `/Users/lordcasser/workspace/projects/jarde/openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py`
- Historical runner SHA-256: `51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33`
- Adapter SHA-256: recorded in `adapter-pin.json`; recompute after any edit.
