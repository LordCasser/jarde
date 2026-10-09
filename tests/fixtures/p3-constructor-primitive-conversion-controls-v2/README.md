# Same-handler constructor conversion control v2

This fixture replaces the old v1/v2 same-handler positive input with the exact source and two
classfiles frozen in `handler-controls-build-v4`. `sameHandler(I)Ljava/lang/Long;` returns
`identity(new Long((long) markInt(value)))` from inside the `RuntimeException` protected range.
The sole consumer is the `identity` invocation at BCI 12; the `areturn` at BCI 15 is outside the
half-open exception range `[0, 15)`. The helper avoids a result local and catch-slot reuse.

The two classfiles are byte-for-byte copies of the independently compiled Corretto 8 and
OpenJDK 23 inputs recorded in `fixture-manifest-v1.json`. The integration test analyzes each
input and compiles the complete generated class with empty classpath and sourcepath; it does not
execute this no-main fixture. Root's frozen CLI probe reported the javac8 source as complete
Structured/Java with no refusal markers and mapped the identity consumer. The javac23 render also
returned exit 0; its full generated-source compile remains part of root's independent replay.

The earlier handler-control versions remain in the change's `results/` evidence. v1/v2/v3 results
are historical controls; their source-scope refusals are not counted as numerical-conversion
successes. The initializer exception-table proof remains covered by its separate v1 fixture and
test.
