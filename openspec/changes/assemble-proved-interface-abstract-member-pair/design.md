## Context

固定 JADX HEAD 为 `2fb1b16386941660fda07e9017285aec40fcb37f`；现有 [EM-01 `multi` 回放](../../evidence/java-syntax-2026-09-27/em01-declarations/replay.py)表明原/JADX Java 8 根源码运行同为 `2:1` 和 `1:java.lang.Comparable<em01.Generic$A<T>>`，Jarde 缺 `Shape.I`、`Shape.A`、`Generic.A`。先只隔离 `Shape`，避免让另一根 `Generic` 的 Signature/bridge 缺口决定本任务成败。

`Shape.class` 的两条直接 `InnerClasses` row 分别为 `Shape$I` flags `0x0609` 和 `Shape$A` flags `0x0409`，双方 child 均无 `EnclosingMethod`。`I` 的物理 class flags `0x0601`，无构造器、字段、接口或 Signature，两个 public abstract 方法无 Code；`A` 的物理 flags `0x0421`，无字段、接口或 Signature，只有默认构造器与一个无 Code 抽象方法。现有 `src/member_inner.rs::scan_family_root` 的普通类 forbidden flags 拒绝 `I`，且单个 static candidate 限制拒绝 pair；`ClassSourceMemberFamily::Prepared` 只保存一个 relation/child。`A` 单独时已由 `assemble-proved-static-member-declaration-only` 验收。`Generic.A` 还需要类/字段/方法泛型 Signature、Comparable 自引用及 bridge 证明，不能随意放宽当前字段/Signature 门。

## Goals / Non-Goals

**Goals:** 同一选定环境的两条准确成员关系和完整物理声明被联合证明后，在根源码原子写出合法的 `I` 与 `A`，保留 child 物理报告及每个声明的来源；不因接口没有构造器而伪造使用点。

**Non-Goals:** 任意个 child、非静态成员、带正文/字段/注解/Signature 的接口或抽象类、根方法中的成员构造/引用改写、嵌套层级、`Generic.A` 或通用成员拓扑算法。

## Decisions

1. **复用一轮扫描及关系证明，承载受证的集合。** 在现有 root `InnerClasses` 扫描中保留准确物理顺序、child 名称、flags 和唯一性；只在恰好一条接口与一条抽象类直接 row 的形态下产生私有双成员候选。每个 child 仍由现有选定环境读取并用 `child_relation_agrees` 核自述 row、无 `EnclosingMethod`。不从 `$` 名猜成员、不另扫整 jar、不把一个失败 child 忽略后投影另一个。若现有单 child 家族记录不足以表达二者，可在同一家族报告/准备路径中引入明确集合承载；拒绝时在家族报告保留已物理准备的 child，未读或缺失 child 仍可按其物理 identity 独立查询，不伪称已准备。不要并列一套脱离物理 family 的根声明捷径。
2. **分别闭合两个声明。** `A` 复用现有 `static_declaration_only_shape` 与无构造站点证书；`I` 要求 Java 8 合法的 public static abstract interface relation、匹配物理 interface/abstract class flags、空字段/Signature/超接口、零构造器及完整无 Code 的 public abstract 方法表。接口方法按 Java 语法写分号声明，绝不补空方法体。本固定证书的根仅含无成员使用的默认构造器；根/child 的其它字段、方法、使用点或未知成员使候选拒绝，物理报告仍可查询。
3. **根源码一次提交。** 在已有 class-source family checkpoint 内先准备两份 child 文本、头、来源、预算，再把两个嵌套声明一次性装入根 text；任何关系、声明、输出预算或取消失败都不发布半组。根声明顺序沿已证 `InnerClasses`/物理顺序，不用名称排序猜词法顺序。由 relation 与物理 method identity 标记派生来源，不复用 child BCI 充当根方法 BCI；单 child 构造型与静态抽象型路径保持行为。
4. **以完整源码与有效拒绝验收。** 冻结 `Shape` 独立完整类、外部 Runner、原/JADX/Jarde 源码及 class SHA；Java 8 重编、`-Xverify:all` 运行和反射方法数一致。构造 verifier/解析有效的缺 child、冲突 self row、额外直接 child、错误接口方法 Code/flags、额外字段/Signature、根未证使用与预算/取消近邻。拒绝时可保留两份物理 child 报告，但不发布仅 `I` 或仅 `A` 的半份根声明。

## Risks / Trade-offs

- **第二 child 被忽略，根源码似乎完整却少一个声明** → scan 和 source writer 共用同一受证集合，要求两 child 完整且原子提交。
- **把接口当普通 class 或给抽象方法写 body** → 按 flags/方法表分别证明接口，使用已有声明 emitter 的 interface/无 Code 分支。
- **泛型 `Generic.A` 被误放行** → pair 证书只限无 Signature、无字段的 `Shape` 类型形态，保持现有泛型拒绝。
- **家族模型改动回归已有单 child** → 固定 `SingleAbstract.A`、构造型 static member、嵌套 enum/annotation 及预算/取消回归。
