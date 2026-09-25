# 增强 `for` 当前三方基线与拒绝边界

旧 [2026-09-22 冻结审计](../../java-syntax-2026-09-22/enhanced-for/README.md)使用当时的 Jarde CLI；它记录的 `Iterable.hasNext()` 整类编译失败已由 `present-proved-java-structure` 2c.6 修复。本次从当前工作树构建 `jarde-cli`（SHA-256 `f112104ac3c7f2197a76b56c49f87f6240e4b894399e94ae019c0598220bd26e`），原样重放该审计的三个 Java 8 class 与 runner。原 class SHA-256 分别为：`IntArrayForeach` `067bf3c5b231811f97d19e9137c7d4019eb2849f8cce91706285120db7d55b58`，`ObjectArrayForeach` `b4c1b0ba7b5b1857f4e5369fb57245cb835ec727f180ec9d81c1a4d6d60841b2`，`StringIterableForeach` `81a0dcb4a53a51eb721df72f2b30825b33e8fb4dc6f239e0e93167f4012e09f36`。构建目录 `/tmp/jarde-enhanced-for-current-target` 已清理；本轮临时源码、javap、编译及执行记录在 `/tmp/jarde-enhanced-for-current-replay/`。

| 输入 | 当前 Jarde 正文 | JADX 1.5.6 正文 | 完整类执行 |
| --- | --- | --- | --- |
| `int[]`，含一次调用取得数组 | 计数 `for`，先保存数组及长度 | 增强 `for` | 原/JADX/Jarde 均按 Java 8 重编、`-Xverify:all`，6 行逐字一致 |
| `Object[]`，循环体调用 | 计数 `for`，先保存数组及长度 | 增强 `for` | 三者均重编运行，7 行逐字一致 |
| `Iterable<String>`，含异常与调用计数 | `while (iterator.hasNext())` | `while (iterator.hasNext())` | 三者均重编运行；原/Jarde 10 行逐字一致，JADX 仅 null 元素的 helpful-NPE 临时变量描述不同 |

因此当前可确定的 **JADX 语法追平缺口是数组增强 `for`**；`Iterable` 是原源码形态的后续探索，不能写成 JADX 已恢复。数组基线内 Jarde 有效行为并无错误，本变更须保持这一点。对 `Iterable`，原/Jarde 执行文本 SHA-256 均为 `e678e03550bdeb377fbb1a11f39a609809f872f835a90d16bf8cd731f0ab7046`，JADX 为 `6f158e8b0572b88f0aa7ecc85cdac998895b201a2edb374f1ea5e976f56d0952`。

[合法手写反例](Probe.java)以 `javac 23.0.1 --release 8 -g:none` 编译为 SHA-256 `e91644639a67d65d23396aab404effae28c9dec5cd11a46c6224c1a3531a7761` 的 class。`sumAndReturnIndex` 在循环后仍使用终值索引，`mismatchedArrays` 以 `a.length` 为边界却读取 `b[i]`；两个循环都有 `arraylength`、元素读取、递增与回边。JADX 分别保留 `while` 与计数 `for`，均未误投影。原/JADX 重编执行均输出 `cached=9`、`mismatch=ArrayIndexOutOfBoundsException`。不能仅凭 opcode 邻接和循环轮廓吞掉索引，或以 `a` 的长度替 `b` 的访问背书。

本地 JADX 的 `TestArrayForEachNegative` 覆盖错误步长、比较边界、错配数组等更多拒绝形状，但该测试显式 `disableCompilation()`，只断言输出不含冒号；它不能替代上述合法 class 的 Java 8 重编与运行反例。本项目的反例仍须检查原 class 行为，并单独记录 Jarde 当前保守引用的可编译性状态。

本地 JADX 源码 `2fb1b1638694` 的 `LoopRegionVisitor` 在 Region 后用双输入 phi、初值0、步长+1、索引用途、长度与元素读取同一数组值作数组准入，然后隐藏旧更新/读取。Jarde 应复用**用途约束与先证明再投影**的算法思想，但用既有区域、SSA、来源与预算做原子提交；不必照搬 JADX 的可变指令隐藏。特别要保留 Jarde 当前对数组引用单次捕获、不同数组、索引逃逸、循环转移与异常位置的更严格证明。
