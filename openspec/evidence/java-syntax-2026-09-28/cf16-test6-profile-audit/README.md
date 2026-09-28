# CF-16 Test6：debug/no-debug 与 JADX 输入 profile 边界

固定 JADX HEAD `2fb1b16386941660fda07e9017285aec40fcb37f`，`TestTryCatchFinally6.java` SHA-256 `ae58ce7514ad518476e7ce2c58a4e50d7442be1ff05b701754ef4709cc4e0670`。活动 `test()` 断言有 debug 信息时输出 `InputStream is = null; try { ... is = new FileInputStream(...) } finally { if (is != null) is.close(); }`；`testNoDebug()` 则只断言 `if (0 != 0)`，并注明不能证明变量应合并。这两个断言都未显式调用 `useJavaInput()`。[IntegrationTest.java](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/api/IntegrationTest.java) 的默认输入为 `dx`，除非环境变量 `TEST_INPUT_PLUGIN=java`；因此活动测试默认覆盖 **Java 源→DEX→JADX**，不能直接算成 JVM classfile 输入的正向恢复证据。

[Java 8 转写](TestTryCatchFinally6$TestCls.java)保留目标方法体，并分别用 `-g` 和 `-g:none` 编译；两份 `test()V` 的 21 个 BCI/opcode 和唯一 `[2,15)→26 any` 异常行完全相同，差别在调试元数据。它是独立转写，并非从 JADX 测试构建产物提取的原 nested class。固定 JADX CLI 以 **Java classfile 输入**解这两份 class，debug 源把同一 slot 写成 `is`/`is2`，no-debug 源写成 `fileInputStream`/`fileInputStream2`，两者都有空的 `if (newStream != null) {}`，finally 则只检查旧的 null 局部。这不满足该文件默认 DEX 测试的 debug 文本断言；说明 profile 差异真实存在，不能把 DEX 断言外推到 JVM。

主线 Jarde CLI SHA-256 `a28b82213ed89d1902da9aac299369a6c2c25c90facc5ee330c50632633b5acd` 对两个 class 都保守输出 explanation-only，最终说明为 local 0 跨 quoted fallback 区域。原始、JADX Java-input、Jarde 展示源码都能作为 Java 8 class 重编，但这不等于行为相同。[Runner.java](Runner.java)在 `1.txt` 存在与不存在时调用 `test()`，`java -Xverify:all` 输出分别为：

| 输入 | 运行输出 |
| --- | --- |
| 原 Java 8 转写 | `normal:ok`；`missing:FileNotFoundException` |
| 固定 JADX 的 Java-input 源码 | `normal:ok`；`missing:FileNotFoundException` |
| Jarde explanation-only 源码 | `normal:ok`；`missing:ok` |

这两条可观察路径不能证明 JADX 的 `is`/`is2` 资源关闭语义正确；它们只证明当前 Jarde 占位方法丢了缺文件异常。完整产物及两种模式摘要在 [baseline/](baseline/)。可用当前 checkout 的 Jarde CLI 和固定 JADX CLI 重放：

```sh
python3 openspec/evidence/java-syntax-2026-09-28/cf16-test6-profile-audit/replay.py \
  --jarde /path/to/jarde-cli \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out /tmp/cf16-test6-replay
```

当前结论是**取证边界**，不是新恢复任务：若以固定 JADX 默认测试为目标，先需匹配 DEX lowering；若以 JVM classfile 为目标，固定 JADX Java-input 输出本身已失去局部别名关系，应独立定义原字节码语义与负例，不能复制它的源码。后续若恢复此形态，必须联合证明一行 catch-all、局部先为 null 后写入及两个条件清理副本；已有 Test14 字段条件清理证书不能直接当作这个局部变量证明。等 Test11/Test4 的大颗粒 finally 里程碑完成后再决定是否新增窄证书。
