# 延期值跨独立语句的语义错误

本轮以实际patched class为oracle。`run_audit.py`编译自写Java8源码，使用池中已有的mark方法项，在四个方法的最终return前插入独立调用，只改变对应Code与属性长度。补丁类通过`java -Xverify:all`。未打补丁的源码仅说明生产者形状，不能当作最终执行顺序的oracle。

| 输入 | 原class | 当前jarde | JADX |
| --- | --- | --- | --- |
| call、field、array、nullArray、goodCast、badCast，各3个失败模式 | 18行合法执行 | 0处引用、完整javac成功，13行不同 | 完整javac成功，18行一致 |
| `checks/`中的int/long除法与取余、实例字段、数组长度 | 24行合法执行 | 0处引用、完整javac成功，13行不同 | 完整javac成功，24行一致 |
| `allocations/`中的原始/引用/多维数组分配 | 12行合法执行 | 0处引用、完整javac成功，6行不同 | 完整javac成功，12行一致 |
| `../construction-consumers/interleaved-effect/`中的new，各3个失败模式 | 3行合法执行 | 0处引用、完整javac成功，3行不同 | 完整javac成功，3行一致 |

调用和构造的轨迹从12变21，字段已求得值5变9，数组已求得值7变8；本应先发生的整数除法/取余异常、负数组长度异常、null/类型检查可能被后续mark的异常替代。因此问题不是输出美观或未覆盖语法，而是已发布正常源码的行为错误。

根因是Builder的延期准入只证明值存在受支持的读取者，随后render_value在最终消费者处重新构造生产表达式。SSA身份能够回答读取哪个值，但不能授权在独立语句之后重新调用、重新读取或重新检查。现有deferred列表用于失败引用，不是已计算值的保存。仅增加init的消费者名单会扩大错序面。

`preserve-deferred-value-order`已单列规划。修复复用Declare/Local、现有类型和NameTable，在生产者的位置保存需要跨语句保留的值，以Builder内的SSA身份绑定供后续读取。这是必要的最小新状态；不会伪造JVM槽或新增IR、调度pass、类型解析器。共享位置证明必须区分同一嵌套表达式与独立语句，并守住Java作用域、异常范围和循环执行次数。

优先级高于instanceof、位运算和new→cast/aastore覆盖扩展；当前final-static窗口完成后串行实现。该文档记录的是修前实测，未声称已修复。

`exception-scope/`另存三项合法异常区域对照。原class与JADX一致；当前jarde在guard准入阶段明确引用并编译失败，没有发布正常错序代码。这是位置证明必须保留的范围边界，不并入上表57项正面，不为修复延期值而重写guard。

`structured-scopes/`进一步验证已约定的区域正面：分支内直线及if测试前缀共12项，原class/JADX一致；jarde0引用且整类编译通过，但9项错序。与最初57项合计69项、44项不同。该独立输入需要真实有名值保存，不能用未插入独立调用的普通if控制代替。
