# DT-06：匿名父类构造实参与捕获参数

这个最小 Java 8 fixture 把三类值区分开：调用 `Base(String, int)` 的两个显式父类构造实参，以及只用于匿名类方法体的捕获局部变量。三个值通过可观察事件记录求值顺序；匿名方法还读取捕获值并调用 `super.render()`。

冻结 class 的 SHA-256 与 `javap -p -c -s` 分别见 [class-sha256.txt](class-sha256.txt) 和 [javap.log](javap.log)。原始冻结类在 `java -Xverify:all` 下退出 0，输出：

```text
arg:capture|arg:super-label|arg:super-value|base:explicit:17
explicit:17:captured
```

## JADX

JADX 使用本地源码 checkout `2fb1b16386941660fda07e9017285aec40fcb37f`。输出源码是 `new Base(text("super-label", "explicit"), number("super-value", 17)) { ... }`：两个实参进入父类构造器，捕获的 `captured` 留在匿名体内，方法体调用 `super.render()`。JADX 生成的两份完整源码以 Java 8 编译成功，`-Xverify:all` 输出与原始 class 一致。生成源码、编译输出和运行输出见 [jadx-source](jadx-source/)、[jadx-javac.log](jadx-javac.log)、[jadx-run.log](jadx-run.log)；工具链记录见 [toolchain.log](toolchain.log)。

这与 `inner/TestAnonymousClass10.java` 的精确断言相同：父类实参必须保留 `this, a2, a2 + 3, 4, 5, random.nextDouble()` 的表达式顺序。`inner/TestAnonymousClass15.java` 补充 `new Thread(run)`、`super.run()` 和匿名实例初始化块 `setName("run")`。

JADX 的相关实现路径是：`ProcessAnonymous.checkUsage` 决定能否内联；`AnonymousClassVisitor.getArgsToFieldsMapping` 跟踪匿名构造器每个参数的唯一 SSA 消费，将写入合成字段的参数认作捕获参数、将 `CONSTRUCTOR` 消费限制为父类构造调用；`InsnGen.inlineAnonymousConstructor` 在确切的分配点用父类构造实参写匿名表达式，并内联匿名类体。此处只能借鉴其构造器实参映射，不能照搬其唯一性规则：`checkUsage` 按构造器的**使用方法数**计数，同一方法的不同 BCI 可能被视作一个使用点；Jarde 必须按物理分配 BCI 证明唯一性。

## Jarde

重放对归档中的三个物理类 `AnonymousSuperArgs`、`Base` 和 `AnonymousSuperArgs$1` 分别请求 `class-source`，再将全部源码一起按 Java 8 编译。当前源码中的分配表达式为：

```java
new AnonymousSuperArgs$1((java.lang.String) text("super-label", "explicit"),
        number("super-value", 17), local1)
```

第三个实参 `local1` 是 `captured`，是匿名构造器的合成捕获参数；前两个实参才是传给父类 `Base(String, int)` 的值。它们的物理顺序是 `String, int, String`，因此不能把“匿名构造器全部实参”当成“父类构造实参”。Jarde 生成的单独子类构造器又按 classfile 顺序写成：

```java
this.val$captured = arg3;
super(arg1, arg2);
```

冻结字节码在 `invokespecial Base.<init>` 之前写入本类捕获字段；这在 JVM classfile 中有效，但 Java 8 源码要求显式 `super(...)` 是构造器第一条语句。完整源码编译因此退出 1。编译器日志只报告这一处构造器错误（[jarde-javac.log](jarde-javac.log)）；它证明此完整源集未通过编译，不能据此宣称其他源码已经全部通过独立验证。Jarde 分类源码、逐类 CLI 日志见 [jarde-source](jarde-source/) 和对应 `jarde-*.log`。

另有一个独立的恢复缺口：Jarde `event()` 输出在 BCI 17 留有 `StringBuilder.append(char)` fallback（见 [jarde-AnonymousSuperArgs.log](jarde-AnonymousSuperArgs.log)）。因此本轮只能证明：完整源码集的唯一 Java 8 编译诊断来自匿名子类构造器中的 `super` 次序；它没有运行 Jarde 产物，也不能作为完整 Jarde 事件逻辑的语义对照。这个 fallback 属于通用表达式恢复边界，记录在此供后续 fixture 选择时避开，不混入 DT-06 匿名类机制。

## 架构复用边界

Jarde 已有可复用的字节码事实：reader 能解码 `InnerClasses`（包含 `inner_name_index == 0` 的匿名项）和 `EnclosingMethod`，且类源码装配测试确认这些事实以类型化结构传递；同一次方法恢复也会采集 `AnonymousAllocationScan`，逐个记录 `new` 的 BCI、目标类、构造器 BCI、有序参数生产 BCI 与完整性。共享的 `init::sites`/`new@1`、物理方法/成员身份、构造器序言恢复和 source map 可作为后续证明及映射输入。

Jarde 的 `class_source` 已把具名非静态成员类做成物理类族投影：先保留父子物理报告，再独立证明捕获、构造调用，最后将有锚点的派生源码范围投影到外层文本。`member_inner::prove_family_capture` 当前证明面很窄：唯一的 `Outer` 类型 synthetic-final 字段、首个物理参数为 `Outer`、且直接父类构造器是零参数。它不能覆盖本 fixture 的匿名 `EnclosingMethod` 关系，也不能区分 `String, int` 两个显式父类实参与末尾 `String` 捕获值。

DT-06 需要补的是一个匿名类族证明，而不是新的全局 classfile 扫描器：用匿名 `InnerClasses` 行和精确的 `EnclosingMethod` 方法身份连接子类；从构造器 SSA/字段身份证明每个物理参数流向父类构造调用还是合成捕获字段；把该构造器连接到同一输入范围内唯一且完整的物理分配 BCI；最后只在这个分配处将父类构造实参与匿名体合并，并隐藏有证明的捕获实参、字段、写入和构造器脚手架。分配实参生产点及其顺序必须保持不变；任一关系、参数角色、构造调用、匿名体或唯一性证明不完整时，保留物理类文本。

主要语义风险是选错/漏掉父类重载参数、改变实参副作用顺序、把捕获值泄露为调用点参数、跨越 `super(...)` 移动构造器效果，以及把多个物理分配点合成一个匿名类而改变类身份。本 fixture 实测了显式参数与捕获参数区分、效果顺序和 `super` 分派；嵌套匿名类、多分配点及跨类引用仍应独立验收。

重放脚本见 [replay.py](replay.py)：它使用临时 `CARGO_TARGET_DIR` 并在退出时清理。工具版本、原始输出、哈希和完整 `javap` 输出均随报告保存。
