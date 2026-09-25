## 1. 冻结输入与边界

- [x] 1.1 固定 `class-literals/` 的851 B主 class、七项原 class/JADX可执行且相等、jarde 11引用与整类五处缺返回的源/JADX/jarde原样对照；root 复查 `summary.json` 与完整运行输出。
- [x] 1.2 从源审计建立永久 Java 8 fixture/runner，固定 `ldc` 池项、BCI/CP、数组 descriptor、七项输出与前置 RED；补 MethodType/MethodHandle 或不可读类名的拒绝，以及局部同名不遮蔽/类型同名可改绑定的正反对照，证明边界不被误认。

## 2. 只恢复已验证的 Class 常量

- [x] 2.1 仅将真实 Class 池项的 `ldc`/`ldc_w` 解为类常量事实，用内部名/数组 descriptor 解析可写类型；单测覆盖对象、两种数组与非法/非Class池项，保持 `TYPE` 静态字段路径。
- [x] 2.2 在现有常量表达式链增加最小 `T.class` 节点及 `java.lang.Class` 呈现类型，复用返回/调用消费者；永久测试确认五个原拒绝方法含正确字面量、无生产者复制、原有两个基本类型/void方法不变。
- [x] 2.3 保留真实 `ldc` BCI/池项及消费派生来源，类型名冲突时安全限定或来源拒绝，不因同名局部误拒；永久来源、默认/all一致、低预算/取消测试通过。

## 3. 整类语义与 root 验收

- [x] 3.1 原样重编译执行永久完整类，七项原 class/JADX/jarde 的类型身份、数组层数、调用次数逐项相同，jarde `@bytecode` 为0；JADX必须先通过完整源码编译才用作运行对照。
- [x] 3.2 root 独立审读池项准入、类型路径绑定、表达式静态类型与来源；重放完整类及非Class拒绝，复跑常量、数组类型、调用、qualifier和deferred相邻回归。
- [x] 3.3 root 统一 reader census/fingerprint、fmt、适当 Cargo 测试及 OpenSpec strict；浮点/基本类型 `.class` 形态债务保持独立。
