# Primitive conversion integration fixture v1

The canonical two-class fixture at
`tests/fixtures/p3-constructor-primitive-conversion-arguments-v1/` is copied from the frozen
`results/baseline-v1/` original inputs. For each Java leg, both `.java` sources, both compiled
`.class` files, and the exact original stdout/stderr are copied byte-for-byte; the fixture manifest
records the baseline source paths and hashes. No fixed original class was rebuilt. The test asks
class-source for both classes in one snapshot, rejects any body or class refusal marker, and
requires the complete two-source result to compile under explicit empty classpath/sourcepath and
run from its private output directory with `-Xverify:all`. Its normal CI JDK is a host integration
check, not a replacement for the frozen Corretto 8/OpenJDK 23 baseline.

The main class pins all 32 constructor allocation sites across wrapper, boxed-array, ordinary
return, stored-local, fifteen one-op conversions, and three multi-op round trips. It checks unique
presented `NewRecord`s, source-map coverage of allocation/dup/constructor/argument/conversion and
array-store BCIs, visible cast order, and one evaluation of the stored-local producer. Runtime
stdout/stderr and exit are compared byte-for-byte against both frozen oracles.

The separate same-handler positive is the complete one-class fixture at
`tests/fixtures/p3-constructor-primitive-conversion-controls-v1/`. Its v2 build manifest preserves
both javac/javap runs and the same-cover exception-table facts. The earlier v1 attempt remains saved
as evidence of why the return-only shape did not satisfy the handler-coverage precondition.
