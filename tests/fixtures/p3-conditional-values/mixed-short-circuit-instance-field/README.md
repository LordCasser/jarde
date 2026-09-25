# Mixed short-circuit value assigned through an instance receiver

`one(ZZ)V` assigns `(a && b()) || c()` to `target(nullReceiver).result`. The target call increments `receiverCalls` and returns either the shared Box or null. The Runner checks all eight `a/b/c` combinations with both receiver states, including the RHS call counts and whether `putfield` throws after RHS evaluation.

Compile the fixture with `javac --release 8 -g:none -Xlint:-options`. `MixedShortCircuitField.class` and its nested `MixedShortCircuitField$Box.class` are frozen; the Runner is source-only in the permanent fixture and the evidence directory retains its run output.

`MixedInstanceControls.inheritedOwner` is a Java 8 verifier-valid refusal control. The source upcasts a `DerivedBox` receiver to `BaseBox` before the same mixed short-circuit write. javac emits no cast instruction: the receiver stack type remains `DerivedBox`, while the `putfield` Fieldref names `BaseBox.result:Z`. The field plan must refuse the mismatched owner rather than present a possibly shadowed source field. `ControlRunner` checks the original field value on all eight input paths under `-Xverify:all`.
