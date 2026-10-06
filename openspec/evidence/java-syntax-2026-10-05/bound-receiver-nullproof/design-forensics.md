# 绑定接收者非空适配（第 5 族 critical 第 17 锚）—— root 设计取证（2026-10-06，读码）

## 现状拒绝点（读 `lambda.rs` 2026-10-06 主线）

```rust
// lambda.rs ~865-877
let reference_shape = receiver == captures.len() && !implementation.is_generated_body();
if reference_shape
    && parameter_adaptation
    && implementation.reach() == Reach::Receiver
    && captures.len() == 1
{
    return Ok(Verdict { evidence, outcome: Err(Refusal::shape(
        "jre_lambda_sam_types",
        "adapting this bound receiver would move its null failure from functional-value creation to invocation".to_string(),
    ))});
}
let form = if array_constructor.is_some() || (reference_shape && !parameter_adaptation) {
    LambdaForm::MethodReference
} else {
    LambdaForm::Lambda
};
```

拒绝的语义依据：`sb::append` 在**创建期**求值接收者（null ⇒ NPE at creation）；适配为 lambda
`x -> sb.append(x)` 后 null 失败推迟到**调用期**——两种文本对可空接收者行为不同，拒绝是对的。

## 可证安全子集（本片要开的门）

接收者**在创建点可证非空**时，两文本的 null 语义都退化为"永不 NPE"，适配行为恒等：

1. **非空证明（MVP：SSA 分配定义）**——SSA 单定义下，捕获值的定义是 `Operation::Allocate`
   （`new`）⇒ 该值非空。锚形：`StringBuilder sb = new StringBuilder(); o.ifPresent(sb::append);`
2. **捕获-重写互斥**——方法引用捕获的是**值**；lambda 捕获的是**变量槽**。若捕获点后该槽还有
   store，两文本会读到不同对象。MVP 要求：捕获点之后该局部槽**无进一步 store**（operations 扫描，
   与既有 SSA 旧值纪律同族）。

两条都满足 → 走 `LambdaForm::Lambda` 适配（`x -> sb.append(x)`），拒绝解除；任一不满足 →
拒绝**逐字保留**（可空参数/字段读接收者、捕获后重写形维持现状）。

## 派发前必做（最小门控实验，task 1.1）

- 冻结锚 fixture（Optional 巡查的 `sideEffect` 形）双腿（真 javac 8 + javac 23 `--release 8`）；
  确认 `parameter_adaptation` 在该锚为真（若为假则锚走的是 MethodReference 分支、与本片无关——
  拒绝诊断逐字比对定位产生路径，勿凭读码推断）；
- 门控实验：临时放宽该拒绝分支（环境变量或临时补丁）观察锚的行为——确认该分支**确为**锚的
  产生点（handoff「写 spec 钉落点前必须做最小门控实验」纪律；shared-latch 片教训）；
- 负例冻结：可空接收者（参数读、字段读）两形 + 捕获后重写形——放宽门控下**仍须可拒**
  （靠新增的两道门，不是拒绝分支本身）。

## 行为验收（预登记）

- 锚方法恢复（0 引注该诊断），剥离编译 exit 0，`-Xverify:all` 运行输出与原 class 逐行一致
  （原 `[S]/[]` 形）；
- NPE 时机对照负例：可空接收者形在原 class 上创建期 NPE 的行为必须仍被拒绝呈现
  （不得产出"可编译且 NPE 时机不同"的文本）；
- 既有 `typed-functional-method-references`（10/10）全部锚零回退。

## 查重与归属

- `recover-typed-functional-method-references`（已闭 10/10）proposal 明写"证据不足继续明确拒绝，
  不把空值失败推迟到调用时"——本片是其预告的"证据足够"后继，**不重开旧 change**；
- 拒绝码 `jre_lambda_sam_types` 的其它拒绝面（SAM 类型不符等）零触碰；
- `type-immediate-functional-receivers`（已完成）是**消费位**（直接调用链）域，与本片
  （SAM 适配位）正交。
