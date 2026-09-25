# Negative table boundaries

The class files in this directory are Java 8 inputs for negative proof tests. They use the frozen
`EnumSwitchSubject.class` and runner, with only the helper class replaced. `multi-write` writes
`RED.ordinal()` twice (first 1, then 2); `duplicate-key` writes both RED and BLUE to integer key
1. Both class sets run with `java -Xverify:all` and produce the recorded outputs, so these are
valid class files whose table semantics a fixed-map proof must refuse. Their runtime trace also
shows why the repeated stores cannot be erased as harmless initialization noise.

`missing-enum/classes` omits `Hue.class`. Running it with `-Xverify:all` exits 1 with
`NoClassDefFoundError: Hue` / `ClassNotFoundException: Hue`. That is an unresolved selected
dependency, not proof that Hue is absent from all possible environments.

The matching Java helper sources are retained beside their compiled class files. `javap.txt`
records the actual write instructions and exception ranges; `runtime.log` records the VM result.
