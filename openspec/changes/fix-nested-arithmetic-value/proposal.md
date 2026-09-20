## Why

一个 `Structured` 方法体可以计算与它字节码不同的值，且没有任何平面或诊断提示。受控复现（standalone class，`javac --release 8 -g:none`）：

```java
int inverse32(int d) {                 // ModLike.inverse32(I)I
    int i = d;
    i = i * (2 - d * i);               // 这一行重复四次，每块都是 iload_1; iconst_2; iload_0; iload_1; imul; isub; imul; istore_1
    return i;
}
```

每个块的真实字节码是 `iload_1; iconst_2; iload_0; iload_1; imul; isub; imul; istore_1`，恢复文本却是四行 `local1 = local1 * 2 - arg0 * local1;`，报告为 `quality=structured`、`content=contains_statements`、`execution=complete`、无诊断。正确表达式是 `local1 * (2 - arg0 * local1)`：外层 `imul`（BCI 8）缺失，`isub` 的左操作数（常量 `2`）被写成乘法，产物实际是 `(i * 2) - (d * i)`。同一形状在 bcprov 的 `org.bouncycastle.math.raw.Mod.inverse32(I)I` 上复现（文件 `weblogic/weak_password/decrypt/lib/bcprov-jdk15on-152.jar`，字节码形状相同）。后果经测量：`d = -1` 时原 class 返回 `-1`，恢复文本返回 `-81`（四轮迭代发散）。

现有测试与绿色门禁不覆盖这条形状；`quality`/`content`/`execution` 在这些输入上同时为「结构化/含语句/完整」，说明它们是结构性平面，不能替代值正确性。

## What Changes

- 算术的操作数是另一个算术的结果时，呈现 MUST 使用该操作数自己的值与其实际求值点，例如 `local1 * (2 - arg0 * local1)`；MUST NOT 省略外层运算或把操作数改写成另一种运算。
- 一份呈现文本 MUST NOT 计算与其字节码不同的值；层不能证明某操作数的值或求值点时 MUST 拒绝该区域（保留 bytecode 与 origin），而不是发布已知算错的表达式。`quality`/`content`/`execution` 是结构性平面，不能作为这条判据的证据。
- 纳入受控 fixture 与执行对照：原 class 与呈现文本在一组输入上逐项比较（含 `d = -1`），并沿用 `tests/p3_execution_comparison.rs` 的既有基线驱动模式；恢复错误呈现的变异必须让对照变红。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：算术操作数按自己的值与求值点呈现；不能证明时拒绝而不是呈现。
- `recovery-validation`：把「呈现值与原字节码的执行对照（一组输入）」纳入恢复验收，并接受带边界证据的拒绝路径。

当前仅完成修正规划，实施任务全部待办；历史归档与既有验证记录保持原状。

## Impact

实现落在恢复层表达式构建路径（`crates/jarde-java/src/build.rs` 的 `render_value` 族与既有私有 AST）；根因由实施者对照 SSA 值与实际求值点诊断并记录证据，本 change 不预设根因。与 [bound-recovery-recursion](../bound-recovery-recursion/proposal.md) 都涉及 `crates/jarde-java/src/build.rs`，两者必须**串行**实施，不能并行。

交付包含受控 fixture（源码、class 字节、来源 README 记录命令/编译器版本/摘要/逐成员字节码、fingerprint 再生成、reader fixture census 更新）、执行对照与变异。非目标：不新增 crate、依赖或依赖升级；不重做 SSA；不引入通用值物化框架或超出既有私有 AST 的新临时量；不新增公共类型、报告平面或停止分支；不重开 R8/R9 或已归档的停止传播修正；不声称一般语义等价；不做性能工作（`optimize-demand-workloads` 保持 0/22）；不修 body 解码重新解析类的债务。历史归档与既有验证记录保持原状。
