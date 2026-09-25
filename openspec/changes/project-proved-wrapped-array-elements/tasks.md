## 1. 冻结求值顺序基线

- [x] 1.1 root 复核[三方证据](../../evidence/java-syntax-2026-09-24/array-wrapped-binding/analysis.md)的源码／class 哈希、Java 8 重编与 `java -Xverify:all` 运行，确认原/Jarde 逐字一致且 JADX 在 `92→4`、`1091→1003`、`91→3` 三处改变结果；[独立记录](verification.md)保存复核命令和输出。
- [x] 1.2 固定 `a[i] + tick()` 与 `tick() + a[i]`、调用实参、体内前置副作用、同轮两次读取、null 与抛错的语法正反例；[独立记录](verification.md)以原 class/JADX/当前 Jarde 完整类源码和 13 行执行轨迹作为后续门槛。

## 2. 受限表达式绑定

- [x] 2.1 在既有数组 `ForHeader`/SSA 证明上识别首条体语句表达式中的唯一 `Index`，证明其前缀无可观察操作、索引独占使用及异常边相同；以 `a[i] + tick()` 正例、写数组与两次读取反例的定向测试验收。
- [x] 2.2 用新鲜元素局部替换已证明的 AST 子树，保留后续调用和原始顺序，并原子产生 `ForEach`；以首句声明、累加表达式两种正例及 `tick() + a[i]`、`consume(tick(), a[i])` 拒绝例的完整类 Java 8 重编运行验收。
- [x] 2.3 保留折叠前真实 BCI、默认/all 正文一致和预算／取消边界；以来源选择、低限停止及拒绝后计数循环未半折叠的定向测试验收。

## 3. 独立验收

- [x] 3.1 root 重建 CLI，独立重放原/JADX/Jarde 完整类，核对安全正例转增强 `for` 且运行轨迹与原 class 一致，三条 JADX 错误路径在 Jarde 仍保留计数循环并产生 `92`、`1091`、`91`；[独立记录](verification.md)保存命令、哈希和结果。
- [x] 3.2 root 审读局部证明和相邻数组/循环回归，运行定向 Cargo 测试、`cargo fmt --check` 与 `openspec validate project-proved-wrapped-array-elements --strict`；[独立记录](verification.md)列出并发债务和顶部无用声明的后续质量债务。独立 Cargo target 清理 7235 文件/2.7 GiB，可用空间约 16 GiB。
