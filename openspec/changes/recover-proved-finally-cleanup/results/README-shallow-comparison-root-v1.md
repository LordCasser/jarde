# Shallow finally/loop comparison preparation

This preparation uses only the existing `depth-02` inputs and records from
`depth-boundary-root-v1/run/result.json`. It reuses the two original `DeepFinally.class` files,
the fixed complete `Runner.java`, the saved complete Jarde `class-source.txt` and document JSON,
and the exact JDK binary identities already pinned in that result. It also verifies the frozen
returned-array CLI metadata, binary, and candidate source hashes before comparison.

The new output is `shallow-comparison-root-v1/`. The script copies the original Jarde text without
editing it, compiles it with the unchanged Runner using empty classpath/sourcepath, and runs it with
`-Xverify:all` only if compilation succeeds. It freshly decompiles each original class with JADX
1.5.6 under default and `--rename-flags none`, compiles every emitted Java source plus the complete
Runner with empty classpath/sourcepath, and likewise runs only newly generated classes after a
successful compile. A package prefix may be added to the Runner to match JADX's generated package;
generated source and failed methods are preserved.

The original depth-02 oracle is `exit 0`, stdout `run(0)=0 trace=1\nrun(40)=1 trace=2\n`, and its
recorded stderr. Candidate success compares exit, stdout, and stderr byte-for-byte with that leg's
original raw output. Candidate compile failures remain recorded and do not trigger an attempted run.
A complete run with any candidate compile, decompile, or behavioral mismatch is recorded as
`comparison-complete-with-candidate-failures`; raw command results remain available. The script
and this note are preparation only; no command in the new comparison has been executed.

Run after review with:

```sh
python3 openspec/changes/recover-proved-finally-cleanup/results/prepare-shallow-comparison-root-v1.py
```

Root executed the reviewed script on 2026-10-10. Fresh JADX complete candidates pass 4/4;
the saved complete Jarde candidates fail compilation 0/2 with the actual Chinese missing-return
diagnostic. Root verifier v2 independently passes 232 checks, preserving the v1 English-diagnostic
assumption failure. See shallow-loop-finally-analysis-root.md and the closed manifest/inventory.
