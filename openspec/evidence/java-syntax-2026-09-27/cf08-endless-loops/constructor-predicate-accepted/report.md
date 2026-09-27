# 固定 `TestNotIndexedLoop` 完整类恢复

重放命令（仓库根目录）：

```sh
CARGO_TARGET_DIR=/tmp/jarde-cf08-constructor-predicate-target cargo build -p jarde-cli --locked
python3 openspec/evidence/java-syntax-2026-09-27/cf08-endless-loops/replay.py \
  --jarde /tmp/jarde-cf08-constructor-predicate-target/debug/jarde-cli \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out /tmp/jarde-cf08-constructor-predicate-replay-new
```

`--out` 必须是尚不存在或空的目录；本目录保存了本次运行的结果。

脚本核对固定 JADX 源码/算法 SHA、原 `NotIndexedLoop` class SHA `b1e6bd92f8fe92e726e7a2a95461327dd3f7abe65ee7f451f1225777ed7ea6d4`，分别重编原、固定 JADX、Jarde 的**完整类** Java 8 源码，并以 `java -Xverify:all` 运行同一 Runner。三者编译和运行退出码均为 0，四路输出逐项为 `null / null / f / h`；详细哈希、源码、编译和运行日志见本目录的 `summary.json`、`source/` 与各自日志。

同轮 SSA/Code 核验：BCI 25 `new` 写 v28；28 `dup` 读 v28，写 v29/v30，其中 v29 无显式 use；29 参数写 v31；31 `File.<init>` 读 v31/v30，写已初始化 v32；34 保存为 local2 v33。体内 41 `aaload` 写 v36→42 保存 v37→43 重读 v38→44 `getName` 写 v39；47 字符串写 v40，49 `equals` 读 v40/v39 写 v41，52 分支唯一消费 v41。内层 phi64 由 BCI 12 null v43、34 对象 v33、42 数组元素 v37 组成 v7；64 仅 `goto 69`，v7 唯一供给 phi69；外层 BCI 68 null v26 是另一输入，phi69 的 v4 在 69 判空、73 `deleteOnExit`、77 返回读取。BCI 12/68 的两次 null 写分别在已证内层空臂与外层空臂。类型推导复用 `array_of_value`，沿 BCI 38 对同轮 local1 的读取追到 `File[]` 参数来源，再从该数组的元素类型证明 BCI 41 `aaload` 保存的值为 `File`；构造器结果同为 `File`，两次 null 写不施加不同引用类型。

Jarde 方法有唯一 `while (true)`、两个 `break`，没有索引 `for` 或 `@bytecode`；构造、`getName`、`equals`、`deleteOnExit` 在源码中各一次。测试核对所有消费指令的来源映射和 14 个物理块的唯一 Region 所有权。BCI 13 与 64 是无文本的结构性 `goto`，source map 无段，但二者分别由 Region 唯一拥有；其余受检 BCI 都有源码段。不能仅凭无 `@bytecode` 宣称来源完整。九个 verifier 有效变异及重建命令见 `tests/fixtures/cf08-constructor-predicate/README.md`，其编译和 `java -Xverify:all` 输出保存在本目录的 `verifier-negatives-*.log`；各自保持物理引用而不错误认领新循环证书。
