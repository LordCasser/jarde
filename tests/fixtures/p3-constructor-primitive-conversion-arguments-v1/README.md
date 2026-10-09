# Constructor primitive conversion fixture v1

This complete two-class input is a byte-for-byte copy of the original classfiles, Java sources,
and runtime streams saved by the frozen CLI2 baseline. The per-leg `classes`, `sources`, and
`oracle` files are recorded in `fixture-manifest-v1.json`, which binds them to the baseline
manifest SHA. The classfiles were copied directly; this fixture did not recompile them.

The test opens each complete class pair, asks for source for both classes from the same snapshot,
compiles both generated sources with an empty classpath and sourcepath, and runs only that new
class directory with `-Xverify:all`. The host-JDK check is local integration evidence; the root
baseline preserves the separate Corretto 8 and OpenJDK 23 comparisons.
