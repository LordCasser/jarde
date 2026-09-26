# 2.1b Root 验收

`class_source_with_evidence` 仅在两常量候选的基础组证明仍为 `Refused`、同次 `<clinit>` 分配扫描完整且含外部构造 owner 时，沿原选定环境解析该 owner 的唯一物理定义。待证关联按 `new` BCI、`invokespecial` BCI、紧邻的常量字段 `putstatic` 与物理字段顺序一一配对；从选定子类的同一份字节经既有 typed reader 核对 `this_class`、直接父类、匿名 `InnerClasses` self 行、`EnclosingMethod` 和唯一匹配构造器，并核对主类的对应 `InnerClasses` 行。合法匿名行的 `outer_class_index=0`、`inner_name=None` 与 `EnclosingMethod.method_index=0` 不被误判缺失。关系仅进入私有 `serde(skip)` 侧车，不改变组 `Refused`、JSON 或类源码，也不隐藏子类物理报告。

Root 审核初稿时发现按字段循环重复选首个分配点的误绑定风险，最终实现改为先构造并排序两个互不重用的构造点—字段配对。Root 独立运行关系测试 1/1、`enum_constants::tests` 28/28、`--test class_source` 47/47、`cargo fmt --all -- --check`、`git diff --check` 与 `openspec validate recover-proved-enum-constant-bodies --strict`，均通过。端到端测试将冻结源码以 Java 8 `-g`/`-g:none` 编译为同一 JAR：`Op` 精确映射 `(字段 0,new 0,ctor 7,Op$1)` 与 `(字段 1,new 13,ctor 20,Op$2)`，`Mixed` 仅首字段到 `Mixed$1`，`Plain` 无子类关系。同名无关类、缺少/重复 `Op$1` 定义及篡改其合法匿名属性均不形成该关系；`Stage`/`Measure` 仍为 `Proved`，在完整分析步限额下复跑完成且不多读子类。所有这些类在本步仍不输出未证常量体。

`cargo clippy -p jarde --lib` 可完成，新增关系函数无 lint；`-D warnings` 被 `jarde-java` 既存的 enum-switch、region、report 等 20 个 lint 阻断，未混入本变更修复。2.1c 的主类抽象/访问构造器形状、2.2 的独占使用和 2.3 的委托/正文闭合仍未完成；待证关系不得当作成功组证明。
