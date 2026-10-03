## Why

[桥方法呈现巡查](../../evidence/java-syntax-2026-10-04/bridge-method-patrol/README.md)在既有 `project-proved-bridge-forwards`（桥投影机制已合入：`prove_class_source_bridges`/`stage_class_source_bridge_projections`）之上实锤**两道过窄准入门**，两门各自导致同一硬编译错误——桥以成员声明呈现，javac 报 "已在类 X 中定义了方法"（仅返回类型不同的同名同参方法在源码层非法）：

1. **擦除返回门只收 `Ljava/lang/Object;`**（`src/facade.rs:29499`）：覆写**接口方法**的协变桥擦除返回是接口类型（`BR$Base.next()` → `LBR$Node;`），被拒 `the source return type is not a proved covariant subtype of the erased Object return`。这是真实代码最常见的协变形（`Comparable<T>`、容器子类、Builder 覆写接口）。
2. **体形门只收返回值 cast**（`bridge.rs` 的 `jre_bridge_cast_not_erasure`）：泛型擦除桥须先 cast **参数**（`compareTo(Object)` → `checkcast Impl` → forward），被拒 `the bridge casts a parameter … before it forwards`。

两门拒绝后桥保留为完整声明 → 整类不可重编（实测 `BR$Base`/`BR$StrBox` 双双报错）。

## What Changes

- **门 1 扩展**：擦除返回不再要求恰为 `Object`——接受"源返回类型是桥擦除返回的已证子类型"，判据复用 `recover-snapshot-hierarchy-widening` 的快照 header 层级 walk（同快照双物理类；链不可达/单边快照外保持拒绝）。
- **门 2 扩展**：体形接受**参数 cast 转发**——cast 目标等于源级参数类型（擦除宽型→源窄型，正是 javac 为该桥发射的形态）且转发到同类源级目标；参数 cast **不消除**（它可以抛 ClassCastException，语义不可丢），但因成员被投影隐藏而无需呈现。
- 既有准入条件（bridge flag、同次 `bridge@1` 转发证明、同类唯一源级目标、继承需求、可重建擦除签名、效果不纯即拒）全部保持；`project-proved-bridge-forwards` 未勾的 2.4/3.2/3.3（计费/取消与 root 复核）属该 change 自身债务，不在本片范围。
- BR 家族三类（协变接口覆写、泛型容器特化、`Comparable<T>` 实现）可重编且行为一致；既有桥投影正例与负例（`negative/`、`orphan/`）逐字不变。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：接口类型擦除返回与参数 cast 形的编译器桥可被投影隐藏，含协变/泛型特化的类可重编。

## Impact

`src/facade.rs`（擦除返回门）与 `crates/jarde-java/src/bridge.rs`（体形门）及测试；门 1 复用快照层级 walk，门 2 扩展既有体形判据，均无新机制。既有桥投影通道、access$/lambda 消隐零回退。
