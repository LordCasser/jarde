## Context

[巡查证据](../../evidence/java-syntax-2026-10-04/bridge-method-patrol/README.md)：BR 家族五类，两门各自拒绝（`BR$Base` 门 1、`BR$StrBox`/`BR$Impl` 门 2），javac 实测 name-clash 报错两份存档（`results/BR$Base.java`、`results/StrBox.java`）。既有机制与判据位已定位：`src/facade.rs:29499`（擦除返回门，硬编码 `bridge_return != b"Ljava/lang/Object;"`）、`crates/jarde-java/src/bridge.rs`（体形门，`jre_bridge_cast_not_erasure`；其模块文档已声明"参数 cast 不是本形状"）。复用对象：`recover-snapshot-hierarchy-widening` 的快照 header 层级 walk（同快照双物理类、深度上界、A16 计费）。**第一个取证义务**：读两门的完整判据上下文，确认 (a) 门 1 换成层级 walk 后是否影响既有"继承需求"判定的输入；(b) 门 2 的参数 cast 形在 javac 重编时是否**逐字节可重建**（决定"隐藏忠实"的证明强度）。

## Goals / Non-Goals

**Goals:** 两门扩展到真实高频形；BR 三类可重编、行为一致；既有桥投影正例/负例（`negative/`、`orphan/`）逐字不变。**Non-Goals:** `project-proved-bridge-forwards` 自身未勾项（2.4 计费取消、3.2/3.3 root 复核——独立债务，不在本片代办）；`Outer.super` 桥（`project-proved-outer-super-bridges` 域）；效果不纯的桥（既有拒绝保持）；桥的**改名呈现**（JADX 路线，语义不忠实，既有决策已否）。

## Decisions

1. **门 1：擦除返回子类型证明**（复用层级 walk）——`target_return` 是 `bridge_return` 的已证子类型（快照内 extends/implements 链可达）即准入；替换硬编码 `Object` 要求。单边快照外或链不可达保持现拒绝文本。
2. **门 2：参数 cast 属规范擦除形**——准入判据从"仅返回值 cast"扩为"参数按位 cast 到源级参数类型 + 转发到同类源级目标 + 可选返回值 cast"，即 javac 为泛型特化发射的规范形。**cast 不消除**（可抛 CCE，语义不可丢）；忠实性来自"成员被投影隐藏、javac 重编时按源级覆写+继承契约重建同一桥"——故门 2 的证明强度是**可重建性**：cast 目标必须恰为源级方法对应参数类型，否则拒绝。
3. **两门共用既有其余准入**（bridge flag、同次 `bridge@1` 证明、同类唯一源级目标、继承需求、效果不纯即拒）不动；隐藏方式复用既有投影通道（`stage_class_source_bridge_projections`），不新增呈现机制。

## Risks / Trade-offs

- **门 1 放宽误纳非协变形**（源返回与桥返回无层级关系）→ walk 不可达即拒；负例钉死（`Object` 返回但源返回不相关类型）。
- **门 2 放宽误纳真实 cast 语义**（cast 目标≠源级参数类型，即桥做了额外收窄）→ 判据要求逐位相等；负例钉死。
- **隐藏后调用位失真**（外部经擦除签名调用桥）→ 重编后 javac 重建同桥，调用位仍解析；行为由三方 `-Xverify:all` 对照钉死（含经接口引用的调用路径）。
