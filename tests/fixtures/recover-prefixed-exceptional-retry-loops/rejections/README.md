# 合法邻变体（不预设全局拒绝）

两类保持完整字段及8方法。`PrefixRetryEffectfulLatch`仅在inner循环结束后加入`record(9)`；`PrefixRetryCatchOuter`仅在catch末尾加入`continue outer`。root已用双javac8/23和debug/no-debug原样编出8class，再双JVM16运行160原始case，严格验证及CodeSource负控通过。

这只证明输入合法及原始语义稳定，尚不证明新恢复证明拒绝或全局fallback。内部测试应直接核新候选不误接受；既有其他路径若能正确恢复，不要求它退回。

EffectfulLatch的normal出口69→94，94含真实`record(9)`调用后goto@100→outer18；不能当纯bridge。CatchOuter的handler goto@91→outer18，不能当回inner36的continue。实际行号/BCI只描述冻结夹具，不作为产品匹配。javac8/23的同一`record(I)V`常量池索引不同，源码与符号引用相同，按真实CP解析而不固定索引。

来源与工具/运行/raw见OpenSpec change的`results/legal-neighbor-originals-root-v1`；copy-manifest固定本目录class的bytes/SHA和原编译命令。
