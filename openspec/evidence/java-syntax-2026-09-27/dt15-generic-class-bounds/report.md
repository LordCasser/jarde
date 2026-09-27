# DT-15：顶级类与接口的泛型形参上界

固定 JADX 基线为本地 `2fb1b16386941660fda07e9017285aec40fcb37f`。相关测试 `TestUsageInGenerics` 虽含成员类 `B<T extends A>`，实际源码断言只检查 `public <T extends A> T test()`，主要落在 DT-16；`TestGenericsMthOverride` 的断言只检查四个方法头与 `@Override` 次数，不直接断言类/接口的形参声明。因此这里不能仅凭两个测试将 DT-15 判为已追平。

本目录构造了有界正例 `Bounds<T extends Number & Comparable<T>>`、`Contract<X extends Number>` 和无泛型的 `Plain` 控制项。它们只含类/接口形参上界与普通无参构造器，故不会把 DT-16 方法泛型或 DT-18 字段/方法参数化类型混入结论。[replay.py](replay.py) 先以 `javac --release 8 -g:none` 编译，再用固定 checkout 的 JADX 与当前 Jarde CLI 输出三个完整类型；每套输出分别加同一外部 API consumer `Runner.java` 重编，并用 `java -Xverify:all` 读取反射泛型上界。调用方式是 `python3 replay.py /absolute/path/to/jarde-cli`。固定输入、原 class 与双方源码的哈希、Java 8 编译/验证运行结果见 [results.json](results.json)，实际输出保存在 `outputs/`。

原始/JADX/Jarde 的完整类型源码均可重编；三者运行结果逐字相同：

```text
T:java.lang.Number:java.lang.Comparable<T>
X:java.lang.Number
0:true
```

Jarde 当前 `ClassSourceDeclaration::project_generic_signature` 已读取 class `Signature`，解析并核对父类/接口擦除，然后拼写形参和上界。这一顶级薄切片无需新增机制或 OpenSpec 实现任务。它的明确边界是成员泛型类型、参数化父类/接口、type-use 注解、泛型字段和方法签名；上述 JADX 测试还含成员类型或方法覆盖，不能从本次通过推断那些组合也已恢复。DT-15 保持“部分已测”，后续按本固定队列继续验收成员组合。
