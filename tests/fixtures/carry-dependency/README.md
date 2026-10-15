# Saved integer carry with reference dependencies

The original factory receiver counterexample is preserved in `referenceConsumer`.
The complete class also exercises `this` and a parameter receiver. The Runner observes
the result and the order of factory, value, condition and selected arm calls, in both
branches. Original and generated classes must run the same unedited Runner.

Both physical classes are built with the pinned Corretto 8.504.04.1 and Oracle 23.0.1
compilers, `-g -source 8 -target 8 -encoding UTF-8`, with empty class/source paths.
`pins.json` records the exact input bytes and six-line original runtime oracle.
The preceding sixteen-method order probe and all baseline/admission inputs are unchanged.

The separate complete two-method negative class assigns the factory receiver to a
static field before invoking `value`. Its original Runner observes the result, trace
and assignment. The currently unproved receiver assignment must retain the integer
carry's original three-consumer refusal, quote every physical instruction and expose
ExplanationOnly with `jarde_refused_body`, rather than a surviving void return.
