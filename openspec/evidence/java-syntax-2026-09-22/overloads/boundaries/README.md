# Overload boundary audit

This is a read-only audit of self-written Java inputs. It does not edit jarde production code, tests, fixtures, parent overload documents, or build artifacts. The original `.class` in each case is the baseline. Every candidate source is compiled as emitted; no method body was hand-fixed.

Environment:

- `javac 23.0.1`, with `--release 8 -g:none`
- `jadx 1.5.6`
- `/Users/lordcasser/workspace/projects/jarde/target/debug/jarde-cli` (`jarde-cli 0.1.0`)
- baseline classes use `javap -c -p` for bytecode facts

The reproducible per-case driver is `run-boundaries.py`. The first run accidentally merged CLI stdout and stderr; `redo-cli.py` reran only the CLI stage with stdout saved as `jarde.java.txt` and stderr saved as `jarde.report.txt`. All result claims below use the corrected files.

## Commands

For each case, the audit ran:

```sh
javac --release 8 -g:none -d original-classes original-src/Case.java original-src/BoundaryRunner.java
java -cp original-classes BoundaryRunner
javap -classpath original-classes -c -p Case
/Users/lordcasser/workspace/projects/jarde/target/debug/jarde-cli class-source \
  --input Case.class --class Case --policy single-class --release 8 --format text \
  >jarde.java.txt 2>jarde.report.txt
javac --release 8 -g:none -d jarde-classes jarde-src/Case.java jarde-src/BoundaryRunner.java
java -cp jarde-classes BoundaryRunner
jadx -d jadx Case.class
sed '/^package defpackage;$/d' jadx-raw.java >jadx-src/Case.java
javac --release 8 -g:none -d jadx-classes jadx-src/Case.java jadx-src/BoundaryRunner.java
java -cp jadx-classes BoundaryRunner
```

The only JADX edit is removal of its invented `package defpackage;` line so the default-package runner can compile. No recovered method body was changed.

## Whole-class baseline

`whole-class-v3/original-source/OverloadEdges.java` and the class copied from it are the baseline. Original output is:

```text
nullObject=1
arrayObject=3
boxObject=5
lambdaRunnable=7
lambdaSupplier=8
methodRefRunnable=7
```

The exact jarde whole-class source is in `whole-class-v3/jarde-source/OverloadEdges.java`. It fails the real `javac --release 8 -g:none` compile because the recovered `lambda$lambdaRunnable$0()` collides with javac's compiler-synthesized symbol; see `whole-class-v3/jarde-javac.log`. This is retained as a real failure, and no source workaround was applied. JADX source is in `whole-class-v3/jadx/sources/defpackage/OverloadEdges.java`; after only package-line removal it compiles and executes with the six baseline values, recorded in `whole-class-v3/jadx-run.txt`.

## Independent cases

`original-run.log`, `jarde-run.log`, and `jadx-run.log` are the exact execution outputs. A jarde compile failure is recorded in `jarde-javac.log`; `not run` in its run log means no output was claimed.

| Case and source location | Baseline | jarde emitted source/result | JADX result | Boundary fact |
| --- | --- | --- | --- | --- |
| `NullObject` (`(Object)null`), `cases-v1/NullObject` | `run=1` | compiles, `run=2` | `run=1` | `choose(null)` selects `String` instead of descriptor `Object` |
| `ArrayObject` (`(Object) String[]`), `cases-v1/ArrayObject` | `run=3` | compiles, `run=4` | `run=3` | array upcast is lost; `String[]` overload wins |
| `BoxObject` (`(Object) Integer.valueOf`), `cases-v1/BoxObject` | `run=5` | compiles, `run=6` | `run=5` | boxing value keeps `Integer` static type instead of `Object` |
| `LambdaRunnable`, `cases-v1/LambdaRunnable` | `run=7` | **compile fails**: `lambda$run$0` compiler-synthesized symbol conflict | `run=7` | emitted targetless lambda plus recovered synthetic method is not a compilable source pair |
| `LambdaSupplier`, `cases-v1/LambdaSupplier` | `run=8` | compiles, `run=8` | `run=8` | Supplier-compatible body happens to retain the same overload |
| `MethodRefRunnable`, `cases-v1/MethodRefRunnable` | `run=7` | compiles, `run=8` | `run=7` | `System::nanoTime` loses `(Runnable)` and resolves as Supplier |
| `MultiArgs` (`pick((Object)null,(Object)"x")`), `cases-v1/MultiArgs` | `run=1` | compiles, `run=2` | `run=1` | both argument static types are lost; String pair wins |
| `CtorOverloads` (`new CtorOverloads((Object)null)`), `cases-v1/CtorOverloads` | `run=1` | compiles, `run=2` | `run=1` | constructor overload reselects the String constructor |
| `NarrowOverloads` (`(byte)3`, `(short)3`), `cases-v1/NarrowOverloads` | `runByte=1`, `runShort=3` | compiles, `runByte=2`, `runShort=4` | `runByte=1`, `runShort=3` | JLS invocation narrowing casts are dropped; int overloads win |

The key emitted expressions are preserved in each `jarde.java.txt`: `choose(null)`, `arr(arg0)`, `boxed(java.lang.Integer.valueOf(arg0))`, `action(java.lang.System::nanoTime)`, `pick(null, "x")`, `new CtorOverloads(null).code`, `onlyByte(3)`, and `onlyShort(3)`.

All original and candidate source/class/log files are retained below this directory. SHA-256 values are in `/tmp/jarde-overload-boundaries/class-sha256.txt`.
