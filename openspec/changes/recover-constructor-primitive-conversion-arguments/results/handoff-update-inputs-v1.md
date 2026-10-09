# Handoff 更新输入（2026-10-10）

此文件汇总供后续更新 handoff/账本使用的已存证据；本次不改 handoff、ledger、tasks、产品或测试。

## 已完成的语义对照

- 数值候选的两条真实 JDK 腿均完整成功：`javac8` 与 `javac23` 各自对完整两类、两份生成源码做空 classpath/sourcepath 编译，并仅运行新生成 classes 的 `-Xverify:all`。两腿都是 exit 0，原始 stdout/stderr 与冻结原程序逐字节相同；27 个目标方法、32 个唯一构造点、AASTORE 来源均通过。依据：`results/candidate-root-v1/manifest.json`、`results/candidate-root-verification-v1.json`（独立核验 221 checks、0 errors）。
- 历史回归合计 22 腿成功 18 腿：旧 18 腿为 16/18；另有完整六类 factory 两腿 2/2。完整 direct 六类仍是 0/2，javac8 与 javac23 都在完整源编译阶段 exit 1，不能将恢复的局部方法当成整类成功。BigDecimal 两腿仍因对照流不匹配而失败。依据：`results/legacy-regressions-root-v1/manifest.json`。其中原程序依据只使用旧 case 冻结的 exit/stdout/stderr SHA；没有声称重新运行原程序或取得旧原始流字节。
- handler 正例现使用 controls-v2：完整源报告在两 JDK 下都以空 classpath/sourcepath 编译成功，但此无 `main` 单类未运行。`sameHandler` 的 identity 调用是 BCI 12 的唯一 consumer，并在 `[0,15)` handler 范围内；v1/v2/v3 的作用域拒绝均保留为历史，不计为数值成功。依据：`results/handler-controls-build-v4/manifest.json`、`results/root-handler-v4-render-javac8/`、`results/root-handler-v4-render-javac23/`、`results/root-handler-v4-compile-javac8/`、`results/root-handler-v4-compile-javac23/` 及 `tests/fixtures/p3-constructor-primitive-conversion-controls-v2/fixture-manifest-v1.json`。
- Boolean mutant 测试只分析报告与来源，不执行被修改 class。依据：`results/root-boolean-v2/result.json` 和 `results/boolean-body-boundary-audit-v1.md`。

## Reader census 与 P5

- Reader census 专项门禁通过（1/1）。当前记录的普查元组为 `1063 / 4616 / 463 / 2711 / 8`，相对旧 pin 新增 8 classes、84 bodies、4 handlers，没有新增 branch 或 subroutine。依据：`results/root-reader-census-v2/result.json`、`results/root-reader-census-v2/stdout` 和 `verification-root.md`。
- P5 旧 pins 的复跑 exit 101，两个旧计数断言失败；更新 pins 后 P5 为 5 passed、0 failed、1 ignored。六个 corpus case 合计 `ir_items +202`、`analysis_steps +1982`，两个旧 per-method arms 同增；其他七个计费维度、文本和 outcomes 不变。wall-clock 仅为观测值，不作为断言或性能结论。依据：`results/root-p5-old-pins-v1/` 与 `results/root-p5-updated-pins-v1/`。

## 门禁及未完成范围

- `tasks.md` 的 3.1 已勾选；3.2（双 seed、MSRV、fmt、clippy、显式 Java 对照、15 opcode、P5/指纹/diff 等）和 3.3（对抗验收、71 单元 handoff、提交推送及 CI）仍未勾选。
- 第一次 workspace seed 的永久记录为 exit `-15`；停止前磁盘降至 `16,294,436,864` bytes。随后只清理本仓 `target`，记录显示释放约 13.1 GiB。完整原始 argv/stdout/stderr 保存在 `results/root-workspace-seed1-v1/`，失败记录不覆盖。
- 新 disk-guard runner 的 seed1-v2 只有 `results/root-workspace-seed1-v2/start.json`，当时 free-before 为 `40,933,036,032` bytes；尚无 `result.json`，因此 workspace seed 与 CI 都不能表述为已完成或已通过。CI 尚未新跑。

下一次 handoff 更新应保留上述边界：完整数值候选 2/2 已验收，旧 direct 整类仍 0/2，handler 正例没有 runtime 对照，Boolean mutant 未执行，workspace/CI 尚未完成；不得把历史局部恢复写成 EM-18 整体追平。
