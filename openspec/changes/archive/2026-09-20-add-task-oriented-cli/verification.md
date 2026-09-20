# 验证记录

实现提交见仓库历史（`85828c4`，紧邻本文件归档提交之前），本机门禁如下；CI 结果随后回填。

## 契约实现

两个表面、一个二进制（`crates/jarde-cli/`）：既有 `--request FILE`/stdin 的 JSON 控制面**逐字节未变**（退出 0/1），另加五个子命令；同时给出两者是用法错误（退出 2）。

| 子命令 | 主要参数 | 调用的库入口 |
| --- | --- | --- |
| `list-classes` | `--evidence declarations\|candidates`（默认 declarations） | `Engine::list_class_declarations` / `list_class_candidates` |
| `list-members` | `--definition <PhysicalDefinitionId JSON\|@FILE>` | `Engine::list_members` |
| `references` | `--class-name`、`--member`、`--member-name`、`--descriptor`、`--relation`、`--consumer`（可重复，≥1）、`--max-items` | `Engine::query` + `ReferenceGrouping::from_query` |
| `class-view` | `--class-name` \| `--definition`、`--body`、`--body-method`（均可重复） | `Engine::class_view` |
| `recover` | `--method` \| (`--class-name`+`--method-name`+`--descriptor`)、`--policy`、`--root`、`--release/--multi-release/--layout` \| `--profile`、`--loader` | `Engine::recover_target` |

公共参数：`--input`、`--scope`（默认 `snapshot_all`）、`--budget DIM=LIMIT`（可重复，经 `BudgetOverride::new` 校验）、`--format text|json`（默认 text）、`--output FILE`。JSON 模式写出库报告自身的序列化（含 `OperationOutcome` 标签），无信封、无改名；文本模式是同一文档的投影，`limits`/`usage`/`coverage`/`execution`/`diagnostics` 走 stderr，其余与逐字节相同的 `recovered.recovery.text` 走 stdout。

## 逐任务证据

- **1.1 只传参、不自行读取**：`list-classes`（两种证据等级）与 `list-members` 的 CLI JSON 与 `serde_json::to_value(库报告)` 逐字段相等（仅剔除 `elapsed_millis`）；候选列举另断言 `class_headers == 0`、`class_bytes == 0`，确认列举断言 `class_headers == items.len()`——自行读取的适配器会移动这些计数。
- **1.2 文本与 JSON 同源**：五个命令的每条 `path = value` 行都按它声明的路径在 JSON 文档中查到并比较；`recover` 的 stdout 与 `recovered.recovery.text` 逐字节相等。
- **1.3 诊断与正文分离**：以已提交的 ECJ 样本 `finallyPath(I)I`（真的会告警 `jre_region_unaccounted_instruction`）验证——stdout 只含恢复正文，stderr 携带 `recovered.recovery.diagnostics.0.code = "jre_region_unaccounted_instruction"` 及其它计费平面；测试断言 stdout 与 `text` 字段**逐字节相等**且不含任何 `recovered.recovery.diagnostics…` 行与该诊断码（之所以用逐字节而非子串搜索：恢复文本自身会把同一条理由作为注释引用，见边界）。
- **2.1 输出文件**：JSON 文件与 stdout 文档逐字段相等（两次运行只可能在 `elapsed_millis` 上不同），文本文件与 stdout 逐字节相等，使用 `--output` 时 stdout 为空；写出失败时**不留下文件**，失败文档（stderr）陈述 `"dimension":"output_bytes"` 与已发生的用量，而运行自身的文档仍以真实 `partial` 平面交付。
- **2.2 退出状态**（各有证明 fixture）：0 = 报告发布的每个执行平面都是 `complete`；1 = 文档无法交付（如输出目录不存在 → `cli_create_output`）；2 = 用法/输入错误（缺 `--consumer` → `cli_consumer_kinds_missing`；`operation_target_not_found`；`output_bytes` 拒付）；3 = 名称有多个身份且库未运行任何东西（两个 origin 上的 `p/Base`，两个候选 + `outcome: ambiguous`）；4 = 执行未完整（`--budget class_headers=1` → `partial`、维度 `class_headers`、1 个条目，**退出 4 而非 0**；坏 child 的树枚举与声明的 external root 同样如此）。
- **3.1 发现面**：用 CLI 自身的 `enumerate_artifact_tree` 报告取得真实 root container；普通 scope 只把嵌套库作为 *resource* 条目、不产生嵌套候选，`--scope artifact_tree` 才到达嵌套类并保留枚举自己的 `ContainerOrigin`；坏 child 给出库的 `partial` + provenance 诊断 + 退出 4，并与库逐字段比较；不生成任何 root（报告回显的正是声明的环境身份，声明的 `LoadRoot::External` 保持库自身的不可用状态、退出 4）。
- **3.2 任务链闭环**：在受控归档上跑（1）`list-classes --evidence declarations` → 原样取 `items[i].definition`；（2）`list-members --definition <该 JSON>` → `method_bodies == 0`，取 `items[j].identity`；（3）`references …` → 与库 `ReferenceGrouping::from_query` 逐字段相等，调用点归到调用方法的身份下；（4）`recover --method <该身份>` → 与库 `OperationOutcome` 逐字段相等；`class-view --body-method <同一身份>` 只计一个 body。同样四步在已提交样本上端到端跑通。

