# `Outer.super` source-binding counterexamples

This directory isolates three ways an emitted `Outer.super.m(args)` expression can stop denoting the exact method named by a physical synthetic bridge. The cases exercise overload applicability, generic substitution recorded only in `Signature`, and checked exceptions attached to an overload. They are compiler-produced Java 8 class files and source-level counterexamples; they do not claim a JVM verifier rule beyond the executions explicitly recorded here.

## Reproduction and toolchain

From the repository root, run:

```sh
python3 openspec/evidence/java-syntax-2026-09-27/outer-super-source-binding/run_evidence.py
```

The script compiles the positive fixtures with `javac --release 8 -Xlint:-options -g:none`, executes each class with `java -Xverify:all`, records `javap -v -c -p` output, and if JADX is installed decompiles and recompiles its complete source output with the same Java 8 release setting before rerunning it. All class files and other build products live in a temporary directory that is removed on exit. The evidence was produced by `javac 23.0.1`, `java 23.0.1`, and JADX `1.5.6`; `--release 8` produced class-file major version 52, but this is **not** a run on a Java 8 VM.

The deliberately invalid source variant is compiled after the positive classes are built. Its equivalent command is:

```sh
javac --release 8 -Xlint:-options -g:none \
  -classpath <classes> -d <temporary-output> \
  openspec/evidence/java-syntax-2026-09-27/outer-super-source-binding/src/checked/CheckedNarrowed.java
```

Exit status 1 is expected. The frozen diagnostic replaces the machine-specific source root with `<source>`; it reports that `IOException` must be caught or declared. The command does not leave class output in this directory.

## Evidence

**Inherited same-name overload.** [`OverloadCases.java`](src/overload/OverloadCases.java) gives `OverloadParent` an `Integer` overload and inherits `OverloadAncestor.select(Number)`. Both calls pass the same runtime `Integer`, but the source static types select different methods: `numberStaticType(Number)` prints `ancestor-number`; `integerStaticType(Integer)` prints `parent-integer`. In [`overload-OverloadCases-Member.javap.txt`](overload-OverloadCases-Member.javap.txt), each Member call is at BCI 5 and calls a different exact helper descriptor. In [`overload-OverloadCases.javap.txt`](overload-OverloadCases.javap.txt), helper `access$001` BCI 2 invokespecials `OverloadParent.select(Number)`, while `access$101` BCI 2 invokespecials `OverloadParent.select(Integer)`; each returns at BCI 5. So a source projection with a narrowed `Integer` static type binds to a different overload than the physical `Number` bridge. The runtime values alone do not recover this source binding.

**Generic superclass Signature.** [`GenericCases.java`](src/generic/GenericCases.java) extends `GenericParent<String>`, where `choose(T)` erases to descriptor `(Object)String` and competes with `choose(CharSequence)`. Substitution makes `choose(String)` the more specific source member, and the run prints `type-variable`. Its member call at BCI 5 reaches an `(Object)String` bridge; that bridge BCI 2 invokespecials `GenericParent.choose(Object)`. The class `Signature` is `GenericParent<String>` and the generic method signature is `(TT;)String`; see [`generic-GenericCases.javap.txt`](generic-GenericCases.javap.txt) and [`generic-GenericParent.javap.txt`](generic-GenericParent.javap.txt).

[`GenericRawCases.java`](src/generic/GenericRawCases.java) is the control with a raw `extends GenericParent` edge. The same String argument now selects `choose(CharSequence)` and prints `char-sequence`; its Member BCI 5 calls a bridge with a `CharSequence` parameter, and bridge BCI 2 invokespecials `GenericParent.choose(CharSequence)`, as shown in [`generic-GenericRawCases-Member.javap.txt`](generic-GenericRawCases-Member.javap.txt) and [`generic-GenericRawCases.javap.txt`](generic-GenericRawCases.javap.txt). The generic method's raw descriptor remains `(Object)String` in both cases. Therefore that descriptor by itself does not determine the source-level overload selected by `Outer.super`; the parameterized superclass edge and method Signature contribute binding information.

