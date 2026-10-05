## Why

[丢弃分配巡查](../../evidence/java-syntax-2026-10-05/discarded-allocation-patrol/README.md)实证（判别完整）：**结果丢弃的分配语句** `new N();`（独立语句、构造器副作用、结果无任何消费方）整方法拒绝——"the instruction at BCI N belongs to no shape this run verified: an allocation, a copy or a cast…" + "the copy … has no proved local assignment"。三种消费形（赋值/链式调用/实参）**全部恢复**，唯一变量是结果是否被读取。**jadx 完整呈现**（`new DN.N();`）。

**真实命中面**：副作用构造形——`new Timer();`（自注册）、`new Thread(...).start();` 之外的裸 `new Foo();`、测试/初始化代码中的死分配。响亮拒绝（行为安全）但整方法损失。

> **root 追加锚（2026-10-05，ctor-throw 巡查）**：`CE.main` 的 try 块内 `new CE(5); new CE();`（委派目标**抛异常**的构造——catch 捕获路径）同因拒绝（BCI 0/3/9/12 同族诊断）；ctor 侧（初始化器并入、提前 `throw` + else/return、委派后 unreachable 语句）**全部忠实**（行为逐行验证：`sc/ic/ctor ok:5/ic/caught:neg:-1`）。即第 5 片的 try-块变体 + ctor-抛异常交互已归档（[ctor-throw-interaction-patrol](../../evidence/java-syntax-2026-10-05/ctor-throw-init-patrol/README.md)），实现时以 `CE` 为追加验收锚。

**根因推断（root 读码，待实现者插桩确认）**：现有分配证明链要求实例**恰有一个 Java 拼写位**（`single_use_at`/读者门 `readers.len() != 1` 的"实例只有一个拼写位"不变量）——丢弃形的读者数为 **0**，落入门外。注意这与"多处真实消费仍拒"是**同一不变量的两个方向**：>1 拒（防两个拼写位）、==1 恢复、==0 现状拒。

## What Changes

把丢弃分配识别为合法语句形：当分配的实例**零读者**（无任何指令读取该实例值）且构造器调用本身可证（实参链正常证明）时，呈现 `new N(实参…);` 独立语句——**不放宽 >1 读者拒绝**（那是防多拼写位的核心不变量）、不碰 `single_use_at` 本体（它约束的是"恰 1 个 use 在 at"的成员构造路径）。

## Impact

- **代码**：`crates/jarde-java/src/init.rs` 的分配证明区（读者门所在；与 alloc-qualifier 片刚交付的 `discarded_null_check_tail` 相邻域）。
- **测试**：`DN` fixture（四形判别已冻结）+ CD.main（`new C();` 级联解锁）+ 零回退（消费三形 + 既有分配测试）。
- **账本**：summary.md 登记行关闭。

## Non-Goals

- **不**放宽多读者拒绝（核心不变量的 >1 方向）；
- **不**做"丢弃后重新分配同 slot"等复杂形（后续按实测扩验）；
- **不**碰成员构造/分配限定符路径（alloc-qualifier 片域）。
