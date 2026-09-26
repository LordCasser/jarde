# 2.1c 独立验收：零参数与主类成员待证形状

同次类请求只在已选定匿名子类的 `Op`/`Mixed` 关系里保留两条按字段序排列的常量记录、`(String,int)` 物理构造描述符、`$VALUES`/辅助方法物理表索引、主类无 Code 抽象声明和唯一 synthetic 访问构造器的原始 marker owner。marker 必须属于同组已选子类；字段名和 ordinal 明确是**待核对的预期值**，不是 `<clinit>` 实参证明。`Plain` 没有子类关系，直接由原物理方法表证明零参数、无桥控制。本步不改变原基础 `prove_group` 的一构造器/全 Code 成功门，正例仍 `Refused`，文本与 JSON 均不产生类体投影。

代理对冻结 `Op`、`Mixed`、`Plain` 的 Java 8 `-g`/`-g:none` class 以及 `Stage`/`Measure` 运行定向测试。Root 独立复跑 `enum_constant_body_relation_tests` 1/1、基础 `enum_constants::tests` 28/28 和 `class_source` 47/47。残缺 `$values` synthetic 旗标使关系为空且组证明 `Refused`；错误 constructor owner 的变体令 IR 帧停止，关系为空且组证明如实 `Stopped`，没有伪装成可完整判定的拒绝。紧预算也没有留下关系或投影。

`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-proved-enum-constant-bodies --strict` 通过；普通 `cargo clippy -p jarde --lib` 成功，但仓库既有 `jarde-java`/`class_source`/`facade` lint 告警仍在。P1 `p1_xref_golden` 的 `metadata.json` 中 `type-and-annotation-combined-sup` 回放失败；Root 在本 change 之前的干净 `5c0c4d63` worktree 复跑同一项也失败，因此不归因于 2.1c/2.2a。基线 worktree 已删除，整仓门禁债务另行处理。

2.2b 的独占使用、2.3a 的逐边参数/哨兵委托、2.3b 的正文闭合及 2.3c 的整组成功证明仍未完成；2.1c 的待证字段不能被这些阶段直接当作已验证的实参或效果。
