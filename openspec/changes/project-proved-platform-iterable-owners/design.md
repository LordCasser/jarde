## Context

见 [proposal.md](proposal.md) 和[固定的 owner 三方对照](../../evidence/java-syntax-2026-09-24/iterable-subtype-owners/analysis.md)。已验收的 [direct `Iterable` 候选](../project-proved-enhanced-for-loops/verification-root-iterable.md)在 `jarde-java/src/build.rs` 内利用 `CallTarget`、SSA、effect handlers、名称表和现有 `ForEach` AST 完成证明与原子提交。`List`/`Collection` 在 Java 8 中是 `Iterable` 的平台接口子类型，但 javac 将调用符号 owner 写为它们自己；单类分析没有外部自定义接口层级。

## Goals / Non-Goals

**Goals:** 用可审计的 Java 8 平台接口事实扩展同一个候选入口，同时保留已有全部语义门槛及预算/来源契约；有无调试信息都给出可编译、可执行的完整类。

**Non-Goals:** 不建立任意 classpath 子类型求解器，不从 `Signature` 猜泛型元素声明，不支持用户自定义子接口或任意 `iterator()` 短签名，不处理 quoted fallback 跨异常区的 P0 结构债务，不清理数组投影后的无用局部声明。

## Decisions

1. **有限平台事实放在现有候选边界。** 对 `java/lang/Iterable`、`java/util/Collection`、`java/util/List` 的 `iterator()Ljava/util/Iterator;` 分别精确匹配 invoke kind、interface flag、owner/name/descriptor；接收者 `Expr::presented` 的 Java 源类型须与符号 owner 同名。前两种新增 owner 的 Java 8 `Iterable` 关系由平台契约及冻结 `javac --release 8` 复编证据确立。相比把任何接口的同名方法视作可迭代，这不会使自定义非 `Iterable` 输出非法源码；相比先建设通用层级求解器，此处没有新的依赖读取或全局机制。列表仅表达固定平台事实，不改变通用类型转换规则。
2. **复用 direct `Iterable` 的整个证明和提交。** 只扩展源类型与第一处调用 owner 的准入，`Iterator.hasNext/next`、SSA 独占消费、首动作、cast 原位、逐 invoke handler 集合、外部赋值清理、来源、预算与取消均沿用现有候选。不得另起第二套循环识别器或先修改 AST 再判类型；已有数组投影和 direct `Iterable` 结果应逐项回归。
3. **原始类型用 `Object` 绑定。** 当前类源码可把泛型 `List<String>`/`Collection<String>` 参数呈现为 raw 类型，`for (Object element : values)` 是 Java 8 合法语法。保持 `next()` 原有 `checkcast` 的 AST 位置，比借调试表把元素直接标为 `String` 更可证明。若后续 `Signature` 恢复泛型，该质量提升另立类型任务，不改变本次等价证据。
4. **JADX 只提供算法线索。** 本地 JADX 1.5.6 在有调试表时认领 List/Collection，无调试表时退回 while；它在[异常边界探针](../../evidence/java-syntax-2026-09-24/iterable-exception-boundaries/analysis.md)对 `iterator`/`hasNext`/`next` 分区的投影可改变 catch/finally。Jarde 沿用逐指令 effect 集合而非照搬其可变 Region 标记。现有 reader/SSA/Region 足以处理本有限范围，新增库无法替代来源证明，还会增加维护与许可审计成本。

## Risks / Trade-offs

- [平台 owner 与接收者类型不一致] → 精确比较字节码调用 owner 和已呈现源类型；无法证明即保留 while。
- [泛型看似可用但方法头是 raw] → 固定 `Object` 元素加原 cast，Java 8 重编和坏元素执行对照，不做无依据的类型收窄。
- [异常边界或前置副作用被移动] → 继续要求同一处理器集合及首动作证明；以负例检查拒绝路径。
- [有限白名单遗漏更多合法子接口] → 记录覆盖缺口，用户自定义类型须等请求环境能提供依赖定义和传递层级证明时单独扩展。

## Migration Plan

无持久数据迁移。先冻结 List/Collection 的有/无调试信息基线和原/JADX/Jarde 执行，再扩展一个候选门槛并完成完整类/负例回归。回退只撤销这两个额外平台 owner 的准入，direct `Iterable` 和数组投影保持原行为。
