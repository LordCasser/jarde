# Task 1.2 — the frozen anchors and negatives, both legs

## The anchors

The anchors are the patrol's own artifact and the path-A fixture's two legs; this slice adds no
anchor fixture of its own:

| anchor | artifact | legs |
| --- | --- | --- |
| `DB` (both forms) | `openspec/evidence/java-syntax-2026-10-05/double-brace-patrol/fixture/db.jar` | the patrol's javac 8 bytes |
| `DB` (both forms) | `tests/fixtures/proved-java-structure/double-brace-capture/{v23,v8}/` | `javac --release 8` and a real javac 8, frozen by that fixture's `freeze.py` |

Both anchors present the double-brace form, and the whole class strip compiles and prints `2/z`
identical to the original classes — under `javac --release 8` **and** under the real javac 8
(`/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac`), verified with
`java -Xverify:all`:

```text
$ javac --release 8 -g:none -d out23 DB.java && java -Xverify:all -cp out23 DB
2/z
$ <jdk8>/bin/javac -g:none -d out8 DB.java && <jdk8>/bin/java -Xverify:all -cp out8 DB
2/z
```

Run over three class sets (the patrol jar, `v23/`, `v8/`) — six compile+run pairs, all `2/z`, the
same output the original classes print (`results/o-DB_1.txt`, `o-DB_2.txt`). The recompiled host
alone is the strip: both companions are hidden from its text, so `DB.java` needs no companion file
(the companion files, were they compiled beside it, would collide with the anonymous classes javac
mints for the same source positions — the family compile in `tests/double_brace_capture.rs` still
passes, because both candidates behave identically).

## The negatives (new fixture set, dual-leg)

`tests/fixtures/proved-java-structure/double-brace-allocation-site/` freezes one positive control
and three negatives, each differing from the control in exactly one criterion (the README is the
fixture's own record):

| case | which criterion fails | render |
| --- | --- | --- |
| `controls/single-site/` (`DBS`, `DBS$1`) | none — the control | the double-brace form |
| `negatives/methods/` (`DBM`, `DBM$1`) | the companion declares `void mark()` | `new DBM$1(s)` |
| `negatives/unspellable-super/` (`DBN`, `DBN$1`, `Carrier`, `Carrier$Nested`) | the superclass's pool form is not a source name | `new DBN$1(s)` |
| `negatives/multi-site/` (`DBS2`, `DBS2$1`) | the companion is constructed twice | `new DBS2$1(arg0)` at both sites |

Both legs are frozen for every case: `v23/` from the ambient `javac --release 8 -g` and `v8/` from
the real javac 8 (`JARDE_JAVAC8`). The multi-site host is **hand-made bytecode** (`freeze.py`,
following `capture-super-arg-probes`' precedent): no Java source states a companion constructed
twice, so the script compiles the child from the control's source (renamed `DBS2`) and writes the
host — `make(Ljava/lang/String;)Ljava/util/List;`, which the child's own `EnclosingMethod` names,
and `again(Ljava/lang/String;)Ljava/lang/Object;`, one `new DBS2$1(arg0)` site each — copying the
compiled host's own `InnerClasses` row. Two **methods** rather than two statements in one, so the
allocation point's own one-site discipline is not what refuses the shape: the criterion under test
is the owner census's single use. The frozen host loads under `java -Xverify:all`.

The render of every case is recorded under `results/render-<leg>-<case>.{txt,err}` and the
negatives' text is asserted byte-for-byte by `tests/recover_double_brace_allocation_site.rs`
(`the_three_negatives_keep_the_presentation_they_had`).

## What the fixtures move

* the render fingerprint (`results/fingerprint.txt`): **+20 lines**, zero changed, zero removed;
* `tests/fixtures/corpus-fingerprint.json`: +105 lines, zero rewrites;
* the reader crate's fixture census: `(879, 3829, 334, 2445, 8)` → `(899, 3867, 334, 2445, 8)`
  (+20 classes, +38 bodies, no handler record, no branch target, no subroutine).

```text
$ python3 tests/fixtures/proved-java-structure/double-brace-allocation-site/freeze.py
froze 20 class file(s)
```
