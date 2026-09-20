## Context

动机与范围见 [proposal](proposal.md)。基线为 `cd6f2f0`；两条反例经真实 Java 8 CLASS、公开 Engine 入口复现，记录见 [verification](verification.md)。现有 MethodIr、SSA、effect 与 origin 足够表达这些事实，缺口位于产物重建，不需要新增中端。

`Builder::render_value` 的直接 local 检查已阻止旧的 `return x++` 错误，但 Arithmetic 递归把子节点的使用位置改成生产者 BCI。字段读取在 instruction 分派时不发语句，fallback 的依赖收集则主要登记 deferred invoke 与被覆盖的 local load；因此已识别的静态字段也可能没有产物位置。旧 P3 修正有效于原反例，尚未覆盖这两个组合。

## Goals / Non-Goals

**Goals:**

- 按实际发射位置与路径验证表达式值及求值顺序，复用已有局部值等价检查。
- 让降级范围覆盖其未呈现的可观察生产者，并映射到调用方方法内的真实 BCI/CP。
- 以公开入口反例、有效结构对照和库/CLI 一致性证明修正，不把错误移到另一个消费位置。

**Non-Goals:**

- 不要求本轮为所有跨写入值引入临时变量；可靠 fallback 是当前契约允许的最小修正。
- 不新建通用 value/effect 框架、cache、并行或独立身份体系；不读取 External 的 Body 来推测是否存在类初始化副作用。
- 不扩大已归档阶段的功能覆盖。MethodParameters、类级事实、现代源码恢复和更大性能语料分开安排。

## Decisions

### 1. 生产来源与实际求值上下文分开

保留 producer BCI 作为 origin；另将最终产物实际求值点/所在 Region 贯穿递归检查，不能递归一层就换成该算术指令的原始 BCI。先使用现有 SSA 值、local 等价检查和 effect/path 证据验证能否重建；证据不足时降级相关表达式或区域并保留闭包。

仅对顶层 Load 加特判已被嵌套反例否定。普遍引入临时变量会扩大声明、作用域与预算改动，暂不作为前提；若最小可靠修正确实需要物化，应在既有私有 AST/build 中实现并按现有预算计费，不重做 SSA。不能仅比较数值 BCI 大小证明跨路径值相等。

### 2. fallback 收集实际未呈现的 effect 来源

沿现有 SSA 操作数与 effect 事实定位依赖，覆盖字段读取、延迟调用和其它可能抛异常/初始化的生产者。已经由可靠语句执行的操作不重复写；无法安全安排语句时，把需要保留的生产者与消费者共同引用。遍历需去重、有界并沿用取消/预算，不把单个 fallback 扩成无条件全方法复制。

`getstatic External.value` 的 BCI 属于当前 caller 方法，CP 指向 External 字段；映射复用既有 PhysicalMethodId，不凭字段 owner 构造 callee 方法身份。独立 FieldRecord 只能证明识别发生，不能替代文本/source map 中的 effect 保留。

### 3. 独立对照检验产物，不只检验中间事实

扩充现有 `p3_local_rewrite`、恢复/编译执行对照体系。若嵌套表达式仍输出 Java，原输入 7 与生成方法都必须返回 16；若降级，核对相关 BCI、物理方法及诊断。字段 cast 反例核对 BCI 0/3/6 的保留，使用受控 External 初始化计数证明原读取可观察，但不执行不可编译的 Mixed 产物。

继续保留简单调用只执行一次、bump/doubleIt 正常输出、分支作用域和 accessor 双来源对照。变异必须让“递归改回 producer BCI”及“fallback 删除 getstatic”分别变红。原有 CFG ledger 检查继续保留，但不能把“属于一个块”解释为“已在产物保留”。

### 4. 复用与依赖

继续使用已准入的 reader、SSA/effect、AST/emitter 与现有测试工具；本问题没有需要新库承接的能力缺口，不重新评估版本或引入依赖。真实 javac/Java 只用于自建受控 fixture 的测试，生产引擎保持离线、不执行目标代码。

## Risks / Trade-offs

- 保守检查扩大降级 → 以未跨写入的嵌套算术、简单调用和现有结构样本作正向对照，禁止无条件拒绝全部算术。
- 收集依赖导致重复 effect 或乱序 → 以单次调用、初始化/异常次序及多个消费者对照，并区分已发射与仅识别的节点。
- 递归/闭包增加成本 → 沿用深度、item/output 预算和取消，在停止时不发布成功半成品。
- 对照两侧共享同一错误 → 预期数值取自原 CLASS 的独立执行，来源完整性另按原始指令检查。

## Migration Plan

先关闭实际求值上下文，再关闭 fallback producer 保留；随后纳入永久 fixture、证伪和公开库/CLI 对照，执行当前候选的完整门禁。没有存储/协议兼容迁移。通过后同步本 change 的 delta 并归档，更新完成判断；未通过前保留当前不完成结论，不改写 P0–P5 历史验收。
