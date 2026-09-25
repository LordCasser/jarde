# Generic method reference adaptation

`GenericReference.java` 是普通 `javac --release 8 -g:none` 输入，没有 bytecode 补丁，
也没有 `lambda$` 合成 helper。它以 `ToIntFunction<String>` 与
`ToIntFunction<Object>` 两个目标引用相同的 `pick(Object)` / `pick(String)` 重载。
冻结本轮编译输入为 1014 bytes，SHA-256
`206aca33d0d8bd537a40f50205107eea358addbf4198c714070338da60ff2ef6`。

完整类的原始执行与 JADX 6 项一致。jarde CLI
`c050b502ba3fc23f33937ac607b1da7b51447e02e1a6d41b1d060c2b93b5b8d9`
输出 0 quote，完整重编译成功，但 String 目标的 3 项全部错误：文本/null 从 2 变成 1，
Integer 本应抛 ClassCastException，却也返回 1。Object 目标的 3 项正常。

根因跨越两个现有呈现事实：class-source 使用 descriptor 写出 raw functional interface；
`lambda::plan` 虽读取 SAM 与 instantiated 描述符，但将引用只按同 shape 放行，选择
MethodReference 时没有保存这层动态参数限制。结果两个工厂都输出 `GenericReference::pick`，
重新编译时都选择 Object 重载。普通调用的实参类型修复不覆盖这条 Builder 直接构造 Call 的路径。

`hypothesis.java.txt` 仅把 String 工厂改成
`(java.lang.Object p0) -> GenericReference.pick((java.lang.String) p0)`。
完整候选类的 6 项与原 class 相同。这证明最小修复可以复用 Lambda/Local/Cast/Call，
不必先实现完整泛型 Signature、类型层次搜索或 lambda body 内联。候选代码不计作实现验收。

动态类型可在调用时约束输入，捕获、调用是不同阶段；这些信息由现有 bootstrap
三种 descriptor 提供。规范概述为
[LambdaMetafactory](https://docs.oracle.com/en/java/javase/23/docs/api/java.base/java/lang/invoke/LambdaMetafactory.html)。
扩展审计继续检查实例接收者、构造器和返回转换，避免仅修本例却遗漏相同链中的检查。

`expanded/return-probe/` 同时给出重要的正确行为边界：实现/SAM 均返回 Object 而
instantiated 返回 String 时，raw Supplier 仍能返回 Integer，typed 调用才由调用者
自身 checkcast 失败。三方完整类在两个状态的 4 行均相等。不能把参数适配对称地复制到
返回端并添加 String 检查。OpenJDK 23 的
[转发方法生成器](https://github.com/openjdk/jdk/blob/jdk-23%2B37/src/java.base/share/classes/java/lang/invoke/InnerClassLambdaMetafactory.java#L470)
也按擦除 SAM 返回类型转换实现结果，与这项观察一致。

`wider-target/` 进一步隔离动态参数限制：同样是普通源码，工厂动态参数为 String，
唯一实现方法却接收 Object。原/JADX 的 3 项一致，jarde 0 quote 且完整 javac 成功，
但 Integer 从 `ClassCastException:0次调用` 变成 `7:1次调用`。884-byte 输入 SHA-256
`97e69596434f2fdf8373c74ba29853aa4e267252b949088f2822256ca7f584c3`。
这证明只按 implementation descriptor 改源码类型仍不够；动态 String 检查不能被最终
Object 参数覆盖掉。它与初始重载组独立计数，共 9 项有 4 项真实错值。
