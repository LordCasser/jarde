# root-v6 私有 patch

root-v6 基于已接受 apply-check 的 root-v5 组合 patch，只收紧既有 `andWhile` 和 `orWhile` 两个断言中的 derived-origin 谓词：and 要求 derived origin 自身 `bci() == 20` 且 method identity 匹配 `and_while.item.identity`；or 要求 `bci() == 19` 且 method identity 匹配 `or_while.item.identity`。这样不能由同一段上的其他派生来源替代目标 latch anchor。其余 v5 测试条件和 root-v4 产品 delta保持不变。

`loop-latch-origins-root-v6-test-only.patch` 是叠加到 v5 的两处谓词增量；`loop-latch-origins-root-v6.patch` 是 v5 加该增量的组合 patch。未执行 Git apply 或任何验证工具。自动 hunk 计数校验：增量 `2` 个，组合 `14` 个，均匹配。

SHA-256：

- root-v5 patch：`1c6afb3fc86f1f91f59e207e2b7814c5ae83fdf8a14ad1e0d5e3cee8a13c9387`
- root-v6 test-only delta：`c0e4e2ba89952a15b1ebfbc785d0fffc9b22a7b5647bb718791c35ea9b3c3ef6`
- root-v6 组合 patch：`aedfdfbfa16d6d7c1feb8e7a9be72e242893917bbc433af1ff087a4a2b59f425`
- validation runner v3：`ffd135a3df3751d730034216de6aaf441700cbd94082991ea4b7f229581801e3`
- validation runner v3 README：`c3bed60820576ac0e15dd1963df8c9a31f336d3656b03b84d60748fe2a070dc9`
- JDK controls manifest：`ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec`
