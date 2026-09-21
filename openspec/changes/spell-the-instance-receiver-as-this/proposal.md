## Why

恢复文本把实例方法的接收者当成普通局部变量：有 `LocalVariableTable` 时槽位 0 的名字通常是 `this`，而 `this` 是 Java 关键字，于是被确定性别名规则改写成 `this_`；没有该表时按槽位序数写成 `arg0`。两者都不是 Java 里那个位置的自然拼写，且同一份产物在每个实例方法里都重复这一处噪音。

已确认的证据（2026-09-21，主工作区 commit ad0ffa6 + 本轮实现）：

- 语料 `/Users/lordcasser/workspace/vulnerability/vulhub/struts2/s2-009/S2-009.war` 的 `WEB-INF/classes/org/apache/struts2/showcase/action/EmployeeAction.class` 里 `getEmpId()`：`LocalVariableTable` 明确写 `Slot 0 Name this`，`jarde-cli class-source` 输出 `return this_.empId;`（该类共 7 处 `this_`）。
- 同一语料上 `jadx 1.5.6 --single-class` 输出 `return this.empId;`。
- 无 `LocalVariableTable` 的类（`bcprov-jdk15on-152.jar:org/bouncycastle/asn1/ASN1Object`）输出 `local2.writeObject(arg0);`——接收者被写成 `arg0`。
- 根因锚点：`crates/jarde-java/src/names.rs`（`is_java_identifier` 判定 `this` 为关键字、`alias_for` 追加 `_`）与 `names.rs:457`（`format!("arg{slot}")` 按槽位序数命名）——规则本身没错，缺的是"槽位 0 在实例方法里是接收者"这一 JVMS 事实。

## What Changes

- 实例方法（含构造器）的槽位 0 MUST 以接收者形态呈现为 `this`：字段读取 `this.f`、方法调用 `this.m()`、作为实参 `this`，而不是 `this_` 或 `arg0`。
- `LocalVariableTable` 对槽位 0 的名字不再作为普通局部名参与关键字别名化；其它槽位的既有规则完全不变：参数用表中的名字，无证据时用 `arg<slot>`，局部用 `local<slot>`。
- 静态方法、以及槽位 0 不是接收者的任何形态，MUST 保持原有拼写（本 change 不修改它们）。
- 既有分组/求值语义不变：`java8-recovery` 关于 receiver 表达式分组、求值位置的要求继续约束新拼写。
- 受影响文本、golden 语料与测试预期同步更新，不保留旧拼写；不新增兼容开关。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增"实例接收者按其身份拼写"的可观察要求与场景；既有命名、分组、fallback 与 refusal 要求不变。

## Impact

实施涉及 `jarde-java` 的命名与发射（`names.rs`、`build.rs`、`emit.rs` 中把槽位 0 解析为接收者的那一处），以及恢复产物文本。受影响验证：P3 文本级测试、golden fixture（`tests/fixtures/p3-*`、`p4-golden`、`p2-golden` 等含文本预期的文件）、CLI 文档测试、以及受控 JDK 编译执行对照（A13）中的 receiver 样本。`jarde-reader`/`jarde-jvm`/`jarde-query` 不受影响：本 change 只改呈现文本，不改 fact、身份或控制流。

非目标：不引入类型推断命名（例如把 `Object` 形参命名为 `obj`）、不改参数/局部命名规则、不解析 imports、不改变拒绝与 fallback 契约。完整可编译工程装配仍属 `add-parallel-bulk-recovery` 的非目标；本 change 只让同一份文本更像它描述的那个程序。
