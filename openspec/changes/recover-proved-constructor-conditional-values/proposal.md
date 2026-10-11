## Why

固定 JADX `TestSequentialLoops2.asciiToLowerCase` 双 JDK 完整三方对照确认：Jarde 已恢复两个循环和静态数组，但最后条件值中的 `new String` 被拒绝，八份完整生成类均编译失败。真实同次 IR 已确认构造结果只经一个 stack Phi 到唯一 return；当前 Site 忽略无物理 BCI 的 Phi use，误判为没有消费者。

## What Changes

- 对已有构造 Site 增加严格的一跳条件值所有权关系，保留构造身份、实参顺序与既有唯一消费者约束。
- 继续使用现有 Builder 的完整条件表达式证明，不新增 CFG/alias/Phi 遍历机制；只有实际输出中的唯一构造节点才能记为已恢复。
- 摘要、诊断与可选 NewRecord 使用相同的最终呈现判定，保留失败原因和完整物理来源。
- 用固定上游及双臂构造、副作用/异常、共享消费者、错误所有权反例做完整类重编运行对照，验收停止和选择一致性。

### Prerequisites and Non-Goals

前片 for-condition 产品02ecd63ff自身CI38154998619、关闭文档b9ae71783及实际最终clean audit已接受。SequentialLoops2固定基线29命令和同IR诊断1/0/0已接受；该诊断只到init/Sites，不宣称后续Builder已通过。本片先补真实控制基线，再应用候选。foreach、泛型List、循环自然环与continue、一般alias/多层Phi、普通Site旧发布债务、源map既有fallback缺口均单列；不据此关闭整个CF11/71单元。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证明的条件表达式可在分支中构造对象，保持行为、来源和统一呈现证据。

## Impact

仅私有init/Sites所有权、现有最终Program呈现扫描及report消费者、必要fixture/API测试。复用现有SSA、Budget和Builder，保持纯Rust分层；不改公开请求、IR、crate、依赖或主机协议。
