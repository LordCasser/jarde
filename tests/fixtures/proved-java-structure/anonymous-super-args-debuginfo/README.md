# Anonymous superclass arguments — same shape compiled with debug info

The `-g` control leg for `recover-anonymous-local-decl-site` tasks 1.3 (dual debug-information
check). The sources are byte-identical with `../anonymous-super-args/`; only the compilation flags
differ, so this classfile carries `LocalVariableTable` (5 tables in `AnonymousSuperArgs.class`,
verified with `javap -l -p`) where the frozen anchor carries none (`javap -l` count 0).

The frozen `.class` files were produced with:

```sh
javac --release 8 -g -d . AnonymousSuperArgs.java Base.java
```

The check this leg pins: the projected left-hand type comes from the initializer's own `new`
operand in both legs — `Base instance = new Base(...) { ... }` here and
`Base local2 = new Base(...) { ... }` there — never from the debug information. The source local
names (`captured`, `instance`) return through the pre-existing naming channel. The recompiled
source set (root + `Base`) exits `javac --release 8` 0 and `java -Xverify:all` replays the original
event log line for line; see
`openspec/evidence/java-syntax-2026-10-04/recover-anonymous-local-decl-site/two-leg-debuginfo/`.