## 任务链最后一步的读取范围（逐字）

受控 fixture 类 `p/Base` 声明 4 个方法（`foo()V`、`foo(I)V`、`bar()V`、`nativeCall()V`）与 1 个字段；`recover --method <foo()V 身份> --policy plain-jar`：

```text
{"analysis_steps":54,"archive_entries":13,"attribute_bytes":92,"class_bytes":221,"class_headers":1,"code_bytes":8,"method_bodies":1,"ir_edges":9,"ir_items":107,"read_bytes":221,"result_items":12}
outcome: performed | presentation execution status: complete | text length: 248
```

`class_headers: 1`、`method_bodies: 1`、`code_bytes: 8`——只读一个 Header、只读被请求的那一个 Body。已提交样本（`HistoricalControlFlow`，3 个方法）同样 `class_headers: 1`、`method_bodies: 1`。

逐 body 计费（`class-view --definition <身份>`）：`foo()V` → `method_bodies 1`/`code_bytes 8`；`bar()V` → 1/3；`nativeCall()V` → 0/0 且 `not_declared`、`no_body_kind: native`。

## 门禁

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo test -p jarde-cli --test task_cli --locked` | 9 passed / 0 failed |
| 既有 CLI 套件（未修改） | 6 + 16 + 11 passed / 0 failed |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1249 passed / 0 failed / 6 ignored**，连跑两次均一致（基线 1240，+9 task_cli） |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed / 0 failed（17.6 s） |
| `openspec validate --all --strict --no-interactive` | 20 passed / 0 failed（归档前） |
| 实现提交的 CI（`85828c4`） | [run 35502305001](https://github.com/LordCasser/jarde/actions/runs/35502305001) **四 job success**：stable（fmt、clippy `-D warnings`、两轮固定 seed 全量测试、JDK 25 oracle、P3 编译执行对照、依赖边界、OpenSpec strict、`git diff --exit-code`）、MSRV 1.88.0、supply chain、fuzz smoke |

未新增 fixture 文件：`task_cli.rs` 全部内存构造（STORED-only ZIP writer + class-file writer），真实样本复用已提交的 ECJ v52，已在 `tests/fixtures/README.md` 索引，corpus fingerprint 未变。

## 设计缺口与本人决定的取舍（已披露）

- **库层无法表达的一处（记录为边界，而非在本 change 内补齐）**：`references` 步**不能接受物理方法身份**——`Engine::query`/`declaration_references` 取 `SymbolRef`，其 owner 是声明类的内部名，而 `PhysicalMethodId` 只有 location/class-bytes digest/variant，名字不在其中；要得到它就得在 CLI 里重读类 Header 或从 entry 路径推断名字（两者都被明令禁止）。因此 `references` 接受上一命令打印的**拼写**（`this_class.escaped` + name + descriptor）加声明的 consumer 类型；承载身份的边是 `list-classes → list-members`（`PhysicalDefinitionId`）与 `list-members → class-view/recover`（`PhysicalMethodId`），测试逐字驱动这两条。规格的链路场景正是「列类、列方法取得物理身份，再用该身份发起恢复」，故不构成规格缺口；若要「按物理身份查引用」，应在任务导向库层新增入口。
- **自定取舍**：五个子命令（不外露纯分析的 `analyze_target`，分析仍在既有 `analyze_method` operation 上）；`--evidence` 选择库自身的两级列举；默认 text、`snapshot_all`、release 8 / MR disabled / layout generic / loader `app`；`--consumer` 至少一个库自身的 snake_case kind（schema 版本 1，报告回显实际运行的 schema）；类名含 `/` 视为内部名、否则视为点分名（用库自身的 `ClassNameQuery` 构造器）；`--root`/`--profile` 作为友好旗标拼不出的状态的口子；失败把库 `Error` + usage 写 stderr 并据以分类（失败不是报告），退出 1 保留给交付失败以免破坏既有 0/1 语义；参数在打开 artifact **之前**解析，打开后的失败陈述已发生的用量。
- **既有弱点、本次未改**：(a) 恢复层会把诊断消息作为注释写进生成的 Java，因此「正文里没有诊断」只能以「stdout 与 `text` 字段逐字节相等」证明；(b) `usage.output_bytes` 由库在运行中计费，CLI 的交付计费在该维度上是叠加的（输出文件用例据此从实测用量推导上限）；(c) `ClassViewReport.execution` 不折叠 body 级停止或 body 拒绝（body 自带平面），CLI 在判定退出状态时折叠每个 body 的平面——这是对库已发布事实的 CLI 读法，已在文档说明；(d) `PhysicalScope::ArtifactTree { root_container }` 命名的是 container 而非加载位置，调用方仍需单独声明 prefix root（README 已说明）。

## 状态

实现与门禁证据如上；本 change 的实现提交同时修掉了 `tests/navigation.rs` 的一处假红（比较停止平面时带了 `elapsed_millis`，已在归档的导航验证记录中登记）。README/支持矩阵的「当前复核边界」行需要随本 change 之后的状态一并更新，在归档提交中处理。
