## Context

以 P1/P2/P3 完成为进入条件。本 change 规划消费 P1 的物理/运行视图与 query API、P2 的 Resolver/IR、P3 的 recovery/source map；当前尚未实现。现代结构可在 P0/P1 逐步被读取，但本 change 负责把 module、nestmate、condy、record/sealed、现代 concat、RuntimeMatrix 和 X2/X3 语义深度纳入可验证契约。

## Goals / Non-Goals

**Goals:**

- 用 release-bound registry 统一现代 classfile 合法性、preview 和 output-level 诊断。
- 以显式 RuntimeMatrix/LoadDomain 支持跨版本、模块和 loader 的 dispatch 查询。
- 以有界 pattern 和 versioned plugin 扩展 framework/resource 查询，保持 evidence/coverage。

**Non-Goals:**

- 不承诺所有 Java 27 源码恢复，也不把结构扫描等同于完整现代 decompiler。
- 不执行 bootstrap、reflection、launcher、JNI 或网络资源；不把 Unknown 变成猜测。
- 不回写 P1 X1 原始 edges，P4 产生的派生语义始终可区分。

## Decisions

1. **Registry 绑定 release，而非单一允许 major。** 为每个版本记录 tag/attribute/flag/opcode 约束和 preview 条件；相比只提高 major 上限，能避免错误承诺合法性。
2. **RuntimeMatrix 批量共享 PhysicalView。** 物理扫描与 profile 选择分离，选择函数按区间可证明时才合并；相比每个 profile 重扫归档，结果更一致且可解释。
3. **X2/X3 显式假设。** X2 返回 KnownCandidates/OpenWorld，X3 仅在有界常量传播和注册规则下给 pattern_inferred_target；相比单一 confidence 数字，证据和假设能复核。
4. **插件仅做派生 facts。** plugin descriptor 包含 schema/rule/evidence/coverage，且不改变核心 X1 事实；相比把框架字符串塞进通用 scanner，边界与回归更清晰。
5. **输出级别冲突先降级。** record/sealed/modern concat 在 Java 8 output level 下返回冲突和 fallback，而不是隐藏语法转换；源映射仍保留原现代 origin。

## Risks / Trade-offs

- [Risk] registry 与 JDK release 演进速度不一致 → 每条能力独立登记、fixture 驱动，未知 release 保守 partial。
- [Risk] RuntimeMatrix/loader 组合爆炸 → 共享 physical scan、显式 profile 上限和选择函数压缩，仅在稳定区间合并。
- [Risk] X3 模式误报动态目标 → 强制输入常量/传播/loader 假设和 Unknown，拒绝开放式猜测。
- [Risk] plugin 规则污染核心结果 → schema/rule version、独立 coverage 和资源预算；未来隔离另立 change。

## Migration Plan

以 P1/P2/P3 结果契约为输入；modern facts、RuntimeMatrix 和 plugin result 是本 change 规划的扩展。若契约需要改变，必须通过 OpenSpec 明确修订，不作旧接口持续兼容的承诺。为每个现代特性增加独立 registry/fixture 后再进入 release matrix；P5 可在被选优化阶段的基准和正确性稳定后评估优化。
