## Context

见 proposal。产品基底 `73adae72c6d89e8eba0a34e3cd92fe81e899cb3e`，Builder SHA256 `3bce6972b9b8516a270725148e8e3e662b9dce09ff86c11207c6bb3b124d20f7`；foreach关闭文档 `7c0848c9d5677d4f2fa3fc4ee634f81040bc5348` 已推送，outside final audit实际exit0/main=origin/仅main/15checkouts干净且无target。原始同IR证据见 `../../evidence/jadx-feature-inventory-2026-09-27/cf11-type-diagnostics-accepted-root-v1/README.md`：local1/index0有store2/String与store25/Object，同Phi2→Load29→Stack Value14唯一Return30、返回descriptor Object；实际accept-v5只接受观察。baseline-v1原完整类8次编译失败/no runtime另存，不借baseline-v2种子调用的成功冒充该literal案例。

当前 `decide_types` 2434附近统一定型，2873附近的全写复查只比较reference/primitive类别。`direct_reference_write_type` 3011、`written_type` 32114和`reference_lifetime_type` 32133提供不同强度事实。root scoped Atlas search只核结构位置，cached行有一行偏差，实际源码逐段核对；不以结构索引声称SSA流证明。

## Goals / Non-Goals

**Goals:** 在既有声明计划内纠正准确Object写被首个窄引用声明漏检的普通body local；读边界严格只允许每个Load输出唯一直接Object Return，统一结果由提升/就地声明与赋值共享。拒绝必须解释所选声明与写/读约束不能同时满足。

**Non-Goals:** 不计算任意引用公共类型，不扩大null-leading/char/boolean/专门nullable/generic证明，不改参数descriptor、reuse identity、SSA/Phi类型或foreach门，不插入cast，不清理空声明，不修LVT descriptor或结构化void泛型body。不扩大其他调用/数组读取位置准入。

## Decisions

1. 使用现有 `LocalVariable` key及完整 `SlotUse`，不新建服务。仅普通first-write fallback的窄标量引用候选且slot不属于参数/resource/catch header，至少一个stored operand由既有直接producer helper精确给出 `java.lang.Object`，才进入本证明；Unknown frame被拼为Object不是这个触发事实。保留所有专门类型优先级与null-leading边界。若body含未覆盖handler写或访问path缺失，候选拒绝，不能靠跳过caught掩盖完整性。
2. 全写证明使用同identity、同次SSA，逐物理write核stored/written均为已初始化reference或实际null；Object能接收这些reference是Java普遍赋值事实，不是LUB或具体层级。首写null继续原规则，数组型首候选不通过此规则。直接Object写不能填入窄引用声明时，不得让原category-only复查静默通过；读证明失败用既有 `NoType::ConflictingWrites`/refusal通道，必要精确修其文案，避免谎称不存在任何Java声明（Object虽可容纳写，却未证明读）。不新建refusal类别或通用类型转换。
3. 对所有read逐一核 `Operation::Load`、SSA指令实际local读取与reuse.variable_at是同identity；严格核其单一Stack输出、定义BCI、全部uses中唯一消费者、消费者SSA指令实际读该Stack/value、operation是Return、物理方法descriptor返回exact Object且无额外写出。Phi输入不用构造LUB，既有identity/CFG事实负责local归属；不能用相邻BCI、源码 `return target`、打印名称或仅Phi→Load当完整读证明。任何copy/dup/stackPhi、多个消费者、receiver/argument/array/null-test读均不通过此首片候选。
4. 证明只在局部类型结果发布前返回统一Object决定；不修改SSA、AST或声明计划中间状态。每个新增access/SSA consumer/instruction遍历在读事实前charge现有AnalysisSteps并poll，不能把instruction_at的全表扫描当常数工作；优先复用既有索引/读序遍历。Stop传播原Result，全部候选完成前不写入结果，不产生部分方法文本/map。公开late Stop必须按新真实计费测量，旧阈值不借用。
5. 参考本地JADX `TypeInferenceVisitor.initTypeBounds/attachBounds/mergePhiBounds` 同时看assign与usages、跨Phi汇聚约束的原则；其全图mutable TypeUpdate不适配本仓只读IR和一次source decision，故不复制通用推断pass。标准库BTreeMap/现有SSA、Operations与类型helper足够；无新依赖、许可或维护面。parse、JVM verification、Java static assignment和source compile/runtime分别验收，frame reference类别不是窄类型可赋值证明。

## Risks / Trade-offs

- CharSequence的String/StringBuilder写与length读取，以及CharSequence[]真实数组读写是合法反证 → 不Object化、不插cast；沿原规则保持可靠结构或准确fallback，不宣称接口公共类型已恢复。新片仅Object直接写触发，另补Object写后显式checkcast调用的合法控制以实触读拒绝门。
- 同slot不同index及无debug → key隔离、分别真实debug/-g:none，不能从源码变量名推slot reuse；捕获javap/SSA事实并保留原文。
- handler/resource隐式写不全 → 此候选拒绝；原handler证明不放宽，控制必须记录到达哪一门而非仅看编译失败。
- 泛型/旧golden变化 → 单独定位实际输出/成本，最小更新准确预期；不把CI失败混成类型新机制。

## Migration Plan

先冻结正向literal与无debug、反例完整Java/Runner及原/JADX/旧Jarde基线；Luna只写private候选、root对抗审阅后串行应用。完成定向真实API/profile/source maps/Stop、双JDK完整生成类对照、fresh workspace、fmt/同CI Clippy/global strict、冻结最终CLI/精确产品自身CI和干净main，再关闭窄片。仅root运行工具链/Git，无新worktree；5GiB最低机器余量/target1GiB/一秒进程组守卫/默认Rust栈/finally clean，固定cargo clean可独立低空间回收并记录。
