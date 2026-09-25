## 1. 固定调用点证据与拒绝门

- [x] 1.1 冻结 `SimpleOuter/UseInner` 及前置/嵌套效果对照的 Java 8 class、SHA、`javap`、原/JADX/Jarde 运行结果；核对原与 JADX 重编的 `-Xverify:all` 输出，见 `simple-member/analysis.md` 和 `negative-controls/analysis.md`。
- [x] 1.2 构造并验证目标及外层各自的 `InnerClasses` 关系错误、物理首参与限定值不同身份、空值检查在参数效果之后、目标缺失的独立控制；能做成 verifier-valid class 时冻结执行和三方结果，否则写直接 proof-unit 拒绝测试并明确证据级别，验收为每个门都有红/绿断言而非推测。

## 2. 物理成员关系交接

- [x] 2.1 在 class-source 的选定环境中按需、带预算地唯一绑定构造目标及其外层定义，复用 typed `InnerClasses`/`EnclosingMethod` 与现有 class 读取，并互证目标和外层的成员关系；用同名多定义、缺失目标、外层关系错形及预算/取消测试证明不越过物理选择边界。
- [x] 2.2 在目标类内验证公开非泛型成员关系、准确 `<init>` descriptor、唯一的合成外层字段及 prologue 的首参写入，向调用点交付最小的内部事实；用篡改关系/descriptor/捕获字段控制证明任一缺证都拒绝。

## 3. 构造站点与语义 AST

- [x] 3.1 在现有 `new@1` 中证明分配实例唯一、隐式首参和限定值的 SSA 身份、准确 `requireNonNull` 栈形与区间、普通实参顺序；单测覆盖 1.2 的每条拒绝门、区间内独立 `void` 调用及普通 `new` 不变。
- [x] 3.2 扩展现有 `New` 表达式以承载已证明的限定接收者，发射 `outer.new Inner(args)` 且源级参数不含物理首参；编译器穷尽遍历并用源映射、预算/取消及旧普通构造回归验证所有消费者。
- [x] 3.3 验证新增成员站点的 `new@1` 记录与已提交正文一致，保持物理实参 BCI 含义及 essential/all/range 的正文与决定一致；定向报告测试验证成功、保守拒绝和停止。若发现普通站点的普遍报告债务，另建 change 处理。

## 4. Root 独立验收

- [x] 4.1 用冻结的原成员 class 作依赖，只重编调用方 Jarde Java 8 源；原/JADX/Jarde 的成功、null、前置效果和嵌套效果输出与次数逐行对照，方法质量、来源 BCI 与缺失/错误目标拒绝均记录在 `verification-root.md`。
- [x] 4.2 跑适用的 `cargo fmt`、定向测试、Clippy 与 `openspec validate --strict`，记录既有失败而不混改；清理本任务私有 Cargo target，验收为磁盘空间回收及验证文档完整。
