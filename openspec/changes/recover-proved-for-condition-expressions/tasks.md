## 1. 真实首拒与对照输入

- [x] 1.1 验收固定TestNestedLoops2基线、同次IR首拒与JADX/架构审阅。验收：root核29命令完整双JDK三方九行行为、实际10节点/12边/28SSA/19trace、outer init3/update41/phi与Invoke@6单branch消费，inner真实拒绝和源码恢复/clean；静态候选、私有原稿与实际运行分开。
- [x] 1.2 冻结调用顺序/初次及后续异常、动态field与array/null/越界、local-boundary写及continue控制的真实Java8基线。验收：原class双JDK编译运行作oracle，当前JADX/Jarde完整类对照和所有失败raw保留；预测不当结果，不改生成源码消除失败。

## 2. 复用条件证明与原有消费

- [x] 2.1 Luna准备私有最小candidate，root审查后扩既有ForHeader条件producer匹配。验收：生产caller已test_is_pure；保留Load角色、init/update/phi/全部CFG和invariant写/外部访问门，非local-writing producer准入与设计相符；不新建purity/IR/pass/扫描，不放开Store/Increment/Copy，不改Builder求值位置。
- [x] 2.2 增加真实fixture/API和必要所有权/effect反例。验收：固定outer一个for/inner while，获证控制准确for、side-effect/throw次数次序同原；local边界写/独立statement/共享effect不能证明时维持可靠拒绝，所有物理BCI/UTF8来源/default-all一致，旧for/foreach/gateway/continue/return-arm控制通过。
- [x] 2.3 核预算/取消及失败传播。验收：真实晚Stop dimension/usage/location可追溯、无部分正文/map/产物，预取消零usage；现有proof计费/poll/状态顺序保持，不以预取消代替中途Stop，也不写无依据的新成本pin。

## 3. 同版本验收与主线交付

- [x] 3.1 root冻结最终CLI并核固定和控制双JDK完整三方。验收：实际115命令/28整类生成runtime成功腿，exit/stdout/stderr同原、循环形态与全部physical来源准确；JADX差异独立记录。最终CLI与已实测candidate-v3逐字节相同，4718项当前闭包与fresh workspace绑定，后续仅3份测试源码和README变化；明确复用同字节真实整类结果，没有在最终路径重复115命令，不借不同旧CLI或局部片段。
- [x] 3.2 完成必要回归、fresh workspace、CI等价Clippy/fmt与全局OpenSpec strict。验收：实际fresh v4 95批/360目标恰好一次，3461/0/97；独立核3558个named outcomes、98命令/196raw、4718闭包与live恒同。fmt/Clippy/strict352/0/finalbuild通过，5GiB free/target1GiB一秒守卫，失败v1/v2/v3全保留，结束clean。
- [x] 3.3 提交推送实际产品并独立核它自身CI。验收：准确product head/run全部jobs/steps/双seed与named outcomes同本地闭包，不借旧CI或仅green状态。
- [x] 3.4 更新verification/固定账本/handoff并交付干净main。验收：实际HEAD=main=origin/main、分支/worktree/target/冻结CLI核准；generic独立债务和CF11/71未完成范围明确，不新增功能分支占用。
