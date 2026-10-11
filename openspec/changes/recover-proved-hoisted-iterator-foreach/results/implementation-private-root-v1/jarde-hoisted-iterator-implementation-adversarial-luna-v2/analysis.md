# Path ancestry is not statement-list identity

This is a supplemental, read-only review of the frozen private candidate v1. It does not edit that candidate or the shared source, and it does not run Rust/Cargo/JDK/JADX/CLI/Git. It compares path construction, Builder vector lifetimes, declaration placement, and the existing direct Iterable fixture.

## Conclusion

The candidate's `owner.len() < loop_path.len()` test is a valid lexical-ancestor test, but it is not a valid proxy for “the declaration is in a different statement list.” A normal direct Iterable loop can have a strict-prefix lexical owner and still have its blank declaration, initializer, and loop all in the same active `self.stmts` vector. The frozen candidate then applies the new blank-only ancestor-gap rule to a legacy same-vector case and refuses an existing positive fixture.

The concrete risk is `RawIterableIterator.rawCast`, which current `tests/p3_iterable_foreach.rs:122-139` requires to recover as Object foreach for both debug and no-debug class inputs. The source at `.../remaining-4-1/original/RawIterableIterator.java:6-14` is `Iterator it = values.iterator(); int result = 0; while (it.hasNext()) ...`. Its pinned `RawIterableIterator.nodebug.javap.txt:10-31` shows iterator initialization/store at BCI 1/6, `result = 0` at BCI 7/8, test at BCI 10 with backedge target 9, and next/cast in the body. Thus the literal result initialization lies between iterator initialization and the loop.

## Why paths and vectors diverge

The declaration planner's paths are lexical region paths, not identities of Builder statement vectors:

- `region_paths` starts each top-level region at `child([], index)` (`build.rs:4349-4365`). `collect_paths` recursively gives a loop's body child paths (`:4396-4403`), while `own_blocks(Loop)` assigns the loop header/test blocks to the loop itself (`:4539-4573`). A preheader block and a later loop-test block therefore naturally have sibling/different paths.
- `common_owner` takes the common path prefix (`:3670-3682`). If init is in the preheader and test/next are in the loop, the shared declaration owner can be a strict prefix of the loop path. The planner uses `Elevated` when the first write is outside that common owner (`:1895-1906`); a strict prefix says the local is lexically visible there, not that its declaration occupies a separate vector.
- Class building emits `declare_at(&[])` and then visits top-level regions without resetting `builder.stmts` (`:8220-8225`). `Sequence` likewise visits child regions directly. A declaration emitted at an ancestor path can consequently be present in the same active vector as a child loop. Loop building does take the current vector while it builds the loop body and restores it before calling the iterable candidate (`:18878-18890`, candidate call/removal `:18988-19013`), but this only identifies the list active at candidate time; it does not make path prefix and vector identity equivalent.
- Other constructs (`arm`, try/catch, etc.) can intentionally isolate statement vectors. So neither “strict prefix always different list” nor “all ancestors share a list” is valid; the helper's actual active-list match is the relevant physical-list fact.

For the direct `rawCast` control, path analysis predicts a preheader region and a loop region, making their common owner a strict prefix, while root-level `declare_at` and the top-level region walk share one `self.stmts`. The exact normalized region indices are not recorded in the pinned direct fixture, so this is a static derivation from the pinned bytecode and path-building code, not a runtime diagnostic claim.

## Candidate impact

Candidate v1's helper can find the exact blank Iterator declaration in the active vector. It then suppresses that index for a strict ancestor (correctly retaining the declaration), but independently returns `ancestor_owner=true`. The caller uses that bool to require *all* rows after init to be blank declarations. On the direct fixture, the `result=0` declaration is a nonblank `Declare(Some(Integer(0)))`, so candidate v1 declines the foreach before the existing exact next/cast/effect proof finishes. That conflicts with the existing same-list acceptance contract in `tests/p3_iterable_foreach.rs` and the design statement that the old same-list path remains.

The candidate's current statement-index search already distinguishes whether the exact declaration row is in the current list. Current OpenSpec design/spec now make the rule explicit: if that exact identity/owner-validated blank declaration is in active `self.stmts`, return and remove only its current-list index and retain the old same-list gap predicate, even when the owner path is a strict prefix. If the row is absent, accept only a fully visible strict-prefix `Elevated` owner, return no ancestor index, preserve that other-list declaration, and apply blank-only gap policy. `owner.len() < path.len()` establishes lexical visibility; it does not establish different-list membership. Root has recorded this correction in the current design/spec, and collector v2 is reported to use active-row presence for the partition. The frozen v1 does the opposite for strict-prefix active rows and is therefore not acceptable. Collector v2 is structurally corrected on this point but remains uncompiled/unrun and is not thereby accepted.

## Existing test obligations and limits

The exact existing positive test is `tests/p3_iterable_foreach.rs:122-139`, covering `RAW_DEBUG` and `RAW_NODEBUG` `RawIterableIterator.rawCast` plus two `IterableForEach` inputs. The exact fixture source/javap hashes and candidate v1 hash are pinned in `pins.json`. The historical `.../jarde/RawIterableIterator.nodebug.java` is an older saved output showing `while`; it is not evidence for current test behavior and should not be used to override the test contract.

No test was run in this review. Root must run candidate v2 against the direct positive and target nested positive/controls; the v1 issue is a static regression prediction, not an actual failed-test claim.
