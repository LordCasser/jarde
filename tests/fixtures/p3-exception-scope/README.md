# Exception-region local-scope fixture

`v8/ExceptionScope.class` is compiled from `ExceptionScope.java` with javac 23.0.1 using
`--release 8 -g:none`. It has no `LocalVariableTable`; the only names available to recovery are
derived local names.

The class holds two complementary cases. `catchOnly(Z)I` declares and reads `local` only inside its
handler, so that declaration must stay inside the catch body. `assignedAcrossTry(Z)I` stores the same
local on the normal protected path and in the handler, then reads it after the join. Its local slot
is present in the verified join frame, and the two reaching SSA definitions are both stores; this is
the narrow case where a declaration can be lifted to the method block.

Reproduce and verify the checked-in bytes:

```sh
javac --release 8 -g:none -d v8 ExceptionScope.java
shasum -a 256 v8/ExceptionScope.class
java -Xverify:all -cp v8 ExceptionScope
```

The class-file version is 52.0, its size is 588 bytes, and its SHA-256 is
`4ebd20bfc2146e02f8c77cd33ff472e090d440a0299304edf37226a86ee28fcf`. The commands above passed
with the recorded input. `javap -v -c` confirms the exception table and shows no local-variable
debug attribute.
