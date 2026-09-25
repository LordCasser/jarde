# Finally completion evidence

Source-only Java 8 audit of `try/finally` completion order, return/throw override, and side effects. See `analysis.md` for findings and the OpenSpec catch-all boundary. `run_audit.py` replays compilation, `-Xverify:all` runs, JADX/Jarde output, and complete-source compilation. No generated source is edited. Jarde compile stderr is retained even though it prevents a runtime comparison.
