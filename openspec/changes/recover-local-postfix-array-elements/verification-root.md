# Root 独立验收

2026-09-25 从当前工作树独立构建 `jarde-cli`，SHA-256 为 `7021d42e75444dc963c8626d296b97335b1609b12029ac4641359b3175a47465`。冻结正例 `ArrayPostfixElement.class` 的 SHA-256 为 `b3ec3d79d577f6483952fac584b96bdcdd69ca814615b338f4797112744352bb`，从 fixture 源码用 `javac --release 8 -g:none` 重编后逐字节相同，class major 为 52。独立 CLI 的完整 JSON 与源码分别保存在[正例](../../evidence/java-syntax-2026-09-25/array-postfix-element/root-after-positive.json)、[`+2` 控制](../../evidence/java-syntax-2026-09-25/array-postfix-element/root-after-plus2.json)和[不同槽控制](../../evidence/java-syntax-2026-09-25/array-postfix-element/root-after-different-slot.json)。

正例 `make(I)[I` 为 `java`、零 fallback，发射 `return new int[]{1, arg0++, arg0 * 2};`，没有独立的 `iinc` 赋值。`javap` 的 18 个真实 BCI `{0,1,3,4,5,6,7,8,9,10,13,14,15,16,17,18,19,20}` 均在 source map 中，其中旧值 load BCI 9 和更新 BCI 10 分别锚定到同一个 `arg0++` 表达式。原 class、JADX 输出和 Jarde 输出的完整 Java 8 类均重编成功；以同一 Runner 在 `java -Xverify:all` 下五行逐字相同，包括 `Integer.MAX_VALUE` 溢出，记录见 `root-after-*-run.txt`。

两个控制 class 都通过 JVM 验证：`iinc +2` 输入 0 输出 `[1,0,4]`，另槽 `iinc` 输出 `[1,0,0]`；如果错误写成第二元素的 `arg0++`，后续值会不同。Jarde 对两者都没有发射 `++` 或部分数组字面量，仍保留不完整正文的引用。正例只在相邻 `iload` 旧 SSA 值、同槽 `iinc +1` 和唯一 `iastore` 闭合时提交，现有 `PostIncrement(Local)`、数组计划和类型/来源检查即可表达；没有新公开 AST 或通用局部重写机制。代码审读确认 `iinc` 同时成为元素来源和数组计划 owner，普通语句被一次性抑制，证明或渲染失败不会留下半个 postfix。

Root 运行新测试 5/5，并复跑一维/多维/部分维度数组、已有后置值、handler 边界、局部重写和短路数组七个目标共 29/29。`cargo fmt --all -- --check`、`git diff --check`、OpenSpec strict 均通过；`cargo clippy -p jarde-java --lib` 退出 0，仍有 17 条与本 change 无关的既存 warning，严格 `-D warnings` 门禁不能据此宣称通过。`tests/fixtures/corpus-fingerprint.json` 的本 fixture 八项已登记并由实施代理逐字节核对；root 另核对八项路径、大小与实际文件 8/8 对齐，任务 1.3 的局部合同闭合。共享树另有 121 个其它 fixture 未登记，属独立语料清单债务，不能据此声明全局清单通过。Root 的私有 Cargo target 已清理，`cargo clean` 报告移除 3.6 GiB，目录已删除。
