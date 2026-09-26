# Root 独立验收（2026-09-26）

`new@1` 只读复用已证明的 `concat::Plan`。链尾必须是构造调用的实际实参值，且只在该调用消费一次；整条链的物理区间、依赖和 owner 均须闭合。构造跨度与唯一消费者还须具有相同的异常处理覆盖。没有链证明时，单独的 reserved 集合仍在内层 `Allocate` 处拒绝。

Root 以重建后的 CLI 直接从冻结 `input.jar` 生成完整 `Probe.java`，未编辑生成文本。默认与 `all` 的正文逐字相同；`thrown` 和 `constructed` 分别恢复 `throw new ArithmeticException("arm-" + label)` 与 `return new ArithmeticException("arm-" + label)`，报告的所有方法均无 fallback。生成类用 `javac --release 8 -g:none` 编译、`java -Xverify:all` 执行，五行输出与冻结原版逐字相同。重编 `Probe.class` 的 SHA-256 为 `d28a756995fba8e47b2b57ded7f47c9d240a5568184b8c52445dfc81a8905289`，也与原版和 JADX 重编版一致。

`cargo test -q -p jarde-java` 全通过（206 个库测试及全部集成测试）；`cargo fmt --all -- --check`、`git diff --check` 和 `openspec validate recover-concat-constructor-arguments --strict` 通过。定向测试覆盖 reserved-only、错实参、重复实参、边界变化和异常覆盖变化的拒绝。严格 Clippy 仍因 24 条本轮之外的既有告警失败；本次不把这些无关债务混入改动。验收后清理隔离 Cargo target。
