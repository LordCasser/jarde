# Java 8 postfix lvalue returned values

`v8/PostfixLvalueValues.class` is the sole permanent compiled fixture. It was built from this
source with `javac -Xlint:-options --release 8 -g:none`; the Rust integration test creates its
runner in a temporary directory. Rebuild the subject class with:

```sh
javac -Xlint:-options --release 8 -g:none -d tests/fixtures/p3-postfix-lvalue-values/v8 \
  tests/fixtures/p3-postfix-lvalue-values/PostfixLvalueValues.java
```

The 1,051-byte class has nine `Code` methods and SHA-256
`7548934ba19c23d533517a76560d525011f8ec261c8a6670b46c1860314cbe21`. Its effectful field and
array updates have the same `dup_x1`/`dup_x2` postfix stack shapes as the frozen source-only audit
in `openspec/evidence/java-syntax-2026-09-22/postfix-lvalue/`. The two extra-consumer refusal
fixtures are referenced directly from that audit's verifier-checked boundary cases and are not
copied into this fixture.
