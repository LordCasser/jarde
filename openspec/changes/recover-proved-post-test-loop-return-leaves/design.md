## Context

动机见proposal。已接受完整基线由产品412245d51/CLId29b6fee执行。四profile同轮探针完成1 test/0 fail，源码逐字恢复、target absent。DoWhileBreak2真实自然循环header0、成员0/19、latch19、出口17/31，无异常表；0块先next4、checkcast9、store12、比较14，17为aload/areturn，19为hasNext23/回边28，31为循环后返回。公开拒绝为StatementFree@9与uncovered17/19/31。

现有loop_region与latch_tested_loop先要求外侧路径符合post_test_break_prefix_proved的goto合同，后者正确拒绝areturn17。已有loop_terminal_returns可证明该独占返回叶块，Frame::loop_body也已承载terminal_returns，但调用顺序太晚。探针使用Sites::empty，只证明同轮CFG/SSA/Region行为，不能代替公开Builder验收。

## Goals / Non-Goals

**Goals:** 在原后测循环路径中复用已有返回叶块证书，保持body0、return17、latch19和普通exit31各自职责；由公开完整源码与事件trace验证。

**Non-Goals:** 不把checkcast/赋值移入纯测试，不放宽goto桥证书，不强行拼成for/while，不新增IR/Region/CFG转换，不修独立raw Iterator泛型展示或旧映射/runner债务。

## Decisions

- 提前取得并复用同一候选的loop_terminal_returns结果，使header预检和latch外侧路径检查仅跳过确已认证的返回叶块，并把相同集合交给现有body scope与完整owner核对。缓存/参数采用已有集合契约，不为一次传递新造框架。
- 新接纳的头部分支须恰好通向返回叶块与自然latch，并且已有`test_is_pure(..., true)`拒绝该头部。能满足StatementFree的条件（含已有dup/store条件赋值）继续由原头测路径处理。首版实测曾抢占char条件赋值并使旧测试失败，故不能只凭返回叶块证明改变恢复优先级；不以fixture名、BCI或类型硬编码分流。
- 保留goto前缀原有证明及未知边拒绝。单看areturn opcode不足以接受叶块，仍需精确canonical入口、无出口、物理Return操作及父scope约束。不得用笼统“循环外块”绕过证书。
- 惰性取得证书，非候选固定形状先拒绝；不对每个循环无条件新增全图扫描，不重复收费或退款。上轮单出口scan的CF12预算回归是明确反例，旧365@29/130@1测试不可改。
- 标准库BTreeSet与现有Runtime/Reader、循环与region生成器已足够，无新增库/SDK或图算法依赖。JADX的出口与return判断仅作对照参考，独立保持本项目物理证书和预算契约。fixture沿用原JADX Apache LICENSE/NOTICE。
- 解析class、Java展示、runtime选择、VM验证与动态语义分别记账；JDK8 framework55/52不能掩盖Jarde源码失败，也不要求不存在的框架兼容性。

## Risks / Trade-offs

[提前返回被当成break] → 验证独占return17而不是普通exit31，比较null后无hasNext/next事件。

[提前扫描改变旧预算] → 固定形状短路、一次证明复用，保留原精确预算门并完整广验。

[失败后遗留visited/scope] → 复用既有checkpoint，负例、预取消、预算与晚期Stop核对完整状态和不退用量。

## Migration Plan

Luna仅写新私有补丁，root审查后集成；真实fixture与公开API先验，再冻结新CLI重做完整三方及扩展Iterator事件控制，执行旧关键门、完整workspace与本产品自身CI。所有本机Rust串行，默认stack、≥5GiB余量、target≤1GiB、每秒进程组守卫及finally精确clean；工具链运行期间不共享修改/Git mutation/网络/人工并行JDK。未通过全部验收保持tasks未完成，CF09/71/612不扩大声明。
