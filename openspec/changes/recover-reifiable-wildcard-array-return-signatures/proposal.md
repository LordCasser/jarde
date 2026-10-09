## Why

子数组片的真实六类双JDK回放已恢复三个grid正文并通过原始输出对照，但`collectionGridDirect`仍把原始`Collection<?>[][]`声明降为擦除形式并保留Signature拒绝。根因是既有泛型返回候选不接受完整`NewArray`，不是数组Signature解析失败；须独立闭合这个已定位的呈现接缝。

## What Changes

- 完整、无形式参数的单return数组正文，使用同次Program/SSA的实际创建类型和物理来源提供窄泛型返回证据。
- 当原始Signature数组的叶类型全部为无界`?`、维数及擦除与实际数组完全一致时，保留原始参数化返回声明。
- 有界wildcard、具体类型实参、类型变量、不同rank/leaf、缺失正文或方法参数变化仍保留拒绝，不能借原class编译。
- 完整six-class双真实JDK回放、Signature负控、来源/次数/停止和全仓CI独立验收；回验子数组片挂起的3.1。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：恢复已证明的无界wildcard数组返回声明，不把擦除正文可运行等同泛型声明已恢复。

## Impact

既有`crates/jarde-java/src/report.rs`的`GenericReturnCandidate`交接、`src/class_source.rs`声明投影及focused测试。候选枚举可增加一个必要的数组形状分支；不新增pass、AST、类型服务、JDK关系表、crate或依赖。

## Prerequisites and Non-Goals

依赖子数组片已验收的2.1/2.2及冻结candidate-root-v4：完整家族双JDK运行2/2、旧22腿20/22、数值2/2，但Signature仍拒绝；这些不是本片成功。子数组原规划不扩大到一般泛型，其3.1/3.2/3.3保持未完成，直到实际所需证据闭合。排除一般泛型推导、参数读写转换、泛型数组创建文本改写、嵌套成员类型、bounded/具体实参、BigDecimal及其它已登记债务。
