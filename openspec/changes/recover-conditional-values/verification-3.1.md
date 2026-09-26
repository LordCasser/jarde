# Root 整类行为验收

2026-09-26 使用主线当前 CLI SHA-256 `4c44f62d4b1dfdf7e83ae486f8619b501eb4d0a781b9a62b980089b4b9b6f8a4`，在独立临时目录重放 `conditional-values-accepted/replay.py`。`TernaryCore` 2 行、`TernaryValues` 16 行、`ConditionalBoundarySwitch` 4 行的原 class、JADX 1.5.6、Jarde essential/all 均以 Java 8 重编并经 `-Xverify:all` 执行相同；两种 Jarde 证据选择的正文 SHA 各自相同，三类均无字节码引用。抛错臂的异常类型、trace 和 String 重载目标由完整 16 行对照覆盖。

同一 CLI 重放 `assert-core/replay.py`：原/JADX/Jarde 在 `-ea` 为 `1|0;bad|2|1`，在 `-da` 为 `0|0;0|0`；将 `<clinit>` 类字面量换成 `StringBuilder` 的有效补丁在选择性 `-ea:AssertCore -da:java.lang.StringBuilder` 下为 `0|0;0|0`，三者一致。非 0/1 布尔字段补丁与静态 final 左值在 [assert-core-current](evidence/assert-core-current/README.md) 和 [field-writes](evidence/field-writes/README.md) 的重编/低位对照中已验收。

Root 另用当前 CLI 对冻结的 `AssertProbe.class` 直接取整类源码、与原 class 分别编译同一 source-only runner，执行 `java -Xverify:all -ea`、`-da`；两种模式各 5 行逐字相同，Jarde 完整类无 `@bytecode`/`not recovered` 标记。它仍呈现显式 `$assertionsDisabled`、`if`/`throw`，不宣称已恢复源级 `assert`；该原子跨成员投影留给 `project-proved-assert-statements`。更宽的异常边/局部作用域和一般 Phi 边界不由此项行为对照放宽。
