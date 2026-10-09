## Context

动机见 proposal。上一片冻结 CLI SHA `196bb3e1e1bd074bb851f363938951a6d2dea81d8e895cd67e1a2560b3c2f761` 的普通两腿 BigDecimal 仍失败。前片 results 下 `bigdecimal-selected-headers-v1/manifest.json` 固定六次真实渲染：原 Main bytes 不变，仅增加从准确 rt.jar 提取的 BigDecimal.class，现有 snapshot hierarchy walk 即命中其 superclass Number；增加 Number.class 没改变结果。`bigdecimal-selected-headers-execution-v1/manifest.json` 固定完整 Main 生成源的两真实 JDK 空 CP/SP 编译/验证运行，四条显式 header 正例通过、两条 Main-only 负基线仍失败。root verification 逐字核对历史原始双流及 56 文件闭集，不将附加的 platform header 作为生成源码或 runtime classes。

现有 `known_scalar_initializer_reference_widens` 与普通调用入口共享 `platform_interface_argument_widens`。后者 NUMBER_FAMILY 是 release-8 六装箱直接边，显式排除 BigDecimal；所有增长遍历与递归仍用既有 Budget。本片只补有限数据，不建立任意类型可赋值推断。

## Goals / Non-Goals

**Goals:**
- 普通 Main-only 两腿完整恢复及语义通过，不要求使用者为此提供 rt.jar。
- 复用现有闭集 scalar 关系、实际 store 值身份和 ordinary call 渲染，直接边来源可核对。
- 对照覆盖实际生产正负行为，保留所有原始失败和回放分母。

**Non-Goals:**
- 不扩展 BigInteger/atomic Number 子类、Comparable/Serializable 或任意平台类型。
- 不改 concat whitelist；`jre_concat_interleaved_effect` 可作为优化拒绝保留，不能再被当完整正文失败。
- 不改 facade snapshot walk/Stop 债务，不建立 JDK 自动下载、依赖扫描、type service、额外 AST/pass。

## Decisions

1. 只向既有 NUMBER_FAMILY 添加 BigDecimal→Number 直接边，准确注明 Corretto8 `rt.jar` SHA `b27515a608ee447566b688e2bbb2257b1f0d8eceb96c307b87eb28d90a6630f4` 及其真实 class header。更新当前注释与 BigDecimal 旧负断言，但保留历史 evidence 的六装箱原范围。新开独立平台表或任意继承框架会重复现有入口；只要求调用者提供 snapshot header 虽已证明有效，却不能修复账本普通 jar 使用方式。
2. 用原完整 StringBuilder 链实现行为闭环。真实诊断证明 Builder 在 concat plan 缺席时已递归呈现 new/append/toString，显式 `.append((Object)local1[0])` 保留原 overload。v1 静态审计曾建议放行 ArrayLength，v2 与 root 完整运行已给反证，保留两版过程但不实施额外优化。
3. 既有 release-8 gate 和已支持 reference-array lifting/accurate store 来源保持。不同 release/目标/源类型的有限负控采用实际生产报告与 helper 断言；不凭无 runtime facts 的正例宣称任意选定环境/平台版本都能证明关系。旧闭集假设及覆盖面属于共享架构债务，不扩本片。
4. Luna 实现一行关系及必要注释/真实测试，root 串行 Cargo、全源语义与对抗验收。产品新 CLI 必须核对 init/report/build/class_source/Cargo.lock 身份，旧 header 诊断不是新 Main-only 产品验收；不下载依赖。现有 Runtime/Reader/AST/标准库已够用，不新增库或许可成本。

## Risks / Trade-offs

- [误把 concat warning 当正文丢失] → 比较完整正文、source-map、原调用次数及新类运行双流；保留正确的优化拒绝诊断。
- [helper true 掩盖实际 store/descriptor 错误] → 真实 Main-only 两腿和 Number 调用 descriptor 正例，错目标与表外值继续拒绝；物理 aastore15 与构造6/9/12来源核对。
- [有限平台事实被泛化] → 只新增一个精确对，release-8 gate 不变，其余旧负控保留；不承诺其它平台关系。
- [空间影响全仓覆盖] → root 使用既有20GiB停建线、串行Cargo和小调试产物；无法本地双seed时采用确切新产品SHA的真实CI，记录差异，不借前片CI。

## Migration Plan

前片组合产品确切 CI 先通过；完成本片规划并严格验证后再 apply。原始两个 jar/完整 Main 成员不删减，所有旧失败保留。新回放覆盖24腿，目标24/24仍需实际证明；P5 pins仅在实际数据解释后更新。产品提交推送、确切CI通过后验收本片并更新handoff/71账本，清本仓target，保留冻结CLI。没有格式/API迁移。
