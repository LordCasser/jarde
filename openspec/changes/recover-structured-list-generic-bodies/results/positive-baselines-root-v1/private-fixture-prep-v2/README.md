# CF11 structured List controls v2 (private source prep)

This bundle preserves v1 unchanged and adds two isolated positive source programs. The prior v1 combined source remains at combined/src/cf11/ with all 17 control cases. Independent programs live under isolated-string/src/cf11/ and isolated-integer/src/cf11/; each contains one complete public fixture class and its own public Runner, so a refusal or compile problem in the combined class cannot block the String and Integer positive checks. The class names DistinctStringFixture and DistinctIntegerFixture differ from both UpstreamNestedLoops and one another.

The isolated fixture methods are deterministically extracted from the pinned legal v1 source (renamed and numbers) by derive_isolated.py; generated files are frozen and pinned here. The only additions to those method bodies are a fixture effects counter increment immediately after add/remove/clear operations. This makes successful side-effect calls observable; it does not change their arguments or ordering. No decompiled source was hand-edited. The integer Runner also performs a legal raw List write of a String into a List<Integer> to exercise runtime heap pollution.

No compilation, runtime execution, JADX, Jarde CLI, or workspace modification occurred during this prep. Content validation checks source pins, balanced delimiters, class/method names, derivation inputs, and exact case labels only.

## Projection expectations (not results)

Expected positive generic projection methods are the combined test, renamed, numbers, castBoundary and isolated DistinctStringFixture.renamed plus DistinctIntegerFixture.numbers. These are expectations for root to test, not claims that any compiler/decompiler succeeded.

Expected generic refusals in the combined controls are rawAliasEscape, reassigned, wildcardNull, handled, and indexOverload. rawLists is a raw-signature method and must be reported as an ordinary raw path, not a generic refusal. The indexOverload refusal expectation follows the currently frozen finite call set, which contains remove(Object) but not remove(int); its original behavior must still preserve exact overload binding and order. A compile failure is no-runtime, never a passing refusal.

## Exact oracle labels

The combined Runner has the original eight fixed CF11 inputs plus nine controls, for 17 rows:

baseline-both-empty, baseline-unmatched, baseline-five-character-match-add, baseline-non-five-match-remove, baseline-remove-continues-and-clears, baseline-null-first-list, baseline-null-second-list, baseline-null-element, renamed-string-positive, integer-nested-add-positive, explicit-cast-object-boundary, raw-list-method-control, raw-alias-heap-pollution, parameter-reassignment, wildcard-null-write, try-catch-handler, remove-object-and-index-overloads.

The isolated String Runner adds five rows:

string-isolated-empty, string-isolated-add-cme, string-isolated-remove-object-cme, string-isolated-null-element, string-isolated-null-list.

The isolated Integer Runner adds six rows:

integer-isolated-empty, integer-isolated-add-cme, integer-isolated-remove-object-cme, integer-isolated-null-element, integer-isolated-null-list, integer-isolated-heap-pollution.

There are 28 distinct named oracle cases. Each output row records the exception class or none, both List states, and the real fixture mutation effect counter. The fixed eight v1 inputs remain unmodified in the combined Runner.

## Root-owned serialized run plan

Run roots and class variants serially with fresh output directories. Keep all execution under root's pinned tool/JDK receipts and do not run the product candidate or compiler from this prep:

1. Run validate_fixture.py and retain its output. Compile the combined original source pair on each pinned JDK (Java 8 and Java 23), with debug and the root-requested no-debug comparison. Run the combined Runner with -Xverify:all and retain stdout/stderr and exact class-set hashes.
2. Independently compile each isolated source pair on both JDKs, again retaining the complete class and Runner. Run each dedicated Runner against its same-JDK original class to freeze the five String and six Integer oracle rows. Do not combine the two isolated Runner.java files in one source root.
3. For every original classfile leg, run the pinned JADX 1.5.6 modes selected by root and render complete class source. Then compile the complete recovered class with its matching dedicated Runner on both JDKs, run with -Xverify:all, and compare all relevant rows with the same-JDK original.
4. Render each original class with pinned Jarde in default and all; retain source and report untouched. Compile each complete generated class with the matching dedicated Runner on both JDKs, run with -Xverify:all, and compare against same-JDK originals. Repeat separately for the 17-row combined controls. Record the actual generic result/refusal, method/source provenance, all commands, hashes, class inventory, stdout/stderr, and row comparisons; never infer slots from source names.
5. Interpret independently: a combined class compile failure does not block the isolated String/Integer positive-class checks; an isolated compile failure is still no-runtime for that leg. Do not count a raw-path normal method as a refusal. Do not claim a success before root has run and reviewed these legs.

This plan does not declare any tool output or projection successful. The original and isolated sources are input fixtures only.
