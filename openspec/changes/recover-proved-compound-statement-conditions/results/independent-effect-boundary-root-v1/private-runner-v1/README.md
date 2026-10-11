# Compound condition negative full-source runner (prepared only)

`run.py` is a private, unexecuted runner for the v2 source-only Java fixture. It requires a fresh
`--out` path, refuses an existing output, pins the source and standalone Runner by SHA-256, verifies
the frozen plan and Jarde CLI, resolves and verifies both JDK plans, and validates the plan's full
57-jar JADX product list.

The execution plan is intentionally one-class and bounded: compile the original source with the
pinned Java 8 compiler; capture its actual `javap -v -p -c`; decompile that complete class with the
plan-pinned JADX jars; capture complete Jarde `class-source` JSON and full text in default/all modes;
compare complete text and per-method text/source-map rows; then compile each available complete class
source plus the independent Runner with both pinned JDKs and run under `-Xverify:all`. A JADX package
namespace adaptation, if needed, changes only the Runner import. If a Jarde source fails compilation,
the raw command is retained and runtime is marked unavailable rather than reported as executed.

Every command records argv, cwd, exit, duration, raw stdout/stderr and their byte/SHA pins. It records
class sets and individual class pins, tool/source/plan pins, source-map comparison, and expected-output
match observations. The existing one-second process-group guard is set to a 5 GiB free-space floor,
1 GiB repository `target` ceiling and 1 GiB output ceiling, with a 60-second per-command timeout and
SIGTERM/SIGKILL group termination. The initial and post-command guard checks use the shared workspace
and output paths, matching the root runner convention.

Expected output is the eight lines `hit:3`, `hit:1`, `miss:-1`, `hit:-1`, repeated for the assignment
method. The runner intentionally does not infer whether refusal occurs in region proof or Builder,
and it does not auto-accept a result. Root must run it with a new output directory and review actual
reader/recovery/build evidence.

No Java/JDK/Cargo/CLI command was run while preparing this file. Root-owned artifacts are untouched.
