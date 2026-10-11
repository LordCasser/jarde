# Private v3 adversarial-test proposal

Status: optional test-only preparation. The product candidate v2 is unchanged. This folder is not applied to the shared checkout, and these tests have not been compiled or run. No Git, Cargo, Rust compiler/formatter, Java, JADX, or CLI command was run. The evidence below is read-only from the accepted control-baseline archive.

## Real control evidence

The primary fixture is the frozen `ConditionalConstructionRefusals.class` from the javac 23 original leg. Its javap output and the original-input Jarde report are pinned in `pins.json`; the accepted `root-acceptance.json` pins that archive's actual results. The report is pre-change evidence, so its old `jre_new_shape` messages document the missing behavior, not the v2 candidate's output.

The bytecode gives four distinct boundaries:

| Method | Observed bytecode / current report | Candidate coverage |
|---|---|---|
| `localBoundary` | `new@8; dup@11; invokespecial@17; goto@20`; false arm loads `original@23`; both reach `astore@24; aload@25; areturn@26`. The frozen report has no New presentation because the old Site reader census does not follow the stack Phi. | Candidate test expects the one-hop marker at physical store `24`; this is the positive local-value consumer. |
| `sharedFieldAndReturn` | Same construction through `invoke@17`; both arms reach `dup@24; putstatic@25; areturn@28`. The old report quotes `dup@24` and the later two uses. | Candidate test requires the Site to remain refused: one immediate `dup` is not a text-writing consumer, and its copies feed both field store and return. This covers the real shared/multiple-use case without synthetic SSA mutation. |
| `nestedConditionals` | Constructor calls at `28` and `45` feed a join at `48` reached from the inner true arm, inner false arm (`goto@32`), and outer false arm. | Candidate test requires both New Sites to remain refused by the narrow two-input one-hop ownership rule; this does not preclude a separately proved nested-expression feature. |
| `independentBranchEffect` | `independentEffect@8` occurs in the true arm before `new@11…invokespecial@20; astore@23`; the false arm stores its own value at `27…28`. The frozen report already presents the New inside the structured `if` and preserves the independent call. | Candidate test requires an ordinary Site with no conditional marker. This protects the existing statement/effect path from conditional-specific accounting. |
| `handlerAroundConditional` | The exception table covers `[0,24)` and targets `25`; the candidate result would return at `24`. The frozen report retains the catch but cannot structure normal-flow block `24`. | Do not assert candidate Site acceptance here. The full Builder's existing CFG/handler proof must decide it; any final New receipt must still come only from the final Program walk. |

No constructor-result Phi with a `PhiInput::Itself` back edge appears in these methods; their source has no loop. No test should invent or mutate an internal SsaTable to manufacture one. The implementation's explicit `PhiInput::Value` requirement continues to reject that shape; a future loop-carried constructor fixture would be separate evidence.

## Optional tests

The v3-only `candidate.patch` adds four tests under the existing `init.rs` unit-test module, using the already checked-in accepted baseline class and the existing `analyzed_caller`/`Sites` analysis path. It does not construct or mutate private SSA records. The test method facts state the actual static method flags (`0x0009`) and descriptor parameter-slot counts. The tests assert the local store consumer, preservation of the shared `dup` refusal, refusal of the three-way nested joins, and the independent-effect ordinary path.

The tests are review candidates only. Static bytecode evidence supports their intended shapes, but compiler/runtime results do not exist yet. In particular, `handlerAroundConditional` is intentionally left as a Builder-level control rather than a Site assertion, and this patch does not claim that `conditional_phi_consumer` alone proves a conditional expression.
