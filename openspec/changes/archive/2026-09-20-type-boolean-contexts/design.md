## Context

动机与复现见 [proposal](proposal.md)。已知事实（来自 [完成复核](../../completion-review.md) 的“新确认”一节，本 change 按下列代码位置复核过一次）：

- `crates/jarde-java/src/build.rs` 的 `Return` 分支（`Some(Operation::Return)`）只做 `stack_operands(instruction).last()` + `render_value(value, at, 0)`，**没有读成员自己的返回 descriptor**，所以 `iconst_1; ireturn` 在 `Z` 方法里照字面写成 `return 1;`。
- 同文件 `condition` 的 boolean 特判形如 `if let Test::Zero(op) = test && builder.boolean_parameter(operands[0].1)`，而 `Builder::boolean_parameter` / 自由函数 `parameter_boolean` 都只接受「值是单条 `Operation::Load { slot }` 的定义，且该槽在参数 descriptor 里是 `Z`」。返回 `Z` 的调用结果没有这条证据，于是落回 `Test::Zero` 的普通分支，写出 `flag() != 0`。
- 同 crate 已有正确的先例：`typed_arguments`（P3-R5 的参数定型路径）按 **callee descriptor** 把字面量 `0`/`1` 实参改写成 `false`/`true`，并明确写下边界——只读 descriptor、只改字面量、其它类型一律保持。它证明「descriptor 是类型事实的来源」这一读法在本层已经成立。
- `ExprKind::Boolean` 与 `emit.rs` 的 `Emitter::expr` 已经能打印 `true`/`false`；`RecoveryFacts::method()` 的 descriptor 在 `report.rs` 的构建入口处可读（`parameter_types()` 就在那里派生），`CallTarget::descriptor()` 给出被调用方的完整 descriptor（含返回类型）。因此本 change 不需要新节点、新平面或类型推断。

## Goals / Non-Goals

**Goals:**

- boolean 上下文里的值 MUST 以 boolean 呈现：`Z` 返回的 `return`、以及操作数被证明为 boolean 的条件测试。
- 证据不足时 MUST 拒绝该区域；MUST NOT 发布另一个编译器拒绝的文本。
- 以「成员自己的签名 + javac 编译 + 两侧执行」把这两个形状永久化，并保留 int 形态的正向对照。

**Non-Goals:**

- 不建通用类型系统、不做数据流类型推断；不使用返回 descriptor、参数槽类型、callee descriptor 之外的新证据。
- 不改 `value_type` 对 `int`/`long`/引用等其它类型的决定，不新增 `Type` 变体、报告平面、诊断 code 或停止分支。
- 不改 `emit.rs` 的打印规则（`ExprKind::Boolean` 已存在）。
- 不重开 R8/R9、已归档的递归界与打印修正；不声称一般语义等价或覆盖整类类型缺陷。
- 不做性能工作（`optimize-demand-workloads` 保持 0/22）；不修 body 解码重新解析类的债务。

## Decisions

### 1. 上下文清单：只做两个位置

本 change 只闭环两个已被实测评判为错的 boolean 上下文位置：

1. **`Z` 方法的 `return`**：上下文类型由成员自己的返回 descriptor 给出（`Z`）。此时值 MUST 以 boolean 呈现：字面量 `0`/`1` → `false`/`true`；已证明 boolean 的值（下述证据）原样呈现；两者都不是时拒绝该区域。
2. **条件的分支测试**（`condition` 的 `Test::Zero`）：当操作数被证明为 boolean 时写成真值测试（`if (flag())`、`if (arg0)`、必要时 `if (!arg0)`），而不是 `!= 0`/`== 0`；无法证明时保持既有整数比较（本 change 不把「无法证明」改成拒绝，见决策 3）。

### 2. 「被证明为 boolean」的候选证据（实施者按本次运行复核，不得新增推断）

- 该值是单条 `load` 且其槽在参数 descriptor 里是 `Z`（既有 `boolean_parameter`/`parameter_boolean`，P3-R5）；
- 该值由返回 descriptor 为 `Z` 的调用产生（`Operation::Invoke(CallTarget)` 的 descriptor 已在本层可读，与 `typed_arguments` 读的是同一事实）；
- 该值是 `true`/`false` 字面量（`iconst_0`/`iconst_1` 在 boolean 上下文里）；
- 该值是上面任一种的 `!` 组合（`Not`），或一个由上述证据声明过的 boolean 局部（`declare` 已经会把它声明成 `boolean`）。

这份清单是规划期候选。实施者 MUST 在当前代码上逐条复核，把**最终**清单写进 verification；`Z` 返回的 `if (x == 0) return true; return false;` 这类形状会经过 phi/区域合并，若某条证据在合并值上不成立，结论是「那个形状走拒绝路径」，不是「按直觉补一条推断」。

### 3. 拒绝只用于「无法定型」，不用于「无法证明 boolean」

两类「无法证明」必须分开，否则会用普遍拒绝换反例通过：

| 情形 | 处置 |
| --- | --- |
| 上下文要求 boolean（`Z` 返回），值没有 boolean 证据 | **拒绝该区域**：`representation=Mixed`、`quality=Fallback`、诊断点名 BCI 与物理方法；MUST NOT 写 `1`/`0` |
| 条件测试的操作数没有 boolean 证据（例如真正的 `int` 比较） | **保持整数比较**（`arg0 == 0`），这不是缺陷，不得拒绝也不得改写成真值测试 |
| `Z` 返回的值是已证明 boolean 的调用结果/局部/`!` | 原样呈现，不得再包一层转换 |

