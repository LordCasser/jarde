# Bootstrap-descriptor fixture for the B2 counterexample

`v8/LambdaSample.class` is real `javac` output for a lambda, checked in so the test that a
type named only by a bootstrap descriptor is reachable from a `Type`-only request runs
against compiler output instead of only against hand-built structure.

## Provenance

- **Compiler:** `javac 23.0.1` (`/usr/bin/javac`, reported by `javac -version` as
  `javac 23.0.1`).
- **Source:** `LambdaSample.java` in this directory, compiled with

  ```text
  javac --release 8 -g:none -d v8 LambdaSample.java
  ```

  The run prints the expected "source/target value 8 is obsolete" deprecation warning; it
  changes no output bytes. `--release 8` is what makes the sample the Java 8 lambda shape
  (one `invokedynamic` site, one `BootstrapMethods` entry, three static arguments).
- The compiler is a generation-only input. It is not checked into this repository and is
  not a runtime or test-time dependency: the test reads the checked-in `.class` bytes.

## Output

| file | classfile version | bytes | SHA-256 |
| --- | --- | ---: | --- |
| `v8/LambdaSample.class` | 52.0 | 731 | `9588c94b91d6da16750e52969c6557dcf416a4d1929c796f47d52e142fac4083` |

## What the test relies on

`javap -v -p` on the checked-in file reported:

- one `invokedynamic` site at BCI 0 of `run()V`, namely `#7 = InvokeDynamic #0:run:()Ljava/lang/Runnable;`;
- `BootstrapMethods` entry 0: handle `#24 = MethodHandle REF_invokeStatic
  java/lang/invoke/LambdaMetafactory.metafactory:(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodHandle;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/CallSite;`
  with static arguments `#20 ()V`, `#21 REF_invokeStatic LambdaSample.lambda$run$0:()V`,
  `#20 ()V`;
- the implementation member is `private static synthetic lambda$run$0:()V`.

The pool holds **no** `CONSTANT_Class` and no standalone `CONSTANT_Utf8` for
`java/lang/invoke/MethodType`: the type occurs only inside the metafactory descriptor
`#30`, which is reachable through the bootstrap method handle. A `Type` hit for it
therefore cannot come from a `CONSTANT_Class` entry, from a member descriptor or from the
dynamic site's own descriptor (`()Ljava/lang/Runnable;`).

`javap` is a verification aid here and not a test-time dependency.
