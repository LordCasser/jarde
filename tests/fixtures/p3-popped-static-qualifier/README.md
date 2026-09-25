# P3 fixture: a discarded invocation before a static field write

`v8/StaticQualifierProbe.class` is the real Java 8 class used for the initial whole-class audit.
`StaticQualifierProbe.java` and `StaticQualifierRunner.java` are its source-only generation and
execution inputs. They were compiled by **javac 23.0.1** (OpenJDK 23.0.1) with:

```text
javac --release 8 -g:none -d classes StaticQualifierProbe.java StaticQualifierRunner.java
```

The checked-in class is 721 bytes, class-file version 52.0, with seven `Code` attributes and no
debug attributes. Its SHA-256 is
`21cdace20a84accd06e52567c250b0359ef76e059df993cc6c00a9fe1d6b7266`. The source SHA-256 is
`15b9fac8342c6dad5dcadffa7ff63a7fc5bacd478655e7acbc3e49ce0d1a31a4`; the runner SHA-256 is
`98ebf6ab2338cd732d68bf1e15d4e1d7812d080b893eca80e53f071f21147236`.

The complete pre-fix replay is reproducible with:

```text
python3 tests/fixtures/p3-popped-static-qualifier/run_audit.py
```

It checks the frozen Jarde CLI at `/tmp/jarde-cli-null-root-final` against SHA-256
`30481c1449b5a1869773d21b934789d42bb1d6a9cab8d81f873d50055af4e34c`, verifies that javac
reproduces the checked-in class, decompiles it with JADX and Jarde, then compiles and executes each
whole class with `-Xverify:all`. The four-line output, complete generated sources, class disassembly,
compiler/runtime diagnostics, statuses, and summary are written under
`openspec/evidence/java-syntax-2026-09-22/static-field-qualifier/fixture-replay/`.

## Pre-fix result

Original and JADX both compile and run to:

```text
read=5:select=1:rhs=0
write=7:select=1:rhs=1
sum=12:select=1:rhs=1
fail=ISE:select=1:rhs=0:value=5
```

Jarde's complete output also compiles and runs, but `write` calls `receiver()` twice:

```text
read=5:select=1:rhs=0
write=7:select=2:rhs=1
sum=12:select=1:rhs=1
fail=ISE:select=1:rhs=0:value=5
```

In `write()`, javac emitted `invokestatic receiver @0; pop @3; invokestatic rhs @4; putstatic value @7;`
followed by the read and return. The source uses the receiver expression to qualify the static
field. Its value is discarded before the RHS call. The current decompiler writes `receiver();`
for the discarded call and then writes `receiver().rhs()` as the next static call's qualifier. The
second evaluation is observable even though this whole class remains compilable. `read()` and
`writeSum()` are controls: their output still evaluates the receiver once. In the failing receiver
case, the first call throws before either RHS execution, so `rhs=0` remains the expected result.

The independent regression in `tests/p3_popped_static_qualifier.rs` asserts one receiver call and
one RHS call in `write()`, and keeps the read and compound-write controls. It has not been run in
this fixture-preparation step; the Rust/Cargo window is reserved for the implementation and root's
independent replay.

The root agent independently reran `run_audit.py` after fixture handoff. It exited 0 and confirmed
the 721-byte/7-Code input hash, byte-for-byte source recompilation, four matching original/JADX
lines, Jarde's `write` mismatch, and the unchanged frozen CLI hash.
