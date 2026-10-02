## 1. 取证与基线

- [x] 1.1 重放固定 P1/P2/P3（SHA 核对）：读 `place_inner_finally`/延迟保留机制，确认 direct 形锚点差异（表尾 vs 体内）与复用面；记录 P3.voidBodySoloFin 基线（BCI 表见巡查 README）。
- [x] 1.2 构造并冻结至少四个 verifier 有效变体/负例：finally 含 return、多层 TWR 外层带 finally（登记现状）、副本断链（拒绝）、finally 与资源行交叠（拒绝）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 尾随子证书与呈现

- [x] 2.1 TWR 证书尾随 finally 子证书（design 决策 1，锚点按取证结论）；P3/P1 恢复、整类重编运行一致（`t[c]f`/`b[c]f`）；inner 形与纯两域 diff 断言逐字不变。
- [x] 2.2 变体边界正确；负例保持拒绝；预算/取消原子性不变。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 TWR 全家族、finally 全家族、twr-inner-finally、nested-monitor）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘纪律：每轮构建前查 `df -h /`，轮间与完成必 `cargo clean`。
- [x] 3.2 P1/P3 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [x] 3.3 root 独立复核锚点判据、副本等价与三方行为，更新账本与巡查记录。（root 于合并主线 c5a3de3f 复核：P3/P1 呈现 `try (…) { … } finally { … }`、2818/0、fmt/openspec 244/244、磁盘纪律达标（worktree 已清）。`place_finally_copy` 共享重构（一份副本判据、行几何锚点互斥）复核认可；`TF.solo` 随片翻转为其切片测试同步属自然推进。三子句显式拒绝、finReturn/inner×trailing 复合登记现状；B 形态归下一片。）
