## Context

隔离结果见 `openspec/evidence/java-syntax-2026-09-27/dt29-reference-cast-audit/report.md`。接口 cast 闭环已通过三方完整源码重编及运行，不再进入实施范围。字段 fixture 固定为一个根类、静态嵌套 `A/B`、单 setter：`B.set(boolean, boolean)` 写 `A.visible` 与 `A.hidden`。Runner 通过 binary name 反射调用和检查字段，避免将源级嵌套组织方式混入行为判据。

Javac 对 `A.visible` 生成 owner 为 `A` 的直接 `putfield`；Jarde 的 field recovery 只接受成员已证明由 receiver 自身声明的路径，因而拒绝 `B` 的父类字段写入。`A.hidden` 的 private 写入生成唯一静态访问器 `A.access$002(A, boolean)boolean`；`B.set` 对该调用的 receiver-to-parent 实参转换被拒绝，helper 本身也因写入效果无法形成返回表达式而缺少方法体。原始和固定 JADX 输出均能用相同反射 Runner 重编、验证并打印 `true:false`。

## Goals / Non-Goals

**Goals:** 用输入中已证明的父类关系和精确 field reference 恢复一次父类 public 字段写入。仅当 private accessor 唯一、描述符精确、正文是目标 private 字段的单次更新且返回值在调用方被丢弃时，保留 private 写入效果，并允许已证明的 `B→A` receiver 转换。结果类族经 Java 8 重编与反射 Runner 后打印 `true:false`。

**Non-Goals:** 不处理 fixed `TestFieldCast` 的完整多类组合、泛型类型参数、额外 child、多个 accessor、隐藏字段、复杂控制流、字符串拼接或任意名称类似 `$access` 的方法。不改变已经通过的接口 cast 切片。

## Decisions

1. 保留字段指令声明的确切 owner `A`，只在完整选中类定义证明 `B extends A`、`A` 声明该 name/descriptor，且当前写入 BCI 的 receiver 确为 `B` 时允许该写入。源码应显式通过 `A` 类型接收者（例如 `((A) this).visible`）选择原字段；裸 `this.visible` 在 `B` 隐藏同名字段时会误选。现有 `field@1` 的严格 receiver==owner 门仍适用于没有这组逐 BCI 证明的其它指令。
2. 对私有写入只认领精确的 accessor 调用链：唯一调用目标、receiver 的已证明父类转换、一个目标 private field store、与原描述符一致的返回值，以及调用点对 accessor 结果的丢弃。优先完整呈现原物理 helper 和调用，不为达成此闭环引入跨类 accessor 内联；若采用内联也必须证明类族中 helper 可安全省略。条件任一不符即拒绝折叠或 helper 的结构化恢复。
3. `B.set` 要么呈现 public store 与 private accessor 的完整合法调用，要么沿现有来源协议保留未恢复区域。不得静默丢弃 `putfield`、在 access$ 方法中留下缺 `return` 的文本，或将拒绝结果写成完整源码。
4. 复用现有 class relation、field reference、invoke 与 producer-consumer 来源事实。无需添加通用类型闭包、名字启发式或新的分析 pass。
5. 反例验证至少包含 owner 错配/父类关系缺失、多个候选 accessor、accessor 额外字段写入；隐藏同名字段是保留 owner 的正向判据，它不能被输出为裸 `B.visible`。
6. fixed `TestFieldCast` 的多字段、多个 child 与泛型 D 组合继续作为后续集成验收；本 change 不把它的所有代码输出问题纳入。

## Risks / Trade-offs

- JVM symbolic owner 与 Java 源接收者类型不同；仅凭 receiver 为子类重绑定字段会破坏隐藏字段语义，因此必须保留 owner 并核对声明。
- private accessor 常使用栈重排表达赋值结果；本项只接受已完整证明的固定形态，不能把任意 `dup*` 都解释成赋值。
- 单 fixture 通过只证明已列形态，不宣称所有 JDK 编译器版本或 private accessor 生成方式兼容。
