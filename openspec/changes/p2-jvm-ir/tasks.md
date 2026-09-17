## 1. 基础契约与第一片交接

执行前确认 [harden-p1-validation](../archive/2026-09-17-harden-p1-validation/verification.md) 的维护提交及对应 CI 通过；该维护已提交（`555c785`、`acbba49`）并由 CI run `35238994798` 证明（四个 job success），并已归档。本清单只跟踪 P2，第一轮仅做 1.1–1.3，验证并只读复核后再进入 2.x。所有任务均未实现，既有债务不并入本清单。

- [x] 1.1 固定解析/声明查询与方法分析的请求、provider 绑定、阶段结果、origin 和预算计费契约；交付可编译的最小类型/API 及示例，验证缺少运行环境不能隐式启动解析，Bytecode/NotJava/NotPerformed 与 coverage/execution 可分别表达（A13、A17）（2026-09-17 完成：契约先经只读复核定稿，`src/environment.rs`/`resolver.rs`/`ir.rs` + 三个 `Engine` 入口 + 示例 + 28 条契约测试；实现复核首轮 Approve 的 F1–F4 与复审的 N1–N3/D1/D2 已关闭，证据见 verification 的 1.1 节）
- [ ] 1.2 在现有 noak reader 适配中提供内部类型化操作数与目标校验；用 wide/iinc、正负 branch、switch default/key/target、handler 边界、溢出/跳入操作数反例及 P0 oracle 回归验证，不另建 decoder（A09、A10）
- [ ] 1.3 扩展现有 Budget 的闭包/IR 存储/边/步骤/克隆/依赖深度计费；验证零/恰好/超限、节点/槽位/边/克隆的计费样例、步骤耗尽、取消、预算不重置以及两类 depth 独立；真实 Frame/SSA 膨胀与 fallback 联调由 3.5/4.3 验收。记录第一片命令、反例和只读复核结果后再交接

## 2. Header providers 与声明解析

- [ ] 2.1 实现不可变 CLASS/JAR 和平台 Header providers、domain/parent/root 映射与内容身份；以 ParentFirst/ChildFirst、同名有序 root、同位置歧义、缺失/循环 parent、未知策略和 module mode fixtures 验证选择与诊断（A14）
- [ ] 2.2 实现按需 Header 闭包、读取 reason 与去重；用深链/高扇出/缺失依赖/循环引用验证预算和取消，证明无关 Body 读取为零、同 bytes 不同 loader/origin 不合并（A14、A16）
- [ ] 2.3 实现字段、class/interface method、访问/静态性、构造器及 invokespecial 解析；用合法/非法对照和 Java 8 default conflict fixtures 验证状态，signature-polymorphic/数组方法有明确支持或 unsupported 分支，不以同名递归替代规则（A11）
- [ ] 2.4 实现显式运行环境的声明引用查询，复用结构 consumer；Base.foo 查询须发现 Sub owner 的真实 use-site，未使用 CP 不算引用，缺失依赖及预算停止保留未决候选。验证 P1 MentionsSymbol 仍区分原符号（A11）
- [ ] 2.5 分离声明 resolution 与已知范围 dispatch；用多实现、外部子类、未知 loader/transformer 验证 open-world，单一已知候选不声称唯一运行目标。完成本片只读复核与交接（A11、A16）

## 3. Raw CFG 与有界 legacy normalization

- [ ] 3.1 完成 petgraph 候选准入：复核候选版本/维护状态并验证 MSRV 1.88、许可/feature、平行异常边、不可达节点、自环、多出口、稳定排序和预算/取消粒度；通过后引入并仅调整 CI 的 petgraph 禁令，保存依赖树与准入证据，未通过须记录可复现阻碍和替代比较
- [ ] 3.2 实现固定 phase/PassDescriptor 静态依赖与 invalidation 校验；用缺失前置、环、错误顺序和 CFG 变更后拒用旧分析的反例验证，不引入动态调度框架
- [ ] 3.3 实现 raw CFG、指令级 throw sites、handler order、保护区间和 effect facts；用分支/switch/不可达块/重叠 handlers 验证异常边来源及 locals/effect 状态，P1 原 BCI 与引用数量不变（A09、A17）
- [ ] 3.4 实现 raw returnAddress/调用上下文分析；真实历史 finally、共享/嵌套子程序和异常路径须有可核对的返回点及受影响 locals，非法 51+ jsr/ret 明确违规（A09）
- [ ] 3.5 实现有界 jsr/ret 克隆规范化及 CanonicalCFG；验证一对多 origin、异常范围、恰好/超界克隆、取消和 bytecode fallback，保存失败反例及本片只读复核（A09、A13）

## 4. Frame 与 stack/local SSA

- [ ] 4.1 实现 descriptor 驱动的 Frame 与 category-1/2、双槽、dup/swap；以合法/非法组合及缺失 debug/StackMap fixtures 验证状态、不变量和 NotPerformed，不能把推导成功当 verifier 成功（A10）
- [ ] 4.2 补齐 uninitializedThis/new-site/初始化转换、handler entry、null/数组/未知引用合流；用同一 block 不同 throw-site 的 locals 对照、缺失依赖和循环收敛/超限验证，不伪造确定类型（A10、A14）
- [ ] 4.3 实现 stack/local SSA、正常/异常 predecessor 的 phi、定义/use、origin 与 effect 顺序；以 diamond/loop/不可约/异常合流、高扇出 phi 和多槽位预算验证，保留最后有效阶段并完成本片只读复核（A10、A13）

## 5. Bytecode 产品与最终验收

- [ ] 5.1 接通方法分析库与薄 JSON CLI；验证 Bytecode/Conservative/Fallback、NotJava、NotAttempted、独立语义证据/verification、阶段 coverage/execution、成员失败隔离、abstract/native 无 Body，库/CLI 逐字段一致（A13）
- [ ] 5.2 加入实际入口的读取/构造计数：单方法不加载无关 Body，P1 X0/X1 不启动 resolver/CFG/SSA/Region/Java AST；重复运行输出身份与顺序一致（A14、A16、A17）
- [ ] 5.3 建立固定 replay 名单的 P2 golden、性质与有预算 fuzz：45–52 历史 class、缺失依赖/debug、非法版本、共享 jsr、异常重叠及资源边界均可到达；复用维护后 harness，断言执行状态和阶段不变量，不仅断言形状或不 panic（A09–A11、A13、A14、A16）
- [ ] 5.4 同步五维支持矩阵、库/CLI 文档和验证记录；运行 fmt、clippy -D warnings、测试及 oracle、MSRV、两个 workspace 的 supply-chain、规定时长 fuzz、OpenSpec strict 和 diff 检查。记录精确 commit/CI run 与各片只读复核结论；确认未宣称 Java/Region 恢复，全部通过才归档
