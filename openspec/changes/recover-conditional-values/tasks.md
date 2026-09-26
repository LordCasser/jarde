## 1. 冻结合法的值汇合输入

- [x] 1.1 从 `ternary-values/core/` 建最小永久 Java 8 fixture，保存 424 B/6 Code class、源码与 runner，两项原/JADX相同和当前 jarde 整类缺返回语句；另保存完整 1414 B/15 Code、16 项样本及真实阶段，root 统一 census/fingerprint。root 在独立临时目录重编三份 subject class 并逐字节核对 SHA，三个 source-only runner 在 `-Xverify:all` 下分别输出 2/16/4 行；reader census 为 `(164, 1152, 98, 381, 8)`，407 文件的指纹 5/5 通过。多入口第三份归 1.2 的拒绝边界，不替代条件值正例。
- [x] 1.2 补真/假臂单独抛错、局部/算术/调用消费、`null` 引用、重载目标与未选臂不执行；合法 JVM 的多入口、循环/异常或类型不明反例不得被本项条件值规则认作双臂 `?:`，普通 switch/循环/异常路径仍可按各自证明恢复；不使用无法通过验证器的伪字节码。`TernaryValues` 的 16 行重编对照锁住正向分支；新 950 B `ConditionalBoundaryCases` 及三个类型依赖均由 Java 8 javac 生成。root 在独立目录重编 subject SHA `04d7291d…` 并以 `-Xverify:all` 得到七行原行为，当前 Jarde 对循环、异常和缺层级引用汇合保持拒绝标记，证据在 `evidence/boundaries/`。其中普通 try/catch 局部作用域的不可编译文本是另一项架构债务，不被视为条件值误认。
- [x] 1.3 保留 `assert-syntax/` 的 1346 B Java 8 class 与 `-ea`/`-da` 原/JADX对照及 class-literal 修后 CLI 25bf 的 BCI 13 历史 RED；另冻结 `assert-core/` 的最小源码、class SHA、外置 runner、当前原/JADX/Jarde 完整类和两个模式。仅改 `<clinit>` 一个 `ldc` 的错误状态来源补丁在 `-ea:AssertCore -da:java.lang.StringBuilder` 下保持相反输出；有效 BCI 8/12 `2`/`3` 补丁在 `-ea`/`-da` 下反转，Jarde 没有把非 0/1 Phi 猜成布尔。root 独立重跑 `assert-core/replay.py` 后 summary 与 `evidence/assert-core-current/replay-summary.json` 字节相同，并对 non01 class SHA `b52d39dd…` 单独重编 Jarde 完整类、两种 JVM 模式逐行比对原类通过。当前 CLI `a3a29b59…` 复用既有字段收窄，无新增机制。

## 2. 复用 region/SSA 构建条件值

- [x] 2.1 有界证明 `Region::If` 两臂与唯一变化的 stack Phi 输入的身份、前驱及唯一消费；透传的同值 stack Phi 可留在外围算术或调用表达式中，其他变化 Phi 拒绝；没有证明不得在一般 `Definition::Phi` 上猜条件值。root 修正预算和实际类型编译错误后，`a_conditional_proof_binds_each_straight_arm_to_one_stack_phi_and_consumer` 覆盖返回、赋值、算术、实参、引用、重载和抛错双臂，switch 拒绝测试通过；本项仅是证明函数，尚未接入表达式构建。
- [x] 2.2 在现有表达式树/发射器增加条件表达式，保持分组、静态类型、Methodref 目标和一次求值；不得把两臂提前保存或重复输出调用。root 审读预构建、条件类型及优先级，重跑 `p3_conditional_values`；新 CLI `a3a29b59…` 的完整 `TernaryValues` 源含七个 `?:`，重编执行的 16 行值、trace、异常及 String 重载均与原/JADX 一致。
- [x] 2.3 成功时归属测试/臂/跳转/消费者来源，失败时完整引用或预算停止；默认/all/replay 正文一致。真实 fixture 测试逐方法核对 all SourceMap 的 branch、两臂 producer、transfer、consumer，并在 IR/输出预算或取消下确认空文本、空来源和正确停止；root 的 `conditional-values-accepted/replay.py` 对 core/full/switch 三类按原 class SHA 重编并验证 essential/all 文本哈希一致、零引用、Java 8 重编和 `-Xverify:all` 输出一致。更宽的负向控制流边界仍列在 1.2/3.2 验收。
- [x] 2.4 核对已证明的双臂整数值接到真实 `putstatic`/`putfield` 唯一消费者时，既有 `Z` 字段写入收窄是否已完整保真；普通 `0`/`1` 和有效 `2`/`3` 均须按 JVM 实际低位结果重编执行，沿现有 final 字段左值规则，不把一般整数 Phi 或别的字段写入误类型化。`AssertCore` 静态 final 路径与新 `field-writes` 的静态/实例 `Z` 写入均通过；root 独立重放整类 Java 8 编译及 `-Xverify:all`，原/Jarde 0/1 和 2/3 各两行完全相同。`putfield I` 不被布尔收窄，既有 duplicate-Phi 多消费者保持拒绝；JADX 1.5.6 的 2/3 输出无法编译。复用现有字段写入规则，无生产代码变更。[证据](evidence/field-writes/README.md)

## 3. 整类语义与 root 验收

- [x] 3.1 原样重编译执行最小与完整样本，要求被证实的条件值零相关引用、原/JADX/jarde 的值、trace、异常和目标选择逐项相同；`assert-syntax/` 与 `assert-core/` 在独立 `-ea`/`-da` JVM 中核对条件/消息调用次数和异常身份，`assert-core/` 补丁还须在选择性启停下保持原类与错误来源类的不同结果。JADX 只有编译成功才算执行 oracle；本 change 不以是否输出源级 `assert` 或 synthetic flag 等价为成功标准。[Root 独立重放](verification-3.1.md)
- [x] 3.2 root 独立审读 join/Phi 准入、条件表达式类型与来源，复跑 if/switch、局部、算术、调用、deferred 顺序及拒绝边界。[Root 独立审读与负例重放](verification-3.2.md)
- [ ] 3.3 root 统一 census/fingerprint、fmt、适当 Cargo 测试及 OpenSpec strict；一般 Phi/区域债务保持独立。
