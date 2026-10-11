# CF11 outer BCI 13 assignment-first diagnostic prep v3

Status: private static candidate; not applied, compiled, or run. Base bytes were captured from the current shared production source after its test-only `contains` correction. The exact base SHA is in `pins.json`. This candidate is freshly generated from those bytes; it does not derive from or overwrite with v1/v2's full candidate.

## v2 review correction

Root review rejected v2 because Rust eagerly evaluated `format!` arguments before `log_foreach_candidate` could test scope, and the body-first refusal built AST debug text unconditionally. In v3, the logger accepts `FnOnce() -> String` and formats only inside its `if scope` branch. Any AST debug formatting is itself inside `if outer_assignment_probe`. Operation/instruction rows are also built only inside that exact branch. Thus a non-scoped candidate call does not construct diagnostic strings, traverse AST debug values, or collect SSA debug rows.

## Scope and behavior

The env opt-in is `JARDE_FOREACH_CANDIDATE_DIAGNOSTIC`. Class and physical method identity are checked before reading the environment variable; the identity is `cf11/UpstreamNestedLoops.test(Ljava/util/List;Ljava/util/List;)V`. Assignment traversal additionally requires test BCI 13 and path `[1]`. Only there can body-first shape admit an existing `Assign` whose existing expression is inline `Cast(Call(..))`. Both extraction points borrow its real `name` and `value`. No Declare, type, target variable, or slot-to-variable relationship is synthesized.

`iterator_declaration_for_loop` and `iterator_phi_use_ownership` remain byte-identical and execute through their ordinary gates; no bypass is introduced. Proof stage is logged at existing gate groups. A fully proved candidate returns `None` before synthetic-name allocation, AST rewrite, or caller-list removal. Non-scope paths preserve the production eligibility behavior; diagnostic strings and SSA/AST debug work are lazy/scoped.

AST origin/name/expression and operation/instruction rows use separate log events. Instruction rows report opcode/reads/writes at the selected BCIs and explicitly say `slot_to_target_identity=not_emitted`.

## Static checks and limits

The private unified patch was replayed against `build.rs.orig` using `patch -o`; replay bytes equal `build.rs.candidate`. Both existing helper bodies were compared byte-for-byte. The base contains the current `.contains(&PhiInput::Value(iterator))` test-only form. No Cargo/JDK/JADX/Git command ran and no shared source was edited. This preparation makes no claim about any later outer gate or successful foreach output; the same-IR run remains outstanding.
