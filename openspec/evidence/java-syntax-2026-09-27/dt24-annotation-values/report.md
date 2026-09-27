# DT-24：复杂注解值的顶级隔离对照

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestAnnotationsMix` 同时含 String/int、float、double 数组、`Class<?>`、enum、嵌套注解等元素，不过测试对这段注解源码并无直接正向断言；它只排除错误的布尔局部写法。`TestReplaceConstsInAnnotations2` 精确检查整数常量名，但叠加了 A/C/C2 多个嵌套类及常量引用。先用顶级 `Mix`、`Simple`、`Mode` 和 `Tagged` 隔离**值编码与源码拼写**，避免 DT-22 的词法家族缺口遮蔽这层事实。

`replay.py` 固定 `javac --release 8` 与 JADX checkout，从完整 class 集合分别生成原始、JADX、Jarde 全源码。三套源码都能以 Java 8 重编，`java -Xverify:all` 后反射 `Tagged` 的运行时注解，输出逐字节一致：

```text
b:7:9.87:[0.0, 1.1]:java.lang.Exception:TWO:false:[3, 5]
```

Jarde 的 `@Mix` 声明保留所有元素：float/double 用可精确回编的十六进制字面量，`Exception.class`、`Mode.TWO`、`@Simple(false)` 与整数数组都在同一 annotation value tree 中。输入/class SHA 和三方结果见 `results.json`，固定生成源码在同目录。这里没有发现需要新增解析器的值类型缺口；现有 `ElementValueFacts`、`resolve_default`/`spell_member_annotation_uses` 已足够覆盖这个隔离样例。

本项仍是**部分已测**。JADX 两个测试都含嵌套类型；常量名替换还需要证明字段的源级资格和 owner，不能因为整数值反射相同就声称 `C.INT_CONST` 原样恢复。DT-22 成员注解与多 child 布局落地后，需用原组合重编，并单独审计常量引用的来源，而不是把这两个问题混入现有值树。

重放命令：

```sh
JARDE_CLI=/absolute/path/to/jarde-cli python3 openspec/evidence/java-syntax-2026-09-27/dt24-annotation-values/replay.py
```
