# 验证记录

实现提交见仓库历史（紧邻本文件归档提交之前），本机门禁结果如下；该提交的 CI 结果随后回填。

## 契约实现

`LoadRoot`（唯一公开形状变更，`crates/jarde-reader/src/view.rs`）收敛为三态：

```rust
pub enum LoadRoot {
    StandaloneClass { snapshot: SnapshotId },
    Container { origin: ContainerOrigin, prefix: ArchiveNameBytes },
    External { id: String },
}
```

旧变体不保留、无别名：`{"kind":"snapshot"}` / `{"kind":"artifact_tree"}` 不再反序列化。旧→新按语义一一对应：standalone snapshot → `StandaloneClass`；ZIP 根 → `container` + 空 prefix；tree 根 → 其 container + 空 prefix；`external` 不变。

- **运行期开关点**（全部穷举匹配，无遗漏）：`runtime_matrix::root_covers`（prefix 故意不参与——它是 container 内的查找键）、`environment::validate_domain_roots`（新增闭合错误码 `InvalidRootPrefix`，`ALL` 变为 10）、`providers::probe_root` / `root_snapshot` / `position_label`、`dispatch::declared_evidence`。
- **仓库内所有构造点一次迁移**：测试侧按 `snapshot.kind()` 选择变体的 helper（忠于旧 `Snapshot` root 的分派语义，不是兼容层）、container-only fixture、显式 standalone fixture、以及旧 `ArtifactTree{root}` → 其 container + 空 prefix；金标 JSON 7 份随之迁移（`p2-golden/*` 93 条 → `standalone_class`，`p1-golden/multi-release.json` 6 条 → `container` + 空 prefix），并重生成 corpus fingerprint（diff 只含这 7 处）。
- **prefix 拼接**：`position_entry_name(prefix, internal_name)` 是本仓库唯一拼接点，checked length，`prefix + internal_name + b".class"`，不 trim、不解码、不折叠大小写或点段、不依赖 ZIP 目录条目；拼接结果交给上一轮落地的 `ArtifactSnapshot::container_candidates`（container origin + raw name），**不重新枚举树、cache key 不含 prefix**。
- **binding 补齐**：`this_class` 与请求内部名的一致性检查落在所有 archive/container 候选共同的选举行 `providers::decide` 中（读记录保留，拒绝可定位）；歧义位置不作断言（它本就不选候选）；`known` 复用路径的唯一调用方只在声明自身的 `this_class` 名下搜索，已在原地注明。新增 crate 私有诊断码 `resolution_definition_name_mismatch`，绝不退化为 `Missing`，也绝不「试下一个 root」。
- **CLI 闭环**：新增薄 operation `enumerate_artifact_tree`（直接返回库 report）；测试证明「CLI 枚举 → 真实 container origin + entry 身份 → 调用方显式声明 prefix → `recover_method`」可用，且 source map 仍指向真实的 `WEB-INF/classes/com/demo/App.class`，`BOOT-INF/classes/` 样本走同一机制（明确标注为 prefix 样本，**不宣称** Boot loader 支持）。

## 关键证据

| 场景 | 结果 |
| --- | --- |
| 无目录条目的 WAR、空/非法 prefix、布局节点单独存在 | prefix 可绑定；非法 prefix 在 `Root{app,0}` 给 `invalid_root_prefix`、`NotPerformed`、零读取 |
| raw 字节（非 UTF-8、多字节、大小写、点段、反斜杠、把 `p/A.class` 当名字） | 全部精确匹配、不规范化；`P/A` Missing；`p/./A.class` 与 `p/../A.class` 是两个不同定义 |
| `this_class` 与请求名不一致 | 在该候选自己的位置拒绝（`resolution_definition_name_mismatch`），后续诚实 root **不会被搜索**，`reads.len() == 1` |
| 形状混用 | `not_zip` / `not_standalone_class`，在读取处拒绝 |
| 声明顺序 vs 扫描顺序 | 声明顺序胜出（即便胜出 entry 的 ordinal 更大）；ChildFirst/ParentFirst 各自移动它声明的多个位置 |
| 应用目录与嵌套库同名、同位置重复、同字节不同 origin | 分别是两个位置 / `Ambiguous` 且保留各自 ordinal / `class_bytes` 相同但定义不同 |
| 坏候选、不完整目录、换 prefix | 在自身位置停止并带 provenance；不完整目录是 `BudgetExceeded` 而非 `Missing`；换 prefix 是新环境、复用任何旧结论都失败（off/on 对照） |
| 环境身份 | 只有 prefix 不同的两个 root 不相等；`analyze_method` 带 prefix 为 `Completed`，同一定义同内容但空 prefix 为 `Failed{resolution_definition_unbound}` |

