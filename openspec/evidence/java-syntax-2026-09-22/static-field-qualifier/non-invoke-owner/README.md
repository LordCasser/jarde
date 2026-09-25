# Non-invocation qualifier with a mismatched static target owner

This source-only negative checks OpenSpec 2.2's non-`Invoke` boundary. Replay from the repository
root with:

```text
python3 openspec/evidence/java-syntax-2026-09-22/static-field-qualifier/non-invoke-owner/run_audit.py
```

It uses javac 23.0.1 with `--release 8 -g:none`, the frozen Jarde CLI at
`/tmp/jarde-cli-null-root-final` (SHA-256
`30481c1449b5a1869773d21b934789d42bb1d6a9cab8d81f873d50055af4e34c`), JADX, and the JVM verifier.
The script writes each command's output/status, the original and patched class files, generated
JADX/Jarde sources, and `summary.json` in this directory. It does not modify Rust or Cargo files.

`NonInvokeQualifierProbe.java` has SHA-256
`6335b22e1578549ccd562a855e021c0b83aaf5dfd173046289e45d45f48128a4`. It compiles to a 224-byte
class with SHA-256 `e56d77cbee428319c37cd5f43e63efee7554a06892b1ab7d9cf0188a4329e962`; both methods
have `Code` attributes. Its `call(Child arg) { return arg.ping(); }` body is:

```text
0: aload_0
1: pop
2: invokestatic #7 // Method Child.ping:()I
5: ireturn
```

`Child extends Base`, and `Base.ping()` returns 31. `javac` legally compiles the expression-
qualified static call and puts `Child.ping:()I` in Methodref #7. The invocation has no receiver on
the operand stack: `arg` was loaded and discarded at BCI 1.

The replay changes only the constant-pool class name behind Methodref #7 from `Child` to unrelated
`Other`; `Other.ping()` has the same descriptor and returns 41. The owner name has the same UTF-8
length, so the patch changes exactly bytes 73–77. The patched class hash is
`5b6875ef447c448fa6dfaa65d5fdbeb0aa7e0848400bec8308df8e7649ce1180`. `javap` shows the same BCI and
instruction sequence, with only the Methodref now naming `Other.ping:()I`. The original and patched
classes both pass `java -Xverify:all`; the original prints `value=31`, and the patched class prints
`value=41`.

JADX's full generated class compiles and runs correctly in both cases: `Child.ping()` yields 31 and
`Other.ping()` yields 41. Jarde's full generated class compiles and runs in both cases, but it writes
`return arg0.ping();` for both Methodrefs and prints 31 for the patched class. That Java expression
selects `Base.ping()` through the declared type `Child`, so the owner-only patch silently rebinds the
static call. Here the popped producer is an `aload`, so it cannot be emitted as an independent Java
expression statement. This confirms that the expression-qualified path needs proof that the
qualifier's Java type selects the original Methodref target; reference-typed alone is insufficient.

All commands, the two complete `javap -c -p` listings, generated sources, hashes, and exact runtime
outputs are recorded in `summary.json` and the adjacent `.stdout`, `.stderr`, and `.status` files.
Root copied this whole source-only directory to a temporary location and independently reran
`run_audit.py` with the same frozen CLI. The replay exited successfully; the original and patched
class hashes, JVM outputs 31/41, JADX patched output 41, Jarde patched output 31, and unchanged CLI
hash matched the recorded evidence. This is a separate non-`Invoke` owner-selection negative for
OpenSpec task 2.2, not a new implementation mechanism.
