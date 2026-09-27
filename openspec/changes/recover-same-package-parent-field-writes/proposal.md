## Why

DT-29 的单 setter 已恢复直接父类 `public` 字段与 Java 8 private accessor，但固定 `TestFieldCast` 组合的 `B.self(Z)V` 仍在 BCI 7/12 拒绝对直接父类 `A` 的 `protected`、包可见字段写入。两条 `putfield` 的 CP owner 均为 `A`，SSA 接收者均为 `B`；现有父类字段证书只给 `public` 发证。原 class 和固定 JADX 的完整源码可按 Java 8 重编运行，Jarde 的这两处仍带 `@bytecode`。

## What Changes

- 在同运行包的 `B extends A` 下，为准确指向 `A` 声明的非 static、非 private `protected` 或包可见字段的 `putfield` 发出现有逐 BCI 父字段证书。
- 继续由现有 field recovery 核对接收者类型与 CP owner/name/descriptor，输出显式 `((A) receiver).field`，在子类隐藏同名字段时仍写入 `A` 的字段。
- 用单 setter 隔离 fixture 对照原 class、固定 JADX、Jarde 的完整 Java 8 源码重编和 `java -Xverify:all`；保留 DT-29 组合中跨对象、泛型接收者及根类方法的独立差距。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：扩展已证明的直接父类实例字段写入至同包 `protected` 和包可见声明。

## Impact

只调整 `src/facade.rs` 中现有父字段证书的准入，并用 `crates/jarde-java/src/field.rs` 及已有 source builder 的 owner cast 消费路径验收。无需新 pass、类层级机制或字段重绑定。固定 JADX checkout 为 `2fb1b16386941660fda07e9017285aec40fcb37f`；组合证据见 `openspec/evidence/java-syntax-2026-09-27/dt29-reference-cast-audit/combined/`。
