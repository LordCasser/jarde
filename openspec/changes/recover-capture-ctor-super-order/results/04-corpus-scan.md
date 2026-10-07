# Task 3.2 — corpus two-leg scan, every delta classified

Date: 2026-10-07 (UTC). Legs: **baseline** = `8519530e` built from `git archive HEAD` into
`/tmp/jarde-baseline` (`CARGO_TARGET_DIR=/tmp/jarde-baseline-target`; verified to render
`DB$2` in the pre-change order), **change** = this worktree's `target/debug/jarde-cli`.

Scripts (self-tested: equal capture counts, no empty capture, both binaries executable):

* `scan-corpus.sh` — every `tests/fixtures/**/*.class` (857 classes) through
  `class-source --policy single-class --class <file-stem>`;
* `scan-evidence.sh` — every `.class` entry of every `openspec/evidence/**/*.jar` (254 jars) and
  every loose `openspec/evidence/**/*.class` (2725 captures), `--policy plain-jar` /
  `--policy single-class`.

Both capture stdout and the exit status per class into two trees and `diff -r` them.

## Result

| corpus | captures per leg | render (`.txt`) deltas | stderr deltas beyond `elapsed_millis` |
| --- | --- | --- | --- |
| `tests/fixtures` | 857 | **2** | 3 |
| `openspec/evidence` | 2725 | **1** | 1 |

Every render delta is a capture-form companion's constructor order, and nothing else:

```
$ grep '\.txt' /tmp/jarde-scan-ctor-order/diff.txt
diff -r .../a/proved-java-structure_double-brace-capture_v23_DB_2.class.txt .../b/...   # the new fixture, javac-23 leg
diff -r .../a/proved-java-structure_double-brace-capture_v8_DB_2.class.txt  .../b/...   # the new fixture, javac-8 leg

$ diff .../jarde-scan-evidence/a/java-syntax-2026-10-05_double-brace-patrol_fixture_db.jar::DB_2.class__84aacc18.txt .../b/...
12d11
<         this.val$s = arg1;
13a13
>         this.val$s = arg1;
```

That is: the patrol's own `DB$2` is the *only* evidence-corpus render that changes, and the two
new-fixture legs are the only fixture-corpus renders — the four order-sensitive control fixtures
(`anonymous-super-dispatch`, `anonymous-super-args`, `anonymous-capture`, `anonymous-top-level`),
both probes, `TH$1`/`TH$2` and every other frozen class stay byte-identical (0 exit-status
differences in either corpus).

The stderr deltas beyond `elapsed_millis` (classified by re-parsing every capture and dropping
that one field; 437 / 1340 stderr captures differ on timing alone):

* the three fixture-corpus entries: the two `DB$2` legs (their `text_digest` and their
  `source_map` segments — the presentation moved, which is the change) and the `CallArg$1` probe
  (`execution.usage.analysis_steps` 109 → 110: the argument walk's one charge for visiting the
  call-bearing argument — bookkeeping, not presentation);
* the one evidence-corpus entry: the patrol's `DB$2`, for the same text/source-map reason.

No other plane, diagnostic, exit status or byte changed anywhere in either corpus.

## Exposure, read off the scan

The defect's exposure in the frozen corpora is exactly the classes whose companion constructor
has the certified group and a call the group's fields cannot be observed through: the patrol's
`DB$2` (and the two legs of its new fixture). Every other frozen capture-form companion declares
a method of its own (the four control fixtures, `TH$1`'s `run()`), or is a named member class
whose super is `Object` and was already reordered.
