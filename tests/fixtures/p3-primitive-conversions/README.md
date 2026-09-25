# Primitive conversion fixture

`PrimitiveConversions.class` is the only permanent class. The support, effect, and runner
classes are source-only inputs used by the ignored complete-class JVM comparison.

The probe contains one direct method for each of the 15 explicit JVM primitive conversion
opcodes, conversion results passed to overloads, intermediate float/double round-trip chains,
and a left-to-right effecting producer whose results are widened after the calls. The runner
uses representative integer, floating-point, NaN, infinity, zero, and 2^24/2^53 boundary values.
The evidence summary records the exact output line count as the fixture's runtime case count.
