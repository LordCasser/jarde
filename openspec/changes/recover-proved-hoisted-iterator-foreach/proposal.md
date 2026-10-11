## Why

固定 JADX `TestNestedLoops.test(List,List)` 完整三方基线已确认 Jarde 两个 while 与原行为一致，JADX 则呈现两个增强 for。真实同次 IR 定位内层 BCI42 首拒：iterator 空声明属于祖先 region，初始化赋值属于当前列表；当前 helper 只在当前列表找声明，无法继续既有证明。

## What Changes

- 复用 split-aware LocalVariable 和 DeclarationPlacement/at_region 核对 iterator 的完整可见声明 owner，支持已被 planner 提升的声明。
- 保留调用 descriptor、完整 header Phi、唯一消费者和 effect/handler 证明；对额外 None-use 按 block 核对精确 Phi input 次数，只接受已替换且有界解析到同一 iterator 的平凡历史输入，不增加通用 liveness/alias 机制。
- 变换只移除当前列表的已证明初始化赋值，保留祖先空声明；使用已有 Object foreach 头和原 cast/元素绑定。
- 固定完整类、异常及迭代调用顺序控制和错误 owner/slot/消费反例，验收预算、来源、default/all一致性。

### Prerequisites and Non-Goals

构造条件值产品 c956050611f63a06f57caf0b1280238e81faeb80 本地全验证及自身CI已root独立接受，关闭证据提交026de6ae4的实际clean audit亦接受，最终任务文档推送后另按真实HEAD审计；root可在守卫及原字节恢复约束下并行执行下一片临时诊断/只读完整类观察；当前产品关闭交付后再应用永久候选。TestNestedLoops既有基线及同IR-v3首拒证据已接受，当前 owner 及后续 SSA 首拒已实际接受：init/test/next 是同一 LocalVariable，Phi97 的三个输入对应三条 None-use，输出 V22→V12→V37；新所有权门及更后面的 next/effect 证明尚未执行。外层BCI13首条Assign、typed String头、generic Signature/List<String>、跨frame可变AST编辑、死声明清理和一般Iterator语法均独立；不扩大CF11或71/612完成数。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：在已有完整iterator循环证明满足时，消费已提升局部的可见声明事实以恢复内层增强for。

## Impact

主要涉及 jarde-java Builder 的既有 iterable candidate 和最小测试，不改变 planner、SSA/CFG、公开API、crate/依赖或Runtime边界；JADX只作算法和对照参考。
