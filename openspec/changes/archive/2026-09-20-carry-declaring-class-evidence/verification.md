# 验证记录

固定提交 `618de49`（实现），归档提交见仓库历史；该提交的 CI 结果见下表。

## 契约实现

- `MethodDeclaration` 增加 driver class 的 raw internal name（`this_class`）与 class `access_flags`，用 reader 自己的字节表示（identity 比较不做 lossy 转换），绑定到同一物理定义的 `identity`；新增只读 getter `class_name()`/`class_access_flags()`，构造入口仍是 crate 私有的 `new`，没有公开的可变或任意构造 `MethodIr` 的路径（seam guard 用例继续通过）。
- 事实来源是同一次 Header 读取：`engine.rs::read_driver_method` 的 `read.header.facts`（`ClassFacts`）——与 version/pool/bootstrap/`FrameDeclaration.this_class` 同一个值；交接发生在同一处 struct literal 里。没有第二次 Header 读取、没有第二次扫描、没有扩大 Body 闭包、没有把 `ClassFacts`/CP 复制进 IR。
- `src/facade.rs::recovery_facts` 用既有的 `DeclaringClass` 适配这两个字段（`with_declaring_class`）；名称经与同函数既有成员名相同的 `String::from_utf8_lossy` 显示，不新建名称体系（原因见「边界」）。
- `crates/jarde-java/src/declaration.rs` **未改逻辑**：`declaration@1` 现有规则即可表达普通实例/static、interface `default`/`static`/abstract、`<init>`、`<clinit>`；只修正了声称「载荷没有任何类级事实」的模块文档。同一事实也到达 `init@1` 与 `field@1` 的 uninitialized-`this` 分支。

## 逐例证据（`tests/p3_declaration_handoff.rs`，8 项）

| 输入 | 结果 |
| --- | --- |
| `Scope.receiver(J)J` / `simple()I` / `<init>()V` | `InstanceMethod` / `StaticMethod` / `Constructor`，`declaring_class` 来自本次读取 |
| `Guarded.<clinit>()V` | `StaticInitializer` |
| `Shape.scaled(I)I`（interface `default`） | `DefaultMethod`，`interface=true`，flags `0x0001`，envelope 行 `// @declaration an interface's default method of \`Shape\`, member flags 0x0001` |
| `Shape.sum(II)I`（interface `static`） | `StaticInterfaceMethod` |
| `Holder.<init>(I)V` / `value()I` / `of(I)LHolder;` / `<clinit>()V` | 各自 form 正确 |
| 同一 internal name、class flags 不同（`0x0031`→`0x0601`，按常量池定位并对照 reader 自己的 header read） | `value()I` 一侧 `InstanceMethod`、另一侧 `DefaultMethod`，都拼作 `Holder`，各自绑定自己的 digest |
| `Shape.sides()I`（abstract / native） | `DeclaredWithoutBody`，`declaration == None`，恢复 `Stopped(IrTableMissing{canonical})`，`text == ""`，`method_bodies=0` |
| `class_headers: 0` 预算 / 预取消 | `Partial{BudgetExceeded{ClassHeaders}}` / `Cancelled`，0 次读取，无声明；低层 `jarde_java::recover` 在无事实时仍给 `jre_declaration_class_not_in_run` |

## 无新增扫描（确定性字段逐项相同）

`Scope.simple()I`：`read_bytes 532, class_bytes 532, attribute_bytes 277, code_bytes 4, output_bytes 532, class_headers 1, method_bodies 1, ir_items 53, ir_edges 3, analysis_steps 24, normalization_clones 0`（仅 `elapsed_millis` 0→1）；`Scope.receiver(J)J`、`Guarded.<clinit>`、`Guarded.fin` 同样逐项相同。另加源码级守卫：`read_driver_method` 只有一处 `HeaderDemand::DriverMethodBody`、一处 `method_code_facts(`、没有 `class_facts(`，交接两行读的是 `read.header.facts.*`。

## quality 未被声明升级

`Guarded.fin()V`：`Fallback / Mixed / ExplanationOnly / NotJava`，`fallbacks == ["jre_guard_finally_copy", "jre_region_uncovered_blocks"]`，compile/semantic/verification 状态未变；只有 envelope 多了一行声明，`declaration@1` 进入 `rules`，诊断由 `jre_declaration_class_not_in_run` 变为 `jre_declaration`。

