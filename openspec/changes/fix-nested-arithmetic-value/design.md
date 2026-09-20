## Context

动机与复现见 [proposal](proposal.md)。已知事实（不构成根因结论）：每块字节码的算术依赖是 `t0 = d * i`、`t1 = 2 - t0`、`t2 = i * t1`；恢复文本 `local1 = local1 * 2 - arg0 * local1` 既缺外层 `imul`（BCI 8），又把 `isub` 的常量左操作数写成了乘法。`render_value` 已经按消费位置验证值（P3-R8 的求值点规则），本缺陷是同一族里的另一条：**操作数自己的值**没有被使用。同一份报告同时是 `quality=Structured`、`content=contains_statements`、`execution=complete`、无诊断，所以这三个平面在这条反例上全部为真，而产物仍然算错。

## Goals / Non-Goals

**Goals:**

- 算术的操作数是另一个算术的结果时，呈现使用该操作数自己的值；无法证明就拒绝该区域。
- 以受控 fixture 与一组输入的执行对照证明「呈现文本与原字节码同值」，并把该对照永久化。
- 让缺陷的根因由 SSA 值与实际求值点的证据确定，而不是按文本形态猜测。

**Non-Goals:**

- 不预设根因或修正位置；不得只在 emitter/文本层按形状打补丁。
- 不重做 SSA、不新增公共类型、不建通用 value/effect/物化框架；只允许在既有私有 AST 内做最小必要表示。
- 不把「拒绝所有算术」当修正：已有的正确算术样本必须保持 Structured。
- 不新增/升级依赖、crate、报告平面或停止分支；不重开 R8/R9 与已归档的停止传播修正；不做性能工作；不修 body 解码重新解析类的债务。

## Decisions

### 1. 根因由证据给出，但不变式先固定

实施 MUST 先在本次运行的 SSA 值表上核对每个算术操作数的定义、消费指令与实际求值点，记录缺陷发生在哪个边界（例如操作数枚举、求值点传递、或渲染映射把另一个定义/另一个操作数当成输入），并在 verification 里以具体 `ValueId`/BCI 与指令序列陈述。禁止只按 `local1 * 2 - arg0 * local1` 的文本形态改输出：那是把反例拟合掉，不是修根因。

固定的不变式：

- 每个算术操作数都以其**自己的 SSA 值**呈现，并在它实际被求值的位置验证；
- 一个算术的结果被另一个算术消费时，呈现保留这一嵌套；缺少证明时该区域走既有 refusal 路径（引用 bytecode、保留 origin、`quality=Fallback`、`representation=Mixed`），而不是省略操作数继续；
- 已证明正确的算术呈现（`(x + 1) + (x + 2)`、`p3-local-rewrite` 语料与既有 controls）逐字段不变。

### 2. 结构性平面不是值证据

`quality=Structured` 只说明区域结构成立，`content=contains_statements` 只说明产物含发射语句，`execution=complete` 只说明本次工作跑完；三者都不构成「产物与字节码同值」的声明。本 change 的任何验收 MUST NOT 用这些平面代替执行对照或操作数证明；这一边界同时由 [`clarify-structural-output-planes`](../clarify-structural-output-planes/proposal.md) 写进文档契约。

### 3. fixture 提交真实字节，对照执行原 class

受控 fixture 放在 `tests/fixtures/p3-nested-arithmetic/`：

- `ModLike.java`：受控源码（上面的四轮写法），保持与反例完全一致的形状；
- `v8/ModLike.class`：`javac --release 8 -g:none` 的真实输出；
- `README.md`：编译器版本与确切命令、字节数、SHA-256、逐成员字节码，沿用 `p3-nested-eval`/`p3-refused-cast` 的记录格式；
- `Baseline.java`：原 class 的驱动，打印该输入集合的原值（对照的基准侧）。

选择提交字节而不是内存生成，因为本条的判据是「原 class 与呈现文本在同一输入上执行结果一致」：原 class 必须是被真实执行的对象，基准答案必须来自它自己的运行，而不是从恢复侧或文本形态反推；语料 fingerprint 与 reader census 也需要稳定的输入字节。对照沿用 `tests/p3_execution_comparison.rs` 的既有流程：从本次事实派生方法声明、`javac --release 8` 编译生成文本、两侧 trace 逐行比较、`#[ignore]` 常备并由 CI 显式运行（`cargo test --test p3_execution_comparison --locked -- --ignored`）。输入集合至少含 `d = -1`（已测量的发散点）与一个正向输入；只用一个输入不算覆盖。

### 4. 拒绝路径的验收

如果最小修正后该形状仍无法证明（即层选择拒绝），验收核对：被引用的 BCI（缺外层 `imul` 的位置与相关操作数）、物理方法映射、`quality=Fallback`/`representation=Mixed`，以及**没有任何**声明为结构化而算错的文本。不得把「没有生成 Java」当作通过；拒绝本身就是产物的一部分（P3-R9 的既有规矩）。

### 5. 复用与依赖

不需要新库：`Expr`/`ExprKind::Binary`、`render_value` 的求值点机制与既有 refusal/origin 词汇足够表达修正。`javac` 只是测试侧的外部 oracle（固定版本、非构建依赖、缺席如实报告），生产引擎保持离线、不执行目标代码。语料新增会改变 `tests/fixtures/corpus-fingerprint.json` 与 `repository_class_fixtures_validate_without_false_target_rejections` 的计数，两者都按既有流程显式再生成/更新，并记录实测值。

