# Implicit cleanup finally fixture

Minimal source-only Java 8 fixture where `finally` contains only a cleanup call that may throw internally. It checks cleanup completion against both a pending return and a pending try exception. Read `analysis.md`; replay with `python3 run_audit.py`. JADX output is comparative evidence, not an oracle.
