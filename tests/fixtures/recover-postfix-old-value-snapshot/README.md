# `recover-postfix-old-value-snapshot` fixtures

The postfix old-value **consumer positions** of `recover-postfix-old-value-snapshot`, frozen on both
compiler legs. Eight classes, each compiled twice from the same source:

```sh
javac --release 8 -Xlint:-options -nowarn -d v8 *.java
/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -nowarn -d v8-javac8 *.java
```

`v8/` is javac 23.0.1 `--release 8`; `v8-javac8/` is real javac 8 (Corretto 1.8.0_432). Every
`*.class` under both legs is listed in `sha256.txt`, together with the eight sources.

## The classes

| class | shape | anchor |
| --- | --- | --- |
| `CM` | the postfix-old-value patrol's four-shape discriminator: `incDec` holds `int j = i++`, `int k = ++i`, `int l = i--`, `int m = --i`; `compound`/`compoundInExpr`/`compoundField`/`incField` are the healthy shapes the patrol recorded as already recovered | the local snapshot position and the healthy controls |
| `CM2` | `immUse` (`int j = i++;`), `postfixExpr` (`return i++ + 10;`) and the `crossStmt`/`prefix` controls | the two local consumer positions the patrol recorded as refused |
| `AD` | `void add(Object t){ elems[size++] = t; }` — the `ArrayList.add` core, byte-identical to the array-store patrol's `ad.jar!AD.class` | the flagship: the array reference survives the field `putfield` dance |
| `GA` | the same shape over a generic collection's own `Object[]` field, byte-identical to the patrol's `ga.jar!GA.class` | the generic form of the flagship |
| `PT` | `pos < src.length ? src[pos++] : null` — a static field's postfix as an array index inside a ternary arm | the conditional-arm position |
| `SR` | `a[i] = i++` (the store's right side is the old value) and `a[i--] = a[0] + 100` (the local postfix as the store's index) | the array-store right side and the local index write |
| `IX` | the local postfix as an array index (`a[i++] = 10`, `a[i--]`) and the static field's postfix as an array index (`IX.arr[IX.idx++] = 20`, `IX.arr[IX.idx--]`) | the index positions, local and static |
| `NG` | the negatives: `i = i++`, `i = i--`, `f = f++`, `i += i++ + 1` (the multi-consumer form) and `while (xs[i++] != 0 && …)` (a Phase-B condition position) | the Non-Goal and the out-of-scope gate |

`CM`, `CM2`, `AD` and `GA` are the patrols' own sources, copied byte for byte
(`openspec/evidence/java-syntax-2026-10-05/postfix-old-value-patrol/fixture/{CM,CM2}.java`,
`.../array-store-soundness-patrol/fixture/{AD,GA}.java`). The `AD`, `CM` and `GA` class files this
leg produced are **byte-identical** to the patrols' frozen jars:

```sh
shasum -a 256 v8/AD.class                                  # 8418a5c4…7008c
unzip -p openspec/evidence/.../array-store-soundness-patrol/fixture/ad.jar AD.class | shasum -a 256
```

## Behavior

The classes that recover as a whole compile with both compilers and run under `-Xverify:all` with
the same standard output and exit status as their own class files. The tests in
`tests/recover_postfix_old_value_snapshot.rs` pin that on both legs; the change's
`results/behavior.sh` records the same runs from the command line. Two fixtures are exempt from the
whole-class replay for reasons that are **not** this slice's:

* `GA.main` names its nested interface through the pool spelling (`GA$Cfg`), so the whole class does
  not compile as Java source — a pre-existing presentation of that member, present at the parent
  commit too. The anchor is verified through the isolated `add` probe in the change's results.
* `NG` is the negatives' class: every body it holds is quoted, and a stripped text that does not
  compile is exactly the *safe* form the soundness invariant asks for. The refusals are pinned as
  text, and the replay asserts that the stripped class does not compile.
