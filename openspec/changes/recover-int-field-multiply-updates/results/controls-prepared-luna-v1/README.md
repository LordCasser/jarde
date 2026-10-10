# Nested field multiply controls (prepared)

This directory contains a frozen Java 8 control fixture and a collector prepared for root review. No JDK, JADX, Jarde CLI, Git, or Cargo command has been run for this prepared family.

`InputFieldMultiplyControls` keeps the exact private static nested `A` / public `a` shape, the explicit-add and multiply cases from JADX's `TestFieldIncrement2`, and adds only `multiplyDivide(int)` with `this.a.f *= 8 / n`. The separate Runner creates a fresh receiver for each observation and uses reflection so the product family remains exactly the outer class plus its nested `A`.

The Runner's cases cover ordinary multiplication, signed multiplication, int overflow (`MAX_VALUE * 3`, `MIN_VALUE * -1`), zero, null target on `test2`, successful division RHS, division-by-zero retaining the old field value, and null receiver with divisor zero. The last two distinguish receiver-field read failure from RHS division failure according to the actual original execution; their expected exception names are Runner checks, not a substitute for the collector's captured original raw streams.

The collector takes a frozen Jarde CLI and metadata path plus both SHA-256 values. It also pins the existing dual-JDK manifest and JADX launcher. Each JDK original family is freshly compiled and run; JADX receives exactly the javac23 outer and nested class files (no Runner), uses default and `--rename-flags none`, and each full generated source family is compiled unchanged with only Runner package adaptation. Jarde default/all full class-source text is compiled unchanged on both JDKs. All generated-source compile legs use empty classpath/sourcepath and `-Xverify:all`. Runtime outcomes are compared as exit/stdout/stderr bytes to the same-JDK original. Operator spelling and member-family projection are recorded as observations; they do not gate semantic success.

The collector writes only `baseline-root-v1/`, refuses to overwrite it, retains command argv and raw streams, copies original source inputs, captures original physical class/javap facts, and closes its file inventory. No candidate result is predeclared as passing.
