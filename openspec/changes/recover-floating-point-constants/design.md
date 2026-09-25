## Context

`../../evidence/java-syntax-2026-09-22/floating-constants/`保存完整基线、原始位模式、独立拼写实验和NaN池补丁。reader的ImmediateValue及CP项已保存u32/u64，frame已区分Float/Double；Java层ConstantValue与ExprKind缺少相应值。现有literal仅被render_value使用，decode把这些常量归为Other。无需修改解析、验证或目标运行时选择。

有限值536项独立实验验证了精确十六进制拼写。八个特殊值表达式实验验证正负无穷和标准正quiet NaN；**对NaN加负号仍产生相同正NaN**，不能据此恢复负NaN。三组特殊NaN池补丁在本地JDK保持原bits，JADX每组丢失两个位模式。后续binding-hypothesis验证非final局部能使fneg/dneg留在运行时，两项raw bits与原class一致；它仍是方案实验，不代表当前jarde已支持。

## Goals / Non-Goals

**Goals:** 填补既有常量消费链的缺口，以原始bits为唯一输入；只接受本项可精确表示的值。证明范围见proposal，不把本地NaN实验扩大为跨平台signaling NaN保证。

**Non-Goals:** 不做浮点求值优化、十进制最短字符串算法、任意NaN构造、字段声明初始化或新区域识别。JDK编译执行只是自写输入的验收，产品不执行目标代码。

## Decisions

1. ConstantValue增加保留原始bits的Float(u32)/Double(u64)，既有fconst/dconst ImmediateValue和ldc/ldc_w/ldc2_w CP事实直通，不转换成宿主f32/f64再比较或重新编码NaN。有限值表达式增加对应叶子，并由叶子直接声明Float/Double类型；这是已有常量缺失的忠实实体，无需通用数值类或第二套类型推断。
2. 有限叶子只承载有限bits。emitter用整数分解符号、指数和尾数生成精确十六进制字面量；float尾数23位左移一位，写6个十六进制位，double写13位，指数使用各自偏置。零单独保留符号，次正规值使用零整数部及最小正规指数，后缀f/d始终明确。文本从统一put/node输出，不能绕过输出预算或先拼整方法。使用现有整数格式化即可；引入浮点格式化库会增加依赖维护和许可面，且不能解决NaN语义，本项无收益。
3. 正负无穷与标准正quiet NaN分别由既有Binary(Divide)及有限的±1/0、0/0表达，保持float/double类型，复用现有优先级与分组。它们是原常量的呈现，子表达式和除法均从该常量BCI派生，不伪造独立原始指令。没有必要新增Infinity/NaN节点或构造运行时Call。
4. 只接受NaN bits `0x7fc00000` / `0x7ff8000000000000`。其它NaN的符号、payload或signaling状态不可归一化。共享的位模式准入判断由既有常量构造、指令呈现及失败生产者路径复用：不支持的Push必须在自身BCI明确引用，消费它的表达式不能被误称为已恢复；必要引用仍包含常量和消费者。不能仅改decode为Push后把原本可见的Other引用吞掉，也不把未呈现值当成可重放表达式。
5. 不用Float.intBitsToFloat/Double.longBitsToDouble补全其它NaN：这会把无调用的常量改为静态调用及类初始化，并且平台可能在signaling NaN传递中改变bits。该边界有原始字节码和明确拒绝，优于默默改变可被raw-bit API观察的值。若以后支持，应独立证明这种转换的语义与可移植性，不在本项发散。
6. 保持既有最终消费者at、消费一次、局部类型及quoted生产者协议。数值比较中的纯值常量自然复用；不得重新选择NaN分支极性。取负的既有词法分组要识别负有限字面值的符号位，尤其-0，否则可能生成`--0.0f`。表达式穷尽遍历、来源及commit/replay同步适配，无新pass、全局索引或缓存。
7. 不能假定javac的常量折叠与原JVM操作序列逐位一致。`nan-negation/`的合法补丁直接在标准NaN的ldc后增加fneg/dneg，原class输出负NaN，`-(0.0 / 0.0)`却被javac折叠为正NaN。`binding-hypothesis/`证明将canonical NaN先存入非final局部再取负，可在本地记录环境保留两种负NaN bits。因此本项依赖先完成`preserve-deferred-value-order`的有名值保存：在既有算术/取负构造中，用深度有界的AST判据识别真实JVM操作将变成闭合Java常量表达式的情形，复用同一生产位置、声明提交及SSA绑定，使必要操作数成为非final局部，保留运行时操作。不能再建浮点专用临时变量表、命名器或调度pass，也不执行浮点常量求值。有限叶子及可证明不改变bits的有限取负无需额外保存；非恒定树也不因此新增局部。若现有绑定无法证明放置、类型或作用域，明确引用必要常量、操作及消费者，不能冒险折叠。标准NaN直接返回、存储、实参及比较继续正常支持；未支持的NaN payload仍受第4项限制，不能借保存局部放宽准入。
8. 所有新浮点Push在失败生产者追溯中保留自身BCI，闭合常量树无法安全保存时也保留相应操作BCI，避免只留下最外return。原常量合成出来的除法与真实JVM除法必须以来源/构造路径区分：前者是已证明常量的写法，不再经过真实操作的准入判定；后者不能借用前者的许可跳过折叠边界。

依据：[JLS 3.10.2](https://docs.oracle.com/javase/specs/jls/se23/html/jls-3.html#jls-3.10.2)及[Float.intBitsToFloat](https://docs.oracle.com/en/java/javase/23/docs/api/java.base/java/lang/Float.html#intBitsToFloat(int))。观察到的特殊值raw-bit保证以明确记录的JDK/平台验收为准；不声称Java算术规范区分全部NaN payload。

## Risks / Trade-offs

- 次正规值舍入或负零消失 → 边界与固定随机536项必须经实际恢复正文重编译，再比较raw bits；方案实验不能代替实现验收。
- float被写成double导致选错重载 → 所有叶子及合成特殊值保留类型，以不同返回值的重载运行验证。
- 特殊值的合成除法分组改变周边语义 → 固定嵌套乘除、取负、比较及实参，使用统一优先级而非裸字符串。
- 新Push吞掉NaN拒绝证据 → 本身及失败消费者均有真实来源，覆盖直接返回、存储、算术和已延期调用组合。
- 新叶子或合成常量绕过计费 → 树规模固定有界，沿用构造/发射计费；验证输出不足、来源不足、默认与完整请求文本一致。
- javac折叠闭合操作树改变NaN符号 → 用合法fneg/dneg和0/0字节码验证真实运行时操作，复用已验收的有名值保存；无法证明位置则拒绝，不引入浮点求值器。
- 相邻语法点共用生产文件 → fixture可独立准备，生产与Cargo仅在root移交后使用。
