# CF-11 shared-latch nested loops: frozen fixtures

All classes are compiled with **real javac 8** (Corretto `1.8.0_432`,
`/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac`,
`javac -d <dir> <source>`; `javac -version` → `javac 1.8.0_432`). The `.class` bytes are the
frozen inputs; recompiling replaces bytes and breaks the recorded baselines.

## Anchors reused from the patrol (do not recompile)

`S5.class` (`bee1eb6c6b65f4eb51526bedd22d29705e5b4664ed3473bf2da509e92995a625`) and `S3.class`
(`79880859d6c63704b863b51481222c6e95b9a6c1faf03410d85db8779fed6826`) are byte-identical copies of
`openspec/evidence/java-syntax-2026-10-04/shared-latch-patrol/fixture/` (patrol baseline
`30e54613`). Their SHA-256 records there stay authoritative; the copies exist so the CI test can
`include_bytes!` the anchor without reaching outside `tests/fixtures`.

- `S5.outerContinueInner` / `S5.labeledOuter` — outer `continue` + nested counted loop, plain and
  labeled (`continue outer`); refused before `recover-shared-latch-nested-loops`, recovered after,
  recompiled behavior identical to the patrol baseline `13/9/20/13` (`o5.out`).
- `S5.outerContinueOnly` / `S5.innerOnly` — the untouched single-continue and no-jump shapes; the
  CI test pins their rendered text (byte-identical before and after the change).
- `S3.nestedBreak` — the `Svc.lookup` business form (Map.Entry double for-each + prefix-filter
  `continue` + inner `break`); recovered; recompiled behavior identical to `o3.out` (`[px:3]`).

## New variants (this slice), sources included

Compiled from the `.java` beside each `.class`:

| class | blake-of-record (SHA-256) | role |
| --- | --- | --- |
| `ThreeLevel.class` | `c0cff86079910b911df02dbfb01a7815a72715136b46da17098229172f46db17` | three-level shared-latch composition — recovered after the slice, recompiled behavior `24` identical to the original |
| `ExitDiverges.class` | `e460d671ac8f0d14b1961eff80123f3c8b02d36458e2919afe6dbcd762b5a94d` | inner loop's exit lands on a block that does not route straight onto the back edge (`if (s > 100)` after the loop) — stays refused |
| `LabeledBreakOuter.class` | `978b087ef7aa887692f304c2a80cb263260c0da2a7dcd5c08680f2443aaf7af6` | labeled `break outer` escapes past the back edge — stays refused |
| `Overlap.class` | `6f5a452db50bc7f873c643b4bd160cf388ecd0323279a8a7212333488c547ef4` | outer `continue` + inner `break` + inner `continue` — refused before and after (the inner two-jump body is the double-jump slice's closed domain); registered residual |

Original behaviors (`java -Xverify:all`, n=6): `ThreeLevel` → `24`; `ExitDiverges` → `13`;
`LabeledBreakOuter` → `1`; `Overlap` → `300`. Each `.render`-leg check in the CI test either
asserts the recovered text and (for `ThreeLevel`) the recompiled output, or asserts the loud
refusal that the change preserves.

`Svc` itself is not frozen here: its family jar lives in the patrol evidence directory, and its
`lookup` body recovery is anchored by `S3.nestedBreak` (the same loop form) plus the render
leg recorded in this slice's evidence directory.
