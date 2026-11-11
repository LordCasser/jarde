# S2b inherited generics: explicit new baseline

These are new, source-built Corretto 8 development inputs for
`tests/s2b_engine_declared_methods.rs`. They replace that test's dependency on
three never-tracked, unavailable `dev-v3-inputs` JARs. They are **not** recovered
historical bytes, and their new hashes do not establish historical equivalence.
The `s2b-signature-paths-r1/custom` files retain their original dev-v2/javac23
provenance and are neither copied nor relabeled here. This is a development
baseline, not an independent holdout or a new recovery acceptance criterion.

The test builds its application and platform JARs in memory using its existing
`jar(entries)` helper. The platform is the same 13 existing frozen class inputs
already used by the other four tests, with unchanged physical loader/definition
binding assertions. No deleted result archive or developer-local path is needed.

## Coverage

- `PlatformChild extends ArrayList<String>`: both inherited get consumers,
  declared `List<String> names`, and the generic field read return.
- `CustomChild extends Mid<String>`, `Mid<X> extends Parent<List<X>>`:
  two-edge substitution for the inherited getter and inherited field, plus
  a declared generic echo method.
- The existing five tests and their assertions remain. The inherited test still
  requires complete execution, projected parent declarations, boot-loader
  `String` dependencies for both consumers, nonempty re-emittable member bodies,
  and the declared field/return types.
- `Driver` checks indexed and first-element reads, empty and invalid-index
  exceptions, null values, inherited set/get, direct inherited field writes,
  generic identity, and alias mutation for the complete two target classes.

## Rebuild and validate

Use the exact JDK identified by `toolchain.json`. The generator checks the
actual `bin/javac`, compiler implementation `lib/tools.jar`, `bin/java`, and
`jre/lib/rt.jar` hashes before executing anything; it also checks `javac -version`.
The recorded compiler is Corretto 8, not a later javac targeting Java 8.

From the repository root:

```sh
python3 tests/fixtures/s2b-inherited-generics/generate.py \
  --jdk /path/to/amazon-corretto-8.432.06.1-linux-x64 \
  --output /path/to/new-s2b-build
```

The output directory must not exist. The script compiles all five sources twice
in separate class directories, requires byte-identical output and Java 8 version
52.0, then runs the original complete classes with `-Xverify:all` and an isolated
classpath. Ambient Java options and `CLASSPATH` are removed. Sources, classes,
compiler inputs, and the reused platform headers have byte counts and SHA-256
pins in the generated `manifest.json`. `commands.json` is local execution evidence
and includes machine-local command paths; it is not a replacement for the manifest.

To validate complete recovered classes, capture the public Engine's unchanged
`report.text` for both targets as `CustomChild.java` and `PlatformChild.java`, then:

```sh
python3 tests/fixtures/s2b-inherited-generics/generate.py \
  --jdk /path/to/amazon-corretto-8.432.06.1-linux-x64 \
  --output /path/to/new-s2b-recovered-build \
  --recovered-sources /path/to/engine-emitted-sources
```

This separately compiles both complete recovered targets against only original
`Parent`/`Mid` support classes. It then compiles the complete `Driver` against that
recovered class set and runs it with no original target classes on the classpath.
It records `recovered-validation.json` and requires the original/recovered output
to match. A baseline-only run explicitly does not claim recovered-source runtime
validation. The Driver establishes the listed cases, not general semantic
correctness or independent holdout coverage.

The checked-in `.class` files and `manifest.json` are generated artifacts from
these sources. Regeneration must reproduce their bytes with the pinned compiler;
never replace the pins with historical dev-v3 claims.

## Public Engine runtime regression

The normal suite keeps its original five tests. A separate ignored test runs the
complete-class compile/runtime comparison through the public Engine:

```sh
S2B_CORRETTO8_HOME=/path/to/amazon-corretto-8.432.06.1-linux-x64 \
  cargo test --locked --test s2b_engine_declared_methods \
  engine_inherited_generics_complete_classes_match_original_runtime \
  -- --ignored --exact --nocapture
```

This requires `python3` on `PATH` and the exact Linux x64 Corretto toolchain in
`toolchain.json`. The test fails rather than skips when explicitly invoked without
that prerequisite. It uses the same 13 frozen platform headers, checks complete
Engine execution and no fallback methods, writes both complete `report.text`
values unchanged, and invokes the generator's isolated original/recovered Driver
comparison. Temporary source/class directories are removed after the test, and
successful recovered validation evidence is printed to the test log.
