# correct-proved-legacy-clone-frame-coverage 私有 v2 实现候选

## 产物与 pins

- 候选源码：canonical.rs，SHA-256 eed8f1d731896f0a7a873e2f0d4c44b790d688a8f537ab9fbe98848fb8945869
- Unified patch：canonical.patch，SHA-256 367edadd30d102f3082789081140b43d194c6fa0682a7be84909b9ef4f3bdcb9
- 基线：crates/jarde-jvm/src/canonical.rs，SHA-256 3770e6f0bb295c361d9855545c30a93f33986dcaed626506a97b584c4c26558e
- 本文沿用的架构审计 artifact SHA-256：ca0ab91877534773db9a4d27aaafb61ce3bae0b218760ea94095807288e3e29a

读取的 change 是 openspec/changes/correct-proved-legacy-clone-frame-coverage。planning/spec artifacts SHA-256：

- proposal.md：c8128a1e309ec856db6ce9cc546ec62c8eb291d5b495708017b9c2795c8fa890
- design.md：bc1051d428e6a87d96a9656145c0231bf3d3baf322e309847731fabb252e3057
- tasks.md：cab440343484fbb7365106dd25ce0525b3b1c605f8023dd4d7b2ce30556f5c3d
- specs/jvm-ir/spec.md：979fb325abca91d2320f547e08be049c1726c2c9f72952ddbbaa0598e5f0e866

## 设计落点

候选只改 canonical.rs。assemble 的既有 outgoing 索引仍复用 root-reachability 构建过程，但其值改为借用的 edge refs，使覆盖遍历可按真实 edge 读取目标和类别。既有 owned-ID root walk、无 clone 时的 ordinary CFG 路径、原 unreachable 计算均保留。仅当 clones > 0 时，coverage helper 从所有现有 path=[] IDs 开始前向闭包。

进入闭包之前，所有 graph edges 都校验 call path 关系：Normal 和 Exception 同路径；Call 精确 push 其 call site；Return 精确 pop 与 source path 尾部相同的 call site。闭包只能沿这些真实边传播，故内部普通/异常边和嵌套 Return continuation 可覆盖，断开的 clone/self-cycle 及错误路径不能自证。全部现存空路径节点继续作为 seed，因此显式 dead callsite 保持原 unreachable truth。

coverage helper 借用 ID 存放 seen/worklist，不复制 path Vec。每个新 ID 分别在 seen 和 worklist 增长前收费 IrItems；每次边形状验证先按固定检查成本与两端 path 长度收费 AnalysisSteps；edge 遍历、worklist pop、seed 扫描、最终 clone 扫描逐项收费。Assembly checkpoint 在 helper pop 时复用现有取消 seam。预算与取消均以原 Norm::Halted 传播，不转换为图缺陷。移除的是 postcondition 中无预算的 clone 直接入边检查；来源、edge 端点、throw site、handler row 和 exception ordinal 检查保留，并在 postcondition 文档中说明 clone coverage 已由 budgeted assembly 完成。

## 候选新增测试

候选新增九个测试。正常图由 synthetic MethodCodeFacts 经既有 raw/context/normalize 流程构造；canonical graph mutation 只用于对抗覆盖拒绝，不称为 JVM 合法 classfile。

1. 两个调用点共享含条件分支的子程序，两个 clone path 各自覆盖分支内部块；fixture 指令连续覆盖至 ret leader。
2. 移除 clone 内部块入边但不加环，验证单纯断开会拒绝。
3. 在同一断开节点加自环，验证 self-cycle 不能自证覆盖。
4. 分别加入错 path Normal、改错 Call site、改错 Return site 和错 path Exception，验证边路径关系拒绝。
5. 已覆盖的 nested clone Return 到正确 outer continuation，验证该 continuation 可继续覆盖。
6. 删除 nested Call、留下其 Return，验证未覆盖的 Return source 不会覆盖 outer continuation。
7. 显式 dead path=[] callsite seed 仍保留及列入 unreachable。
8. 使用 synthetic proved handler facts：jsr 0→4；return@3；astore@4；aconst_null@5；athrow@6；handler astore@7；ret@8，异常表 [4,7)→7。经真实 raw/context/normalize 后验证 clone path [0] 的异常边进入 handler clone path [0]、ordinal 0，handler ret 返回 caller continuation BCI 3，并验证 throw-site 来源 path 与 handler row 的 clone protected source。该用例明确不宣称是 JVM 已验证 class。
9. 从 helper 对完整覆盖测量实际 usage，再分别以 IrItems 和 AnalysisSteps 的完整需求减一运行并检查真实 BudgetExceeded dimension/consumed/limit/requested；Assembly seam 在 helper 已产生收费后取消，检查 usage 和 Norm::Halted(Cancelled)；既有完整 canonical assembly 取消断言仍保留。

Unified patch 已用私有脚本按 hunk 应用回基线，并逐行验证重建字节等于候选源码。未运行测试，因此候选不是测试通过证据；expected diagnostic 和新增 handler fixture 都仍需 root 在仓库应用后运行验证。

## 执行边界

遵循只生成私有候选的授权：没有写仓库，没有运行 Git、Cargo、JDK、CLI 或测试。没有重新验证完整 class Reader/Frame/SSA/region 集成回归。OpenSpec apply-change 技能说明依赖 openspec CLI；本任务禁止 CLI，因此直接读取四个 artifacts，不运行该 CLI。历史双 JDK 验证视为 root 的既有实测记录，不表述为用户确认。
