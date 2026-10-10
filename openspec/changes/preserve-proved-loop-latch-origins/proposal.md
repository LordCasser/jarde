## Why

普通 while 控制基线中 `PlainOneArmLoops.noPrefix(ZI)I` 已输出结构化循环，但双 JDK/default-all 的 source map 均遗漏原始 `goto 6` 的 BCI14。缺口独立于单臂续接：结构化标志不能代替物理指令来源覆盖。

## What Changes

- 已证明普通 header-tested loop 的末尾直线 body 隐式回边，将准确 terminal goto/goto_w BCI 保留在现有 loop origin 字段，经已有 emitter 发布来源。
- 保持正文、正常/异常控制流、physical block owner 与 evidence 请求语义；显式 break/continue、内层循环和非本 loop 目标不得误归来源。
- 以冻结 noPrefix class 的逐方法来源测试、跨层 transfer 反例、预算 Stop 及双 JDK/default-all 实际对照验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 已证普通 header-tested loop 末尾直线隐式 latch 的物理 transfer 来源不能随语法结构吸收而消失。

## Impact

仅涉及 region.rs 成功 loop 构造处的来源登记和定向测试。前提是已有 canonical/SSA、自然循环与完整 body/ownership 证明；复用 `Region::Loop.gateway_origins`、`OriginSet`、emitter 和预算扫描，不增公共 IR、pass、依赖或机制。

不修改单臂续接、loop shape 接受范围或正文；不追求本片覆盖全部嵌套/多 latch/for/do-while 来源形态，也不处理已登记的 exit gateway/update transfer 或其它 Arithmetic 来源债务。原控制完整类仍因另三个单臂方法拒绝而编译失败，不能据本片 method 来源修复声称整类运行通过；后续单臂片最终验收必须再次核全部 BCI。71/612 账本完成数不变。
