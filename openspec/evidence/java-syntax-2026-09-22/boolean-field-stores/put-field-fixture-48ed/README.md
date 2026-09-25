# Minimal Java 8 `Z` field-store fixture

`ZFieldStores.java` is compiled with `javac --release 8`, then only the
`instanceFlag` and `staticFlag` field declarations and their matching Fieldref
descriptors are changed from `I` to `Z`. The resulting class is run with
`java -Xverify:all`. `patch_field_stores.py` freezes every method's Code-array
SHA-256 before and after this descriptor-only edit; all 11 Code attributes are
unchanged.

`ZFieldStoresRunner` exercises direct and produced instance/static stores with
`-2`, `-1`, `0`, `1`, `2`, `Integer.MIN_VALUE`, and `Integer.MAX_VALUE`. It also
checks a null direct receiver, a producer that throws before a null field
store, a successful producer followed by the null store, a normal produced
receiver store, producer call counts, and ordinary boolean parameter/literal
writes to instance and static boolean fields.

Replay all stages with:

```sh
python3 run_audit.py
```

The replay is pinned to `/tmp/jarde-cli-narrow-field-stores-48ed`, whose SHA-256
is recorded before and after each run. It saves each command, stdout, stderr,
and status. The host used for this snapshot reports `javac 23.0.1`,
`java 23.0.1`, and `jadx 1.5.6`; the class files target Java 8 (major version
52).

Snapshot results:

- Source compile/runtime: `0` / `0`; source output has 40 lines.
- Patched runner compile/runtime: `0` / `0`; patched class passes
  `-Xverify:all` and produces 40 lines.
- jarde class-source / full-class compile/runtime: `0` / `0` / `0`. The frozen
  pre-implementation output is RED against the patched JVM on 26 of 40 lines:
  all six int-to-`Z` writer bodies become no-ops, so producer calls and null
  exceptions disappear. The six ordinary boolean control outputs still match.
- JADX decompile / full-class compile: `0` / `1`. The generated source retains
  six assignments from `int` expressions to `boolean` fields, so it does not
  compile; the runtime stage is marked not run.

Class SHA-256 values and all Code-array hashes are in `class-sha256.txt` and
`code-sha256.json`. Stage status and output-difference counts are in
`summary.json`; raw logs and replay commands sit beside this file.
