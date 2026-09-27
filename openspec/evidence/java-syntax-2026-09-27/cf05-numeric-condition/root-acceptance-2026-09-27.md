# CF-05 主线独立验收

在主线 `f808a62e`，root 审查了 `ConditionalValueProof` 的唯一 Phi 消费、字段 descriptor、目标 `(B)/(S)` 描述符和逐臂范围证明；实现只复用既有条件值 AST 和调用参数适配，没有新增全局数值转换规则。错字段类型、错目标、超范围、额外 Phi 使用、异常边与停止均有负例。

以主线重新构建 `jarde-cli`，独立运行 [固定脚本](replay.py)至 `/tmp/jarde-cf05-root-replay-20260927`。原 class、固定 JADX、Jarde 的完整 `ConversionCases` 均以 Java 8 重编、`-Xverify:all` 运行且 20 行逐字一致；`ConversionBasic` 三方十行逐字一致。Jarde 源码 SHA-256 分别为 `65d2dd14e44052796e41850170b423539be605191e610bf26c346dc7be068225` 与 `b6c8f8138cf019264648073a527a99e1e0d62f2b4b0c170bd0c108f159aa617f`，与实施者的修后证据相同。`p3_patterns` 65/65、`cargo fmt --check`、`git diff --check` 和 OpenSpec strict 通过。

这只验收准确 B/S 目标下的有界条件实参；三个 Smali 测试的泛化数值恢复以及 fixture 外的 `final` 字段初始化没有由此获得证明。