### 4. 最小布线

返回 descriptor 目前没有进入 `build.rs::Builder`（`Inputs` 只有 `parameter_types`）。实施者 MUST 在既有构建入口（`report.rs` 的 `build::build` 调用处，`request.facts.method()` 已可读）把返回类型交给构建器，像 `parameter_types` 那样作为**本次运行的事实**传入；MUST NOT 让实现层自己去重新解析 descriptor 或以文本形式猜测。`emit.rs` 与 `ExprKind` 不改。

### 5. 受控 fixture 与「javac 接受」的验收

fixture 放在 `tests/fixtures/p3-boolean-contexts/`：

- `BooleanContexts.java`：受控源码，含两个缺陷形状与三条正向对照；
- `v8/BooleanContexts.class`：`javac --release 8 -g:none -d v8 BooleanContexts.java` 的真实输出；
- `README.md`：编译器版本与确切命令、字节数、SHA-256、逐成员字节码（沿用 `p3-nested-arithmetic` 格式），并记录修正前的 javac 拒绝信息；
- `Baseline.java`：原 class 的驱动，打印输入集合的原值。

规划期候选成员（成员名、正文与精确文本由实施者按实测记录，下表用于验收对照，不得据此反推）：

| 成员 | 形状 | 修正前 | 修正后 |
| --- | --- | --- | --- |
| `isZero(I)Z` | `if (x == 0) return true; return false;` | `return 1;` / `return 0;`（javac 拒绝：`int cannot be converted to boolean`） | `return true;` / `return false;` |
| `flag()Z` | `return true;` | `return 1;` | `return true;` |
| `parity(I)I` | `if (flag()) return 1; return 0;` | `if (flag() != 0)`（javac 拒绝：`incomparable types: boolean and int`） | `if (flag())` |
| `count(Z)I` | `if (b) return 1; return 0;` | `if (arg0)`（既有 P3-R5 路径） | 逐字不变 |
| `nonzero(I)I` | `if (x != 0) return 1; return 0;` | `if (arg0 != 0)` | 逐字不变（int 对照） |
| `answer()I` | `return 1;` | `return 1;` | 逐字不变（int 返回对照） |

验收沿用 `tests/p3_execution_comparison.rs` 的既有流程：从本次运行的事实派生成员声明（descriptor、access flags、参数槽类型）、`javac --release 8` 编译呈现文本、原 class 与生成体两侧打印同一 trace 并逐行比较。该文件已经支持 `boolean` 参数与返回（`java_type`/`sample_values`/`default_value` 都有 `boolean` 分支），因此本 change 的 javac 接受与执行对照不需要新的对照框架。稳定回归（精确文本与拒绝边界）放在新的 `tests/p3_boolean_contexts.rs`。

### 6. 复用与依赖

不需要新库：两个修复点都在 `build.rs` 的既有渲染路径上，且已有 `typed_arguments` 的读法、`ExprKind::Boolean` 的打印与 `facts.rs` 的参数类型派生。`javac` 仍是测试侧外部 oracle（固定版本、缺席如实报告）。新增 fixture 会改变 `tests/fixtures/corpus-fingerprint.json` 与 reader census 计数，两者都按既有流程显式再生成/更新并记录实测值。

## Risks / Trade-offs

- **过度泛化把 int 改成 boolean** → `nonzero(I)I` 与 `answer()I` 两条对照 MUST 逐字不变，并进执行对照。
- **把 `== 0` 一律写成真值测试** → int 比较对照必须保持整数比较；只有被证明 boolean 的操作数才走真值形态。
- **phi/合并值缺乏证据时硬写** → 合并值上证据不成立就拒绝（决策 3 第一行），不得按形状猜。
- **只修一处** → 两个形状各有 fixture 成员与对照；只修 `Return` 或只修 `condition` 都会让另一个形状的检查变红。
- **对照两侧共享同一错误** → 基准侧运行原 class 自己的 driver，不复算呈现侧。
- **语料变更被静默接受** → fingerprint 再生成器与 reader census 都要求记录实测计数。

## Migration Plan

1. 在固定行为基线上先记录两个反例：恢复正文、报告平面、把正文套进正确签名后 javac 的拒绝信息（修正前完成，作为前置证据）。
2. 复核「被证明为 boolean」的候选证据清单，写出最终清单与每个形状的处置（呈现或拒绝）。
3. 接入返回 descriptor 的事实并实施两个位置的最小修正，保持 int 形态不变。
4. 提交 fixture 与来源 README，执行 fingerprint 再生成与 census 更新，加入精确文本回归、javac 编译/执行对照、拒绝边界与变异。
5. 跑固定提交门禁并写 verification；同步 delta 与状态引用后归档。

与 `group-call-receivers`、`spell-array-types` 串行实施（同一 crate，本 change 第 2 个）。回退按本 change 的独立提交进行；回退后必须恢复「产物无法在成员自己的签名下编译」的公开事实。

## Open Questions

无。`Z` 返回形状里哪些合并值能带 boolean 证据，由实施者按当前 SSA/region 事实查证并记录（该形状若不能证明即走拒绝路径），这属于本 change 的诊断任务，不是需要上游裁决的设计歧义。
