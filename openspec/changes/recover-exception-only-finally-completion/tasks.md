## 1. Establish the actual input and baseline

- [x] 1.1 对两907B封存输入实际运行Facts/SSA/guard探针，记录完整CFG/异常边/effects/值身份及真实首拒；root核raw、输入pins、临时源码恢复与资源清理收据，见results/initial-facts-root-v1。
- [x] 1.2 固定完整独立ExceptionOnlyFinally源/driver与双javac8目标输入，运行原始九组合oracle及当前CLI/JADX红基线；原完整FinallyCompletion七方法另行保留，记录全部编译/运行成功或失败，不删方法或修文本。

## 2. Implement the proved completion

- [x] 2.1 增加有界exception-only finally proof，逐项核单row/两块/全边与throw-sites、Caught phi及store/load/rethrow身份、完整指令分区与ownership；永久正证明须对两编译器和封存原class成立，generic proof近邻结果保持。
- [x] 2.2 在既有FinallyCompletion显式表达Throws，并仅将Shape::Finally字段normal_cleanup改为cleanup；所有构造/匹配点显式处理，验证旧FinallyOnce、finThrow/nestedFin与返回型finally正文/来源不变，无Void伪装或新Shape/solver。
- [x] 2.3 Builder保留body原throw且只写一份finally cleanup，Region不开放无关loop/normal continuation；建立完整checkpoint并验证body/cleanup/declare_at/push失败及未呈现local均原子恢复，不发布部分ownership或正文。

## 3. Pin boundaries and public evidence

- [x] 3.1 为row缩放/竞争、wrong caught/rethrow identity、cleanup再次受保护、额外入口/出口与finally非局部完成增加真实classfile邻变体；先断言reader/IR完整再断言拒绝，并保留JVM严格验证和具体首拒收据。
- [x] 3.2 增加early/late budget、真实取消与期限停止测试，证明已进入proof/Builder后仍按原Stop及usage返回，公共入口无部分正文/映射；精确目标selector实际运行通过。
- [x] 3.3 公库及完整class-source default/all正例核全部物理方法/BCI/身份、无fallback目标及来源UTF-8界限；完整原FinallyCompletion仅tryThrows改善，其余文本/两显式finally红项保持，原样编译错误如实保留。

## 4. Accept the complete product

- [x] 4.1 最终冻结CLI执行两输入compiler × default/all × 两重编compiler × 双JVM的完整独立类矩阵，九组合逐字等原始oracle并核注入对象身份、78恰好一次、异常类型/message及覆盖；JADX固定同输入新输出与真实失败单独保存。
- [x] 4.2 实际运行reader/corpus fingerprint/P5、旧CF16与finally邻接回归、fmt/当前CI同款Clippy/MSRV和完整workspace双seed；默认栈/5GiB/1GiB/每秒守卫/最终exact clean全部通过；不无条件更新预算或文本基线。
- [x] 4.3 形成并推送精确产品提交，独立验收该提交自身CI及最终来源恒同；root复核计划范围、完整raw与双版本strict后更新handoff和CF16叶片记录，不增加71/612总账，也不把未覆盖显式finally完成称为已恢复。
