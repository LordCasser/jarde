# 绑定接收者非空适配恢复（recover-proved-nonnull-bound-receivers）

## Why

[Optional 链巡查](../../evidence/java-syntax-2026-10-05/optional-chain-patrol/README.md)固定 critical 第 17 锚（第 5 诊断族唯一 critical 形）：`o.ifPresent(sb::append)`（接收者=刚分配的捕获局部）整条调用语句被吞，幸存文本可编译但**静默丢失副作用**（`[]` vs 原 `[S]`）。拒绝点 `lambda.rs`（`reference_shape ∧ parameter_adaptation ∧ Reach::Receiver ∧ 单捕获` → `jre_lambda_sam_types` 拒绝"adapting this bound receiver would move its null failure…"）的语义依据是**可空接收者**上方法引用（创建期 NPE）与 lambda 适配（调用期 NPE）的时机差。已闭片 `recover-typed-functional-method-references`（10/10）当时明写"证据不足继续明确拒绝"——接收者**可证非空**时该时机差消失，两文本行为恒等，拒绝过宽。

## What Changes

> **root 重设计（2026-10-06，门控实验证伪原两门逃逸的充分性后）**：见 [redesign.md](redesign.md)。原两门保留为必要条件；追加"站点识别自有丢弃空检查尾 + 经尾读回接收者 + 尾三 BCI 入站点所有集合（BCI 序跨块窗口）"；尾识别复用 `facts.rs::is_discarded_null_check` 与 `init.rs::discarded_null_check_tail` 的既有谓词与单用纪律；所有权走 `init::Sites` 既有通道；窄口径（仅两门通过的形）。

在既有拒绝分支内增加**两道门**的放行逃逸（`lambda.rs` 同一 verdict 点，无平行处理器）：

1. **非空门**：捕获接收者的 SSA 定义是 `Operation::Allocate`（SSA 单定义 ⇒ 该值非空）；
2. **捕获-重写互斥门**：捕获点之后该局部槽无进一步 store（方法引用捕获值、lambda 捕获槽，重写会使两文本读到不同对象）。

两门全过 → 按 `LambdaForm::Lambda` 适配（`x -> sb.append(x)`）；任一不过 → 拒绝**逐字保留**（可空参数/字段读接收者、捕获后重写形维持现状，含诊断文本与拒绝码）。

- 不改 `reference_shape`/`parameter_adaptation`/`Reach::Receiver` 判据本身；
- 不触碰 `jre_lambda_sam_types` 的其它拒绝面与 soundness 六族守卫表（build.rs 注册表为子串识别，放行只是减少发生集）；
- 不新建非空证明通道：SSA 分配定义即既有事实，MVP 不做 null-check 支配等更强证明（登记为后续扩验）。

## 硬不变量

1. 既有 `typed-functional-method-references` 全部锚（含绑定实例拒绝形）渲染逐字节不变；
2. 可空接收者负例（参数读/字段读）拒绝文本逐字不变；
3. 不得产出"可编译且 NPE 时机不同"的文本（第 17 锚的负例守恒）；
4. soundness 守卫六族清单零变化。

## 验收

- 锚方法恢复（该诊断 0 引注）、剥离编译 exit 0、`-Xverify:all` 输出与原 class 逐行一致（`[S]` 形）；
- 门控实验先行（task 1.1）：确认该分支确为锚产生点、`parameter_adaptation` 在锚为真；
- 负例三形（可空参数、可空字段、捕获后重写）维持拒绝；
- 全门禁 + corpus 指纹。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：可证非空且无捕获后重写的绑定接收者按 lambda 适配呈现，方法行为完整。
