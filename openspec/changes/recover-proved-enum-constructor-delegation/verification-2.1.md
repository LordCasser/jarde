# 2.1 同轮构造器委托边验证

2.1 在 `src/enum_constants.rs` 增加私有 `prove_constructor_delegation_edge`。`DelegationEdgeInput` 将基础 `prove_group` 同轮已验证的常量三元组、`$VALUES` 字段名、物理成员表和 `capture_method_code` 收集的 `<init>`/`<clinit>` Code 候选打包传入；`method_code` 继续按物理方法表索引和同轮 member identity 核对候选。没有新增 `MethodIr` 分析，也不从恢复文本提取语句。共享 `prove_initializer_prefix` 现接受有序的构造器描述符与整数实参约束：基础单构造器路径仍使用 `AnyLiteral`；双构造器路径要求 `ZERO` 精确调用 `(String,int)` 且不传源整数，`ONE` 精确调用 `(String,int,int)` 且传字面量 `1`。

定向正例从 Java 8 编译的枚举经 class-source 同轮候选证明，委托构造器只含入口 `this`、name、ordinal、字面量 `0`、唯一终端构造器调用和 `return`；两处 `<clinit>` 构造调用的 BCI 为 `[7,21]`，描述符依序是 `(String,int)`、`(String,int,int)`，源整数为 `[无,1]`。该私有事实不会提升为类级 `Proved`：class-source 报告仍为整组 `Refused`，常量字段和两条物理构造器均保留，避免在终端效果及投影未证前发布源码形状。

定向拒绝测试覆盖常量错选重载、name/ordinal/委托零值被改写、`ONE` 整数实参改变、桥内额外调用、桥和 `<clinit>` 分支/异常处理器、物理构造器表重复及 Code 候选重复；另验证预算耗尽与取消作为停止返回，不会变成普通通过。预算按新增的字段/方法表、候选表和指令访问收费。异动候选属于证明函数级拒绝控制；JVM 可运行的九组 verifier-valid 反例与哈希/BCI 记录见 [1.3 控制集](../../evidence/java-syntax-2026-09-25/enum-constructor-delegation/negative-controls/analysis.md)。

本阶段只证明调用边和共享常量前缀。终端构造器异常表、用户 helper/字段效果、`Signature` 到源码参数的对应、同轮构造器 AST 交接以及整组源码投影仍未证明，留给 2.2–2.4；因此这里不声称双构造器枚举可投影，也没有重编投影类或宣称其反射形状等价。

验证命令：

```text
cargo test --target-dir /tmp/jarde-enum-delegate-edge-target -p jarde enum_constants::tests --lib
cargo clippy --target-dir /tmp/jarde-enum-delegate-edge-target -p jarde --lib --no-deps
rustfmt --edition 2024 --check src/enum_constants.rs
openspec validate recover-proved-enum-constructor-delegation --strict
```

聚焦模块测试 19/19 通过；Clippy 退出 0，报告仍为仓库其余位置的 10 条既有警告，本函数未新增参数数量警告；格式检查与 OpenSpec strict 均通过。Cargo 编译产物保留在专用 `/tmp/jarde-enum-delegate-edge-target`，本轮未清理，由 Root 独立验收后处理。
