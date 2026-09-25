# Java 8 postfix lvalue return audit

This is an independent, source-only fixture for the old value returned by
`return receiver().value++` and `return array()[index()]++`. It does not change
Rust/Cargo sources or OpenSpec planning files. Rebuild every artifact with:

```sh
python3 openspec/evidence/java-syntax-2026-09-22/postfix-lvalue/run_audit.py
```

The script compiles the hand-written inputs with `javac --release 8 -g:none`,
runs the original classes under `java -Xverify:all`, records `javap -c -p`,
decompiles the complete `PostfixProbe` class with JADX and the frozen Jarde CLI,
then compiles and runs each complete generated class with the same helper and
runner. It never edits generated methods. `summary.json` records tool paths and
SHA-256 values before and after the run, source hashes, the primary class hash,
class size and number of `Code` methods, and each compile/run result. The
original class files are kept under `original/classes/`.

Root also copied this directory to a temporary path and reran the script independently against
the same frozen CLI. The replay exited successfully and reproduced the 1,228-byte/11-`Code` class
hash, all 19 original/JADX lines, the two Jarde missing-return compiler errors, and an unchanged
CLI hash. This verifies the baseline for OpenSpec task 1.1; further negative bytecode boundaries
remain under task 1.2.

## Results

| Variant | Whole class compile | Whole class run | Runtime observations |
| --- | ---: | ---: | --- |
| Original javac class | 0 | 0 | 19 expected observations |
| JADX 1.5.6 | 0 | 0 | All 19 lines match original |
| Jarde `class-source` | 1 | Not run | javac reports missing return statements in `postReceiver()` and `postArray()` |

The Jarde source and compiler error are preserved verbatim in `jarde/`. Since
the complete generated class does not compile, this audit makes no Jarde runtime
equivalence claim. The local increment, simple field post-increment, and simple
field pre-increment controls remain in the fixture; no generated source was
patched to make the class compile.

Original-class SHA-256 is
`7f9e8105b6ed95267ac7eb20c792eeb42303144bd2a49af3f496ce280ff26893` (1,228
bytes, 11 methods with `Code`). Frozen Jarde CLI:
`/tmp/jarde-cli-bitwise-root-after`, SHA-256
`88f2e7aa9b5b8172da02f5af5d4d2c029e72a52d9b3298b46b82774e0bd5fdfd`. JADX
binary: `/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx`, SHA-256
`64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7`.
Full hashes and executable paths for `javac`, `java`, and `javap` are also in
`summary.json`.

## Bytecode shape and observations

Offsets below are bytecode indices from `original/original-javap.stdout`:

| Method | BCI sequence | Meaning |
| --- | --- | --- |
| `postReceiver()` | `invokestatic receiver@0; dup@3; getfield@4; dup_x1@7; iconst_1@8; iadd@9; putfield@10; ireturn@13` | Keep receiver once; `dup_x1` retains the old field value for return while the incremented value is stored. |
| `postArray()` | `invokestatic array@0; invokestatic index@3; dup2@6; iaload@7; dup_x2@8; iconst_1@9; iadd@10; iastore@11; ireturn@12` | Evaluate array then index once; `dup_x2` retains the old element value while the incremented value is stored. |
| `localIncrement(int)` | `iinc@2; iload@5; ireturn@6` | Ordinary local increment control. |
| `postSimpleField()` | `aload_0@0; dup@1; getfield@2; dup_x1@5; iconst_1@6; iadd@7; putfield@8; ireturn@11` | Simple instance-field post-increment control. |
| `preSimpleField()` | `aload_0@0; dup@1; getfield@2; iconst_1@5; iadd@6; dup_x1@7; putfield@8; ireturn@11` | Prefix control returns the new field value. |

The original and JADX runner observations verify old/new values `41/42` for
the receiver field and `70/71` for the array element, with exactly one call in
the order `R` and `AI`. Null receiver throws `NullPointerException` after `R`;
null array evaluates both array and index (`AI`) before `NullPointerException`;
an out-of-range index evaluates `AI` before `ArrayIndexOutOfBoundsException`.
Incrementing `Integer.MAX_VALUE` returns `2147483647` and stores
`-2147483648`. The local control returns `9`; field postfix returns `12` and
stores `13`; field prefix returns and stores `14`.

## Files

- `PostfixProbe.java`, `PostfixBox.java`, `PostfixRunner.java`: hand-written fixture.
- `run_audit.py`: deterministic rebuild, decompile, whole-class compile and run.
- `original/`: original class files, javap listing, and run logs.
- `jadx/`: full generated source, compiler output, and runtime output.
- `jarde/`: unmodified full generated source, report, and compiler output.
- `summary.json`: machine-readable hashes and result summary.
