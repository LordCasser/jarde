## 1. 扩展 class-level int argument proof

- [ ] 1.1 为 literal、一次 `getstatic:I` 和字段读加一个 int literal 建立闭合的证明值，并由同次 Code facts 精确绑定操作数顺序、BCI、constructor call 和常量字段写；运行 enum proof 单测确认现有 literal control 保持通过、fixture 三种已声明形状分别得出完整组。
- [ ] 1.2 在 class-source 常量投影中发射证明过的字段读与加法实参，复用现有 Java field/arithmetic 拼写且保持 getstatic 次数与参数求值位置；对 IntArgs 使用 javac `--release 8` 重编并运行 runner 验证三个值与顺序。

## 2. 保持拒绝边界并验证整组

- [ ] 2.1 为方法调用、第二次静态字段读取、非直接加法、constructor/ref 不匹配、initializer suffix/use census 不满足分别添加 refusal fixtures；运行 class-source 检查完整 group 原子拒绝且不部分发出 enum constants。
- [ ] 2.2 用完整 enum fixture 验证 original 与 Jarde 重编运行结果；执行相关 enum/class-source 测试和 strict OpenSpec validation，确认预算/取消与现有 class-source contract 未被新 grammar 绕过。

## 3. 保持 String varargs 独立

- [ ] 3.1 运行 DT-11 frozen replay 并保留 `StringVarargs` 的现有拒绝和 JADX/source 对照；确认当前实现及任务清单没有引入 String[] constructor 参数或 Signature 尾参支持。
