`MixedLocalControls.java` is compiled with `javac --release 8 -g:none -Xlint:-options`.
Each method begins with the same mixed short-circuit 1/0 graph as `MixedBooleanLocal.one` but
violates one local-consumer premise:

- `numeric` uses the stored int in arithmetic, so no Boolean descriptor proves its type.
- `rewritten` stores to the same local twice.
- `duplicated` uses `dup` to write the value to a field before `istore`, so the store does not
  directly consume the unique stack Phi.
- `crossing` writes the local in both a protected range and a handler, and reads it afterward;
  its declaration cannot be placed on the one-Store proof.

All classes are Java 8 verifier-valid. The integration test asserts conservative recovery and
source-map coverage of the emitted quote. The protected-range fallback currently quotes its
canonical block starts; improving that older fallback's per-instruction origins is separate.
