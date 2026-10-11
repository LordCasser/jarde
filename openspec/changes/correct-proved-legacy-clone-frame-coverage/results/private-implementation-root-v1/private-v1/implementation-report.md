# correct-proved-legacy-clone-frame-coverage 私有实现候选

## 产物

- 候选源码：canonical.rs
- Unified patch：canonical.patch
- 基线：crates/jarde-jvm/src/canonical.rs SHA-256 `3770e6f0bb295c361d9855545c30a93f33986dcaed626506a97b584c4c26558e`
- 候选源码 SHA-256：`88707b54f8ca724f10af000411df2dd9a7fba49f993352d41261a37280362a51`
- Patch SHA-256：`0e357268b983443eb6070d962533eb7070ec9808ed3fc39f96c2177da9e3de67`

## OpenSpec 输入 pins

本次读取的 change 为 `openspec/changes/correct-proved-legacy-clone-frame-coverage`。四个 planning/spec artifacts 的 SHA-256：

- proposal.md：`c8128a1e309ec856db6ce9cc546ec62c8eb291d5b495708017b9c2795c8fa890`
- design.md：`bc1051d428e6a87d96a9656145c0231bf3d3baf322e309847731fabb252e3057`
- tasks.md：`cab440343484fbb7365106dd25ce0525b3b1c605f8023dd4d7b2ce30556f5c3d`
- specs/jvm-ir/spec.md：`979fb325abca91d2320f547e08be049c1726c2c9f72952ddbbaa0598e5f0e866`

架构审计 result artifact SHA-256：`ca0ab91877534773db9a4d27aaafb61ce3bae0b218760ea94095807288e3e29a`。

## 候选内容

候选只改 canonical.rs。assemble 的既有 outgoing 索引改为保存借用的 CanonicalEdge，方法 root reachability 仍从原 root 计算 unreachable，仍只在 clones > 0 时运行新增覆盖。覆盖从所有现存 path=[] ID 起步；所有边先校验准确的 call-path 变化，再从已覆盖 source 沿真实 edge 前向传播。Normal/Exception 保持路径，Call 精确 push 其 call_site，Return 精确 pop 对应 call_site。clone ID 未进入闭包则拒绝。Return continuation 只能从已经覆盖的 nested source 到达。

seen/worklist 持有借用 ID，避免复制 path Vec。每个新 seen 槽和 worklist 槽分别在增长前各收一个 IrItems；边形状检查、pop、edge 遍历和 clone 完整性检查收 AnalysisSteps。预算错误与取消通过 Norm::Halted 原样传播。直接 clone 入边检查从无 Budget 的 postcondition 移除；来源、edge 端点、throw site、handler row、异常 ordinal 检查不变。普通图跳过 clone coverage。

## 新增测试覆盖

候选新增七个测试，均以 synthetic MethodCodeFacts 创建正常规范化图，再在 canonical graph 层做 mutation；mutation 用例明确标为发布覆盖层的对抗图，不宣称其是 JVM 合法 classfile。

- 两个调用点共享含条件分支的子程序，两条 clone path 各自保留分支内部块。
- 删除一条内部入边并给 clone node 加自环，验证断开自循环不能自证覆盖。
- 分别加入错 path 的 Normal edge、改错 Call site、改错 Return site，验证全图边路径形状检查拒绝。
- 从 nested clone 删除 Call edge但保留其 Return，验证未覆盖 Return source 不能覆盖 outer continuation。
- 正向调用已有 nested-call 合成事实，验证合法 Return 到匹配 outer continuation及后续覆盖。
- 通过 goto 跳过一个合法 jsr 的 synthetic facts，验证显式 path=[] dead callsite seed 和其 clone 保留在 graph，且仍列于 unreachable。
- 对覆盖 helper 分别设置 IrItems/AnalysisSteps 不足并断言具体 Error dimension/usage；通过既有 Assembly cancellation seam 断言返回 Cancelled、不返回 CanonicalOutcome graph。

Unified patch 已在内存中按 hunk 对基线重建，结果与候选 canonical.rs 字节一致。现有 shared/nested/历史 handler、fusion、预算和取消测试仍在候选源码中；未运行测试，不能将静态候选当作测试通过证据。

## 边界与限制

遵循委派约束，只读取 change/source 并在 /private/tmp 下生成候选和 patch；没有写仓库，没有运行 Git、Cargo、JDK 或 CLI。因而没有运行 OpenSpec status/instructions、rustfmt、cargo test，也未验证完整 class Reader/Frame/SSA/region 回归。root 按要求独立审阅后再应用和执行验证。OpenSpec apply-change 技能说明原本依赖 openspec CLI；本任务明确禁止 CLI，因此没有调用它，直接读取其四个 artifacts。
