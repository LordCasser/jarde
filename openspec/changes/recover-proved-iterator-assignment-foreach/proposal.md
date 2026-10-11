## Why

固定 JADX `TestNestedLoops` 中，Jarde 已恢复内层增强 for，外层仍是 while：元素局部在循环外声明，首句因此是 `Assign(Cast(next()))`，而现有投影入口只接受初始化声明。root 同次 IR 实测已确认，借用这条现成赋值的 RHS 后，全部既有 iterator proof 与来源/删除计划均通过；无需新增证明机制。

## What Changes

- 在既有 iterable foreach 投影中接受首句局部赋值，但本片仅支持 inline `Cast(next())`。
- 保留原赋值目标、声明位置、强转、物理来源及循环体，只把唯一 next 表达式替换为已有 Object foreach 合成元素读取。
- 保持所有 iterator 身份、声明所有权、Phi/use、调用、间隔和异常证明；拒绝形态和真实 Stop 维持原子性。
- 用固定完整类和赋值变量在循环外可观察的控制，验证正文、来源、调用次数、异常顺序及双 JDK 重编执行。

### Prerequisites and Non-Goals

基底产品 `993e9f8c0f413bb7c31538e6f7c2c92a742e7af4` 已提交推送，本地完整验收已接受；自身 CI 与最终2246f6719关闭主线审计已实际接受。当前片在该断点之后应用，定向回归已真实通过；完整自身产品交付仍按tasks逐项验收。临时诊断不曾发布改写，4732 输入按字节恢复且 target 已清理。

不处理无强转 Assign、typed String 头、泛型 Signature、声明清理、旧同列表 literal gap 时序债务、任意调用 iterable 或一般 iterator 语法；不关闭整个 CF-11，不增加 71 单元 / 612 文件完成数。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：完整 iterator 证明成立时，循环首句元素局部赋值不再阻止保留绑定的增强 for 投影。

## Impact

仅既有 jarde-java Builder 的 iterable candidate 及必要测试/fixture。无新 IR、pass、变量身份服务、类型推断、公开 API、依赖或 crate。JADX wrapped cast 路径作为算法参考，其可变泛型类型修订不作为本仓证明。
