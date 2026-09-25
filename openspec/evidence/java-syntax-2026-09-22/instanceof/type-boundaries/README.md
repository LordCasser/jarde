# instanceof 操作数的源码静态类型边界

InstanceOfTypes 是自写 Java8 源码，原 class 经 javac 和 `java -Xverify:all` 通过，11行输出保存在 original.txt。六个测试方法覆盖 String→Integer、String[]→int[]、String→Runnable、相关 CharSequence 对照、返回 String 的有副作用调用及命名方法引用（另有空void目标）；调用另有自身抛错输入。

前三种不相关类型使用 `(Object) value instanceof Target`，javap 明确显示加宽未留下 checkcast。原 class 合法，不是手工非法字节码。若恢复只按 JVM 指令写 `value instanceof Target`，Java 的静态类型规则会拒绝。当前 jarde 尚不恢复该 opcode，完整输出 javac 失败；实际 JADX 输出也省掉了加宽，在前三方法与 called 共四处编译失败，不能执行比较。相关 CharSequence 为正常对照。

naive-hypothesis-javac.log 是从原源码单独构造的“删去 Object 加宽”假设实验，明确不是 jarde 生成结果，不计入产品验收。run_audit.py 保留每一步；jadx 原包与生成方法体均保持不变，只有 source-only helper/runner 放入对应包。

架构结论：只增加 instanceof 事实和boolean表达式还不够。已有 Expr.presented 需约束左操作数的 Java 拼写；对具体引用类型可用现有 Cast 安全上溯 Object，避免为判断编译期 cast-convertibility 引入外部继承解析器。Object/null 不必重复包装，已有运行时 checkcast 仍必须在内部保留，不能把这些已知 false 测试折叠为常量从而丢失生产者调用/异常/目标类型操作。最终仍在实际消费者处递归呈现并验证求值位置。

命名方法引用还需先保留工厂Runnable目标，不能直接cast到Object；其字节码是invokedynamic后直接instanceof，中间没有checkcast。调用参数恢复已有这一有限目标类型逻辑，可复用其Cast步骤；原执行functional=true，不需要增加lambda识别模式。

依据：[JLS 15.20.2](https://docs.oracle.com/javase/specs/jls/se8/html/jls-15.html#jls-15.20.2) 与 [JVMS instanceof](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-6.html#jvms-6.5.instanceof)。受控源码与 javac/javap/执行结果比静态阅读更直接固定了本项边界。
