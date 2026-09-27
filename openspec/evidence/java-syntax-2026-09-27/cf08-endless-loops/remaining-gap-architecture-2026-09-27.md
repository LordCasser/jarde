# CF-08 剩余 `NotIndexedLoop` 缺口定位

在主线 `d73aa754`，root 用 `javac --release 8 -g:none` 重建固定 `NotIndexedLoop.java`，物理 class SHA-256 仍为 `b1e6bd92f8fe92e726e7a2a95461327dd3f7abe65ee7f451f1225777ed7ea6d4`。合并态 CLI SHA-256 `9518aa8e0d5e34ab6d5ce71b6222db5a38b009ebea6467a228553d3db44cc4f9` 的类报告仍给出 `jre_region_arms_do_not_meet`（block 0）与未覆盖块 `[19, 25, 38, 55, 58]`。其后的 `local 2 crosses a quoted fallback region` 是局部绑定层对前述区域缺口的保守拒绝，不能靠放宽局部检查修复。

`JRE_JOIN_PROBE` 显示外层 BCI 0 分支的立即后支配点为 BCI 69，内层 BCI 4 分支的为 BCI 64。`javap` 中 BCI 13、35、55 都正常转移至 BCI 64；BCI 64 再转移至 69。现有 `continue_inner_join_arm` 只续接两个均为 `Region::Straight` 的 `If` 分支，并要求续接起点恰有两个分支末块的入边、尾部为单一直线段。内层 BCI 4 的一臂包含循环，故当前证明域不能表示它的 BCI 64→69 续接。这是由代码约束和物理 CFG 得出的定位，尚未证明放宽该约束足以恢复整个方法。

循环本身还有独立边界：BCI 19 的头部越界路径在 BCI 25–35 构造并写入 `File("h")`，BCI 55 的命中路径直接跳向 BCI 64。现有 `loop_exit_gateway_pair` 仅证明无效果直接转接网关；这里的 BCI 25 含可抛的构造与赋值，不能套用已验收的纯网关证书。下一步应把“外层分支中的循环后续接”和“带效果的双循环出口”分别构造成 Java 8 最小正反例并重放，再为确证的最小缺口写 OpenSpec；两个假设不能在一个宽松的 CFG/局部重写中混修。
