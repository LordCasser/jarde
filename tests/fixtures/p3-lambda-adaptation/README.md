# Lambda descriptor adaptation fixture

`LambdaAdaptationProbe.class` is the primary permanent Java 8 class. The support, box, and runner
sources are source-only inputs for the complete class comparison.  The probe has no generated
`lambda$` helper methods: every functional site is a method reference in the main class.

The primary positive class covers erased String/Object overload selection, dynamic String to an Object
implementation with a call counter, String array adaptation, an unbound receiver with primitive
return, a constructor parameter, primitive same-type adaptation, and the generic raw-Supplier
return control.  Bound-null and unsupported boxing/return-narrowing variants stay outside the
permanent positive class and are recorded in the change evidence.

`v8/CapturedLambdaAdaptationProbe.class` is a second permanent class for the captured dynamic
adapter. Its `captured()` method captures the value loaded at BCI 4 from a producer call at BCI 0;
the invokedynamic site is BCI 5 / CP #13. The captured integer is passed to a generated lambda
implementation whose erased SAM input is `Object` and implementation input is `String`, so the
runtime Object-to-String check is real while the class contains no `checkcast` for that check.
`v8/BoundNullLambdaAdaptationProbe.class` keeps the creation-time null check as a separate
refusal control. The added support and runner sources let the original, JADX, and recovered
positive class be compiled and run as a complete small project.
