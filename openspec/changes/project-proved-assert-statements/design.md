## Context

见 [proposal.md](proposal.md) 与 `../../evidence/java-syntax-2026-09-24/assert-core/analysis.md`。`AssertCore` 的 71753f… class 在 BCI 0–16 的 `<clinit>` 中从 `AssertCore.class.desiredAssertionStatus()` 写合成 `Z` 字段，`check` 的 BCI 0–24 先读取该字段，再按 guard 结果进入 `AssertionError(Object)`。当前类源码能写出等价的显式字段、条件和抛错；源码形态仍弱于原 `assert`。错误 owner 补丁和非 0/1 补丁均能由 JVM 验证，不能由字段名模式误收。架构的 Query/Decompiler 分层、物理身份与可选源码投影不变。

## Goals / Non-Goals

**Goals:** 对完整类内可闭合的断言组，证明开关重建、所有守卫及异常路径后一次性投影；覆盖无消息、带消息、多条断言、选择性启停和错误补丁。默认详细证据只改变报告量，不改变源码。

**Non-Goals:** 不修条件值 Phi 或任意 `putstatic Z`；不把普通显式 `if`/`throw` 猜为原写法；不从外部类推导状态；不改独立方法 AST/报告；不引入通用跨方法规则引擎或将 JADX 纳入运行时依赖。

## Decisions

1. **类级闭包而非局部文本替换。** `jarde-java` 从同次 CFG/SSA 与原指令产生候选：字段的真实 Fieldref、每个 getstatic/putstatic、guard 分支、条件与消息表达式、`AssertionError` 分配/构造/抛出、汇合和来源 BCI。`class_source` 在同一物理类读取中核查唯一 `Z` 静态 final synthetic 字段、无 `ConstantValue`、唯一 `<clinit>` 写入、所有可达读取均被候选覆盖；字段名只可作为 Java 编译器重建相同物理名称的附加准入约束，不能替代数据流证明。不存在完整闭包就整类不投影。与只看 `check` 文本相比，这能阻止其他方法读字段后被源码删掉。
2. **初始化只接受可重建的真实状态。** 用 `<clinit>` 的指令、CFG、SSA 证明唯一写入值恰是当前类 `Class.desiredAssertionStatus()Z` 的布尔反值，经标准 0/1 选择入 `Z`；证明赋值位于所有其他静态效果之前，或证明保留其原顺序仍等价。当前类 `Class` 常量必须与物理 `this_class` 相同；错 owner 与 2/3 窄化补丁拒绝。`<clinit>` 其余语句原样保留；不能证明编译器隐式赋值与其相对顺序时拒绝。只以字段 flags 或 `$assertionsDisabled` 拼写决定会错过这些差异。
3. **守卫与抛错按求值位置证明。** 每条候选必须有开关真时的直通边、开关假时恰一次条件求值、条件真时汇合、条件假时可选消息恰一次求值和唯一 `AssertionError` 构造/抛出；无额外入口、处理器跨越或未归属副作用。若 constructor overload 依赖未知表达式类型或无法保持同一异常/消息语义，拒绝该候选，进而拒绝整组。新增一个必要的 `StmtKind::Assert` 与发射分支，复用现有表达式、类型、来源与预算，不新建第二套控制流。
4. **在类源码装配边界事务性投影。** 现有 bridge 投影能保留物理方法而换类源码文本，接口初始化器投影能保留物理字段/`<clinit>` 而改变其源码位置；本项只扩展这些已有装配接缝。先保留原方法、字段和 `<clinit>` 物理事实及 `RecoveryReport`，在旁路准备省略合成字段显式声明、移去其已证赋值、替换各守卫的完整文本和带 BCI 的投影证明记录。类级正文目前没有独立 source map，因此不伪造投影后 offset；原方法报告的 source map 仍指原恢复文本，类级证明负责把投影与原 BCI 联系起来。全部预算/取消检查通过后发布整类新正文；任何缺口维持原显式呈现。单方法路径永不应用此投影。此操作可能使重新编译的 Java 编译器生成合成字段；验收同时检查其名称、descriptor、flags 与反射行为，而不是仅比较控制台输出。
5. **拒绝证据必须有效。** 冻结 `AssertCore` 的原/JADX/Jarde 三方重编，`-ea`、`-da`、`-ea:AssertCore -da:java.lang.StringBuilder` 与验证器输出；保留 wrong-owner 与非 0/1 两个合法补丁，再构造额外读、额外写、异常处理器改变和不同构造器的 JVM 可验证样本。每例记录 class SHA、关键 BCI、三方源码和运行或编译状态。JADX 是对照，不是源码真值；原 class 的行为与物理事实是准入判据。

## Risks / Trade-offs

- **Java 编译器可能用不同的合成字段名称或插入顺序** → 只在可证明其重建身份与顺序的形状投影；其他编译器形状保持显式输出，不能靠改名伪装。
- **多个断言共享开关，局部成功会丢掉剩余读取** → 同一类所有读取/写入闭包后整组发布。
- **消息 overload 或异常边改变可观察行为** → 以构造器 descriptor、静态类型及异常表/CFG 为硬门槛，未知拒绝。
- **二次发射使预算或来源失配** → 复用现有类级原子暂存与停止报告，独立方法物理证据不变；低预算、取消和 essential/all 对照验收。
