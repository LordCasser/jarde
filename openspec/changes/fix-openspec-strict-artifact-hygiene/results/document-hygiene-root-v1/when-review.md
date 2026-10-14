# Complete contract scenario WHEN review

1. Standard multi-release selection is evidence-backed [baseline strict long-body warning]
   WHEN 系统处理按请求 RuntimeView 和 scope 生成 container 的 MR selection report

2. Multi-release compliance diagnostics are bounded and non-selecting [baseline strict long-body warning]
   WHEN 系统处理物理路径、Manifest 与 bounded class Header 的 MR 合规检查及诊断

3. Bounded nested and Boot layout traversal [baseline strict long-body warning]
   WHEN 系统处理 physical snapshot 枚举与显式 artifact-tree 请求的 nested、Boot 和 WAR 布局展开

4. Evidence-gated Java 8 recovery [baseline strict long-body warning]
   WHEN 系统尝试从 IR、descriptor、异常/effect 与编译器模式证据恢复并呈现 Java 8 方法

5. Recovery artifact content is observable independently [baseline strict long-body warning]
   WHEN 系统处理恢复报告与产物交付

6. Recovery recursion is bounded and a stop is published [baseline strict long-body warning]
   WHEN 系统执行恢复调用（包含递归下降）

7. Boolean contexts are presented as booleans [baseline strict long-body warning]
   WHEN 系统尝试呈现 boolean 返回、分支测试与其它值上下文

8. One local variable's type is decided once, before its statements are built [baseline strict long-body warning]
   WHEN 系统处理既有 LocalVariable 身份的类型决定、声明规划与语句构建

9. A presented concatenation keeps each append's own conversion [baseline strict long-body warning]
   WHEN 系统尝试呈现 StringBuilder 或 StringBuffer 的 append 拼接链

10. A proved prefix survives a local gap [baseline strict long-body warning]
   WHEN 系统处理方法恢复中的已证明前缀、当前形状、分支与汇合后的走法

11. A named catch is written only when the region is not a refused guard [baseline strict long-body warning]
   WHEN 系统判断并恢复异常表保护范围及处理器

12. A proved lambda or anonymous body is shown at its use site [baseline strict long-body warning]
   WHEN 系统处理类源码视图装配及 lambda 与匿名类使用点呈现

13. Class source spells package, throws, and varargs from the class file [baseline strict long-body warning]
   WHEN 系统处理来自 class file 的类声明、throws 与 varargs 呈现

14. A special call, a compare, and a constant field keep the program they came from [baseline strict long-body warning]
   WHEN 系统尝试呈现特殊调用、比较与常量字段

15. Constructed sources are accepted by a three-way comparison [baseline strict long-body warning]
   WHEN 系统验证构造得到的源码

16. 无 child report 的成员 family refusal 不阻止同类整数常量名称 [baseline strict long-body warning]
   WHEN 系统处理同类整数常量名称投影及其成员 family 准入

17. Complete class source projects multiple proved enum switches in one method [baseline strict long-body warning]
   WHEN 系统尝试在完整 class source 的同一方法中投影多个 enum switch 站点

18. Proved enum constants preserve bounded integer constructor arguments [baseline strict long-body warning]
   WHEN 系统尝试恢复 enum 常量的整数构造实参

19. Interface-qualified super calls require a legal source qualifier [baseline strict long-body warning]
   WHEN 系统判断接口限定 super 调用的源级 qualifier 合法性

20. Coverage and pagination are explicit [baseline strict long-body warning]
   WHEN 系统执行查询请求

21. 根方法返回父类直接超类型的匿名类可内联 [baseline strict long-body warning]
   WHEN 系统处理根方法返回父类直接超类型的匿名类内联

22. 捕获形伴生构造器的 super 次序 SHALL 可编译 [baseline strict long-body warning]
   WHEN 系统尝试恢复捕获形伴生构造器的 super 调用次序

23. Debug names preserve proven distinct reference lifetimes [baseline strict long-body warning]
   WHEN 系统判断引用局部生命周期的 debug names

24. 分配限定符的成员构造在真 javac 8 的 null 检查下 SHALL 恢复 [baseline strict long-body warning]
   WHEN 系统处理分配限定符成员构造并核验被丢弃的 null 检查

25. 限定外部实例构造的 null 检查判据 SHALL 识别两种编译器拼写 [baseline strict long-body warning]
   WHEN 系统判断限定外部实例构造的编译器 null 检查拼写

26. lambda 对原生数组捕获的 SHALL 以有据类型参与一致性判据 [baseline strict long-body warning]
   WHEN 系统尝试恢复 invokedynamic lambda 站点的原生数组局部捕获

27. Multi-resource try-with-resources proves its nested levels [baseline strict long-body warning]
   WHEN 系统尝试恢复两个及以上资源的 try-with-resources

28. A protected prefix and loop retain one exception scope [baseline strict long-body warning]
   WHEN 系统尝试恢复 named catch 保护的前置语句与循环序列

29. Proved enum constants recover literal String varargs [baseline strict long-body warning]
   WHEN 系统尝试恢复 Java 8 普通 enum group 的 String varargs 构造器与常量实参

30. 外层 continue 与内层循环退出共享 latch 时可证明 [baseline strict long-body warning]
   WHEN 系统判断外层 continue 与内层循环退出共享 latch 的 CFG 证明

31. Structured parameterized List bodies are projected as one source decision [baseline strict long-body warning]
   WHEN 系统尝试投影结构化参数化 List 方法的声明与正文调用

32. 真 javac 8 的 TWR 关闭序列 SHALL 恢复为源级 try-with-resources [baseline strict long-body warning]
   WHEN 系统尝试恢复 javac 8 try-with-resources 关闭序列

33. 返回值形写访问器 SHALL 按字段描述符恢复（封闭类型集） [baseline strict long-body warning]
   WHEN 系统尝试恢复返回值形合成字段写访问器

34. Separate representation and validation statuses [baseline strict long-body warning]
   WHEN 系统发布 source result 或验收返回值、可观察行为与 fallback 来源

35. A signal is not an acceptable answer [baseline strict long-body warning]
   WHEN 系统验收曾以进程 signal 结束的恢复输入

36. A boolean context's artifact compiles and agrees when executed [baseline strict long-body warning]
   WHEN 系统验收 boolean 上下文的恢复产物与拒绝区域

37. A local's decided type compiles and agrees when executed [baseline strict long-body warning]
   WHEN 系统验收局部类型决定的恢复产物与拒绝区域

38. A presented concatenation is compared by execution [baseline strict long-body warning]
   WHEN 系统验收拼接链的恢复产物与拒绝区域

39. Evidence-gated Java 8 recovery [dependent MODIFIED coherence]
   WHEN 系统尝试从 IR、descriptor、异常/effect 与编译器模式证据恢复并呈现 Java 8 方法

40. Coverage and pagination are explicit [dependent MODIFIED coherence]
   WHEN 系统执行查询请求

41. Boolean contexts are presented as booleans [dependent MODIFIED coherence]
   WHEN 系统尝试呈现 boolean 返回、分支测试与其它值上下文

42. One local variable's type is decided once, before its statements are built [dependent MODIFIED coherence]
   WHEN 系统处理既有 LocalVariable 身份的类型决定、声明规划与语句构建
