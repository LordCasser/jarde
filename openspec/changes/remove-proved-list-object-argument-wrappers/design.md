## Context

见 proposal 的固定用例差距。`prove_generic_list_parameter_body` 已验证 List receiver 的封闭使用，并为必须的 add(E) 使用检验元素的 next→CHECKCAST→store→load→invoke SSA 链。已有 `ClassSourceInvokeArgumentPresentationCast` 和 opaque retained AST 事务可精确移除展示包装，raw RecoveryReport 与投影文本分离。staging 和 final 均在同一原始 AST 上重新证明并比较计划。

两版实际 SDK javap 均证明 String 为 final 且仅有 equals(Object)，List 有 remove(Object) 与 remove(int)。私有控制的 javac8/23 四配置原始编译和双 VM 实际输出均显示：Integer 引用选择 remove(Object)，primitive int 选择 remove(int)，自定义 equals(Object)/equals(String) 可观察地不同，真实 CHECKCAST 对错误 Integer 抛异常。整个采集尚未终态；以最终收据和 root 独立审查为准，不把准备或中间结果当完整验收。

## Goals / Non-Goals

**Goals:** 只在已有 List body 范围内，利用同次物理和 retained AST 事实去掉固定 equals/remove 包装；引用局部类型不限于 String，但必须经过现有完整元素局部证明。null 如直接复用现有单一物理 aconst_null 证明则可支持；缺证保留。

**Non-Goals:** 不扩展 Builder 的 descriptor-only 入参入口，不处理任意 Call/lambda/method reference/poly 表达式，不根据 assignability 单独删除 cast，不构建完整平台层级或重载求解器，不改变 foreach 准入与物理 CHECKCAST。

## Decisions

1. **复用已有窄证明。** 提取现有 add 的实参证明为私有函数，保留唯一声明/赋值、真实词法祖先、foreach 所有者、物理 cast 类型、唯一 next 生产者、store 和所有局部写闭合、load 及 invocation argument SSA 身份。返回已证实际引用类型/已有空值情况，避免复制同一证明。必须 add 仍按旧规则参与 type_uses；remove/equals 仅产生可选展示编辑，绝不提供元素推断。

2. **明确调用选择。** remove 只接受当前已证 List formal 的精确 invokeinterface java/util/List.remove:(Object)Z；引用局部通过 strict reference widening 匹配 Object，remove(int) 需要 unboxing，不能先于 strict phase。String equals 必须是精确 invokevirtual java/lang/String.equals:(Object)Z，retained receiver 呈现类型必须为 String 并有对应同次物理 receiver/AST 来源，避免把自定义 equals 重载纳入。实际平台事实和编译控制支持此封闭规则；不据名字或 descriptor 单独推广。

3. **可选性不污染核心。** 保留现有必须 List body 证明失败契约；每个可选调用缺证只跳过该编辑。call key、argument index、完整方法身份、wrapper 来源和唯一匹配都需闭合。在现有 staging/final 两处确定性重算同一可选集合；复用现有 retained AST transaction。若可选精确应用可能失败，使用已有事务进行预检或第二次试投影并保留成功核心结果，不新增全局状态或公开模型；必须编辑失败和真正 Stop 不能被吞掉。

4. **输出与预算。** 只变 ClassSource 完整类/方法投影；raw 文本、raw source map、物理来源及执行质量保持。新增遍历/分配按现有 Budget 计费并 poll，Stop 回滚到未发布状态。不能用粗略方法总量替代新增嵌套工作计费。

5. **复用与来源。** 无新依赖；现有 Rust 标准库、opaque AST、SSA、投影事务已覆盖缺失能力。参考 JADX MethodInvokeVisitor 的源级绑定保护与 SimplifyVisitor 的显式 cast 保留思路，采用本仓物理来源证明，JADX 本地 LICENSE 为 Apache-2.0；本片参考算法边界，不复制源码，不引入依赖或许可证变化。

## Risks / Trade-offs

- [Integer 错误反例] → primitive/reference 分开，双版编译产物实际 descriptor 和运行事件作为 oracle；撤回早期静态误判，保留更正记录。
- [可选缺证撤回可靠泛型] → 必须推断与可选清理分开，故意断链控制要求已有输出仍在。
- [真实 cast 被误删/自定义 overload 被改选] → 精确平台调用、receiver、物理值链及原始 wrapper predicate，参数真实 CHECKCAST 不进入展示包装集合。
- [整类反编译存在旧失败] → 完整源码原样重编译；每个失败真实分类，不裁源、不补原始 inner class，不将编译失败计作 runtime 通过。

## Migration Plan

根审查规划后交 Luna 私有实现；根审查补丁、运行必要 Rust 回归和完整原始/JADX/Jarde 比较，保留 default/all 与 raw 不变检查。机器余量至少5GiB、本仓 target 上限1GiB、default stack、每秒进程组守卫；Rust 工具链串行且 finally exactclean。产品提交推送后验其自身 CI；主规范、固定账本和 handoff 按实际结果更新，保留干净 main。无需格式迁移。
