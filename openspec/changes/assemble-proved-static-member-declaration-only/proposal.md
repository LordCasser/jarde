## Why

EM-01 的 `SingleAbstract.A` 只有一个合法静态抽象成员类声明，没有根方法构造它。原 class 和固定 JADX 的完整 Java 8 根源码重编运行通过，Jarde 虽能单独读取物理 child，却因现有静态家族证明依赖构造站点而在根源码漏掉 `A`，使源码无法重编。

## What Changes

- 在已有静态成员家族路径中，把准确双向成员关系与声明装配的证明同构造使用点分开；单个无分配点、无字段的静态抽象成员也可恢复到根源码。
- 保留 root/child 各自物理报告及预算、停止边界；无 Code 的合法抽象方法按其声明写出，不伪造成空方法体。
- 固定 Java 8 三方重编、验证运行与缺失/冲突关系、额外 child、泛型 Signature、部分恢复负例；已有带构造站点的静态成员恢复须回归通过。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 允许已证的唯一直接静态抽象成员在没有构造使用点时出现在根类的完整 Java 8 源码中。

## Impact

影响 `jarde` 既有类源码家族证明和最终 writer，不新增 JVM IR、crate、CLI 参数或依赖。[EM-01 审计](../../evidence/java-syntax-2026-09-27/em01-declarations/report.md)把多 child、泛型 child 父接口和接口成员排为后续独立形态。