## Risks / Trade-offs

- **过度保守把合法算术降级** → 正向对照（无跨写入的嵌套算术、p3-local-rewrite 语料、既有 controls）必须在修正后仍为 Structured；`p3_eval_context` 全部用例保持通过。
- **只修一种文本形态** → 修正范围由 SSA 证据决定；对照集合覆盖「操作数本身是结果」的多种排列（含常量左操作数、多轮迭代），不把单个样本推广成全部。
- **对照两侧共享同一错误** → 基准侧运行原 class 自己的 driver，不复算呈现侧；输入集合与基准输出记录在 verification。
- **语料变更被静默接受** → fingerprint 再生成器与 census 断言都要求记录实测计数，改语料必须显式可见。
- **修正引入新的临时量或语义** → 只在既有私有 AST 内做最小表示；若证明必须物化，须在 verification 说明其计费与作用域影响，且不改变公开契约。

## Migration Plan

1. 在固定行为基线上确认反例：恢复文本算错（不是拒绝），记录 `d = -1` 的原值与恢复值。
2. 按 SSA 证据实施最小修正，保持既有正确算术逐字段不变。
3. 提交 fixture 与来源 README，执行 fingerprint 再生成与 census 更新，加入执行对照、拒绝路径验收与变异。
4. 跑固定提交门禁并写 verification；同步 delta 与状态引用后归档。

与 [`bound-recovery-recursion`](../bound-recovery-recursion/proposal.md) 串行实施（同一文件 `crates/jarde-java/src/build.rs`）。回退按本 change 的独立提交进行；回退后必须恢复「该产物算错值」的公开事实，不能保留完成声明。

## Open Questions

无。根因本身待实施者以证据确定；这属于本 change 的诊断任务，不是需要上游裁决的设计歧义。

## 已定位的根因（本次复核的独立复现，2026-09-20）

规划时要求实施者先定位，本文件补记复核者**已经定位**的结果，实施者应据此直接进入修复与验收，不必重复定位：

**根因在发射器，不在值分析。** `crates/jarde-java/src/emit.rs` 的二元表达式打印只做拼接：

```rust
ExprKind::Binary { op, left, right } => {
    emitter.expr(left)?;
    emitter.put(&format!(" {} ", op.spell()), at)?;
    emitter.expr(right)
}
```

没有任何优先级/结合性判断，也没有括号。表达式树本身是正确的（`local1 * (2 - arg0 * local1)`），但文本 `local1 * 2 - arg0 * local1` 按 Java 语法解析回的是 `(local1 * 2) - (arg0 * local1)`——**产物是另一个程序**，而 quality/content/execution 三个平面全报正常。

**这不是单个方法的孤例。** 用 `javac --release 8 -g:none` 构造四个形状，全部 `structured`、全部丢失分组：

| 源码 | 正确表达式 | 实测呈现 | 文本实际解析为 |
| --- | --- | --- | --- |
| `a * (2 - b * a)` | `a*(2-b*a)` | `arg0 * 2 - arg1 * arg0` | `(a*2)-(b*a)` |
| `a - (b - c)` | `a-(b-c)` | `arg0 - arg1 - arg2` | `(a-b)-c` |
| `a / (b * c)` | `a/(b*c)` | `arg0 / arg1 * arg2` | `(a/b)*c` |
| `a - (b + 1) * 2` | `a-((b+1)*2)` | `arg0 - arg1 + 1 * 2` | `((a-b)+1)*2` |

原反例（bcprov `Mod.inverse32(I)I`，以及用 javac 写 `int i = d; i = i * (2 - d * i);` ×4 得到的同形受控样本）只是其中一例：`d = -1` 时原 class 返回 `-1`、文本返回 `-81`。既有语料没有暴露它，因为此前的嵌套样本（`nestedPlain`、`leftRead`/`rightRead`）都是同优先级左结合，不加括号也恰好正确。

**因此验收必须覆盖优先级与结合性两个方向**（乘法内的减法、减法内的减法、除法内的乘法、减法内的加乘混合），而不是只钉住原反例那一个形状；同时保留第四平面（`quality/content/execution`）不构成证据的说明。修复的正确边界是「打印时保持树的分组」，不是「只让 `Mod.inverse32` 正确」。

## 复核方回传的两条测量侧提示（2026-09-20）

实施本 change 时不要吸收、但要在归档验证里带到记录的两点（来自发现该反例的测量会话）：

1. **它的对照臂有一处自己的 bug**：探针按参数名的首次文本出现顺序编号，在 `comparePriorities(II)I` 上把两个 `int` slot 调换，因此那一行是 harness 缺陷而非引擎问题（排除后它的可比行是 785/784，不是 786/784）。修复后该行仍会不匹配，除非它的对照臂被修正或排除——不要把该行当作本 change 的验收依据。
2. **103 条 `does_not_compile` 只是排除类别、不是判定**：其中可能有与本缺陷同源（打印分组/优先级）的条目，修复后应由测量方重跑该桶确认，而不是永久留作排除项。本 change 不承担该重跑（它是测量工作），但应在验证记录里写明这一待办，避免修复被误读为已解释那些条目。
