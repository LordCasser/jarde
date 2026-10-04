## Why

已交付的限定外部实例成员构造能力（DT-03：`recover-proved-member-inner-construction` 9/9、`recover-inner-class-instance-folding` 7/7）**只识别 javac 9+ 的 null-check 拼写**，而本项目的目标输出层级是 **Java 8**（`classfile.rs:145` 的 `OutputLevel::Java8` 是唯一变体）。root 2026-10-04 巡查用**决定性单变量实验**证明：同一份 `N1.java` 源码，

| 编译工具链 | null-check 惯用法 | 渲染引注 | `arg1.new Inner(9).total()` |
| --- | --- | --- | --- |
| javac 23 `--release 8`（两片验收 fixture 所用） | `invokestatic Objects.requireNonNull(Object)Object` | 3 | 折叠 ✓ |
| **真 javac 8**（Corretto 1.8.0_432） | `invokevirtual Object.getClass()Class` | **18** | **拒绝** ✗ |

即**真实 Java 8 字节码的 `outer.new Inner(…)` 被整方法拒绝**。失败是响亮的（`not recovered` + 引注，整类不投影，不产生"可编译但行为不同"的文本），故不属静默偏离事故；但它是目标层级上的真实覆盖缺口，且 `outer.new Inner()` 是 Java 8 常见形（内部类持有外部实例）。

判据落点两处，均已读码核实：

1. `crates/jarde-java/src/init.rs:985-998`：要求 `Load{..}` → `Duplicate` → `Invoke{ kind: Static, !is_interface_reference, owner == "java/util/Objects", name == "requireNonNull", descriptor == "(Ljava/lang/Object;)Ljava/lang/Object;" }` → `pop (0x57)`。
2. `src/member_inner.rs:126-131`：要求 `check.opcode == 0xb8`（`invokestatic`）且 CP 条目 `MethodRef{ owner: "java/util/Objects", name: "requireNonNull", descriptor: "(Ljava/lang/Object;)Ljava/lang/Object;" }`，否则拒 `"member call has no exact early requireNonNull check"`。

真 javac 8 的形是 `opcode 0xb6`（`invokevirtual`）+ `MethodRef{ owner: "java/lang/Object", name: "getClass", descriptor: "()Ljava/lang/Class;" }` + `pop`——**两处判据均不匹配**。

取证与全部实验记录见 [qualified-outer-alloc-getclass-patrol](../../evidence/java-syntax-2026-10-04/qualified-outer-alloc-getclass-patrol/README.md)。

## What Changes

- 两处判据各**并列**增加 `getClass()` 拼写的识别，把判据从"认某一种拼写"改为"认这一事实的两种拼写"：分配序列内**连续的** `dup` → 一个**返回值被 `pop` 丢弃**的 null 检查调用 → 紧邻 `invokespecial` 构造器。
- **保持既有位置约束逐字不变**：连续性、锚定在分配点、区间检查（`init.rs` 的 contiguous 检查、`member_inner.rs` 的 `bci <= pop.bci || bci >= call_bci`）都不放宽。
- **`requireNonNull` 支逐字保留**：javac 9+ 产物零回退（既有 8 个 fixture 类与两片全部测试必须逐字通过）。
- **不得引入 `java_release` / `major_version` 门**：root 实测两种产物的 class major version **均为 52**，版本字段无法区分；且 `crates/jarde-reader/src/classfile.rs:139-141` 已固化原则——答案"stated over the facts that class really carries — **never over the compiler that produced it** and never over its own `major_version`"。现行判据正是违反该原则（认编译器拼写而非事实），本片按该原则修正。
- 冻结**真 javac 8 编译**的正例与负例（见 tasks 1.2），并新增 CI 测试；两片既有 fixture 与测试零回退。
- **顺带修正文档**：`init.rs` 与 `member_inner.rs` 的拒绝文本与注释中"exact requireNonNull"的措辞须改为覆盖两种拼写的表述（否则拒绝文本会与实际判据不符）。

## Impact

- **代码**：`crates/jarde-java/src/init.rs`（约 985-998）、`src/member_inner.rs`（约 126-131）。**两处判据的所有者不同**（`init.rs` 属 `jarde-java` 的 `new@1` 私有证明，`member_inner.rs` 属根 crate 的成员调用路径），须各自修改并保持判据同形——**不得**只改一处（只改一处会导致同一形在不同路径下接受面不一致）。
- **不涉及**：`emit.rs`、`report.rs`、`ast.rs`（发射侧已有 `outer.new Inner(args)` 通道，由两片交付）；`crates/jarde-reader`（擦除与 CP 事实已覆盖 `getClass` 的 `MethodRef`）。
- **账本**：DT-03（`declarations-types.md` 证据边界列已记载本差距，落地后须更新为"已修复"）；`summary.md` 的 DT-03 状态归属须从"冻结差距已修复，单元待扩验"改为反映新差距已闭合。
- **验收锚纪律**：本片必须**双腿**（真 javac 8 与 javac 23 `--release 8`）都通过——单腿正是导致该缺口不可见的原因。

## Non-Goals

- **不**支持用户显式 `o.getClass();` 语句被误折叠为 null-check（该负例必须冻结并保持拒绝/不误折叠，见 tasks 1.2）。root 已实测二者位置可区分：用户语句在 `new` **之前**，javac 的 null-check 在分配序列**内**且紧邻构造器。
- **不**放宽 `pop` 之外的返回值消费形（如 `x = Objects.requireNonNull(o)` 保留返回值的写法）——那属另一形，须独立取证。
- **不**触及 accessor 写形缺口（`accessor.rs:480` 有意拒绝返回值形写访问器 `(LC;I)I`）——已登记为**独立债务**，须先读 `d09f5dea`（"recover proved parent field writes and private setter helper"）已交付的通路再另行立项。
- **不**做"整个 corpus 用真 javac 8 重编"的工具链迁移——那是独立的大颗粒项（见 tasks 3.4 的登记）。
