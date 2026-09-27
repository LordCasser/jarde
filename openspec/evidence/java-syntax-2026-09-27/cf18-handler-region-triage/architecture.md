# CF-18 handler-region architecture triage

Date: 2026-09-27. Based on local `main` at `31586ffc`; inspected JADX checkout `2fb1b16386941660fda07e9017285aec40fcb37f` and replayed with JADX CLI `1.5.6`. The two inspected implementation files have the SHA-256 values in `comparison.json`. This is an evidence note, not an implementation proposal. No production source was changed.

## Reproduction and what it proves

The existing [CF-18 report](../cf18-exception-regions/report.md) contains the full original/JADX/Jarde Java 8 class replay. The original class compiles and verifies and prints `124:115`. The fixed JADX output compiles and verifies but its outer `IllegalStateException` catch is inside the negative-value `if`, so the exception from `work(2)` escapes (exit 1). Jarde refuses `run()I` at BCI 0 with `jre_region_irreducible`, emits no executable body, and the complete source fails `javac` with a missing return.

[`HandlerLoopProbe.java`](HandlerLoopProbe.java) removes the negative-value branch and most arithmetic while retaining an inner `NumberFormatException` catch with `continue`, a split outer `IllegalStateException` table range, and both handlers rejoining the loop increment. [`replay.py`](replay.py) compiles each complete source with `javac --release 8 -g:none`, runs each successful class with `java -Xverify:all`, checks the fixed JADX revision, and cleans its temporary Cargo target. The generated [`comparison.json`](comparison.json), [`javap.txt`](javap.txt), and [`run-region.json`](run-region.json) record:

| Complete class | `javac` | `java -Xverify:all` | Observation |
|---|---:|---:|---|
| Original | 0 | 0 | `4:110` |
| Fixed JADX | 0 | 0 | `4:110` for this smaller shape |
| Jarde | 1 | not run | `run()I` is a BCI-0 `jre_region_irreducible` whole-method quote; missing return |

The smaller fixture reproduces Jarde's refusal but **does not** reproduce the fixed JADX semantic error. That error remains proven by the original CF-18 full-class runtime, not by textual shape or this reduced output. The reduced class's table is:

```text
  9..16 -> 19 NumberFormatException
  9..29 -> 38 IllegalStateException
 32..35 -> 38 IllegalStateException
```

BCI 19 is the inner handler and lies *inside* the first outer protected range. Its normal transfer goes to the loop increment at BCI 51. BCI 38 is the outer handler and also transfers to BCI 51. The original CF-18 class has the same ownership complication in a larger shape: outer rows `13..25`, `28..52`, `55..69` all target BCI 72, and the inner handler BCI 38 lies in outer row `28..52`.

## Exact gates in Jarde

`NormalFlowView::build` omits exception edges from dominance and loop questions; this correctly prevents an exception dispatch from masquerading as an ordinary Java branch. The ordinary transfer from an exception-root handler back to an in-loop join, however, appears as an extra entry into the SCC. For the reduced class, `run-region.json` names SCC blocks `[4, 9, 32, 51]`; BCI 38 -> 51 and BCI 19 -> 51 enter it from handler roots. These are **exception-origin entries**, not evidence that the source loop itself has two ordinary entries.

`region::recover` calls `proved_loop_catch_joins` before the whole-body `irreducible_blocks` gate. That existing exemption requires *every block touched by the full declared row range* to be in the natural loop and dominated by its header (`region.rs`, `proved_loop_catch_joins`), then requires protected throw-site paths to reach the join. The outer `9..29 -> 38` row includes BCI 19, an inner handler reachable from the method through an exception edge. Under the intentionally normal-only dominance relation, the loop header BCI 4 does not dominate BCI 19. Therefore the exemption cannot establish the outer handler's `38 -> 51` join, and the SCC fails before region walking. The original `28..52 -> 72` row similarly includes the inner handler at BCI 38. This is a false ordinary-loop irreducibility conclusion caused by applying an ordinary-flow dominance condition to a protected interval with an exceptional-only inner entry. It is distinct from a genuinely irreducible normal CFG.

Passing that gate alone would not prove a correct Java `try`. `guard::catches` recognizes rows beginning at one instruction and at most two end points; its nested case calls `nests`, which rejects a handler shared by another range (`own_rows_only`). At reduced BCI 9, the inner `9..16` and outer `9..29` rows appear nested, but outer handler BCI 38 also serves `32..35`, so the nested candidate is refused. At BCI 32, the second outer row can look like a separate `try`, although its handler belongs to the same source-level outer catch as the earlier row. The original CF-18 outer handler is shared by three disjoint table ranges. Region walking also keeps an enclosing loop's `Frame::scope` inside nested `try` and handler walks; whether it can claim both exceptional-only handler roots and their common continuation must be established after ownership is correctly represented. These are separate proof obligations, not reasons to suppress the SCC diagnostic by deleting an edge.

Fixed JADX takes a different route. `ExcHandlersRegionMaker` derives handler exits from splitter/handler path crosses or dominator frontier; `ProcessTryCatchRegions` then wraps blocks reachable through the selected dominator while excluding handler-reachable paths. This explains how it can produce a structured tree, but those placement rules do not certify protection of every bytecode throw site. In the full CF-18 replay, the outer catch appears under the negative branch and misses `work(2)`. Its reduced-class success therefore cannot validate Jarde's future ownership rule.

## Bounded next decision

Do not open a CF-18 implementation OpenSpec yet. A single “allow handler -> loop join” change would only remove the first refusal and risk publishing a misplaced catch. A reviewable implementation proposal needs an explicit certificate that groups disjoint same-handler exception rows into one lexical owner while preserving table dispatch order, maps the inner handler's exceptional entry into the enclosing protected body, and proves handler and continuation blocks are claimed exactly once across the loop frame. It also needs a negative case with the same normal CFG but a genuinely unowned/crossing handler edge, plus original/JADX/Jarde full-class compile, verifier, and runtime comparisons for both the reduced and original fixtures. Keep any local-scope/definite-assignment failures encountered after region ownership as separate debt under the existing `preserve-local-scope-across-exception-regions` change.
