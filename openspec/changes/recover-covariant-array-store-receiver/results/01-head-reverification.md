# 01 · Task 1.1 — HEAD re-verification of the patrol anchors (2026-10-07)

The change was filed after several adjacent slices had landed (`recover-array-element-field-receiver`
and the narrow-local-type family), so the first task is to re-verify the patrol's anchors **at HEAD**
before anything is implemented, and to take the re-verified state as authoritative over the patrol
README.

## Baseline

| what | value |
| --- | --- |
| HEAD | `0471c6c9` (the change's filing commit) |
| baseline binary | `target/debug/jarde-cli` built from the stashed HEAD, snapshot `/tmp/asis/jarde-cli-head`, SHA-256 `fab06c598e745dca04f6f9595dd12ce797049f8d3709be9b3ad5726313a160f4` |
| change binary | the same build after the widening admission, snapshot `/tmp/fx/jarde-cli-change`, SHA-256 `6e1d667619e658c8b9a3980ce2e3a8bde33b39068977a20acad125b7f7d46bb7` |

## The anchors, on both legs

The patrol's `fixture/AS.java` was recompiled on both legs (its `as.jar` entry is a javac 23.0.1
`--release 8` compile of exactly this source — the entry's SHA-256 is `2369235f…f286f6`, and a fresh
`javac --release 8 -Xlint:-options -d leg1 AS.java` produces `cmp`-identical bytes; the report's
`class.class_bytes.digest = 2af17364…` is the same class under the engine's blake3 digest):

| leg | command | class bytes | render |
| --- | --- | ---: | --- |
| javac 23.0.1 `--release 8` | `javac --release 8 -Xlint:-options -d leg1 AS.java` | 1,025 | [head/AS-leg1.txt](head/AS-leg1.txt) |
| Corretto 1.8.0_432 | `javac -d leg2 AS.java` | 1,028 | [head/AS-leg2.txt](head/AS-leg2.txt) |

**Drift: none.** `diff` between the patrol's recorded `results/jarde-AS.txt` class text and the HEAD
render is empty on both legs ([head/patrol-vs-head.txt](head/patrol-vs-head.txt) is a 0-line diff),
and the two legs' texts are byte-identical to each other. The current presentation, verbatim:

```java
    static java.lang.String storeWrong() {
        java.lang.String[] local0 = new java.lang.String[2];
        local0[0] = java.lang.Integer.valueOf(1);
        return "unreachable";
    }
    static java.lang.String storeRight() {
        java.lang.String[] local0 = new java.lang.String[2];
        local0[0] = "s";
        return (java.lang.String) local0[0];
    }
    static java.lang.String storeNumber() {
        java.lang.Integer[] local0 = new java.lang.Integer[2];
        local0[0] = java.lang.Double.valueOf(0x1.4000000000000p1d);
        return "unreachable2";
    }
```

## Current refusal/render state, verbatim

- the signature is the allocation's component type (`java.lang.String[]`, `java.lang.Integer[]`): no
  `LocalVariableTable` states the source's `Object[]`/`Number[]`, so the frames' own array is what
  the local is declared with;
- the store is **written, not refused**: the member's outcome is `produced`, `content =
  contains_statements`, no `@bytecode` quote, and its diagnostics are the ordinary
  `jre_recovery_produced` / `jre_declaration` pair — `compile_status = "not_attempted"`,
  `syntax_status = "unchecked"`. The uncompilability is therefore **silent in the report** and loud
  only where it matters: `javac` refuses the text;
- the stripped text's compile errors (javac 23.0.1 `--release 8`, comment lines dropped the way the
  patrol's own `verify-uncompilable-AS.java` was made) — [head/compile-errors.txt](head/compile-errors.txt):

  ```text
  AS.java:9: error: incompatible types: Integer cannot be converted to String
          local0[0] = java.lang.Integer.valueOf(1);
  AS.java:21: error: incompatible types: Double cannot be converted to Integer
          local0[0] = java.lang.Double.valueOf(0x1.4000000000000p1d);
  2 errors
  ```

- jadx fails on the same shape (the patrol's `results/jadx-AS.java`: the illegal
  `new String[2][0] = 1;` and the `Multi-variable type inference failed` marker).

## The store-side type-check emission point (located)

- the statement is written by **`crates/jarde-java/src/build.rs::array_write`** (the `Operation::
  ArrayStore` arm of the statement dispatch at BCI `at`), which renders the three operands and pushes
  one `StmtKind::IndexAssign { array, index, op: Assign, value }`;
- the element type the write meets is `array_element(ssa, operations, array_value, stated)` →
  `array_of_value` ([`build.rs::array_of_value`], the same component-fact channel
  `recover-array-element-field-receiver` reads for a field receiver) → `element_of_dimension`;
- the store-side **type check** is `meeting_position(value, &element, …, Widening::Position)`, whose
  reference-vs-reference answer is `Conversion::Same` by construction: *"Which class is assignable to
  which is a subtype judgment this layer deliberately does not make"* (`conversion`, the
  `make-required-conversions-explicit` non-goal). So an incompatible reference store is neither
  converted nor refused — which is exactly the silent uncompilable text above.

## Gating experiment (the admission alone)

One edit, and nothing else: `array_write`'s `Some(ty)` arm hands the rendered receiver to
`widen_covariant_store_receiver(array, &ty, &value)`, which widens to `Object[]` when the component
is a reference that is neither `Object` nor the value's own type, and when the value states a
reference type at all.

| observation | HEAD | with the admission |
| --- | --- | --- |
| `AS.storeWrong` | `local0[0] = java.lang.Integer.valueOf(1);` | `((java.lang.Object[]) local0)[0] = java.lang.Integer.valueOf(1);` |
| `AS.storeNumber` | `local0[0] = java.lang.Double.valueOf(0x1.4000000000000p1d);` | `((java.lang.Object[]) local0)[0] = java.lang.Double.valueOf(0x1.4000000000000p1d);` |
| `AS.storeRight` (same-type) | `local0[0] = "s";` | **byte-identical** |
| stripped `AS` compile, javac 23 `--release 8` | exit 1, 2 errors | **exit 0**, no diagnostic |
| stripped `AS` compile, real javac 8 | exit 1 | **exit 0** |
| run, original vs recompiled, `-Xverify:all` | `s`/`ASE1`/`ASE2` | `s`/`ASE1`/`ASE2` on all three runs |

The two negative shapes were measured on the same binaries with the same edit in place:

| shape | observation |
| --- | --- |
| primitive array (`SD.primitiveArray`, `int[] a; a[0] = 7;`) | byte-identical (`local0[0] = 7;`) |
| unproven component (`UB.merged`, two-branch local) | byte-identical, still `local3[0] = java.lang.Integer.valueOf(1);` |
| same-type element receiver (`SD.elementReceiver`) | byte-identical (`local0[0][0] = "r";`) |

Raw renders: [head/](head/) (baseline) and [gate/](gate/) (admission), each for both legs;
[gate/compile-javac23.txt](gate/compile-javac23.txt) and
[gate/compile-javac8.txt](gate/compile-javac8.txt) are both empty (exit 0).

Byte-for-byte diffs of the two binary's renders over the whole fixture and evidence corpus are in
[05-corpus-sweep.md](05-corpus-sweep.md): outside this change's own new fixtures, **zero** render
deltas.
