# Same-handler primitive conversion control v2

`ConstructorPrimitiveHandlerControls.java` is compiled independently on the two JDKs recorded in
`fixture-manifest-v2.json`. Its `sameHandler(I)Ljava/lang/Long;` method places allocation,
constructor invocation, and the assignment that consumes the new value within one RuntimeException
handler range. The manifest records the raw class hashes, `javap -v -p -c` captures, maximum stack,
and exception-table range.

The original version with a return as the sole allocation consumer is preserved in
`recover-constructor-primitive-conversion-arguments/results/handler-controls-build-v1/`; it showed
that the handler range ended before the `areturn` consumer and was not the same-cover positive
control. The v2 classfiles are the canonical test inputs. The integration test analyzes them but
does not execute a modified classfile variant.
