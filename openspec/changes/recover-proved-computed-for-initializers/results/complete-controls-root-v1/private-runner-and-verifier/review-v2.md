# JADX package adapter v2 review

V1 is preserved at `/private/tmp/jarde-computed-for-controls-jadx-root-v1`; this v2 runner defaults to the fresh path `/private/tmp/jarde-computed-for-controls-jadx-root-v2`.

I inspected all six complete-source outputs from v1. Every source has exactly one top-level package declaration, `package defpackage;`:

| Input case | JADX source SHA-256 | Declared package | Qualified runner main |
|---|---|---|---|
| controls-javac8-original | `7a4a9336c93eb88586280535ea51c5ff0c1dc2adf096766340b335c356c4dbfc` | `defpackage` | `defpackage.ComputedForControlsRunner` |
| controls-javac8-pending | `16a67623e633e9377653c1c34285c9f7dc985e9099b98081b31caf3d4c11057f` | `defpackage` | `defpackage.ComputedForControlsRunner` |
| controls-javac23-original | `7a4a9336c93eb88586280535ea51c5ff0c1dc2adf096766340b335c356c4dbfc` | `defpackage` | `defpackage.ComputedForControlsRunner` |
| controls-javac23-pending | `16a67623e633e9377653c1c34285c9f7dc985e9099b98081b31caf3d4c11057f` | `defpackage` | `defpackage.ComputedForControlsRunner` |
| direct-return-javac8-original | `1d195698faa81cbb37b2c7939e2b71358a6f4fb02b2eaef4d60f554515d37653` | `defpackage` | `defpackage.ComputedForDirectReturnRunner` |
| direct-return-javac23-original | `1d195698faa81cbb37b2c7939e2b71358a6f4fb02b2eaef4d60f554515d37653` | `defpackage` | `defpackage.ComputedForDirectReturnRunner` |

Both original Runner files have no package declaration. V2 parses exactly one package declaration from each unchanged complete JADX source, prepends only `package <parsed-name>;\n` to the unchanged Runner bytes, checks that the remaining Runner body is byte-identical, and verifies the complete decompiled source hash is unchanged. Runtime argv uses the correspondingly qualified main. The execution record stores original/adapter hashes and the exact prefix per input/JDK build.

Root's v1 execution has 47 captured commands. All 12 JADX whole-source compile attempts stopped because the Runner could not resolve class references across `defpackage`; no JADX runtime was attempted. The Jarde controls full-source compile attempts failed (whole-class compile only), while the four direct-return Jarde full-source compile/runtime cases succeeded. V2 reruns fresh JADX and Jarde observations in a separate output directory; v1 raw files remain untouched.

Use:

```bash
python3 /private/tmp/jarde-computed-for-controls-jadx-luna-v1/run-comparison-v2.py --output /private/tmp/jarde-computed-for-controls-jadx-root-v2
```
