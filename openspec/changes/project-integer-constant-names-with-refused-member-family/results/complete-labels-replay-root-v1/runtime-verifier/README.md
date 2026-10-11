# CF12 v5 captured-runtime verifier

`verify_v5.py` is a read-only verifier for the actual v5 `execution.json`. It launches no CLI, JDK, Rust, Cargo, or Git process.

It validates all 32 indexed command records and each stdout/stderr file length and SHA-256; binds the old/new CLI, new build record and build-binding CLI; rechecks all 4,695 captured live source pins; checks frozen input classes, probe/JADX/Jarde source and class pins; requires exactly eight fresh renders; and requires the four exact JDK × mode profiles. Each profile must contain successful JADX, Jarde, and probe compiles with inputs matching their pinned sources, exactly two original/JADX/Jarde class entries, and original/JADX/Jarde runtime commands with the correct JVM, verifier option, classpath and probe arguments. Runtime stdout is checked against the exact ten captured label observations and must match byte-for-byte across the three sides in each profile.

Run with:

```sh
python3 verify_v5.py /private/tmp/jarde-cf12-labels-full-replay-root-v5/execution.json
```

Observed result is recorded in `verification.json`. This verifies the captured replay records and outputs; root review and any broader product acceptance remain separate.
