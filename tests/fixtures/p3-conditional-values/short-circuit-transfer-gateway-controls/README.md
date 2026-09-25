`ChainExtraBoundary.class` is a verifier-valid Java 8 negative control derived from the frozen
`short-circuit-chain-extra-entry/ChainExtraBoundary.class`. `GenerateMultiEntry.java` changes the
BCI 12 `ifne 25` target to BCI 8, the pure `goto 25`, then recomputes stack frames. BCI 8 now has
normal predecessors from BCI 5 and BCI 12, with a backedge from BCI 12. The remaining assignment
and consumer are unchanged. `ChainExtraRunner` verifies and executes all 32 inputs against this
class in the integration test; a structured field write is forbidden.

To regenerate on a JDK with the bundled ASM package:

```sh
javac --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED \
  --add-exports java.base/jdk.internal.org.objectweb.asm.tree=ALL-UNNAMED GenerateMultiEntry.java
java --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED \
  --add-exports java.base/jdk.internal.org.objectweb.asm.tree=ALL-UNNAMED \
  GenerateMultiEntry path/to/frozen/ChainExtraBoundary.class ChainExtraBoundary.class
```

`ChainExtraBoundaryException.class` is derived by `GenerateExceptionEdge.java`. It protects
BCI 8–11 (the `goto`) with a `Throwable` handler at BCI 34. The original 32-path output remains
unchanged because a direct `goto` cannot throw, but the physical exception-table edge bars the
gateway from the closed graph. The integration test verifies the complete class before asserting
that Jarde does not publish a structured field assignment. Regenerate with the same commands,
substituting `GenerateExceptionEdge` and writing `ChainExtraBoundaryException.class`.
