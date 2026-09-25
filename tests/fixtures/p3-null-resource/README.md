# Null-resource fixtures

These Java 8 inputs cover the narrow `aconst_null; astore` resource case, the adjacent ordinary
catch case, a verifier-valid damaged suppression chain, and the declaration-type boundary. Only
target class files are checked in; runner and helper classes are generated in a temporary output
directory when replaying the fixtures.

## Frozen positive class

`NullResourceCore.java` compiles with `javac --release 8 -g:none` to the frozen 670-byte
`v8/NullResourceCore.class` (SHA-256
`bf4543b40274c9cc95559ed1ff6d3a5d56f44b02011b29948235c142f8448e2e`). Its
`useNullResource()V` protocol is:

| BCI | Bytecode role |
| ---: | --- |
| 0–1 | `aconst_null; astore_0` initializer |
| 2–10 | protected body and normal-path null test |
| 15 | normal `NullResourceCore.close()V` |
| 21 | primary-exception handler |
| 27 | exceptional `NullResourceCore.close()V` |
| 34, 36 | load primary/suppressed exceptions; `Throwable.addSuppressed` |
| 39–40 | rethrow primary exception |
| 41 | normal return |

The exception table protects `[2, 10)` with handler 21 and `[26, 30)` with handler 33. The
resource class directly implements `AutoCloseable`. `NullResourceRunner.java` expects
`null-resource:bodyCalls=1,closes=0`.

## Classification and refusal inputs

`same-type/NullResourceCore.java` keeps the same declared resource class. Its
`ordinaryNullThenCatch()V` starts with `aconst_null; astore_0`, then has an ordinary
`RuntimeException` handler over the `Object.toString()` call; there is no resource close protocol.
The frozen class SHA-256 is
`03438338c836ef6e890bbeeb50104d5f479687075c2606ad8a34c8105f956adb`. The
additional handwritten `closeWithoutSuppression()` method is only a source-shape note: javac
branches around its close (`goto` precedes the close), so it does not pass the current normal-close
candidate prefilter and is not used as the suppression-refusal assertion. The jarde whole-class
compile consequently still fails in this adjacent method, where its ordinary null local is emitted
as `Object` before `.close()`; the `ordinaryNullThenCatch()` method itself remains structured and
keeps its user `RuntimeException` catch. This existing null-local type-inference debt is outside
this resource-header change.

`broken-suppression/v8/NullResourceCore.class` is derived from the frozen positive class by
`patch_broken_suppression.py`. The script checks that BCI 36 is the exact
`Throwable.addSuppressed(Throwable)V` invocation and replaces the three-byte instruction at BCI
36–38 with `pop2; nop; nop`. The original and mutated class files differ only at file offsets 556
and 558 (the middle byte was already zero). The class remains 670 bytes. The original
`useNullResource()V` Code bytes hash to
`1fac6c6eec836e6e5fbc73541f2e74f5de564ddc7e20aa664eec0dda59099c49`; the mutated Code bytes
hash to `08f802f09b587ea26beb558e483903513959e3597da3e99a72129fa308949083`. The mutation leaves
the method's exception table and StackMapTable unchanged. Its class SHA-256 is
`9b1799db42e4ebafb1f8bf129d7156f870bd57b7dfe1b4ba29224bb7a5a289ee`.

The original and mutated class both pass `java -Xverify:all` with their runner and print
`null-resource:bodyCalls=1,closes=0`; the broken class is the negative input for preserving a
source-bearing `jre_guard_suppressed` refusal.

## Type-boundary and exceptional-body inputs

`inherited/NullResourceInheritedOnly.java` inherits `AutoCloseable` through `NullResourceBase`
and overrides `close()` locally, but its class file declares zero direct interfaces. The fixture
keeps `NullResourceInheritedOnly.class` (SHA-256
`c71386e7dc539ab9ffbc4dac2ec6fcb71b0d30164cbf5bbc93ac2ae03c07424e`) and
`NullResourceBase.class` (SHA-256
`11e9241a3024e480557ee3bc53b847411cbd190b8681309e2b13fb4511cd8207`) so the original hierarchy
can be recompiled. This fixture is not evidence that jarde can resolve an external superclass:
the required behavior is to refuse to invent the current class resource type from inheritance.

`exceptional/NullResourceExceptionalCore.java` places a single helper call in the null-resource
body. The helper throws a caller-owned marker; the runner checks object identity, an empty
suppressed list, one body call, and zero closes. This path ensures the exceptional body arm follows
the proved resource structure even though the resource is null. Its target class SHA-256 is
`ae9d8ea51f6f6e6f715c13ba69d27a60614c44b5c906655bfe781d279ff4f61a`.

The bytecode inspections above used OpenJDK 23.0.1. With final frozen CLI SHA-256
`30481c1449b5a1869773d21b934789d42bb1d6a9cab8d81f873d50055af4e34c`, the reconstructed
exceptional class compiles with its helper and runner using `javac --release 8 -g:none`; its
class-file SHA-256 matches the frozen input. `java -Xverify:all` prints
`null-resource-exception:identity=true,suppressed=0,bodyCalls=1,closes=0`. The full comparison
and the separate JADX compilation limitation are recorded under
`openspec/evidence/java-syntax-2026-09-22/try-with-resources/root-after-null/exceptional/`.
