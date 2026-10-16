# Spec Delta

## ADDED Requirements

### Requirement: Reference-overload proof stops before later recovery work

class-source 的引用实参重载证明 SHALL 保留其必需依赖读取与平台缺失检查中实际发生的共享请求预算、取消、期限停止，以及原诊断、用量和可用定位。停止 MUST 在正文恢复前结束后继证明、正文与可选证据工作，保留已完成 raw analysis 和合法发布的成员前缀，MUST NOT 转换为空证明、普通拒绝或由后续停止改写原因。完整读取后的普通拒绝和值级隔离仍遵循既有契约。

#### Scenario: The first overload proof stop ends later proofs

- **WHEN** 公共 class-source 的引用实参重载证明首次在实际收费或检查点观察到共享请求停止
- **THEN** 系统 MUST 保留原停止，结束后继证明、正文及可选证据收费；不得继续到另一次拒付才把请求标为未完整。尚未交付的当前正文 SHALL 为 Stopped/not_produced，text/source map 为空，已完成 analysis 不重跑、不重复计费

#### Scenario: A direct failure retains its original cause

- **WHEN** 实际证明检查或读取直接返回结束本请求的预算、取消或期限错误
- **THEN** 公共报告 MUST 保留原原因、实际维度、原有额度数字与可用位置；Cancelled 不得改写成预算，期限不得变成输出或 IR 额度停止。合法请求 SHALL 返回分析及恢复报告，不能以空证明或无报告裸错误代替

#### Scenario: A dependency stop report remains distinguishable from absence

- **WHEN** 该证明必需的 owner/source 读取或平台缺失检查返回承载共享请求停止的原执行报告
- **THEN** 原 execution、全部原诊断与真实 usage MUST 进入恢复及类汇总；无可用事实不得成为普通缺失，原报告未提供的 limit/consumed/requested/位置不得被编造。预算报告仍保留原维度

#### Scenario: Cancellation and deadline are observed within real proof work

- **WHEN** 初始未取消的 token 在实际证明收费中置位并被真实 checkpoint 观察，或请求真实期限在该证明内首次被观察到
- **THEN** 系统 SHALL 保留原 Cancelled 或期限停止，停止后无后继证明/正文/证据工作及新增收费；停止前已获准用量不退款。入口预取消、sleep 或伪造错误不能替代该验收

#### Scenario: Publication preserves the paid prefix

- **WHEN** 较早成员已合法交付，而当前重载证明在正文恢复前停止
- **THEN** 已交付正文、来源和逐方法结果 SHALL 保留；当前正文不得伪称已产出。当前 stopped record 仅在既有发布费用允许时交付，否则类汇总 MUST 保留原诊断与真实 coverage 缺口；库及 JSON CLI 表达同一停止，CLI 使用既有未完整退出码4

#### Scenario: Complete absence and isolated value failures keep their meaning

- **WHEN** 读取完整但缺失、歧义、访问/descriptor/调用形态不满足原条件，或出现既有值级 Unsupported/decode 不完整
- **THEN** 系统 MUST 保持既有普通拒绝、物理来源和隔离；不得将所有无事实或所有非 Complete 都升级为共享请求停止。真正完整无候选的平台缺失判据 SHALL 保持，不能把读不到 Header 当作平台不存在

#### Scenario: Wide proof preserves overload binding and source behavior

- **WHEN** 宽预算下原有合法引用实参重载证明完成
- **THEN** 系统 SHALL 保持原调用目标、descriptor、必要显式转换、物理方法及 BCI 来源，不增加可接受重载或放宽缺证。受控完整生成源码按原样重编后的固定输入行为 MUST 与原始类一致，不能用结构平面、单一方法片段或原始源码可编译代替生成源码验收
