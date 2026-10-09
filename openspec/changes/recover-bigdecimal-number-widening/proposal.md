## Why

EM18 账本剩余两条 BigDecimal 数组输入，构造器已经恢复，却因 Main-only snapshot 没有 BigDecimal→Number 事实而拒绝整个方法。root 将真实 Corretto8 BigDecimal header 提供给同一快照后，现有路径完整恢复 Main；四条显式 header 诊断腿在真实 JDK8/23 下独立重编和运行均匹配原始双流，说明缺口是有限平台事实，不需要另造 constructor 或 concat 机制。

## What Changes

- 在既有 release-8 平台 reference widening 闭集中，仅补 `java.math.BigDecimal`→`java.lang.Number` 的准确直接边，复用现有调用实参与数组元素赋值入口。
- 保留实际构造/ValueId/store BCI、既有 budget 与完整方法提交；普通 Main-only 两条输入须完整恢复并通过双 JDK 全源语义对照。
- concat 优化仍可保守拒绝；现有普通 StringBuilder 调用链必须保留实际求值顺序、次数和来源，不调整 concat 白名单。
- 保留六装箱原正例、不同 release/目标/源类型负例及所有旧回放分母；记录已有共享类型事实债务，不混入本片。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：release-8 平台 BigDecimal 值可向 Number 调用形参及数组 initializer component 呈现。

## Impact

仅既有 `crates/jarde-java/src/build.rs` 平台事实与实际生产测试、完整 fixture/对照和验收证据；不新增 pass、AST、type service、依赖或 CLI 选项。前置组合产品 `45b4848c558f4f5d317720535cea32fe431288bf` 及确切 CI 必须先验收，产品冻结期间只完成本片规划。JADX 历史两腿语义通过是比较基线，原始程序仍是 oracle；71 单元及 EM18 整单元分类不因本窄片自动完成。
