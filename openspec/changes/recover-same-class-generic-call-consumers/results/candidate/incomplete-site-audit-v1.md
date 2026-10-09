# IncompleteSite 调用点清单补充审计

## 结论

`IncompleteSite` 是合法的 conditional 单调用正例，不是缺失或不完整 inventory 的负例。冻结 Java 源在 `relay(T x, boolean use)` 中仅当 `use` 为真时调用 `identity(x)`，否则直接返回 `x`。四腿冻结 jar 的完整物理指令转录都确认 `relay` 恰有一个同类调用：BCI 6 的 `invokevirtual identity:(Ljava/lang/Object;)Ljava/lang/Object;`。因此，candidate 把 `identity` 与 `relay` 的泛型头恢复为 `T`，并通过 true/false 两条运行路径，是本例预期的正例结果。不能通过普通 Java 源码把这例当作“缺失 inventory”测试；缺项必须在测试夹具或上游 census 注入边界合成。

## 输入与物理调用证据

冻结源码 `frozen-inputs-v1/corretto8/debug/IncompleteSite/IncompleteSite.java` 的 SHA-256 为 `3d4c9f64d35222cb1e6591e24f64bdee3178f1d0370ca09bd9356b51d4d55e0a`，内容为单个 `if (use) return identity(x); return x;` 条件调用。candidate-v1 manifest 中四个物理 jar 的 SHA-256 分别为：Corretto 8 debug `8caf43e2252d114b114d51193e5cd3153fac096f13a48889fc4e88c4f8a1a27b`、Corretto 8 nodebug `c659525e0fa7cb64236e08ea4c643bf182881786617c4fea5ab820fb9405d55e`、OpenJDK 23 debug `26c16dc2130a6a63193a0535016e519b2f048733d9b68cadef7950ba50bdb94f`、OpenJDK 23 nodebug `52dc3a3143e5b2e20632bcfb592cfcd8a59055c664bc5b6276c5c763f864c7fd`。

新保存的 `incomplete-site-census-v1/manifest.json`（SHA-256 `65d4f42da4e7d48e07ece53d76b88b853e902f8b853c70edcb84b1d7147edc76`）记录四腿实际 `javap -c -p -v IncompleteSite` 命令、输入 jar hash、exit、stdout/stderr hash 与 relay invoke 行。每条均只有 BCI 6 的上述 `invokevirtual`。转录文件分别位于 `incomplete-site-census-v1/{corretto8,openjdk23}-{debug,nodebug}/javap.stdout`，四次命令 exit 均为 0，stderr 为空。Corretto 8 debug 原始 stdout SHA-256 为 `25e14962ae3cfa5f0882929e5adc81826a3437052041a551033f68284851a33a`；其它三腿的精确转录 hash 见该 manifest。

candidate-v1 的 `gc01-08-140/manifest.json` SHA-256 为 `a2f3ef31dad2c5e4c627d5037bfeebf40bda4a377febe93411f33f6e8fe1160a`。其 Corretto 8 debug Jarde stdout SHA-256 为 `56fc806f2c0e6b908118f83d0b6ed0e4bd286205a7f9fcd311d5b25f1d2ef02f`，其中 `identity(T)` 与 `relay(T, boolean)` 都带有从 erased descriptor 投影泛型签名的 marker，relay 正文保留 `if (arg2) return this.identity(arg1); else return arg1;`。同腿 JSON stderr SHA-256 为 `12343f5265b9f5989440ad835097cad951075137981f6d4268036a17694118e9`，保存了结构化 method 记录。既有 candidate 汇总记录本例四腿 compile/probe 成功、行为标记 `both-conditional-paths`、全反射匹配；这些结果与物理 census 相符。

## `report.rs` 的拒绝边界与 12 项测试

