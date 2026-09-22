## Why

同一 snapshot 上的**同一次定义读取**会被重复支付。已测量的事实（`optimize-demand-workloads` §2 O2 与 §3 实验，证据 `evidence/o2-class-reads.md`、`evidence/g1g2-experiment-o2.md`）：

- 同一定义的**第二次请求**仍重做一次受信读取（fixture 2,056 B / bcprov 532 B 的 entry 读取与 CRC/长度/span 复核），保留的只有容器目录/backing 与结构事实；
- W2 的 9 个类连续导航 = 9 次物化；W3 的逐方法臂 77 次；W5 往返 10 次；
- 单因素实验（两臂同一二进制、唯一变量为开关、11 格 × 2 臂 × **10 次交错**、220 个进程）显示：W1 第二次请求 66 → 50 µs（9/10 更快）、W2 控制重问 20 → 8 µs（10/10）、W3 逐方法臂 2178 → 1764 µs（10/10）、W5 往返 498 → 416 µs（9/10），计数 `class_materializations` 1→0 / 9→8 / 36→0 / 10→1，**准备段与 `class_preparations` 不变**，24/24 结果指纹（去 `usage`/`elapsed_millis`）相同，跨 snapshot 判别性用例证明**不误命中**。

这是 `optimize-demand-workloads` 的 O2 准入项，也是该专项八项调查里**唯一**被准入的候选（O1/O4/O5/O7 维持已交付，O3 待子 spec，O5 高层层/O6 预热/O7 的 W6b/O8 的 CLI 固定点暂缓或否决）。它不引入新的分析语义，只停止重复支付已经做过且仍然有效的读取。

## What Changes

- **`facts-cache`**：新增一条要求——**已验证的定义读取可在显式界内复用**：按定义身份（snapshot、物理位置、digest、length、variant）与读取 schema 绑定，只发布完整读取；与容器/CP 层共用同两个界（entries + retained bytes），满则拒绝、不淘汰；命中不计入本次 `archive_entries`/`read_bytes`/`entry_bytes`/`class_bytes`，不重放旧 usage、不重置预算；关闭或拒绝时在剩余预算内走直读。
- **`facts-cache`**（MODIFIED）：身份维度表增加定义读取一层；`Verified backing saves work but does not grant a fresh budget` 的现文要求"随后仍需读取所选 class"，改述为"仍须**定位该 entry 并核对变体与调用方声明的身份**；读取本身可由该定义的已保留读取作答"。
- **`artifact-snapshots`**（MODIFIED）：快照新增一个**受信读取的复用入口**（`jarde-reader` 与 `jarde-jvm` 之间跨 crate 调用），语义为"命中即返回该读取的字节与该读取建立的 digest；不命中读零字节、计零费用"。
- 实现入口：`src/facade.rs` 的身份绑定读取与 `crates/jarde-jvm/src/providers.rs::read_definition_class`（方法驱动与 bulk class task 的读取）经快照侧入口；**名字搜索的候选读取不入此路径**。

## Non-Goals

- 不做全局 `MethodIr` 复用、不做更高层 store、不做淘汰策略（O5 已暂缓，触发条件在其证据里）。
- 不改容器/CP 层语义（O1 的交付面不变，仍由归档 `bound-container-lookup` 唯一负责）。
- 不把本变更读作加速结论：窗口份额上界是 **0.5%**（要 5% 需 13.8 ms），因此只作**工作量**主张。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `facts-cache`：新增"已验证的定义读取可在显式界内复用"；身份与失效表增加定义读取层；一处既有 scenario 的措辞随语义更新（见 delta）。
- `artifact-snapshots`：快照增加受信读取复用入口及其命中/不命中语义。

## Impact

涉及 `jarde-reader`（快照侧入口与身份表）、`jarde-jvm`（方法驱动与 class task 的读取点）、根 facade（身份绑定读取）与它们的测试；不新增依赖、不新增线程、不用 `unsafe`、不引入淘汰与全局状态。回退责任：无 store / 满容量 / 异声明条目 / 取消耗尽 / 身份不符逐条走直读；语义回归撤回该独立提交。
