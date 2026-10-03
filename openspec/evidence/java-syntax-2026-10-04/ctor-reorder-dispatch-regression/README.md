# 构造器重排的构造期虚分派回归（2026-10-04）——已验收切片引入的静默行为回归

**性质：root 验收失误导致的行为回归**，由 2026-10-04 账本审计（`present-proved-java-structure` 未勾项 2.10）发现并实证。实证脚本 [results/repro.sh](results/repro.sh)、渲染产物 [results/jarde-rendered-Base-subclass.java](results/jarde-rendered-Base-subclass.java)，冻结 fixture SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)（`tests/fixtures/proved-java-structure/anonymous-super-dispatch/`）。

## 实证链（逐条可复现）

| 步骤 | 结果 |
| --- | --- |
| 原 class 运行 | `observed=captured-value` / **`visibleDuringSuper=true`** |
| `AnonymousSuperDispatch$1` 原字节码 ctor | `aload_0; aload_1; putfield val$captured; aload_0; invokespecial Base.<init>; return`（**先写捕获，后调 super**） |
| Jarde 呈现（主线 `28eb5755`，切片 `recover-synthetic-ctor-super-order`，合并 daa4fb31） | `super(); this.val$captured = arg1; return;`（**重排**） |
| 该文本 `javac --release 8` 重编后的字节码 | `invokespecial Base.<init>` **在前**、`putfield val$captured` 在后 |
| 重编产物运行 | `observed=null` / **`visibleDuringSuper=false`** ← 行为改变 |

## 根因

`Base()` 构造器在自身构造期间**虚调用** `observe()`（被匿名子类覆写，读 `val$captured`）。原字节码把捕获写入放在 `invokespecial` 之前，故构造期可见；重排后 javac 只能发射"先 super 后写"，虚调用读到 null。

重排判据（`recover-synthetic-ctor-super-order`：pre-super 连续合成字段直传存组 → 移至 super 之后）**未检查 super 目标是否可能虚分派**。

## 为何是"最危险类"缺陷

重排前该产物**不可编译**（`super()` 非首句，javac 报"灵活构造器是预览功能"）——响亮失败；重排后**可编译且行为不同**——静默失败。这正是 `recover-return-in-do-while-false`（已验收）确立的不变量所禁止的形态：不得产出"可编译且行为不同"的呈现。

## 验收失误记录（诚实登记）

1. 切片 `recover-synthetic-ctor-super-order` 的 design/tasks **未把 2.10 的既有反例列为负例**——该反例自 2026-09-25 就冻结在 `tests/fixtures/proved-java-structure/anonymous-super-dispatch/`，且 `present-proved-java-structure` 的 2.10 明写"旧任务所称'改序不损失任何效果'已被运行反例否定，不能作为验收条件"。
2. root 验收（daa4fb31，见该 change tasks 3.3）只核了 C1/C2 两 fixture 的**正常路径行为**，未查该既有反例，也未核"重排是否可能改变构造期可见性"。
3. 教训已记入 handoff：**任何改变指令/语句顺序的切片，验收必须跑既有 order-sensitive 反例 fixture**（本项目已有 `anonymous-super-dispatch`、`ordinary-new-void-effect` 两个），并检查目标 change 之外同域的既有证据目录。

## 判据（精确、可证伪）

已验收的两个正例 fixture 的 super 目标均为 `java/lang/Object."<init>"`（实测 javap）——Object 构造器不可能虚分派到用户代码。回归 fixture 的 super 目标是用户类 `Base.<init>`。故：

**重排仅在 super 目标为 `java/lang/Object.<init>()V` 时安全**；目标为其它任何类（用户类或平台非 Object 类）时保持原顺序呈现（现状：不可编译但忠实，且带引注/诊断——响亮失败优于静默错误）。

## 影响面普查（2026-10-04，javap 扫 `tests/fixtures/proved-java-structure/`）

pre-super 写（`putfield` 早于 `invokespecial`）且 super 目标非 Object 的冻结 fixture 共三处；但**逐一 javap 其 super ctor 后，只有一处真会翻转行为**——判据的关键不是 super 目标是不是 Object，而是 **super ctor 是否在 `this` 上虚分派**：

| fixture | super 目标 | super ctor 是否 `this` 虚分派 | 重排后果 |
| --- | --- | --- | --- |
| `anonymous-super-dispatch/AnonymousSuperDispatch$1` | `Base.<init>` | **是**（ctor 内 `invokevirtual observe()` on `this`） | 行为翻转 `true→false`（本片回归实证） |
| `anonymous-super-args/AnonymousSuperArgs$1` | `Base.<init>` | 否（仅 `Object.<init>`+自身字段+新建 StringBuilder 的 invokevirtual+`invokestatic event`） | 重排安全，产物当前已可编译且行为正确 |
| `anonymous-capture/AnonymousCaptureCases$1` | `AnonymousCaptureCases$Base.<init>` | 否（`invokevirtual` 均在新建 StringBuilder 上，其余 `invokestatic`） | 重排安全，同上 |

故修复判据**不能**简单收窄为"仅 Object super"——那会让后两个**当前正确且可编译**的 fixture 退化为不可编译（反向回归）。正确判据是 design 决策 1 的三档：Object 目标安全；或 super 类在快照内且其 ctor 无 `this` 虚分派则安全；否则不重排。

super 目标为 `java/lang/Object."<init>"` 的 pre-super 写形（`AnonymousCaptureCases$Outer$1`、`Inner$1`、`AnonymousMemberBase$Outer$Base`）落入第一档，不受影响。

修复片 `recover-ctor-reorder-dispatch-guard` 的 1.2 变体须覆盖上述**三处**并分别断言：dispatch 形不重排（回归消除）、两处无分派形仍重排且逐字不变（守反向回归），各自以重编运行对照钉死"不再出现可编译且行为不同"。

## 处置

`recover-ctor-reorder-dispatch-guard`（窄修复片，最高优先）：在重排判据加"super 目标 == `java/lang/Object.<init>()V`"前置条件；不满足时不重排（保持既有逐字呈现与诊断）。C1/C2 及全部既有重排正例逐字不变（它们都是 Object super）；回归 fixture 恢复为"不可编译但行为忠实"，并以该 fixture 建**回归测试**（断言呈现中 `putfield` 语义序早于 `super()`，即文本含 `this.val$captured = arg1;` 在 `super();` 之前）。

原 class 为行为基准（`observed=captured-value`/`visibleDuringSuper=true`）。
