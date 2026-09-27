# CF-08 固定 `TestNotIndexedLoop`：主线独立验收

主线合入 `e665c716` 后，root 独立构建 CLI（SHA-256 `ce82556bf397cec990f86c4f529fecf907de3ddef4e2418db91d33014101c7ab`），用归档 `replay.py` 在新目录 `/tmp/jarde-root-cf08-accept-replay` 从固定原源码、固定 JADX checkout 与实时 Jarde 分别生成并重编**完整 Java 8 类**，再以 `java -Xverify:all` 运行同一 Runner。三者 `javac`、JVM 验证和运行均退出 0，四路逐项输出 `null / null / f / h`；原 class SHA-256 `b1e6bd92f8fe92e726e7a2a95461327dd3f7abe65ee7f451f1225777ed7ea6d4`，实时 Jarde 完整源码 SHA-256 `d404e11abe4089e1e613be1b6ccf925c281536cd1593a6a77826be2c95d66c31`，与实现分支归档源码和执行结果相同。CLI SHA 不同是因为 root 在较新的主线上独立构建。固定 JADX 测试与算法 SHA 均由脚本重核。

root 审阅 Region/Builder：新候选仍要求原三块自然循环、准确两出口和二层三来源 join；`new/dup/File.<init>/astore` 以同轮 SSA 的初始化结果和唯一消费证明，`aaload→local2→getName→equals→branch` 按序且各一次；phi64 的三输入只转给 phi69，外层 null 是另一输入，后继判空、`deleteOnExit` 和返回读取同一值。Builder 的 `File` 局部类型只在此受证 Region 下由构造对象、`File[]` 元素与两次 null 写共同决定，没有全局把 `Object` 猜成 `File`。目标源码有唯一 `while (true)`、两个 `break`，无 `for (` 或 `@bytecode`；所有有文本的物理指令都有 source-map 段，结构性 `goto` BCI 13/64 虽无文本段仍各有唯一 Region owner，14 个物理块均仅被认领一次。

root 独立用 `javac --release 8 -g:none` 重建九个负例 class，与冻结二进制逐字一致，并运行 `java -Xverify:all` 的九路 Runner。随后用实时 CLI 逐方法恢复：构造别名/额外消费/效果参数、谓词额外调用/错误接收者/额外效果、内层额外出口、内层 join 效果和外层非 null 变异全部保留 `@bytecode` 且不误收 `while (true)`。`cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、`cargo fmt --all -- --check`、`openspec validate recover-constructor-predicate-loop-exits --strict` 与 `git diff --check` 通过；专用 Cargo target 已清理。

本次闭合固定 `TestNotIndexedLoop` 的完整类差距，不从这一个 lowering 推断 CF-08 全部多入口与复杂循环变体均追平。
