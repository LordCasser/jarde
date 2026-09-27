# DT-23：顶级类型的注解放置已通过，嵌套组合仍未验收

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestParamAnnotations` 直接断言参数 0/1 的注解源码位置及显式元素值；`TestAnnotationsUsage` 主要检查 use 图，还断言三次注解用法文本。两个测试都把注解或承载类写成成员类型。因此先用本目录的顶级 `A` 与 `Types` 隔离注解**放置**，避免 DT-22 的成员注解声明缺口替这项制造假失败。

`replay.py` 固定 `javac --release 8 -g:none`，将原 class、固定 JADX 和 Jarde 的完整类型源码与同一 `Runner` 重编，再用 `java -Xverify:all` 检查运行时可见注解。`A` 有 `Class<?> c()` 与 `int i() default 7`。`Types` 的类、字段、方法、第 0 参数和第 1 参数分别带 `@A`，末项显式写 `i = 5`，因此同时核对参数索引与元素值。三方输出逐字一致：

```text
class=true
field=true
method=true
param0=true
param1=5
```

Jarde 完整输出中，五个 `@A` 分别落在正确声明和参数位置；顶级注解自身的 `Retention(RUNTIME)` 与泛型 `Class<?>` 元素也能重编。输入/class SHA、工具版本与输出在 `results.json`；固定 Jarde/JADX 源码保存在同目录。现有 `class_source` 从类/字段/方法的 RuntimeVisibleAnnotations 与方法参数表读同轮事实，再由声明 writer 放置；这条路径当前无需另起实现机制。

这里**没有**把 DT-23 整项标为追平。原 `TestAnnotationsUsage` 的 A/B/C 多成员类组合需要先恢复其词法 owner；`TestParamAnnotations` 也使用成员注解。当前 DT-22 已单独证明 `Holder.A` 失败，复杂多 child 组合还需在其后的固定样例复验。它们是声明家族差距，不能归咎于本次已通过的注解放置。下一步在 DT-22 成员注解落地后，按原测试的嵌套形态再次重编并用反射核对。

重放命令：

```sh
JARDE_CLI=/absolute/path/to/jarde-cli python3 openspec/evidence/java-syntax-2026-09-27/dt23-annotation-placement/replay.py
```
