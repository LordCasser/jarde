# 复合条件运行时源码夹具 v2

本目录是 `recover-proved-compound-statement-conditions` 的 task 3.1 私有 Java 8 源码候选，只包含 `CompoundStatementConditions` 与独立的 `Runner`，不依赖 JUnit 或其他库，也不改仓库文件。

`CompoundStatementConditions.java` 保持 v1 源码不变，覆盖 `(a && b) || (c && d)`、反转混合条件 `(!a || b) && (c || !d)`、可选按标签抛异常的同形条件，以及 `!(value < 0.0f) || value < -10.0f`。条件调用和分支 payload/公共前缀后缀都写入 trace，便于对比短路调用顺序、异常时机及共享续接是否重复。

`Runner` 对 16 组输入分别执行普通、混合与 throwing-null 三种正常返回；执行四个可达异常点；再执行四个不可达异常点控制，确认未访问的标签不会抛出。浮点组固定为 9 项：NaN、正负无穷、-10 两侧及边界、正负零和普通正值。总计 65 条记录。`input-matrix.json` 为每条记录列出输入、预期结果、测试调用顺序、完整 trace 和异常状态。这些是按 Java 源语义推导的预期，不表示 Jarde 已通过。

没有在此编译或执行。root 负责对原始源码、JADX 和 Jarde 生成类进行真实编译、runner 执行与完整结果比较。
