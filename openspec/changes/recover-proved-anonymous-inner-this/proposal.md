## Why

DT-04 的冻结 Java 8 对照表明，JADX 将匿名类中的外围实例捕获写成 `Inner.this`，而 Jarde 仍暴露物理 `this$0` 字段，并在 `super()` 前发射其构造器赋值，导致完整 class-source 集无法按 Java 8 重编。Jarde 已有 `QualifiedThis` AST 与具名成员捕获证明，但当前匿名 `EnclosingMethod` 关系不能进入该投影路径。

## What Changes

- 为单个、身份闭合的匿名类分配点证明匿名类的 `EnclosingMethod`、词法 owner、匿名构造器中的合成外围引用字段及其精确使用。
- 在已证明的分配点将匿名类体投影进外围方法；只把经证明的外围引用读取写成 `Inner.this`，并在这个完整投影中隐藏 `this$0` 与其构造器序言，不移动或重排物理写入。
- 对捕获局部变量、多分配点、额外合成字段/构造器、外部引用或不完整源码维持物理类表示；不为物理 class-source 输出一般化 Java 构造器重排。
- 以冻结原/JADX/Jarde 完整源码、Java 8 重编、`-Xverify:all` 与拒绝边界验证行为和来源。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：为有完整匿名捕获证据的 class-source 单元呈现词法外围实例接收者和匿名类源码，并在证明不足时保留物理类表示。

## Impact

涉及 `src/member_inner.rs` 与 `src/facade.rs` 的匿名类关系/闭包证明、`crates/jarde-java/src/build.rs` 的现有 `QualifiedThis` handoff，以及 `src/class_source.rs` 的原子家族文本投影。Reader、JVM IR、公开 CLI/schema 和外部依赖不需要改变。DT-06a 的无捕获匿名父类构造实参恢复仍由 `inline-proved-anonymous-super-arguments` 单独负责；本变更的捕获字段只服务词法 `Inner.this`，不解释或改写父类构造实参。
