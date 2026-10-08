# Class-scope generic constructor evidence

This fixture set tests whether a decompiler can restore a constructor parameter whose `Signature` refers to a class type variable. Each fixture is compiled independently; the test Driver is compiled outside the input JAR and is never part of the decompiler input.

The set has nine full-reflection positive shapes: direct `T`, bounded `T extends Number`, `T[]`, mixed `long`/`double`/`T` slots, stored category-2 values, two class variables with separate field stores, one parameter consumed by two field writes, an unused class-generic parameter, and a raw same-class `new` call from a static factory. Ten conservative boundaries cover `Object`, erased casts, reassigned parameters, conditional/phi values, method-call RHS, constructor type-variable shadowing, `U` flowing to a `T` field, a non-Object superclass constructor, `this(...)` delegation, and exception handling around a method-produced RHS. `PeerNewHold` is a separate partial-positive boundary: a body allocation makes its one-argument constructor and shared field stay erased, while its directly storing two-argument overload can retain the class binder.

`ConstructorDriver.java` constructs each class through reflection, verifies it with `-Xverify:all`, and emits separate `BEHAVIOR|` and `REFLECT|` rows. It compares each type variable by `TypeVariable.getGenericDeclaration()` object identity, so a constructor variable named `T` is distinguishable from a class variable also named `T`. It records every constructor's formal parameter count, every parameter type and binder, constructor type variables and bounds, class type variables and bounds, and each generic field's type and binder. Array component binders are reported recursively. Array values use a stable textual form.

The frozen originals use these compiler legs:

- Corretto 8u432 at `/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home`, with `-source 8 -target 8`.
- OpenJDK 23.0.1 at `/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home`, with `--release 8`.
- Each compiler leg uses both explicit `-g` and `-g:none` modes.

Each class has its own original source copy, class files, one-class JAR, `javap -p -v -c` output, SHA-256 manifest, original execution/reflection output, and JADX decompilation. The external Driver is not in any JAR. The JAR contains only the target class, including the superclass-boundary case, which extends JDK `Vector<T>`.

JADX is `/opt/homebrew/bin/jadx`, version 1.5.6, SHA-256 `64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7`. The command used is `jadx -d <output-dir> <class.jar>`. The replay removes only JADX's default-package line before compiling the emitted standalone class, then runs the external Driver against only the generated class directory and Driver classes.

The frozen Jarde baseline CLI is `/tmp/jarde-class-ctor-baseline-cli`, SHA-256 `34a5288badf6fb40020c5117ef749ae4705545b29ee35a9fefda48de24276b53`. On all 72 inputs, it returned CLI status 0 and a nonempty class-source header. Its source compiled and ran in 64/72 cases; all 64 successful runs matched original behavior. The eight compile failures are `CallHold` and `ExceptionHold` across the four compiler/debug legs: the emitted constructor parameter is `Object`, then the source passes it to a same-class method requiring `T`. They are nonempty source outputs, not body-refusal markers. This same-class generic callee argument proof is a separate recovery boundary.

The initial JADX replay compiled and ran 56/72 cases with behavior and full reflection matching on all 56. Its compile failures are `CrossHold`, `ErasedCastHold`, `ObjectHold`, and `ShadowHold`, each across the four compiler/debug legs. The emitted sources are preserved with the compiler diagnostics.

`RawNewHold<T>` adds a raw `new RawNewHold(value)` call from a static factory whose declared return type is raw `RawNewHold`; its `T` constructor and `T` field remain the positive constructor projection target. `PeerNewHold<T>` directly stores its two-argument constructor parameter, then constructs a second instance into a local variable from the one-argument constructor body. That body also writes its `Object` parameter into the shared field, so the conservative source projection keeps that overload and the field erased. The external Driver invokes both constructors. This records a usable constructor-level binder without claiming full-field restoration. Both inputs are frozen under all four compiler/debug legs and included in the addendum replay.

The baseline CLI compiled and ran all eight inputs from the two new families with behavior matching all eight. It retained erased constructor signatures for both families, so generic reflection differed. The addendum also preserves the original-source execution, JADX source/compile/run evidence, baseline CLI source, and compiler diagnostics for both new families.

The strengthened replay compares candidate compile, run, and behavior results against every baseline input that compiled successfully. `ThisDelegateHold` is also compiled as a whole class and run under `-Xverify:all` for all four frozen legs; its physical erased constructor signature is retained while runtime behavior is compared with the original.

`CrossHold<T,U>` is a separate constructor-parameter boundary: source reflection says the constructor parameter belongs to class variable `U`, while field `v` belongs to `T`. The correct output may preserve the constructor's `U` signature while erasing field `v` to `Object`; whole-class reflection equality is not required for this boundary. The replay reports constructor, field, and class reflection comparisons separately.

## Rebuild and replay

```sh
python3 generate.py
python3 freeze.py                 # all original compiler/JADX evidence
python3 freeze.py --only Hold CrossHold  # selective new evidence
python3 replay.py --baseline /tmp/jarde-class-ctor-baseline-cli
python3 replay.py --baseline /tmp/jarde-class-ctor-baseline-cli --candidate /path/to/candidate-cli
python3 replay.py --baseline /tmp/jarde-class-ctor-baseline-cli --candidate /path/to/candidate-cli --out runs-addendum
python3 replay.py --baseline /tmp/jarde-class-ctor-baseline-cli --only RawNewHold PeerNewHold --out runs-new-families
```

`replay.py` always uses fresh temporary generated-class directories. Its classpath contains only that generated class output and the separately compiled Driver; it never includes the original JAR or class directory. It writes `summary.json`, CLI reports, compile diagnostics, runtime output, and separate behavior/reflection diffs under `runs/` or the selected `--out` directory. CLI statuses other than 0 or 4, missing class-source headers, failed required positive candidate shapes, and regressions on baseline-compilable families are recorded before the script exits nonzero. Negative boundary compile failures remain recorded outcomes and are not counted as restorations.

The evidence tree omits zero-byte `.stdout` and `.stderr` files because they contain no diagnostic or output bytes; their corresponding `.exit` records remain, and replay recreates the log files. Under `results/`, `class-source.stdout` is omitted only when its bytes exactly match the sibling `report.json`; the JSON report is retained as the authoritative copy, and replay can regenerate the duplicate stdout capture. All nonempty logs, distinct reports, source/class/JAR/Javap artifacts, reflection output, exit records, and historical failure evidence remain saved. The frozen-input SHA manifest is kept unchanged during this cleanup; it retains all 240 input entries with their complete relative paths.
