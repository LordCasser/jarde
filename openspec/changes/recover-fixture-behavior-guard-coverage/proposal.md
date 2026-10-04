## Why

**系统性验证债务**，由 root 2026-10-04 验收 `recover-synthetic-ctor-super-order` 引入的静默行为回归时发现：`tests/fixtures/proved-java-structure/` 的冻结 fixture 中有 **8 个未被任何 CI 测试引用**，其记录的行为基线只存在于 README 与**手动** `run.sh`（实测：CI workflow 与全部测试文件均不调用 `run.sh`）。后果已由真实事故证明——ctor 重排使 `anonymous-super-dispatch` 的 `visibleDuringSuper` 从 `true` 翻转为 `false`，而 CI 全绿（2918 passed），回归只能靠 root 手动重放才发现。

**范围更新（2026-10-04 root 实测复核，环 0/环 1 合入后）**：8 个中的 **2 个已被环 0/环 1 的新测试引用**，故本片范围收窄为**其余 6 个**；且其中一项的基线**已翻转**，若照原表钉 golden 会把已交付的改进钉成"回归"——这是本片开工前必须先修正的陷阱：

| fixture | 目录布局（root 实测） | run.sh | README | 当前 CI 引用 | 记录的行为基线 |
| --- | --- | --- | --- | --- | --- |
| `anonymous-member-base` | 平铺（5 个 `.class`，含 `$1`/`$2`/`$Outer`） | 有（手动） | 有 | **0** | 需实测补记 |
| `anonymous-top-level` | 平铺（4 个 `.class`） | 有（手动） | 有 | **0** | 需实测补记 |
| `lambda-body-inline` | 平铺（2 个 `.class`） | 有（手动） | 有 | **0** | 需实测补记 |
| `enum-arity` | **嵌套 `v8/probe/`**（4 个 `.class`） | 无 | **无**（只有 `SHA256SUMS`） | **0** | 需实测补记 |
| `package-info-basic` | **嵌套 `v8/p/`**（2 个 `.class`：`package-info`、`Check`） | 无 | 有 | **0** | 需实测补记 |
| `short-circuit-left-false` | 平铺（1 个 `.class`） | 无 | 有 | **0** | 需实测补记 |

**已被环 0/环 1 覆盖、本片不得重复也不得回退的 2 个**：

- `anonymous-super-args`（**2 个测试文件引用**）：其完整源集在环 1（`recover-anonymous-local-decl-site`，merge `25f2589e`）后已由 `javac` **exit 1 翻转为 exit 0**，root 实测事件日志与原 class 逐行一致。**原表记载的"当前完整源集编译退出 1"已过期**——若据此钉 golden，会把环 1 的交付钉成回归。
- `anonymous-super-dispatch`（**1 个测试文件引用**，来自 `recover-ctor-reorder-dispatch-guard`）：其基线 `observed=captured-value` / `visibleDuringSuper=true` 已由该片的三向负例守卫。**注意它是环 3（`recover-anonymous-parameterized-root`，在飞）的锚**：环 3 落地后其呈现会从物理文本变为投影的 `new Base() { … }`，故本片**不得**把当前物理文本呈现钉为 golden（那会使环 3 无法通过）。本片对它只补**行为腿**（运行输出 golden），呈现腿留给环 3 的验收。

**与既有归属的关系（查重）**：`present-proved-java-structure` 的 7.2 有执行对照机制（`cargo test --test p3_execution_comparison --locked -- --ignored`），但其范围明写"只覆盖**本 change** 使文本可单独编译的 fixture"——上表中 `anonymous-super-args` 等当前**不可编译**，落在 7.2 之外。`audit-corpus-gates`（6 未勾）处理的是 fixture **登记/census 索引**完整性（`p5_corpus_fingerprint` 的文件哈希与 reader 人口计数），**不守行为**——该测试自述"asserts nothing about whether an acceptance row passes"。故本片不重复二者：它把已冻结的**行为事实**变成 CI 断言。

## What Changes