## 被本 change 迫使修正的两处既有测试（已披露，非放宽）

1. `tests/p5_container_lookup.rs`：原 fixture 让多个路径复用同一份 class body（`p/Other.class` 与 `p/Top.class` 同字节），在新的名字绑定检查下必然被拒——现在每个 entry 声明自己的名字，6 个 fixture digest 重新固定并在注释里写明「字节因本 change 移动过一次」。形状与测试主题未变。
2. `tests/p2_members.rs` 的 F1 用例改名为 `an_entry_that_declares_another_name_is_refused_at_its_own_candidate`：它依赖的畸形 artifact（entry 路径与 header 声明不符）现在**不可能**再产出 `Found`，这正是本次补齐检查的目的。原 F1 性质（「一个解析绝不可以在其 header 未声明的名字下成为 `Found`」）保留并改由两条路径的拒绝各自断言；被请求的物理身份改从 snapshot 自身枚举取得，而不是从一个不再可解析的名字反推。

## 门禁

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo test --test p2_prefixed_roots --locked` | 17 passed / 0 failed |
| `cargo test -p jarde-cli --locked` | 6 + 16 + 11 passed / 0 failed |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1187 passed / 0 failed / 6 ignored**（基线 1164，+17 prefix、+6 CLI） |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed / 0 failed（17.8 s） |
| `openspec validate --all --strict --no-interactive` | 23 passed / 0 failed（归档前） |
| `cd fuzz && cargo check --all-targets` | 通过（fuzz 是独立 workspace） |
| 实现提交的 CI（`fbcf06b`） | [run 35497694448](https://github.com/LordCasser/jarde/actions/runs/35497694448) **四 job success**：stable（fmt、clippy `-D warnings`、两轮固定 seed 全量测试、JDK 25 oracle、P3 编译执行对照、依赖边界、OpenSpec strict、`git diff --exit-code`）、MSRV 1.88.0、supply chain、fuzz smoke |

未新增 crate、依赖或 manifest 改动；除枚举变体与 `EnvironmentProblemCode::InvalidRootPrefix`（+`ALL`）外，其余新增符号均 crate 私有。未新增 fixture 文件、未调用编译器（全部内存构造）。

## 边界

- **环境验证只能部分表达设计里的「检查 container origin」**：无字节的验证器无法证明 origin 链由其 snapshot 派生（reader 的 `"root"` container id 是它自己的约定），因此验证器检查 root 的 snapshot/内容与 prefix 形状，origin 在真正搜索的位置校验（`container_candidates` 的 `child_container_mismatch`/`entry_origin_mismatch`，由上一轮用例覆盖）；`Container` 与 standalone 形状混用在读取处拒绝，同样在文档里写明。
- **取消不可经 CLI 适配层到达**（无 token 字段，只有 `elapsed_millis`），因此没有声称 CLI 侧 `Cancelled` 一致性；库侧 `Cancelled` 树报告由既有用例覆盖。
- MR/module/自动布局加载的声明范围未被扩大：`classpath.idx`、`BOOT-INF/lib` 顺序与 launcher 均未触及。
- 顺带记录、本次未改：providers 测试构造器 `class_with_body` 的 `super_class` 指向文本 `#3` 的 CP 条目（`[7,0,3]`，应为 `[7,0,2]`）；CLI 内部标记的 unit 变体忽略未知字段；dispatch range 仍枚举整树（`resolver.rs::search_coverage_with_artifact`，上一轮已记录）。
- 附带影响：`crates/jarde-java/**` 只有 4 行测试/cfg(test) 处的 `LoadRoot::Snapshot` → `StandaloneClass` 迁移（作为 breaking 枚举变更的编译后果，实现该 crate 的兄弟 change 未受影响）。
