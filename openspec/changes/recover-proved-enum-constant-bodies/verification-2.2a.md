# 2.2a 独立验收：精确原始 owner 候选

`CandidateFilter::Owner` 只在现有 P1 `scan_candidates` 的候选匹配点比较原始 owner 字节；`Class`、`Method`、`Field` 保留原始符号，物理遍历、consumer、计费、覆盖率和停止路径不变。公开 query request/schema 没有改动。这是 2.2b 做独占使用证明所需的候选入口，**本步尚未证明任何 enum 子类独占，也不授权源码投影**。

测试的 [OwnerRefs.java](../../../crates/jarde-query/tests/fixtures/OwnerRefs.java) 用 `javac --release 8 -g` 生成 [OwnerRefs.class](../../../crates/jarde-query/tests/fixtures/OwnerRefs.class)；Root 在临时目录重编，确认 1,084 字节逐字节相同，classfile major=52。定向物理扫描覆盖 `new` 类符号、方法/字段 owner、`ldc`/bootstrap 方法句柄；同一输入的 `MentionsSymbol(Class)` 精确查询确实漏掉成员 owner。`max_items=1` 与 `ResultItems=0` 保留 `has_more` 和 Partial 覆盖，不把停止当作零使用。

Root 在隔离 worktree 独立执行：

- `cargo test --locked --target-dir /tmp/jarde-enum-body-2-1a-target -p jarde-query --test owner_candidates --quiet`：2/2；整个 `jarde-query` 包 7 项单测加 2 项集成测试通过。
- `cargo fmt --check -p jarde-query`、`git diff --check`、`openspec validate recover-proved-enum-constant-bodies --strict`：通过。

2.2b 仍须按选定输入范围逐份扫描并解释所有候选及覆盖率，再同 `<clinit>` 精确前缀合证；owner 过滤本身不处理对象别名、隐式元数据或子类构造语义。
