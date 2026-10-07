# `recover-postfix-condition-positions` fixtures

The postfix old-value **condition positions** of `recover-postfix-condition-positions`, frozen on
both compiler legs. Two classes, each compiled twice from the same source:

```sh
javac --release 8 -Xlint:-options -nowarn -d v8 *.java
/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -nowarn -d v8-javac8 *.java
```

`v8/` is javac 23.0.1 `--release 8`; `v8-javac8/` is real javac 8 (Corretto 1.8.0_432). Every
`*.class` under both legs is listed in `sha256.txt`, together with the two sources.

## The classes

| class | shape | anchor |
| --- | --- | --- |
| `CP7` | the postfix-condition patrol's frozen trio: `scan` (`do { last = xs[i]; } while (xs[i++] != 0 && i < xs.length);`), `find` (`while (i < xs.length && xs[i++] != t) { }`) and `cond` (`if (a[i++] > 0 && i < a.length) { return true; } return false;`) | the three condition positions, on both legs |
| `CN` | this slice's two negatives — `twoVariables` (a second variable's position in one condition: `while (xs[i++] != 0 && ys[j++] != 0)`) and `midChain` (a position in the middle of a short-circuit chain: `while (a != 0 && xs[i++] != 0 && i < xs.length)`) — plus the control `chain` (`do { n++; a--; b--; } while (a > 0 && b > 0)`, the compound chain the region layer already presented with no postfix position in it) | the negatives and the byte-identical control |

`CP7.java` is the patrol's own source, copied byte for byte
(`openspec/evidence/java-syntax-2026-10-05/postinc-condition-patrol/fixture/CP7.java`), and the
`v8/CP7.class` this leg produced is **byte-identical** to the patrol's frozen jar entry:

```sh
shasum -a 256 v8/CP7.class                              # 63683000…c68d
unzip -p openspec/evidence/.../postinc-condition-patrol/fixture/cp7.jar CP7.class | shasum -a 256
```

The A-phase traps (`i = i++`, `i = i--`, `f = f++`, `i += i++ + 1`) are **not** copied here: the
suites read the A-phase fixture's own `NG.class` (`tests/fixtures/recover-postfix-old-value-snapshot/`,
SHA-pinned in that fixture's README), the way `recover-dup-store-conditional` reads CF-06's
`ExtraCopy.class` from the fixture that owns it.

## Behavior

`CP7` and the `chain`/`main` members of `CN` compile with both compilers and run under
`-Xverify:all` with the same standard output and exit status as their own class files; the tests in
`tests/recover_postfix_condition_positions.rs` pin that on both legs. `CN.twoVariables` and
`CN.midChain` are the negatives: their bodies are quoted whole, so the stripped text of `CN` does
**not** compile — the safe form the soundness invariant asks for — and the replay asserts exactly
that. The `do { … } while (xs[i++] != 0 && …)` scan is the one anchor whose evaluation count matters
beyond its output: the increment runs once per iteration, so the replay compares the printed
answers, which count the iterations.