审计对象为当前 `crates/jarde-java/src/report.rs`，文件 SHA-256 `e1860c55a49073ebea2f3b8cd1b3661a5b7f7343389904d0acafe499ebc2bcd1`。本地 v14 格式化源码快照 `results/local-gates/source-snapshot-v14-formatted.json` SHA-256 `e2721719b0f92d231bc45b471c5342311d76fb71776d4050bd5aec097a5ba845` 对该文件也记录了相同 hash，因此以下行号和拒绝逻辑对应 v14 快照。

这组 12 个 report 测试覆盖 wrapper/cast 投影、body consumer、精确 site、错误/歧义 key，以及预算和取消传播。与 site 拒绝最相关的是：

- `exact_site_and_source_cast_preserve_the_child_anchor_and_presentation`（第 11875 行）验证精确 site 正例。
- `cast_mutation_skips_a_foreign_method_node_with_the_same_bci_and_name`（第 11944 行）验证相同 BCI/name 但属于另一 method 的 AST 节点不会被误改。
- `body_consumers_capture_only_exact_invocation_statements_and_their_catch_scope`（第 12067 行）验证 invocation statement 与 `CallTarget` 不匹配时拒绝。
- `result_use_is_return_and_wrong_or_ambiguous_site_keys_refuse`（第 12295 行）验证错误 opcode、重复 required BCI、重复 AST 命中均拒绝。
- `required_site_scan_propagates_budget_stop_and_cancellation`（第 12370 行）验证停止信号透传。

拒绝实际发生在 `class_source_invoke_ast_sites()`（第 1892–2000 行附近）：如果 `complete_code` 为假或 `instruction_bcis.len() != instruction_count`，返回 `None`；每个调用方提交的 key 必须对应物理 BCI，`call_targets` 中该 BCI 必须恰有一项且 opcode/target 精确相同，AST 中也必须恰有一个 BCI、method、name、descriptor 参数数量匹配的 Call 表达式，否则返回 `None`。调用 `project_class_source_invoke_argument_edits()`（约第 2997 行）时，site resolver 返回 `None` 会直接成为普通拒绝。也就是说，已提交的 site 缺失、重复或文本/target 不一致，会在 AST-site join 处被拒绝。

但函数文档明说完整 call census 由 caller 负责；并且 `required_sites.is_empty()` 会成功返回空向量。若上游 census 漏掉了某个真实 invoke，caller 只传入剩余 sites，当前 resolver 本身没有独立的完整物理 invoke oracle 来发现遗漏。上述 12 项中没有测试“完整字节码有两个同类 invoke，而上游 census 只交一个”的明确 partial-census 负例，也没有测试“物理 invoke 存在但 AST 中零个匹配表达式”的专门负例。现有 wrong/duplicate tests 不等于这两类覆盖。

## 最小有意义的负例测试建议

建议只增加一个针对 census 与 AST join 的合成负例：准备包含两个真实 invoke opcode 的完整 method fixture，并保留完整指令/opcode 清单；故意把其中一个 invoke 从候选 `SameClassInvokeUse`/site census 中省略，再调用负责验证完整 census 的 aggregation/transaction 入口。断言该受影响的关联泛型组整体回退或拒绝，同时一个无关联 leaf 仍保持其既有恢复结果。关键是测试入口必须同时看到独立的完整物理 invoke 清单与被测 census，不能只把缩短后的 `required_sites` 传给当前 resolver；后者对空列表明确接受，也无法知道调用方漏报了什么。

另一个很小但较窄的补充可以在现有 `result_use_is_return_and_wrong_or_ambiguous_site_keys_refuse` 中删除唯一 Call AST 节点、保留物理 `instruction_bcis` 与 `call_targets`，然后断言对原 key 调用 `class_source_invoke_ast_sites()` 返回 `None`。这覆盖 resolver 的零 AST match 分支，但它测试的是 AST/site 不一致，不是上游漏 census。若只能新增一个测试，优先前述带独立物理清单的 partial-census 负例，因为它验证的正是当前文档归于 caller 的责任边界。
