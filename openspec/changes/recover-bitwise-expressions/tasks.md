## 1. 固定语法与类型边界

- [x] 1.1 从bitwise基础/type-flow审计收成一份最小永久Java8 class及source-only helper/runner，记录hash/Code数和修前真实失败；覆盖整数提升、嵌套boolean、局部复制/分支赋值、实参/条件以及左右效果，原class与JADX267项基线保留。
- [x] 1.2 将mixed-types合法descriptor变体、纯整数0/1对照、旧局部及dup/pop消费边界做内存patch回归；用JVM验证实际输入，并区分已呈现来源与必须引用的延期生产者，不新增永久class或通用class parser。Luna 的原样证据在 `bitwise/boundary-audit/`，root 使用同一冻结CLI独立重放于其 `root-replay/`；原/patch均通过JVM验证，混合descriptor保持拒绝，通用dup多消费者另记边界。

## 2. 闭合事实、类型与发射

- [x] 2.1 解码六个opcode并扩展现有BinaryOp及类型/优先级表；以解码、byte/char提升、long与混合分组测试验证，不增加表达式AST层级或化简模式。
- [x] 2.2 扩展共享boolean证据查询与既有decide_types队列中的组合依赖，保持literal不能独立启动boolean类型；用nested/copied/hoisted/passed、混合boolean/int与纯整数对照验证声明和消费始终同型。
- [x] 2.3 在既有render_value、消费延期与失败来源路径接入位运算，保留最终at和左右一次求值；用左右抛错、false左值仍调用右侧、旧局部及失败消费者测试验证，无关区域不扩张。
- [x] 2.4 验证默认/完整来源同正文、真实运算/操作数/消费者BCI及member、正文与来源预算停止；深嵌套、局部链和取消必须实际验证有界失败，不新增无预算递归。

## 3. 完整执行与主代理验收

- [x] 3.1 使用完整Engine生成类原样javac并执行，覆盖原始267项及实现新增对照；原侧执行冻结class，记录整数结果、boolean结果、调用顺序、计数和异常，不能删换生成方法后报告整类通过。
- [x] 3.2 root独立审查类型证据、队列计费和拒绝完整性，复跑bitwise两组独立输入及算术/boolean/局部/调用/来源预算回归；统一冻结census/fingerprint，执行fmt、受影响clippy与OpenSpec strict，并分别记录既存门禁债务。最终CLI及逐命令记录见 `bitwise/root-after/`，完整类268行原/JADX/Jarde一致，混合Z/I仍引用拒绝；严格Clippy只余既存 `region.rs:1736` 的 `type_complexity`。