**Checked-exception legality.** [`CheckedOverloadCases.java`](src/checked/CheckedOverloadCases.java) has `select(Number)` without checked exceptions and a more-specific `select(Integer) throws IOException`. `Member.numberStaticType(Number)` uses `Outer.super` and compiles without `throws` or `catch`; the physical Member BCI 5 calls a Number-typed helper, whose BCI 2 invokespecials `CheckedParent.select(Number)`. [`CheckedParent`'s javap output](checked-CheckedParent.javap.txt) shows that only the Integer overload has an `Exceptions: IOException` attribute. The control [`CheckedNarrowed.java`](src/checked/CheckedNarrowed.java) changes the source parameter type to `Integer`; now overload resolution selects the checked overload and javac rejects the same no-throws method. This proves that changing the emitted static argument type can change both target identity and source legality. If a projected expression preserves the original Number static type and declaration hierarchy, this fixture does not claim that merely spelling `Outer.super` changes legality.

## JADX comparison and limits

JADX 1.5.6 emits the `Outer.super` expressions in [`jadx/sources`](jadx/sources/); those complete generated sources compile under `--release 8` and pass `-Xverify:all`. Their four outputs match the original class outputs in [`run-results.txt`](run-results.txt) and [`jadx-run-results.txt`](jadx-run-results.txt). This shows that JADX preserves the tested source binding when it retains the source signatures. It does not establish behavior for other decompilers or incomplete-signature inputs.

Jarde has not been run on these fixtures, and no Jarde output or acceptance is claimed. These examples justify a source-binding proof gate; they do not specify its implementation or establish that every possible Java overload, generic signature, or exception shape is covered. In particular, the raw generic control changes the class's declared superclass edge; it is evidence of information needed to distinguish parameterized from raw source binding, not a claim that a correct writer would intentionally discard an available `Signature` attribute.

## Frozen hashes

The SHA-256 values below are the class-file checksums printed by `javap` from the transient compiled files. The corresponding class files are intentionally not retained; rerunning the script recreates them and records their checksums in the listed `javap` files.

| Class file | SHA-256 |
| --- | --- |
| `OverloadCases.class` | `ed89056d7380f7959a5b76175c5d5195bf6b9e7ddd2a0203ec24e4f588727dc1` |
| `OverloadCases$Member.class` | `0265f78361133a5f18d08ac303821ecaf3818770bbfd886b6c014bbf2b9c9ec3` |
| `GenericParent.class` | `8cfa49ba9d85506ba16a51f09682e062227ed2858fac2a13732f452d687a13b1` |
| `GenericCases.class` | `2a1478478641ee8d4aeb606e9dfdcccb3938d1d60c08803d1c852bc9a0f27805` |
| `GenericCases$Member.class` | `6d3bc7dba9caa6130cc3b95e52f9e27907a86edad9ab173aa58b1fe8b465cf68` |
| `GenericRawCases.class` | `966ed7e0e3cc4b342c0930b90640572428051e9c1df91814407e2a1f2b515e2c` |
| `GenericRawCases$Member.class` | `7debf54a62c3df7a5f159f233925a10f5912c479dfd6c54bd5bee7ada7c4ab60` |
| `CheckedParent.class` | `e07ce3ebcf0d5533b4f91939b3cb6a395e585ae0c375d9f7d7b27b229f990edb` |
| `CheckedOverloadCases.class` | `977af307d7d3b525c4f0108a111c504974c10d4c5e391771a5e53bc06dca06a5` |
| `CheckedOverloadCases$Member.class` | `23324f2d790f386974b1a4099c8fd76ec3cc9788f1467d49f090b135a0c02828` |

The representative source fixture hashes at capture time were:

| Source | SHA-256 |
| --- | --- |
| `src/overload/OverloadCases.java` | `03f7cd89c8e3ab378b62c8ee1f989ca7cd8411b8af6025a03c86e1e347f17223` |
| `src/generic/GenericCases.java` | `498dc0a787622a556ff188cb6f3e56c8b4fd1ddc5f7482c0a1f41a61177457a6` |
| `src/generic/GenericRawCases.java` | `b64211c9a4b7ad01f90f67e23c4131013e94fde5397b7b8557376e35208951e6` |
| `src/checked/CheckedOverloadCases.java` | `9a14247d43f267f703b574b2f2151620e47fba1ef7dc2a9e996feed0018c7ebe` |
| `src/checked/CheckedNarrowed.java` | `86efed04304fcc1d6d4c6e66c1d27051271c397adf5ecb0ed46a7b4695e66d0f` |
