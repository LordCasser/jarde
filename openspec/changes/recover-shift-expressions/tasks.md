## 1. 固定纯移位与类型边界

- [x] 1.1 将 root-core 的 723 项独立输入收成一份最小永久 Java8 class 及 source-only helper/runner，补充 short、右侧同优先级嵌套与调用目标对照；确认 class hash/Code 和原/JADX/jarde 完整基线，不能让位运算或 l2i 混入基本正面类。root 统一冻结 census/fingerprint。证据见 [十方法、11 条移位的小切片和独立 723 行重放](../../evidence/java-syntax-2026-09-24/shift-root-core-slice/analysis.md)；[宽度转换三方小样](../../evidence/java-syntax-2026-09-24/shift-width-cast/analysis.md)另固定 `(long)x << 32` 与 `x << 32` 的行为差异。两组都是实现前 RED，不代表 2.x 或整类语法恢复完成。
- [x] 1.2 使用内存 descriptor/Code 变体固定 boolean 左值/距离、旧局部及拒绝消费者来源；原 class 经 JVM 实际验证，记录可保存或应拒绝的差别，不新增永久负例类。证据见 [负边界验证与来源测试](../../evidence/java-syntax-2026-09-24/shift-negative-boundaries/analysis.md)：descriptor 变体在 `-Xverify:all` 下有效但 boolean 移位被拒绝；旧局部重写保持原值及移位 BCI，boolean 返回消费者在 `ireturn` 的实际 BCI 留有拒绝来源。无 verifier-invalid 字节被用作负例，也没有提交负例 class。

## 2. 闭合现有表达式路径

- [x] 2.1 将六个 opcode 接入已有事实并增加三个 BinaryOp，沿用左值独立整数提升，验证 byte/short/char、int/long 和 boolean 拒绝；不补数值转换或新类型框架。
- [x] 2.2 通过统一 emitter 排列乘除、加减、移位、关系、相等及已支持位运算的绑定强度，测试左右嵌套、UNARY/PRIMARY 与实际 javac 分组，不通过局部拼字符串补括号。
- [x] 2.3 在既有 render/reader/refused-producer 路径消费真实 SSA 操作数及最终 at，共享已验收的求值位置/保存机制；验证调用顺序、任一侧抛错、跨独立语句与旧局部，不扩大区域准入。[有副作用与异常顺序的三方整类重放](../../evidence/java-syntax-2026-09-24/shift-effect-order/analysis.md)显示原/Jarde 3/3 行一致；JADX 1.5.6 的输出虽可编译，但把字段读前移而 3/3 行改变。
- [x] 2.4 验证 essential/all 文本相等、实际 BCI/member 来源、IR/工作/来源预算、取消及深度停止，commit/replay 不重复发射或伪造来源。[焦点回归](verification-2.4.md)确认文本及来源契约和可控停止路径；恢复递归深度是无调用方配置的内部上限，现有深表达式夹具依赖运行时 javac 且处于 ignored，因此未伪造深度边界覆盖。

## 3. 完整执行与独立验收

- [x] 3.1 用实际恢复的完整类原样 javac/执行基本 723 项及新增对照，与原 class/JADX 分开比较；原 ShiftAudit 位运算组合和 long 距离 200 项按相邻功能状态另列，不冒称属于本项基础覆盖。[root 独立重放](verification-2.2-and-3.1.md)固定最终 CLI、原/JADX/Jarde 三方 723/56/3 行结果与宽度差异；来源/预算及相邻复核另见 2.4、3.2，整仓门禁 3.3 未关闭。
- [x] 3.2 root 独立重放 root-core、边界与完整永久夹具，复核左值类型、效果保存、括号和失败来源，复跑算术/位运算/比较/negation/invocation/deferred-order/预算相邻回归。证据：[Root 独立验收](verification-root-3.2.md)；相邻比较来源与转换固定计数的既存失败另列，未混入移位实现。
- [ ] 3.3 root 统一验收 Java 包、census/fingerprint、fmt、严格 clippy 和 OpenSpec strict；记录确切既存门禁债务及数值转换边界，不顺带修改无关模块。
