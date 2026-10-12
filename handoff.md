# Jarde 接续入口

继续在当前聊天。产品锚点 `5f5fb2c605d61e18f392bb3abe6bdae6e10bf41e`；OpenSpec `recover-proved-local-reference-write-compatibility` 当前 7/8，剩余干净 main 交付。功能与本地验证已完成，交付未完成前不可写成成功。

## 已有事实

[verification](openspec/changes/recover-proved-local-reference-write-compatibility/verification-root.md) 记录 fmt、当前 CI Clippy、global strict 356/0、CLI build、clean 成功；fresh 本地工作区 96 批/363 targets/3489-0-97/3586具名结果；4784 来源闭包已验收。CLI SHA-256 `4df9ff1c96addfcae2814b72687e35f2883b5aba1d19b0b42cdfa9959d091f95`。自身 [CI 38197592542](https://github.com/LordCasser/jarde/actions/runs/38197592542) 已真实成功4jobs/52steps，root独立verifier v5实际exit0、逐目标核双seed与本地具名结果、绑定确切产品和最终CLI。见[验收记录](openspec/changes/recover-proved-local-reference-write-compatibility/results/own-product-ci-root-v1/README.md)。原v4验收器NameError失败保留；下一步是文档推送后的fresh main审计。

main/15 checkouts/14 辅助 detached 只是旧检查点；本次必须按真实 HEAD fresh audit，不能预称 clean 或全目录删除。空闲磁盘≥5 GiB、target≤1 GiB、守卫间隔1秒、default stack、最后 clean。共享 Git/Cargo/rustfmt/JDK/JADX/CLI 由 root 独占，Luna 只做私有准备或只读审计；勿并行构建或改共享工作树。

## 下一项：CF11

交付本 change 后，固定清单仍为 71/612。只做 CF11 `List<String>` structured-void 候选诊断，分别核对结构化 void body 完整性、参数用途可赋值性和调用处源级参数绑定；先读[实际边界审查](openspec/evidence/jadx-feature-inventory-2026-09-27/cf11-structured-void-generic-static-root-v1/next-boundary-review-root-v1.md)及[root review](openspec/evidence/java-syntax-2026-10-12/cf11-testnestedloops-same-ir-root-v3/next-design-review/root-review.md)，再接续 `/private/tmp/jarde-cf11-structured-void-generic-diag-prep-luna-v3`。v3 已修正动态闭包的 Path/相对键比较，并复用守卫记录最终清理。脚本尚未执行，仅诊断准备，不实施产品改动；静态推断不能记成实测拒绝。当前产品的[双JDK调用探针](openspec/evidence/jadx-feature-inventory-2026-09-27/cf11-structured-void-generic-static-root-v1/current-generic-call-probe-root-v1/README.md)已实测：只改List<String>签名因add(Object)编译失败；再只去掉该add的合成Object cast，手工诊断源码才编译且8路径同原。该手工结果不算产品修复，不得全局删除cast。

## 证据入口

[任务](openspec/changes/recover-proved-local-reference-write-compatibility/tasks.md)、[本地验收](openspec/changes/recover-proved-local-reference-write-compatibility/results/final-validation-candidate-root-v1/README.md)。失败原样保留：批53断言/记录器错误见[记录](openspec/changes/recover-proved-local-reference-write-compatibility/results/full-workspace-failed-root-v1/README.md)，磁盘停止见[记录](openspec/changes/recover-proved-local-reference-write-compatibility/results/full-workspace-resource-stop-root-v2/README.md)，计费断言见[记录](openspec/changes/recover-proved-local-reference-write-compatibility/results/full-workspace-billing-failure-root-v3/README.md)；后续诊断断言与紧预算失败见[记录](openspec/changes/recover-proved-local-reference-write-compatibility/results/final-diagnostic-focus-root-v1/failed-capture/execution.json)。债务仅从[独立账本](openspec/completion-review.md)拆分处理。
