# Shared-true short-circuit fixture

This Java 8 class is a verifier-valid near miss for the short-circuit field-write proof. The `||`
form sends both the outer true edge and inner true edge to the shared `iconst_1` producer at BCI 10;
the shared value is written by `putstatic` at BCI 15. `rhs()` increments `calls`, so true on the
left must skip that call. It must remain quoted until that shared-true value shape has a proof.

Freeze command, run from the repository root:

```sh
mkdir -p /tmp/jarde-shared-true-freeze
javac --release 8 -g:none -d /tmp/jarde-shared-true-freeze tests/fixtures/p3-conditional-values/short-circuit-shared-true/SharedTrueShortCircuit.java
cp /tmp/jarde-shared-true-freeze/SharedTrueShortCircuit.class tests/fixtures/p3-conditional-values/short-circuit-shared-true/SharedTrueShortCircuit.class
```

SHA-256:

- `SharedTrueShortCircuit.class`: `4b3632dd7bc9bd26e640112a67a080d10f2171d0e83bf88f3190f67de412e935`
- `SharedTrueShortCircuit.java`: `a777c3dac33aa01cfa49abe11a68c791d3640f4db4c5750942d1347123dab01a`
