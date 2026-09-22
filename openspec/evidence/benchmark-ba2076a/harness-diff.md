# 协议驱动 vs G0 harness：同一格的交错对照（2026-09-22）

回答 `openspec/benchmark-protocol.md` 里登记的开放项「协议驱动比 G0 基线快 7–8%」。
结论：**差值可复现，是两套 harness 的结构差异，不是负载漂移、也不是仪器开销**；数值上
**1.125×（配对中位 +0.429 s，10/10 同向）**，差值定位在 G0 的 `prepare` 相与其 `request` 相的额外开销。

## 配置（两边完全一致的部分）

- 引擎：`ba2076a`（引擎与当前 HEAD 等值，见协议）；
- artifact：`weblogic/weak_password/decrypt/lib/bcprov-jdk15on-152.jar`（sha256[:16] `5329ddefb3c92927`）；
- 单元格：sink=`discard`、workers=`1`、facts store 容量 `roomy = FactsCapacity::new(1 << 14, 1 << 27)`
  （G0 的 `roomy` 与协议驱动 `bulk_scope_sweep` 内置容量**逐位相同**）；
- 边界：两边都用 `/usr/bin/time -l` 从**进程外**计墙钟（含进程启动、打开、走树、导出、退出）；
- 交错：每一轮先跑 G0、再跑协议驱动，共 10 轮，消除负载漂移。

## 命令

```sh
# G0 harness（在仓库内构建到独立 target，不动仓库文件）
CARGO_TARGET_DIR=/tmp/jarde-bench-p5-target \
  cargo test --locked --release --test p5_optimize_workloads --no-run --message-format=json
# → 二进制约 /tmp/jarde-bench-p5-target/release/deps/p5_optimize_workloads-<hash>

JARDE_OPTIMIZE_ARTIFACT=<bcprov.jar> JARDE_OPTIMIZE_WORKLOAD=w6a JARDE_OPTIMIZE_WORKERS=1 \
JARDE_OPTIMIZE_SINK=discard JARDE_OPTIMIZE_CAPACITY=roomy JARDE_OPTIMIZE_INSTRUMENT=off \
  <那个二进制> --exact measure_workload --ignored --nocapture

# 协议驱动
/tmp/jarde-bench-v5-target/release/examples/bulk_scope_sweep <bcprov.jar> \
  '{"kind":"snapshot_all"}' roots_empty.json discard 1
```

## 结果（10 轮交错，进程外墙钟）

| round | G0 harness | 协议驱动 | 差 |
| --- | --- | --- | --- |
| 1 | 3.859 s | 3.438 s | +0.421 |
| 2 | 3.864 s | 3.421 s | +0.443 |
| 3 | 3.860 s | 3.413 s | +0.446 |
| 4 | 3.888 s | 3.404 s | +0.484 |
| 5 | 3.812 s | 3.419 s | +0.393 |
| 6 | 3.838 s | 3.415 s | +0.423 |
| 7 | 3.845 s | 3.407 s | +0.438 |
| 8 | 3.822 s | 3.401 s | +0.421 |
| 9 | 3.826 s | 3.391 s | +0.435 |
| 10 | 3.812 s | 3.428 s | +0.384 |
| **中位** | **3.841**（3.812–3.888） | **3.414**（3.391–3.438） | **+0.429（1.125×）** |

## 差值在哪里（G0 自己的账本）

| 相（G0） | instrument=on | instrument=off |
| --- | --- | --- |
| open | 2.7 ms | 2.9 ms |
| **prepare** | **259.2 ms** | **266.2 ms** |
| request | 3,465.9 ms | 3,509.5 ms |
| 窗口合计 | 3.771 s | 3.779 s |
| 进程墙 | 3.849 s | 3.856 s |

- **仪器不是原因**：`instrument=on`/`off` 的进程墙中位 3.849 / 3.856 s，差 **<0.2%**。
- 协议驱动的进程墙（3.414 s）**覆盖同样的交付**：19,865 条记录、15,003 声明 / 14,495 方法体，
  与 G0 的 `numbers` 逐项相同（`produced 12045`、`explanation_only 2447`、`not_produced 3`、`no_body 508`）。
- 因此差值由两部分构成：G0 的 **`prepare` 相 ~0.26 s**（协议驱动路径里没有独立出现这一相），
  加上其 **`request` 相比协议驱动的整个进程墙还大 ~95 ms**。
- 两套 harness 各自跨会话稳定：协议驱动 3.438 s（campaign）→ 3.414 s（本次）；G0 3.80 s（基线记录）
  → 3.841–3.856 s（本次）。**差异是系统性的、可复现的，不是会话间噪声。**

## 建议的协议写法

把「协议驱动比 G0 快 7–8%，原因未解释」替换为：

> 两套 harness **不可互换**：同一 revision、同一单元格（bcprov / discard / w1 / roomy）、同机交错 10 轮，
> G0 harness 进程墙中位 3.841 s、协议驱动 3.414 s，配对差 +0.429 s（1.125×，10/10 同向）。
> G0 的账本把差值定位在自己的 `prepare` 相（~0.26 s）与其 `request` 相的额外开销上；
> 仪器开销已排除（on/off 差 <0.2%）。**G0 数字用于归因，不与协议行并列比较**，也不取平均。
