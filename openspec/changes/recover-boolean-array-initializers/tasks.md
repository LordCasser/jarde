## 1. 冻结折叠路径与三方基线

- [x] 1.1 复核[独立审计](../../evidence/java-syntax-2026-09-25/boolean-array-initializer/analysis.md)的 Java 8 原/补丁 class SHA、descriptor/atype/五个 opcode 差异、原 JVM 五元素输出、JADX javac 失败和 Jarde 在 consumer BCI 23 引用；root 独立复放 JVM 与当前 Jarde 阶段。
- [x] 1.2 将 BoolInit 的受控补丁类、源码、runner 和 expected 纳入永久 fixture，并构造能通过现有闭包证明的有副作用元素子样本；只在私有目录重建并逐字节核对 class/Code/hash，分别记录原 JVM、JADX、Jarde 修前阶段与执行/异常次数。

## 2. 已证明 initializer 的逐 store 转换

- [x] 2.1 在现有 `prove_array_initializer` 循环保存元素值与真实 store BCI 的有序配对；以原有 int/reference/byte 初始化器回归和 BoolInit 的五个不同 store 来源验证，不从平铺 `owned` 推断位置。
- [x] 2.2 在 `array_initializer_element` 只对已证明 boolean 组件、真实 `bastore`、B/C/S/I 呈现值调用现有最低位 AST；boolean/0/1 保留原写法，以 BoolInit 奇偶/负值完整类 Java 8 重编执行和五个 store 来源验证。
- [x] 2.3 对 producer 单次求值、顺序与异常、未知/不匹配 store 的拒绝、default/all/replay 正文及预算/取消原子停止做定向测试；复跑既有数组初始化器、普通 boolean `bastore`、字段/返回转换回归。

## 3. root 完整执行与验收

- [x] 3.1 root 独立对永久原 class、零引用 Jarde 完整类及 effectful 子样本作 Java 8 编译、`-Xverify:all`、值/trace/异常逐行对照；保存 JADX 原样编译失败阶段，不改它的源码后续测。
- [x] 3.2 root 审读证明配对、每元素真实 store 准入、来源与预算，确认普通数组写入未受影响；其它局部/phi/跨 handler 形状只在 roadmap 分案。
- [x] 3.3 root 更新 fixture 索引和 corpus fingerprint，运行本案/相邻 Cargo 与 JDK 测试、fmt、适当 Clippy、`git diff --check`、`openspec validate recover-boolean-array-initializers --strict`，记录既存门禁缺口并清理私有 Cargo target。
