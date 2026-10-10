# 已证调用返回值丢弃：root 架构审查

root实读Builder::discarded_evaluations、call_result_is_discarded、instruction的write=None普通Expr(call)、call_statement/accessor fallback、call_expr及qualified/deferred/refused quote、DiscardedEvaluations与emit完整Stmt来源。完整原/JADX/Jarde基线26命令、8运行腿、28方法profile已独立核：仅static4/virtual13/interface15缺来源，其余physical BCI与方法身份全在，两配置精确同。

private v1手写hunk实际git apply --check exit128失败，raw保留。private v2全文审读并实际apply-check exit0；两个statement路径都只读取同次既有discards证据，不重扫或猜producer，不扩pop2/qualifier/跨block/其他Stmt。一个Builder内部helper统一poll/charge准确pop BCI后plus_derived到完整语句，旧call primary不变，push前Stop传播，无新IR/Frame/pass/证书。

permanent v2测试同次ALL真实IR、reader唯一ordinal、trusted method/environment subject；三call完整正文/来源签名仅追加一个pop derived且精确完整语句span，准确owner/descriptor/BLAKE3与全物理BCI、default/all相同；return/local消费对照全文/map恒同。尚未编译，不能据apply-check标完成2.1/2.2。精准预算须root真实观测helper收费前的恢复用量后核实际Stop，禁止另建测试机制或伪造Builder/SSA；公开取消与原子发布须实跑。

静态qualifier-pop潜在遗漏与short-circuit/accessor直接投影路径不在本片，单独债务。不修改类型片或条件fallthrough。
