# Local reference-write public API test draft

This is a private draft only. It has not been copied into the shared checkout and no Rust, Java, Jarde, or JADX tool was run. v3 is based on v2 but fixes its missing class-identity helper using fields verified in `src/class_source.rs`, `src/facade.rs`, and `crates/jarde-reader/src/model.rs`; v2 remains unchanged. Do not treat any `planned` assertion as observed product behavior.

## Fixture copy plan

Copy these exact original compiled classes into the public test's fixture tree, retaining both JDK and debug profile dimensions:

- `debug/javac8/CF11*.class` from `/private/tmp/jarde-local-reference-write-baseline-root-v2-debug/legs/javac8-original/classes/cf11/`
- `debug/javac23/CF11*.class` from `/private/tmp/jarde-local-reference-write-baseline-root-v2-debug/legs/javac23-original/classes/cf11/`
- `none/javac8/CF11*.class` from `/private/tmp/jarde-local-reference-write-baseline-root-v2-no-debug/legs/javac8-original/classes/cf11/`
- `none/javac23/CF11*.class` from `/private/tmp/jarde-local-reference-write-baseline-root-v2-no-debug/legs/javac23-original/classes/cf11/`

The 7 target class names in each leg are `CF11OuterAssignmentTarget`, `CF11CastReadControl`, `CF11CharSequenceControl`, `CF11ReferenceArrayControl`, `CF11ParameterControl`, `CF11SlotReuseControl`, and `CF11HandlerWriteControl`. Destination: `tests/fixtures/p3-local-reference-write-compatibility/{debug,none}/{javac8,javac23}/`. Preserve the original Runner separately only if a future test needs independent runtime oracles; this public API draft currently opens each complete class as a single-class artifact.

The source roots are pinned by the two v2 execution receipts and corresponding collector manifest under `/private/tmp/jarde-local-reference-write-baseline-root-v2-{debug,no-debug}`. The seven classes correspond to the prior source pin `CF11OuterAssignmentTarget.java` through `CF11HandlerWriteControl.java`; original full source SHA pins are in `/private/tmp/jarde-local-reference-write-baseline-prep-luna-v2/manifest.json`. No fixture bytes were copied as part of this draft.

## Test intent and open expectations

`no_cast_iterator_assignment_preserves_object_target_and_full_source_map` is the planned positive contract for `noCastAssign(Ljava/util/List;)Ljava/lang/Object;`: the recovered target is declared as `Object`, seeded with `"seed"`, assigned from `iterator.next()` without an explicit cast, and returned. It compares default, all, and no-rule-details text and source maps, then checks every physical BCI against a same-method origin and any instruction constant-pool index against its mapped origin. The method name and descriptor are checked as exact class-file UTF-8 member bytes.

The controls cover the three existing outer-assignment shapes (`positive`, `bodyEffectBeforeAssign`, `extraNextConsumption`), explicit cast-on-read (`castRead`), interface references (`builderLength`), reference arrays (`lengthAt`), parameter writes (`replace`), physical slot reuse (`afterStringLifetime`), and handler writes (`handlerWrite`). Their method names and descriptors were read from the pinned javac 8 `javap` captures. The matrix iterates every matching control method; it does not select only the first method in a class. The identity helper uses the published API fields: `ClassSourceReport.class`, `ClassSourceReport.declaration.item.definition`, `ClassSourceDeclaration.item.declaration.this_class`, `ClassSourceMethod.item.identity.owner`, and `PhysicalDefinitionId.class_bytes.length`. It compares the exact requested internal name to `this_class`, compares both owner identities to the report's complete physical definition (including digest and location), and checks the captured byte length. Controls accept only the observed complete result shapes `Structured/Java` and `Fallback/Mixed`; they compare quality, representation, content, whole-class text, target method text, and source maps across default/all/no-rule-details requests. This preserves explanatory fallbacks as mixed output instead of calling them Java. Exact baseline text and unsupported/refusal expectations are pending a fresh public API observation. No internal proof gate is named.

Pre-cancel expects an empty class/method result and zero counted work. `late_stop_measurement_probe` is a root-only helper: it measures one complete method at runtime and tries one fewer analysis step. Its numeric limit is derived from the observed run; no claim is made that this always stops within the target method, and no fixed cost assertion is included. `planned_target_stop_shape` is only a helper for later use after actual outcome inspection.

The prior baseline diagnosis observed javac/Jarde CLI generation, not these public API calls. In particular, compile-failure counts and CharSequence debug/no-debug source differences must not be substituted for public API assertions. Exact control expectations and any additional source-map UTF-8 claims remain planned until measured through the public API.
## Public API identity fields read for v3

The helper follows the definitions in the pinned workspace sources: `src/class_source.rs` defines `ClassSourceDeclaration` (line 225), `ClassSourceMethod` (line 434), and `ClassSourceReport` (line 946); `src/facade.rs` defines `ClassDeclarationFacts.this_class` (line 39655) and `ClassDeclarationItem.definition` (line 39724); `crates/jarde-reader/src/model.rs` defines `PhysicalDefinitionId.class_bytes` (line 300) and `PhysicalMethodId.owner` (line 337). The exact source files and SHA-256 values are in `pins.json`. These definitions support every `assert_class_identity` call; the helper compares the owner definition with both report identities, verifies the queried internal name against class `this_class`, and checks the input class byte length.

