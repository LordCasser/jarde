## 1. 冻结有效输入与拒绝边界

- [x] 1.1 `tests/fixtures/p3-array-initializers/` 已保存两份原 Java 源和 1281 B/13 Code 的 Java8 class，三者与冻结证据逐字节相同；root 独立以 `javac --release 8 -g:none` 重建 class 得 SHA `998bdb54…5c86` 并 `cmp` 相同，runner 的 14 行与原/JADX 冻结基线相同，修前 Jarde 四处缺 return 的真实整类失败仍见证据。总 fixture 索引已登记，corpus fingerprint 347 文件经独立既有测试二进制再生及 5 项校验通过。
- [x] 1.2 `array-initializers/boundaries/` 已冻结精确长度/连续索引/单一数组身份及有副作用元素正例，和动态长度、重复/跳跃索引、逃逸时观测默认值、跨块、协变 `aastore` 抛错的合法 Java 8 反例；root 在独立复制目录原样重放，1741 B/15 Code、原始 runner 成功、JADX/Jarde 整类编译失败及冻结 CLI SHA 均与 `summary.json` 一致，拒绝边界与可恢复候选逐项记录在 README。

## 2. 在现有数组创建值上认领初始化链

- [x] 2.1 用块内指令及 SSA 做有限形状证明，逐个检查分配、copy、索引、store、最终消费和类型；每步走既有预算/取消，不放宽 `chained_pair`。
- [x] 2.2 给现有 `NewArray` 增加可选元素序列，仅对已证明的一维链输出 `new T[]{…}`；复用 `render_value`、类型、deferred 顺序及来源，保证一次求值且普通分配不回退。
- [x] 2.3 成功时归属全部链 BCI，失败时完整来源拒绝；默认/all 正文、预算停止及 source map 对照稳定。

## 3. 整类执行与 root 验收

- [x] 3.1 原样重编译执行永久 fixture 和补充有效样本，要求零相关引用、整类 Java 8 编译、所有值/trace/异常与原 class 逐项一致；JADX 只有成功编译才可作运行对照。root 在独立转换实现后的冻结 CLI `8b86c729…756e9` 上重新编译永久 class 与恢复类、分别执行 runner，14 行逐字节一致且零引用；补充有效样本的 11 行等价证据保留在 `verification.md`，协变拒绝仍不算等价。
- [x] 3.2 root 独立审读身份/长度/类型证明与拒绝边界，重放原/JADX/jarde；复跑普通数组读写、部分维度分配、链式赋值及 deferred 顺序相邻回归。完整 14 行 trace 的独立转换依赖仍归任务 3.1，不据此声称本 change 全量语义验收。
- [x] 3.3 root 统一 census/fingerprint、fmt、适当 Cargo 测试及 OpenSpec strict；架构债务独立记录。最终 CLI SHA-256 `d7520a08a37b54ef579d42c770f5b512af17ddb11b05050568458759c78dbf16`，见 `verification.md`。