## 判别性（任务点名的变异）

删掉 `src/facade.rs::recovery_facts` 里的 `.with_declaring_class(...)` 后跑该用例：**3 passed / 5 failed**，其中公开入口用例失败行为

```text
a class's instance method: the class the run's own read declares
  left: None   right: Some("Scope")
```

并有一条专门断言「声明正是交接带来的东西」的失败（`jre_declaration_class_not_in_run`）。变异已恢复，最终树只含预期改动。

## 顺带修复的既有缺陷（本 change 一并交付，已披露）

`crates/jarde-java/tests/p3_patterns.rs::a_body_without_debug_metadata_is_still_presented_with_deterministic_names` 比较两份整报告时把 `usage.elapsed_millis` 也比了进去——而 `recovery-validation` 的确定性要求写明只剔除这一个观测字段。它在本次全量运行中偶发失败，并在**未修改的 HEAD** 上同样可复现（`elapsed 4` vs `0`）。现在只对该字段做归一化，其余字段仍逐项比较；另用 `/tmp` 临时探针证明：纯时钟差异（`+7 ms`）在归一化后不再失败，而结果面改动（`quality`）与计数改动（`ir_items`）仍会使比较失败。这是首次全量绿色运行（`1135/0/5`）。

## 门禁（`618de49`，本机）

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo test --test p3_declaration_handoff --locked` | 8 passed / 0 failed |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1135 passed / 0 failed / 5 ignored** |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed / 0 failed（18.8 s） |
| `cargo test --test p5_corpus_fingerprint --locked` | 5 passed / 1 ignored（files 101→105，`recovery` 维度 carrier 21→23） |
| `openspec validate --all --strict --no-interactive` | 21 passed / 0 failed（归档前） |
| 该提交的 CI（`618de49`） | [run 35495275106](https://github.com/LordCasser/jarde/actions/runs/35495275106) **四 job success**：stable（fmt、clippy `-D warnings`、两轮固定 seed 全量测试、JDK 25 oracle、P3 编译执行对照、依赖边界、OpenSpec strict、`git diff --exit-code`）、MSRV 1.88.0、supply chain、fuzz smoke |

## 边界

- **设计里「非法名称影响 syntax 状态」没有实现**：非法 class name 只落在注释里，实测其 `quality/representation/syntax_status/content/compile/semantic/verification` 与对照完全一致，文本差异只是 `Shape`→`1.hap`；要让非法声明名影响 syntax 需要改 emit/syntax 规则，超出本 change 范围，故如实记录而不是硬凑。
- **显示边界的代价**：reader 的安全显示 `JvmString::escaped()` 在 facade 侧不可达（`JvmString::from_parts` 是 crate 私有，`JvmString` 只能由一次读取产生），因此沿用同函数既有的 `from_utf8_lossy`；两个只在无法解码的字节上不同的非法名会显示成同一个字符串，而 identity 仍然精确。要消除这一显示碰撞需要改 reader，那由其它 change 拥有。
- **范围说明（envelope 之外的行为变化）**：同一个 `DeclaringClass` 也喂给 `init@1`/`field@1`，所以构造器体可以离开 fallback：`Scope.<init>()V` 由 `Mixed/Fallback` + `jre_init_class_not_in_run` 变为 `Java/Structured` 且有 `super();`，`Holder.<init>(I)V` 呈现 `super(); arg0.value = arg1;`。这是设计决定 2 的直接后果，且只发生在 fallback 真的消失的地方；仍有 fallback 的 `fin()V` 未变。
- **其它类级 metadata 仍未支持**：只交接了 `this_class` 与 class flags；superclass、`InnerClasses`、`MethodParameters` 等仍不在载荷。
- 顺带记录、本次未改的观察：`quality == Fallback` 可以伴随空的 `fallbacks` 平面与全 `structured` 的 regions（语句级拒绝在 `program.ragged` 与 shape 记录/诊断里），即 `fallbacks` 不是「质量为何下降」的清单；这是既有事实，另行评估。
