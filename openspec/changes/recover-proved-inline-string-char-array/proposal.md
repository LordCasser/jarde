## Why

EM-27 的固定 Java 8 输入中，`new String(new char[]{'a','b','c'})` 的数组初始化被 Jarde 单独证明，但外层 `new@1` 在更早阶段看到内嵌数组写入便拒绝，完整源码因此缺返回语句。固定 JADX 虽能重编，却把 `new String(char[])` 折为 `"abc"`，把原本不同于字面量的对象身份改成相同；需要在保持身份的前提下恢复这条构造链。

## What Changes

- 在同次恢复中让已证明的单个 `char[]` 初始化链作为准确 `java/lang/String.<init>([C)V` 的构造实参；证明内外分配、数组元素写入、构造调用和直接返回顺序及唯一消费者后，输出 `new java.lang.String(new char[]{...})`。
- 复用现有数组初始化、`new@1`、AST 和来源锚点；在任一关系、效果、handler、预算或取消证据不完整时整体保留物理回退，不发布半个构造表达式。
- 以原/JADX/Jarde 完整 Java 8 源码重编和 `-Xverify:all` 对照，并检查与同内容字符串字面量的 `==` 结果；Jarde 必须与原 class 一致，不按固定 JADX 的错误折叠改变身份。
- 本变更不处理 byte[]/charset、空构造、数组别名/突变、额外副作用、其它对象构造器、J11 invokedynamic concat 或一般常量折叠。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：增加已证明的内嵌 `char[]` 初始化作为 `String([C)V` 实参的保身份源码恢复和原子拒绝契约。

## Impact

涉及 `jarde-java` 已有数组初始化计划、`init::sites` 构造站点证明及方法 builder 的同次证据交接和定向测试。保持物理方法、BCI/来源映射、预算与现有 CLI/schema；不新增表达式节点、通用重写 pass 或依赖。审计证据见 [EM-27 三方对照](../../evidence/java-syntax-2026-09-27/em27-string-concat/report.md)。
