## Why

`present-proved-java-structure` 5.3（类级匿名语法）的**精确剩余范围**，root 2026-10-04 以代码与 javap 双重实证界定：两个已交付片各覆盖一半、各**明确排除**对方的一半，混合参数形（super 构造实参与捕获值并存）落在二者之间的缝隙中。

代码实证（两门互斥）：

- `src/member_inner.rs::prove_family_capture` 第 270 行：`refuse("constructor must invoke the direct zero-argument superclass constructor")` —— 有捕获字段的证明要求父类构造器**零参数**。
- `src/facade.rs::project_class_source_anonymous_super` 第 4606 行：`child_facts.field_count != 0` 即拒绝 —— 转发父类实参的证明要求**无捕获字段**。

措辞实证（两片自述排除）：

- `inline-proved-anonymous-super-arguments`（8/8）：范围是"无捕获、无实例字段和初始化效果"；并明写"**构造参数中混入捕获值**、实例字段写入、重载目标不明、第二分配点、跨类引用或正文不完整时保留物理类源码"。
- `recover-proved-anonymous-inner-this`（8/8）：明写"**无捕获匿名父类构造实参恢复仍由 `inline-proved-anonymous-super-arguments` 单独负责**；本变更的捕获字段只服务词法 `Inner.this`，不解释或改写父类构造实参"。
- `recover-proved-anonymous-local-capture`（6/6）：判据是"唯一直接返回匿名接口实例、一个 synthetic-final `double` 捕获字段、**一个准确构造器参数**"——接口形且无 super 实参。

冻结 fixture `tests/fixtures/proved-java-structure/anonymous-super-args/AnonymousSuperArgs$1`（javap 实证）正是该形：

```text
final java.lang.String val$captured;
AnonymousSuperArgs$1(java.lang.String, int, java.lang.String);
   0: aload_0 / 1: aload_3 / 2: putfield val$captured      ← 第三参是捕获值（pre-super 写）
   5: aload_0 / 6: aload_1 / 7: iload_2 / 8: invokespecial Base."<init>":(Ljava/lang/String;I)V   ← 前两参转发父类
  11: return
```

即物理参数 `(String, int, String)` 中前两个是父类实参、第三个是捕获值，无 `this$0`（静态上下文）。该 fixture 的既有验收证据（`evidence/java-syntax-2026-09-27/anonymous-super-args/report.md`）记录的正是它当前**不可编译**（"完整源码编译因此退出 1"）。

**与 ctor 重排回归的同源关系（架构关联）**：本会话实证的 `recover-ctor-reorder-dispatch-guard` 是过渡性收敛（非 Object super 退回 verbatim → 响亮失败）。其**终局解就是本片**——`present-proved-java-structure` 2.10 原文："等 5.3 的类级匿名语法、捕获值流及整类运行证明齐全，才由 `new Base(...) { ... }` 让 javac 生成等价的前置合成写入"。混合形一旦内联为 `new Base(args) { … }`，pre-super 捕获写入由 javac 自行重建，重排需求消失。

## What Changes

- **合取并集，不新增机制**：把上述两门的排除各放宽一处——`prove_family_capture` 的"零参数父类构造器"改为"父类构造器实参与物理参数的一个**有序子序列**逐位对应"；`project_class_source_anonymous_super` 的 `field_count != 0` 改为"捕获字段与父类实参共存，且**每个物理参数的角色被唯一证明**（转发父类 / 存入某捕获字段）"。两个证明各自保持既有的原子性与拒绝口径，只是判据从互斥变可并存。
- **参数角色划分（本片核心判据）**：物理参数按 SSA 消费点唯一分类——流入 `invokespecial` 父类构造器实参位的为 super 角色（保持左到右序），流入 synthetic 捕获字段 `putfield` 值的为 capture 角色；任一参数两者都不是、或同一参数被两种角色消费、或划分后两角色实参序与物理序不一致，即拒绝。
- **捕获字段的词法替换**沿用 `recover-proved-anonymous-local-capture`/`recover-proved-anonymous-inner-this` 既有通道（已证字段读取 → 根方法参数引用），本片不新建替换机制。
- **MVP 边界**：静态上下文（无 `this$0`）、单个匿名分配点、正文 `quality=structured` 无引用缺口。`this$0` + 捕获 + super 实参三者并存、嵌套匿名、多分配点、跨类引用**不在本片**（如实登记为 5.3 的后续范围）。
- 验收：`anonymous-super-args` 完整源集 `javac --release 8` 通过（当前退出 1）、`java -Xverify:all` 事件日志与原 class 逐行一致；两个已交付片的既有正例（`anonymous-inner-this`、`anonymous-local-capture`、super-args 的无捕获形）逐字不变。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：混合参数匿名类（父类实参 + 捕获值并存）可内联为源级 `new Base(args) { … }`，其构造器脚手架、捕获字段与写入被隐藏且可由 javac 重建。

## Impact

`src/member_inner.rs`（`prove_family_capture` 的 270 行零参数门）、`src/facade.rs`（`project_class_source_anonymous_super` 的 4606 行 field_count 门与参数角色划分）、`src/enum_constants.rs` 若参与装配则同步；`crates/jarde-java` 预期无改动（捕获词法替换与构造器序呈现均已有通道）。既有三个匿名片与 ctor-reorder-guard 的负例零回退。
