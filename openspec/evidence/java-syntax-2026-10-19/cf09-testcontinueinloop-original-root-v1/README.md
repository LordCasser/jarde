# CF09 TestContinueInLoop 完整原始基线

原始外层/目标源码完整保留，目标只有构造器与 test([II)V，字段 f:I；外层 JUnit 不调用。最终布尔片 CLI SHA9ea9177b8992c8444c9f9af0158ca7bc99b3e7dfee4696b342aea7da2e295167，实际46命令。原件 debug/no-debug × 两VM共32个有限用例观察与人工计划一致；Jarde八API的目标方法全部fallback，16完整编译均因显式 jarde_refused_body 拒绝，不运行错误结果。JADX两次完整JDK23编译、四VM/32观察全同原件，两次JDK8框架major55/52失败保留；原始JDK8框架边界失败另列。

Jarde fallback 为 arms-do-not-meet 与 loop-shape，map物理44条、已映射11条，其余33条是fallback观察。本基线只确认差距，尚无修复。default/all的全文、身份、raw map相同。parentless/CodeSource/空cwd/全校验及三个错根负向门成立，数组初值在调用前冻结、数组变更与f值独立对照。

capture.tar.gz SHA `f7da343f3738eb66d2d18202c9788560b70262ff41715a2fd084ca5453e483d9`，418个payload条目（文件与目录见root-review）加inventory已root逐字/mode复开；保留四版prep及其错误、全部编译失败和负向空目录、独立review及精确JADX许可。不拷贝JDK/CLI/依赖。独立review复开4383 unique/13892 pin记录；历史schema顶层generated_class_trees为空，但逐compile树已独立重建核对，不冒称其非空。

同时附另外固定CF09 BreakInLoop/DoWhileBreak及SequentialLoops的数学计划，它们不构成运行证据。DoWhileBreak两case只验证可达返回0/5，不能据此要求保留do-while：等价while也可正确，JADX golden只要求while。CF09/71/612整体不关闭。
