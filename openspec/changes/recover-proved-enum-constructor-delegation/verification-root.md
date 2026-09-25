# Root 最终验收

`recover-proved-enum-constructor-delegation` 的 1.1–3.1 已完成。根证据为 [三方冻结输入](../../evidence/java-syntax-2026-09-25/enum-constructor-delegation/analysis.md)、[九组拒绝控制](../../evidence/java-syntax-2026-09-25/enum-constructor-delegation/negative-controls/analysis.md)、[2.2](verification-root-2.2.md)、[2.3](verification-root-2.3.md) 和 [2.4](verification-root-2.4.md) 分阶段独立验收。

最终 CLI SHA-256：`8299ace8b147b900d7471db3e988a7500836335cf552182a7b472938f47a5df5`。Root 从原始 Java 源分别重编 `-g`/`-g:none` 两套 class，原/JADX/Jarde 完整类均通过 `javac --release 8` 和 `java -Xverify:all`，逐字相同地输出 `values=ZERO:0,ONE:1`、`effects=2:0,1`、`declared-constructors=2,3`。Jarde 的默认/all 类正文 SHA 同为 `c748f28b586e93a7763f83314a7ba9db7e21cf294d4846283de1fa1a4f68ded5`；物理四字段、七方法及两个构造器的 JSON 身份/来源保留。九组合法负例由 Root 在仓库外用最终 CLI 再重放，原/JADX 可验证，Jarde 9/9 未误投影。

实现只在现有 class-source 的同轮 Code/AST 候选上增加有界证明与原子投影：无参构造器必须按身份原样转发注入的 name/ordinal 与真实常量 0，终端构造器的 helper 与字段效果必须按 BCI/消费闭合。此规则借鉴 JADX 定位构造调用的顺序，但不照搬其无条件隐藏前两个实参或 `$VALUES` 辅助方法的启发式；冻结控制已证明这些启发式可能改变 name、ordinal 和效果顺序。未增加通用反编译 pass。

回归：`jarde --lib` 57/57、`tests/class_source` 47/47、`jarde-java --lib` 180/180；`jarde-reader --lib` 排除一条夹具数量旧断言后 176/176，`jarde-cli --test class_source_cli` 排除一条普通类来源注释旧断言后 15/15。两个排除项的原始失败和期望/实测值见 [2.4 记录](verification-root-2.4.md)。fmt、diff、OpenSpec strict 均通过；Clippy 退出 0，仍有 10 条相邻 warning。私有 Cargo target 已在最终复核后用 `cargo clean --target-dir /tmp/jarde-enum-delegate-projection-target` 清理。
