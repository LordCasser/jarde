# 门控实验（A 单独 / B 单独 / A+B / 基线）

脚本：[03-gating.sh](03-gating.sh)（渲染 [02-render.sh](02-render.sh)）；表格
[03-gating.out](03-gating.out)；渲染件 `/tmp/gate/renders/{base,a-only,b-only,ab}`。

四个构型同一 checkout 构建（baseline = `3e45800a` 的二进制；a-only/b-only = 实现后加一行临时门：
a-only 让单组副本仍要求 `row.start_bci == current.bci()`，b-only 让副本长于一组即拒；实验后
`guard.rs` 由参考副本还原，`git diff` 证明无残留）。

## 每形门控（按名，`v8` 腿；三条腿同构）

| 方法 | base | A-only | B-only | A+B |
| --- | --- | --- | --- | --- |
| `ML.nestedLocks` | 拒 | **呈现** | 拒 | **呈现** |
| `ML.interruptibly` | 拒 | 拒 | **呈现** | **呈现** |
| `ML.multiAwait` | 拒 | 拒 | 拒 | 拒（逐字节不变） |
| `MLOrder.nestedLocks` | 拒 | **呈现** | 拒 | **呈现** |
| `MLOrder.nestedLocksThrowing` | 拒 | **呈现** | 拒 | **呈现** |
| `MLOrder.interruptibly` | 拒 | 拒 | **呈现** | **呈现** |

即：**A 单独只翻多语句 finally 体形（nestedLocks 两法），B 单独只翻行外可抛获取形
（interruptibly 两法）**，互不代偿。

## 零回退（27 个输入 × 4 构型逐字节）

`03-gating.out` 的 27 行全部 `all-identical`，除本片的两个锚类（`ml-*`、`mlorder-*`，
A/B 各自 moved）。逐字节不变者包括：

- **LK**：巡查 jar、两条重编腿、`LockGuardNegatives`、`LockGuardProbe`（5 输入）；
- **IO**：巡查 jar、两条重编腿、`IOMidRead`、`IONegatives`（6 输入）；
- 其它已证书形：`p3-handlers/Guarded`、`p3-finally-straight/FinallyNormal`、
  `p3-concat-saved-finally/FinallyOnce`；
- 本片负例与边界：`MLNegatives`、`MLProbe`（4 输入）。

结论：两准入的判据在**每一种构型**下都不触碰 LK/IO 与既有证书的判据与渲染（硬不变量 1），
负例与边界在每一构型下逐字保持（硬不变量 2）。
