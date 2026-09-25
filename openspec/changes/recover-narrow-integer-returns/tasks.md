## 1. 固定返回指令的独立对照

- [x] 1.1 将无显式conversion的窄局部回读及直接B/C/S返回边界整理为最小永久Java8主类，保存源代码、精确descriptor补丁、hash/Code和原class实际验证执行结果；保持helper/runner source-only，root统一冻结census/fingerprint。
- [x] 1.2 固定字段前/后自增、同步返回及已有switch返回下推的窄返回测试；以原class完整运行结果和真实return/operand BCI验证各消费入口，保留Z与boolean到B/C/S的合法拒绝变体。条件stack phi另存边界，不能列作本项恢复正例。Java 8 永久样本、真实 JVM 和来源证据见 [边界验收](verification-1.2.md) 与 [root 独立验收](verification-root.md)。

## 2. 复用返回位置的转换语义

- [x] 2.1 在既有返回消费处根据真实ireturn、方法返回类型和呈现整数类型生成必要Cast；用B/C/S极值、负值及超范围值测试真实截断和符号/零扩展，普通赋值/调用转换回归保持原契约。root 的 49 项永久类、39 项直接边界及 `p3_required_conversions` 除既存冻结计数外 7/7 见 [独立验收](verification-root.md)。
- [x] 2.2 普通、同步、switch下推及字段自增返回共用已呈现Expr的适配；分别验证求值位置与真实return来源不混用，字段只更新一次且完整int字段值不被提前窄化。root 的 49/19/36 项完整行为与真实共享 `ireturn` 来源见 [独立验收](verification-root.md)。
- [x] 2.3 新节点/来源沿用预算、深度、取消及提交规则；测试默认/all正文相等、真实operand+ireturn来源、受限预算/取消停止和replay稳定，未知值及boolean边界仍诚实拒绝。`p3_narrow_integer_return_contract` 3/3 和 `p3_narrow_return_boundaries` 3/3 均由 root 独立复跑。

## 3. 完整类行为与 root 验收

- [x] 3.1 对永久输入及root的NarrowLocals20项、ReturnSinksCore19项、直接B/C/S边界39项实际生成零引用完整类，原样javac/java与原class逐行对照；Z另13项保留边界。JADX完整输出失败时保留阶段事实，不手工修复或假称执行一致。root 隔离复放 49/20/19/39 均逐行相同、零引用；JADX 的成功/失败阶段分别记录在 [独立验收](verification-root.md)。
- [x] 3.2 root独立检查返回事实、特殊路径、转换来源与一次求值；复跑required-conversions、boolean-contexts、field-increment、guard、switch和deferred-order相邻回归，另记录未支持条件phi和Z最低位转换债务。[独立验收](verification-root.md)记录相邻测试、JADX 的 raw-2 反例及分离债务。
- [x] 3.3 root统一Java包、语料census/fingerprint、fmt与OpenSpec strict；严格clippy既存region债务单独记录，完成验证后才勾选实现任务。定向 Java/Clippy、fingerprint 5/5、fmt 与 strict 均通过；全工作区既存门禁债务见 [独立验收](verification-root.md) 与 [roadmap](../../roadmap.md)。
