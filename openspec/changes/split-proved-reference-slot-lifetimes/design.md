## Context

见 [proposal](proposal.md)。`report.rs` 先恢复 Region，再调用 `reuse::plan`，随后建立名称与声明。`build::decide_types` 按 `LocalVariable` 的首写定型；目前普通引用间的槽复用仍会共用一个身份，因此后段可能沿用 `int[]`、String 或旧容器类型。

现有 `typed_split` 已证 Ref/Int 两段与特殊 monitor 清理；`array_retype_split` 已按写入定义、唯一读取 owner 和旧值使用界限分数组段。二者已使用 `Plan::variable_at`、逐段 `SlotEvidence::Split` 和名称碰撞处理。问题是引用间覆盖不足，不是解析失败，也不是 JVM 验证失败。

JADX 固定源码为 `2fb1b16386941660fda07e9017285aec40fcb37f`。`InitCodeVariables` 为 SSA 建 CodeVar，phi 连通分量共用身份；`ProcessVariables` 根据 assign/use 放声明。借鉴值身份先于名称的算法原则，复用本仓同一方法 SSA/CFG；不复制其方法头兜底或 debug-name 合并，也不移植 Dex 可变 IR。JADX 只作参考和对照工具，本片无代码搬运、外部依赖或许可变更。

## Goals / Non-Goals

**Goals:** 八类十六个 no-debug 输入完整重编/验证运行闭环；类型事实与生命周期事实各自有来源，既有声明规划自然消费分段身份。

**Non-Goals:** LVT 同名重用、通用引用可赋值性、null/未知类型反推、alias/非平凡 phi、异常/call-context/克隆块/不可达块、参数与资源头部、完整 LG 或 EM-20 全量完成。

## Decisions

1. **先复用数组分段所有者。** 优先把已有 `array_retype_split` 的定义→读取 owner/使用界限逻辑扩展为普通引用类型分段；不要平行复制整段证明。已有数组路径保持准入与结果，普通引用扩展增加自己的正常 CFG 条件。若复用需要有限的私有类型读取 helper，只暴露已有事实读法，不新增类型服务、pass 或 IR。
2. **类型只读已有直接事实。** 复用 `written_type` 的数组通道、直接常量和明确 named frame 类型；它对 unknown/null 兜底的 `Object` 不是本规则的证据。至少两段完整拼写不同才触发，全部同型写入保持原规则。类型不同不等于 Java 不可赋值；这里只利用已证明的独立、死亡的值链，因此不要求外部继承推断。
3. **唯一 owner 与无跨段使用是核心。** store 顺序只是候选边界，每个 local read 的 SSA 代表值必须有且只有一个该槽定义 owner。平凡 replacement 有界跟随；跨段或非平凡 phi、同值多 store 的歧义、旧值留栈到后段、定义/读取冲突均不分段。每个段使用现有独立 LocalVariable，消费锚点沿 variable_at 的映射，不能另取当前 slot 的新值。 store值的直接uses不足以证明栈上旧值死亡；普通引用规则沿现有load/store/dup/checkcast的SSA身份转发检查派生栈值，仅已有replacement证明的平凡phi继续，无法证明的栈复制拒绝。栈槽编号使用实际SSA读写位置，不把Stack0当作统一栈顶；全部同类型写在类型循环内直接退出。
4. **普通引用扩展补足 CFG 证据。** 只接受完整正常图，拒绝 handler/call-context、clone 与不可达事实。对每个切点证明后段正常路径不能回到前段访问；同一基本块内的先后访问可由指令位置确认，不能因起点块与早段块相同误拒，但任何后续边回到该块必须拒绝。不能仅按 BCI 拆分跨分支/循环值。本片不改变已有数组/monitor 证明的范围。
5. **发布和计费沿现有入口。** 临时计划完整后才交 NameTable/Declarations；所有新增扫描、代表值跟随和可达性遍历按现有 AnalysisSteps/IrItems 纪律计费与轮询。预算计数移动须真实解释，不为了通过指纹而省略计费；不追加无关 cache 或 fixpoint。

## Risks / Trade-offs

- [类型兜底被当作精确来源] → unknown/null、不能拼写的 descriptor 保留，直接事实与 fallback 区分。
- [BCI 分段改变栈上旧值或回边] → 用 SSA owner、全部旧值 uses 和正常 CFG 验证，冻结 held-use、cross-phi、return-path 负控制。
- [既有数组/Ref-Int/monitor 预算或文本移动] → 保持旧路由，逐字源文本对照与全仓指纹检查；新增计费如实重钉。
- [调试表混入功能成功率] → 32 输入按16 no-debug、8不同名 LVT、8同名 LVT 分开统计；后两组保持当前结果。
- [局部成功冒充整个单元追平] → EM-20 五项中 smali/禁编译/未触发分段的测试单列，完整 LG 有其它失败不得删成员后冒充完成。
