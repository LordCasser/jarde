# 普通参数化方法 Signature：架构核查

此文件只记录源码接缝和待验证的设计判断；三方运行与反例见同目录的 `analysis.md`。目标是 Java 8 class 的普通参数化方法类型，如 `Iterable<String>`，不把方法自有 `<T>` 的在途变更并入本项。

## 事实流

1. `jarde-reader::signature::parse_method_signature` 已有带字节、深度、节点和预算上限的完整语法树；`prove_method_signature_erasure` 按参数/返回位置逐一对真实 descriptor 做擦除证明。`ClassTypeSegment` 和 `TypeArgument` 保留分段内类、精确参数与三种通配形态。query 也消费该共享结果，无需第二套解析器或新 crate。
2. `src/class_source.rs::spell_method_declaration` 目前只用 descriptor 的 `Signature`（此处是本地的已擦除参数表）写方法头；`project_method_signature` 虽会读取同一物理成员的 `Signature` 属性并调用 reader 擦除证明，却只允许方法自有单一类型变量、静态方法及少量直接返回形状。普通 `Iterable<String>` 会落到拒绝分支。
3. `crates/jarde-java/src/build.rs::iterable_for_each_candidate` 在方法体阶段以 `Object` 建立增强 `for` 的元素类型，保留原 `checkcast String`。这个结果可执行；单凭 `checkcast` 不能把 raw `Iterable` 的方法声明写成参数化类型。类级 `project_method_signature` 在正文恢复以后运行，故单改最终声明字符串也不会改变已构建的循环 AST。
4. JADX 本地 `SignatureProcessor` 在 `TypeInferenceVisitor` 之前把解析后的方法参数/返回类型写入 `MethodNode`，再由 `MethodGen` 输出。其 `validateParsedType` 用类型比较的非 `CONFLICT` 结果准入，允许 `UNKNOWN` 的一部分形状；Jarde 应继续使用同一物理成员的严格逐位置擦除，而不是复制这一宽松判断。JADX 先传播类型再恢复循环的顺序有参考价值，但不能照搬没有 SSA/异常边界证明的结构改写。

## 分开的两个投影决定

**方法声明**：可复用 reader 的结构化类型及擦除证明，在类源码装配中产生完整声明候选。候选须用现有参数 slot/名字、`Exceptions`、varargs 与注解接缝构造，不按字符串替换旧声明；只有源级名称、上下文类型变量、正文使用和受影响调用绑定均可证明时才原子发布。合法但未覆盖的 Signature 保留物理 descriptor 声明与明确拒绝原因。类自有类型变量若类头尚未呈现，不可在方法声明中孤立使用。

声明候选还要区分 `Recovered` 方法与 `NoBody` 的 abstract/native/interface 成员：后者没有正文可证，仍有泛型反射签名与 Java 头部；不能把当前只处理 `Recovered` 的发布函数当成方法 `Signature` 的完整入口。构造器可能存在合成外部实例参数，参数个数与 `Signature` 不同，需先证明隐式参数映射；当前 reader 的逐位置擦除结果不可直接跳过这一差异。方法/参数/type-use 注解的目标路径也须逐项确认，避免把注解留在错误的类型层级。

**增强 `for` 元素类型**：是后续独立投影。只有方法声明已证明为参数化来源，并能把同一参数 slot 的源类型作为方法体构建事实传入，才可考虑用 `String` 绑定并删除已被该类型蕴含的原 cast。它还需要单独证明 `next()`、cast、handler、SSA 消费及异常/副作用位置；不应为外观生成 unchecked `(Iterable<String>) raw`。现有 `Object` 头加原 cast 在此期间保持有效。

## 待三方证据裁决

- 有/无调试表下，原 class、JADX、Jarde 的方法泛型反射与完整类重编/执行差距。
- `List`/`Set` 等同长度原始类型被篡改的 JVM 可验证 `Signature`：Jarde 的逐位置擦除必须拒绝；观察 JADX 是否接受。
- 参数化返回和参数使用在重载、转换、继承和通配符上的源级绑定边界。不能以反射一致或简单运行值相同替代完整类语义。
- 字段/类泛型与方法声明是否发生上下文耦合；若有，另立债务，避免把本项扩成全类泛型系统。
