# Java 8 try-with-resources source-only audit

Replay from the repository root with:

```sh
python3 openspec/evidence/java-syntax-2026-09-22/try-with-resources/run_audit.py
```

The script compiles each handwritten class with `javac --release 8 -g:none`, saves the real class and `javap -v -c -p` output, runs JADX 1.5.6 and the frozen CLI, then compiles each complete generated class without editing it. A generated class that fails `javac` is recorded and its runtime is skipped. Every runnable variant uses `java -Xverify:all`; the script saves raw stdout/stderr/status, per-line runtime comparisons, class files, generated source, and JSON summaries.

The frozen CLI is `/tmp/jarde-cli-class-literals-root`. Its required SHA-256, `25bf181ccf0d879351a16710818e27efba0df6b3851931a29eb98d9f41adb1e0`, is checked before and after replay and recorded in `cli-sha256-before.txt`, `cli-sha256-after.txt`, and `audit.json`. Tool versions are in `tool-versions.txt`.

| Sample | Runtime oracle | Whole-class source result |
| --- | --- | --- |
| [`core/`](core/) | Normal body/close, body plus failing close, and failed initialization preserve counts and exception identity. Original, JADX, and jarde outputs match line by line. | Original, JADX, and jarde compile and pass `-Xverify:all`. |
| [`multi-resource/`](multi-resource/) | Three close failures are suppressed by identity in order 3, 2, 1. | Original, JADX, and jarde compile and pass `-Xverify:all`; outputs match line by line. |
| [`null-resource/`](null-resource/) | Null resource runs the body once and never calls close. Original and JADX outputs match. | Original and JADX compile and pass `-Xverify:all`. Jarde output does not compile at `Object local0.close()`; its runtime is intentionally skipped. |

The null refusal analysis, its limits, and overlap with existing P3 evidence are in [`analysis.md`](analysis.md). This audit adds evidence only; it does not edit production code, OpenSpec changes, or tests.
