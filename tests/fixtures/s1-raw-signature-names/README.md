# Supplementary generic signature fixtures

`supplementary.jar` contains four standalone class files compiled by OpenJDK 23 with `--release 8 -g:none -encoding UTF-8`. It contains only the target classes, with no authored `SignatureDriver` support class. Their class and method names are ASCII; each generic type variable is `𐐀T`.

- `ClassScope`: class type variable used by a field and method.
- `MethodScope`: method type variable and bounded return/parameters.
- `ConstructorScope`: generic constructor type variable.
- `ThrowsScope`: method-local generic throws type variable.

The corresponding authored sources, compiler receipts, original reflection output, and SHA-256 pins are in `openspec/changes/restructure-demand-driven-recovery/results/s1-raw-signature-names-v1/matrix-inputs-r3/`.
