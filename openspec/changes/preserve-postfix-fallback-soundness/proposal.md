## Why

实证（[postfix-self-assign-soundness-patrol](../../evidence/java-syntax-2026-10-05/postfix-self-assign-soundness-patrol/README.md)）：`i = i++` 渲染 `local0 = local0 + 1; return local0;`——**可编译且行为不同**（原 5、渲染 6）；`i = i--` 同（5→4）；`a[i] = i++` 的数组存语句**整条丢失**（102→2）。这违反项目第一不变量（绝不产出可编译但行为不同的文本）。机制根因：旧值 store 的值来源证明失败（"the value at BCI N is the value local X held at BCI M"诊断）时，引注只圈 store 与旧值 load，但 **iinc 的语句仍以已证明身份呈现**——引注机制实际依赖"fallback 后文本碰巧不完整"保证不可编译（postOther 因缺 return 而不可编译），本族文本碰巧完整且语义错。jadx 对同形行为正确（`return 5`）。

## What Changes

当一条 store 的值来源证明失败（上述诊断族）时，**与该未证明值语义绑定的整条语句**——包括为它呈现的 iinc 自增/自减赋值、以及以旧值为 RHS 的数组存——必须一并落入引注区：呈现文本（去注释后）对本方法**不得可编译出与原 class 不同的行为**（不可编译、或行为一致、或整方法响亮拒绝，三者任一）。触发族泛化（2026-10-05，[field-compound 健全性巡查](../../evidence/java-syntax-2026-10-05/field-compound-soundness-patrol/README.md)）：copy/receiver-dup 证明失败（"the copy at BCI N has no proved local assignment"）同样留下可编译错文本——`this.flags |= 1 << bit`（void/值消费）整语句被引注吞掉、剩余 `return;`/`return this.flags;` 可编译且位操作静默丢失（BF 实证 false/false/false/true/0 vs true/false/true/false/3）；jadx 正确直排。守卫覆盖**四诊断族**（第 12 锚，[multiconsumer 巡查](../../evidence/java-syntax-2026-10-05/multiconsumer-local-soundness-patrol/README.md)）：**多消费者族**（"saved producer at BCI N has M consumers"）——单表达式内局部 ≥3 次消费整链被吞、剩 `local = new …; return;` 可编译壳（无输出 vs 3/10/5/true）；2 次消费对照完整恢复。（第 7 锚，[array-store 巡查](../../evidence/java-syntax-2026-10-05/array-store-soundness-patrol/README.md)）：**依赖链族**（"dependency chain … not bounded"）——`elems[size++] = t`（ArrayList.add 形状，集合代码最常见语句）渲染体仅 `return;` 可编译且存储静默丢失（null/null vs x/y）。边界锐化（field-compound 复核）：**静态复合全类型恢复**（无 receiver dup）、**实例 `this.i++` 语句位恢复**——失败面精确为实例 receiver dup/dup2_x1 复合 RMW 与旧值/copy/依赖链三族；守卫按诊断族识别不受影响。MVP 范围 = 后缀旧值四锚（`i = i++`、`i = i--`、`a[i] = i++`、`i += i++ + 1`——第 4 锚 11→6 实证于 FS）。

- 插桩先行（实现者改动前回答，落盘 change 目录）：Q-i：引注圈的 BCI 集合在哪计算（诊断 "the value at BCI N…" 的产生点），为何 iinc 语句（`local0 = local0 + 1`）不在圈内；Q-ii：把 iinc 语句并入引注后，`j = i++`（现不可编译的 postOther 形）与其他 "local crosses" 拒绝的文本如何变化（逐例对照，不得把现有响亮拒绝变成可编译错文本）。
- 恢复优先级说明：在队 `recover-postfix-old-value-snapshot` 落地后本形将正确恢复；本片保证**恢复落地前**的健全性。两片验收互不替代。
- 拒绝边界保持：诊断文本本身不改（不发明新拒绝码）；健康形（语句位 `i++;`、前缀 `++i`）零回退。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：旧值 store 证明失败时的引注范围与不可编译性保证，保持第一不变量。

## Impact

- 代码：引注范围计算处（诊断产生点同文件的 BCI 集合选择）——实现者按 Q-i 定位。
- 测试：SA/SD 三锚（去注释后不可编译或行为一致）+ postOther 现状不回退 + 健康形零回退。
- 账本：summary.md 最严重级行关闭。