- 为上表**在范围内的 6 个** fixture 补 CI 守卫测试，按既有模式（`include_bytes!` + `Engine::open` + `ClassSourceRequest` + 文本/行为断言，参照 `tests/p3_anonymous_class_facts.rs`）：
  - **行为腿**（有可运行基线者）：原 class 的运行输出钉为 golden；能重编者加重编运行对照，不能重编者钉"当前不可编译"这一事实（**不得**把不可编译当成通过）。
  - **呈现腿**（6 个在范围内 fixture 全部；`anonymous-super-dispatch` 因是环 3 锚、呈现即将翻转，不在本片范围，其呈现腿留给环 3）：关键文本锚钉为断言，使任何改变该形的呈现改动都在 CI 可见。**对 `anonymous-top-level`/`anonymous-member-base` 须特别注意**：它们是环 2（返回父类超类型）/`this$0` 三者并存形的锚，当前**物理文本呈现**，将来相应切片落地后会翻转为投影形——故其呈现腿 golden 须写成"当前物理呈现"并在该形被后续切片解锁时**主动更新**（不得让后续切片误判为回归）。
- 对**无已记录行为 golden**的 fixture 先实测补记（`java -Xverify:all` 运行输出与 SHA）再写断言——不得凭猜测钉基线。root 实测：这 6 个 fixture 的 README **没有一个记录了实际的运行 golden 输出**——`lambda-body-inline`、`anonymous-top-level`、`anonymous-member-base` 的 README 只说明 `run.sh` 会"以 `-Xverify:all` 运行"（描述脚本行为，非钉基线），`enum-arity` 无 README（只有 `SHA256SUMS`），`package-info-basic`/`short-circuit-left-false` 有 README 但既无 `run.sh` 也无运行输出。故 6 个全部须实测补记行为 golden。
- **布局陷阱（root 实测）**：`enum-arity` 的 class 在**嵌套 `v8/probe/`**、`package-info-basic` 在**嵌套 `v8/p/`**，其余四个为平铺。`include_bytes!` 路径须逐个核对，不得假设平铺布局。
- 建立**登记纪律**（写入 handoff.md）：新增冻结行为 fixture 时 MUST 同时加一个引用它的 CI 测试；`run.sh` 是复现工具而非守卫。

## Capabilities

### New Capabilities

无（本片只补验证覆盖，不改恢复能力）。

### Modified Capabilities

- `java8-recovery`：其验收 fixture 的行为基线由 CI 断言守卫，而非仅手动脚本。

## Impact

`tests/`（新增或扩展守卫测试文件）、`tests/fixtures/proved-java-structure/` 下**全部 6 个**在范围内的 fixture 目录（补行为 golden 记录：`anonymous-member-base`、`anonymous-top-level`、`lambda-body-inline`、`enum-arity`、`package-info-basic`、`short-circuit-left-false`）、`handoff.md`（登记纪律）；`crates/` 与 `src/` **无生产代码改动**。

**顺序约束已满足**：与 `recover-ctor-reorder-dispatch-guard` 的重叠面（该片 (a) 项断言 `anonymous-super-dispatch` 不重排）已随其合入 `fc868aba` 落地，故本片可在其后实施；本片把守卫推广到其余 6 个 fixture 并补行为腿，不重复其判据测试。

**与在飞/后续切片的协调（root 2026-10-04 实测确立，实施前必读）**：
- `anonymous-super-dispatch` 是环 3（`recover-anonymous-parameterized-root`，**在飞**）的锚——其呈现将从物理文本翻转为投影形。本片**只对它补行为腿**（运行输出 golden 与呈现无关），**不钉呈现腿**，否则会阻塞环 3。
- `anonymous-top-level`（环 2 的锚）与 `anonymous-member-base`（`this$0` 三者并存形的锚）当前为物理文本呈现，将来相应切片落地后会翻转。其呈现腿 golden 须明确标注"当前物理呈现，后续切片解锁时须主动更新"，避免后续切片被误判为回归。
- `anonymous-super-args` 已被环 0/环 1 的测试引用（2 个测试文件），**本片不得重复守卫**，也不得把它原表的"编译退出 1"钉为 golden（该基线已由环 1 翻转为 exit 0）。
