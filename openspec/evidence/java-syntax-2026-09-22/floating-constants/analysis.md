# float/double 常量恢复

2026-09-23基线使用数值比较实现后的debug CLI。自写FloatingConstants共28个语法方法及构造器，覆盖fconst/dconst、常量池浮点值、正负零、最小次正规值、最小正规值、最大值、NaN/无穷、float/double重载实参、嵌套取负和零阈值条件。原class与JADX完整生成类的36行结果一致，比较了原始位模式和重载结果；jarde有56处引用、完整javac失败。`run_audit.py`及所有原始输出在本目录，class只存在临时目录。

## 缺口与已有事实

reader已用ImmediateValue::Float(u32)/Double(u64)和CpEntryKind的bits保存精确内容，frame也认识相应形状。缺口在java层ConstantValue及ExprKind的有限常量集合：decode将这些操作归入Other，AST只提供Integer/Long等字面值。不能把reader改成宿主浮点值后再重建NaN，否则原始位模式会丢失。Atlas scoped查询本次返回src/call_context.rs类型范围校验失败，源代码已直接复核，未运行全仓index。

有必要补忠实的Float/Double常量事实和表达式；不需要新pass、类型解析器或数值运算求值器。结果类型直接由字面值种类声明，沿用现有算术/比较/调用消费与来源路径。

## 拼写候选的独立验证

`spelling-hypothesis/`是方案验证，不是jarde输出。通过原始bits分解符号、指数和尾数，生成精确十六进制浮点字面量；显式f/d后缀保留重载类型，零单独保留符号，次正规值不经过十进制舍入。javac/java对12种边界加256个固定随机值/类型，共536个输入逐个校验raw bits，全部一致。生成Java和脚本均已保存。

该方式只需要现有整数十六进制格式化，无外部浮点转换库，也不需要在产品中执行Java。实际实现仍须接入统一emitter，并扩展已有Neg的负数字面值词法分组，防止`--0.0f`被读成自减；输出字节和来源计费不能另走通道。

## NaN不是可随意归一化的常量

`nan-payloads/`对原class中唯一的Float/Double NaN池项精确改位，分别构造正quiet、负quiet、正signaling三组；未修改Code。三组均通过当前JDK的`-Xverify:all`，每组36行执行，其中两个NaN方法保留给定符号及payload。JADX均重编译成功，但每组两行raw-bit结果错误，共6处，把它们归一为默认NaN。jarde当前仍引用，尚未引入该错误。

因此不能只用宿主`is_nan()`后输出一个统一NaN，也不能把数值比较中的“NaN结果相同”外推为常量位模式相同。是否使用已有Call表达式呈现bit-conversion工厂、哪些NaN能作精确保证及额外调用语义，须先完成架构判断；当前未派发该生产实现。无穷与通常NaN可用Java标准常量表达式呈现，但仍需验证实际javac位模式，不据名字猜测。

依据：[JLS 3.10.2](https://docs.oracle.com/javase/specs/jls/se23/html/jls-3.html#jls-3.10.2)允许十六进制浮点字面量，并区分浮点类型后缀；[Float.intBitsToFloat](https://docs.oracle.com/en/java/javase/23/docs/api/java.base/java/lang/Float.html#intBitsToFloat(int))明确说明signaling NaN在某些处理器上的复制可改变位模式。这里的实测只证明当前JDK/平台，不把它推广为跨平台payload保证。

## 进一步确认：常量后的真实取负

`special-spelling-hypothesis/`的八个表达式实测表明，javac把正负号两种NaN写法都折叠为正quiet NaN。`nan-negation/`随后在原class两个NaN常量后精确插入fneg/dneg，并调整Code长度；合法补丁经-Xverify:all得到`ffc00000`及`fff8000000000000`，而独立候选源码`-(0.0 / 0.0)`仍输出正NaN。完整JADX输出也重编译成功且有同样两处位模式差异，详见该目录summary及原始输出。这不是jarde已输出的错值，而是实现前发现并排除的候选方案缺陷。

方案现已收成`recover-floating-point-constants`：有限值精确叶子，正负无穷和标准正quiet NaN复用已有常量表达式；其它NaN明确引用。还须在已有算术/取负构造处拒绝不能证明逐位等价的闭合浮点操作树，保留所有必要BCI，不能只补字面值后把javac常量折叠问题藏起来。该判断局部有界，不新增浮点求值器或运行时bit-conversion调用。规划已strict通过，生产尚未派发。
