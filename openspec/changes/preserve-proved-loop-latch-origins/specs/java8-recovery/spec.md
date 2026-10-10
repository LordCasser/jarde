## ADDED Requirements

### Requirement: Implicit straight-tail loop transfers retain physical provenance

当普通 header-tested loop 已满足既有结构恢复与所有权证明，其 body 末尾直线区域的终端正常 goto/goto_w 唯一转至该循环 header，且该 transfer 由循环结构隐式承载时，系统 SHALL 将该物理指令的来源保留在实际循环源码的非空 span 上。该来源 MUST 指向原物理方法及准确 BCI；MUST NOT 改变正文或制造显式控制流，也不能以 structured 标志代替来源覆盖。

#### Scenario: Simple while latch absorbed by loop structure

- **WHEN** `noPrefix(boolean,int)` 的原始 BCI14 为 `goto 6`，已证明的 while body 末尾隐式回至 header6
- **THEN** 生成正文 SHALL 保持现有 while 及条件外 return，BCI14 SHALL 映射到该 while 的实际非空 span；所有已有来源和物理方法身份 SHALL 保留

#### Scenario: Different loop target or explicit transfer

- **WHEN** transfer 指向另一循环、已由显式 break/continue 呈现、处于内层循环，或未满足末尾直线 latch 的准确正常边/所有权条件
- **THEN** 系统 MUST NOT 将其错误地登记为当前循环的隐式回边；已有显式语句来源及准确拒绝 SHALL 保留

#### Scenario: Evidence requests preserve presentation

- **WHEN** 同一物理方法按默认与完整 evidence 请求恢复
- **THEN** 正文与 source map SHALL 一致；新隐式回边来源 MUST 不依赖可选 region/rule/name/read evidence 是否请求

#### Scenario: Source proof interrupted

- **WHEN** 回边来源核验或发射遇预算、取消或既有停止条件
- **THEN** 系统 MUST 遵循既有 Stop 契约，保持停止 outcome 和空正文/source map，不将停止降为普通 shape refusal 或完整成功
