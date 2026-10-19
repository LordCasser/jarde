## Context

见 proposal.md 的 Why 和已冻结三方基线。真实 Reader→canonical→SSA→NormalFlowView→recover 探针对四个原件 profile 已执行，trace SHA-256 为 cddd66ab28b5c4e16536bc5315bee53e32f7c43f47a91f115db16dfcc4888e74。Loop4 自然集合 {5,11,36}、出口 5→42 和 11→17、ForHeader init3/update36 均正确；外侧 DAG 为 17→23/29、23→29/42、29→42。当前 Frame.loop_body 不准入条件块 17/23；forward_join 可把外侧入口视作空臂汇合，最终自然集合覆盖检查通过，但 recover 的全图扫描仍留下 17/23/29。不得把“有 Loop 区域”误记为完整恢复。

## Goals / Non-Goals

**Goals:** 在已有区域架构内证明并拥有外侧条件体，修正本候选的汇合选择，保留完整真实 owner 和 transfer origin。

**Non-Goals:** 不改变自然循环成员；不普遍放宽 forward_join；不把此 DAG 冒充单块 gateway、终端 return 或含子循环的 NestedLoopSideExit。Loop5 命名 catch 的 prefix+loop+return-tail 是独立组合问题，本片不宣称解决它。

## Decisions

1. 复用现有 Frame 作用域、If/Straight/LoopBreak、for header 和 owner 检查；只补本候选缺少的有界闭合证明。单块 loop_exit_bridge、post-test break prefix 和 nested_loop_side_exit 均有更窄不变量，保留其契约，不能通过删条件借用其名称。可用小型私有证明结果承载精确 branch/entry/stay/exit/owners；无证据不得扩域。是否新增此私有结果由实现最小需要决定，不增加公共类型/AST/分析 pass。
2. 候选绑定当前头测循环、本循环 break_target、自然集合内的条件 branch、唯一外侧入口及留在自然集合内并到达原继续/边界的 sibling。外侧 owner 必须有限、无环、无子循环/switch/return，所有正常路线最终只到本循环出口，且不包含出口本身；逐条核对 canonical 边与已解码比较/转移。当前实现先限定无保护行语境；后续命名 catch 组合若需扩展，须另行提供精确行归属证据，不接受任意异常边。
3. 准入前核对所有入边：外侧入口只由指定 branch 进入，内部节点无外部其他入口；拒绝未解释的非 normal 边、已 visited 节点、父 scope 外节点和共享尾边界。只扩当前候选的 body scope 和最终 expected owners，既不扩自然集合，也不把别的外侧块当已证明。
4. 在广义 forward_join/空臂读取之前，对已证明的 branch 选循环内继续/更新汇合。已有遍历必须实际输出外侧 If 和赋值，真实 conditional break 来源 bci26、goto break 来源33都须核对；禁止把整个分支缩成 branch11 的 break。可复用证明结果，或有预算地重算，但不得以“在 scope 内”代替对应 branch 的证明。
   实际探针还确认：route 内 branch23 的父 arm boundary 为 owner29，不能让全图 postdom42 覆盖这个真实直接后继；此局部优先级只绑定同一证明的 owner/边界。branch11 的外侧 arm 复用 loop_body_sequence 收集完整 If17→赋值29→break33 序列，单次 region_at 只返回首个区域，不能据此宣称路线完整。全图 join 和其它 arm 的遍历契约保持不变。
5. 发布前对所有区域块做多重集检查：无重复，等于原自然循环体加本证明 owners 和原已有证书 owners；并比较 newly_visited 的精确增量。候选失败或真实 Stop 必须恢复所有被试探改变的 visited/unclosed tail/catch joins/终端 owner/fragmented/monitor 状态，预算消耗不回退。每次扫描、集合复制和扩容前 charge，遍历和发布前 poll；包括晚期停止。
6. 仅参考本地 JADX LoopRegionMaker 的多出口和 IfRegionMaker 的空臂处理思路，不复制其源码：其 CFG 和可变区域模型不能直接进入本项目不可变事实/有预算证明架构。使用 Rust 标准集合和现有运行时，无新依赖、无额外授权/维护负担。Reader parse、Java 方言、Runtime 选择、字节码验证与语法恢复保持分层；真实原件可读并不等于已通过 JVM verifier 或语法恢复。

## Risks / Trade-offs

- 空臂仍被启发式选择 → 本证明先于通用 join，完整 owner 多重集和实际 Loop4 语义共同验收。
- 范围扩张吞掉共享尾或父循环 → 双向边闭合、当前目标绑定、scope/visited 拒绝和对抗性边变异。
- 循环正常识别却外侧遗漏 → 原自然集合与新增 owners 分开保存，不能只检查 covers(natural)。
- 计费或取消造成半提交 → 扫描前计费、发布前 poll、完整回滚，并测预取消、预算不足和晚期中止。

## Migration Plan

主 agent 审查 Luna 私有补丁后串行集成。Rust 运行须可用空间至少 5 GiB、target 上限 1 GiB、每秒进程组守卫，运行结束精确清理 target；工具链期间禁止共享修改/Git mutation/网络。定向测试通过后冻结新 CLI，重做完整原件三方对照和已有关键回归，再做全工作区及该产品提交的 CI。完整验收前任务保持未完成，不改既有 71 单元/612 文件分母；失败回到明确 refusal。

## 完整广验：先排除不存在的候选

首轮fresh workspace的CF12旧门表明，一般for+switch在本证明已经可由自然出口数不为2排除时，仍先缴纳全candidate scan费用，使原类型证明内365@29停止点提前。必要条件`natural_exits.len()==2`及二元素集合含出口均可在固定形状检查内完成，必须前置于scan收费；只有真实候选扫描仍按既有上界预付并轮询。此处修正避免不执行的工作收费，不调旧测试预算、不退还已执行工作，也不改变Loop4/5两出口候选的证明或产物。先由Luna私有最小补丁，root独占真实CF12/核心/公开门验收；新产品重新构建、对照和自身CI，首轮失败原流保留。
