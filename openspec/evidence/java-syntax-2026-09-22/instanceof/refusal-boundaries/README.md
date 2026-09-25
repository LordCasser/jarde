# `instanceof` refusal boundaries

These are four legal JVM variants derived from the frozen `InstanceOfProbe.class`.  Each variant
replaces exactly one complete `method_info` by matching its frozen method header and Code
attribute; no class-file parser or permanent class file is added.  The exact replacements are
duplicated in `tests/p3_instanceof.rs` so the Engine regression uses the same byte sequences.

| variant | method and bytecode | JVM behavior | recovery boundary |
| --- | --- | --- | --- |
| `pop` | `called()V`: `invokestatic value`, `instanceof Integer`, `pop`, `return` at BCIs 0, 3, 6, 7 | producer runs once and prints `pop=returned:1` | the test and its discarded result cannot become a bare Java expression statement; producer and consumer sources remain mapped/quoted as needed |
| `duplicate` | `local(Object)Z`: `instanceof`, `dup`, `istore_1`, `istore_2`, `iload_1`, `ireturn` at BCIs 1, 4, 5, 6, 7, 8 | `duplicate-true=true`, `duplicate-false=false` | both consumers of the duplicated test value remain real source boundaries |
| `stale` | `local(Object)Z`: `instanceof`, `istore_1`, `iconst_0`, `istore_1`, `iload_1`, `ireturn` at BCIs 1, 4, 5, 6, 7, 8 | both inputs print `false` because the old local is overwritten | the original test is not reused as the later local value; the overwrite and read remain covered |
| `int` | `branch(Object)I`: `instanceof`, `iconst_1`, `iand`, `ireturn` at BCIs 1, 4, 5, 6 | `int-string=1`, `int-number=0`, `int-null=0` | JVM int-stack compatibility does not justify emitting Java boolean-to-int bitwise syntax; the `iand` consumer stays explicit |

The patched classes all pass `java -Xverify:all`.  `javap -p -c -v` output, exact SHA-256 values,
the original frozen runner output, and each variant's runner output are stored beside this file.
`RefusalRunner.java` is source-only and is compiled into `/tmp` for the checks.
