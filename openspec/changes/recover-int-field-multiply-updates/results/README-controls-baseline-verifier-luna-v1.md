# Multiply-controls historical baseline verifier

`verify-controls-baseline-luna-v1.py` authenticates the closed `controls-prepared-luna-v1/baseline-root-v1` evidence and writes `controls-baseline-independent-acceptance-luna-v1.json` only when every check passes. It refuses to overwrite an existing result. It is scoped to this historical controls baseline and its frozen CLI v2 metadata; it does not claim any candidate implementation or candidate runtime acceptance.

The verifier imports the already accepted EM23 v3 verifier only after checking its fixed SHA-256 (`efead211ceb2ecde7540a4d0a8a0493f74a47009899be0dff603f8afcbfb9161`). It reuses only generic byte/identity helpers: `recorded_bytes`, `parse_javap`, `owner_identity`, and `class_members`. All controls-specific roots, source/Runner pins, expected output, JDK/JADX/Jarde pins, command plan, member census, and expected physical BCI lists are independently fixed here.

It verifies all 126 inventory files and 33 raw command records; the fixed two JDKs, JADX 1.5.6, CLI binary and archived metadata; exact two-target original/JADX jars; complete isolated Java 8 source recompiles and `-Xverify:all` raw triples for the two original and four JADX legs; and all four recorded Jarde compile failures with no emitted class files or runtime attempt. Both default/all source reports must preserve identical outer/nested text and method source maps matching every original physical instruction BCI.

The controls Runner oracle checks the full raw sequence, including signed overflow (`2147483645`, `-2147483648`), divide-by-zero mutation behavior (`ArithmeticException,field=17`), and null receiver precedence (`NullPointerException` before division). The verifier separately checks `multiplyDivide(I)I`: exact physical descriptor/flags/BCIs, `putfield` at BCI 13, receiver reload at 17, nested-field post-read at 20, and `ireturn` at 23. The Jarde explanation-only report must retain source-map origins for all those physical instructions. These are baseline observations; because all four Jarde full-source compiles fail, the report makes no Jarde runtime claim.

Preparation consisted of `py_compile` and read-only manifest/schema checks only. The final verifier was not executed.

```sh
uv run --no-project --with blake3==1.0.11 python -B \
  openspec/changes/recover-int-field-multiply-updates/results/verify-controls-baseline-luna-v1.py
```
