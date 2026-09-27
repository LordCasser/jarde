# EM-19：数组读写与下标求值次序首片对照

固定 JADX 的 `arrays/TestArrays.java` 和 `code/TestArrayAccessReorder.java` 分别要求数组字面量后按索引读取，以及带循环、下标和 `i++` 的数组读写。两份测试文件的 SHA-256 由 [replay.py](replay.py) 核对。[input/em19/](input/em19/) 在 `javac --release 8 -g:none` 下缩小出完整 `Access` 类和共同 `Runner`，涵盖合法索引、越界异常、二维数组长度及三元素逆序负值写入。固定 JADX 和 Jarde CLI (`8f1f0012324350e4fc65c7fef4b3e3835102e6d4fda00df280b466270c98e727`) 的完整 `Access` 源码与相同 `Runner` 均通过 Java 8 重编和 `java -Xverify:all`。独立第二次重放的摘要及两份反编译源码逐字节相同。源码、版本、编译/运行日志及哈希在 [acceptance/](acceptance/)；三方输出相同：

```text
1:5
AIOOBE
2
[-15, -10, -5]
```

JADX 在简单读取处内联 `new int[]{...}[i]`，Jarde 保留与原源码一样的局部数组再读取；两者均只建数组一次，越界异常一致。循环中 Jarde 保留先 `arg0[local3]`、后增加索引、再写 `local2[local4]` 的顺序；其 `local3 = local3 + 1` 与原始/JADX 的 `i++` 在此位置运行等价，后缀源形态属于 EM-23 后续验收。现有 SSA 值身份和结构化循环/数组表达式路径足以恢复本首片，不需要为数组访问新增机制。

本次初始探针还放入了 `jbc/TestDup2x1.java` 对应的 `return this.value = v`。它让 Jarde 在 `dup2_x1` 上回退、整类不可编译；该字节码是**实例 long 字段赋值表达式**，与数组访问无关，已从 EM-19 的验收夹具拆出并转入 EM-07 单独审计。`TestDup2x1` 在固定 JADX 使用 Java 11 profile，但本地 `javac --release 8` 也产生同一 `aload_0; lload_1; dup2_x1; putfield; lreturn`，因此差距仍有 Java 8 复现基础。

EM-19 只移动到“部分已测”：其他下标副作用、异常边、别名以及更多循环重排尚未覆盖；这里不把字段赋值的失败混入数组实现任务。
