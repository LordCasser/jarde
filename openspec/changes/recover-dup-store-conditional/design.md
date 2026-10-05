# Design：dup-store 舞蹈的双分支呈现

## Context（root 已实测）

- 字节码：`iload_0; iconst_1; iadd; dup; istore_0; ifle 12`——dup 一份给 istore（赋值）、一份给 ifle（条件）。
- jadx：`return i + 1 > 0;`（参数槽 store 不可观察→消除）。
- 家族四员同诊断文本；#8/#9/#11 已立项（dance/postfix/putfield）。

## 决策 1：两分支判据 = store 目标的后续读者数

- **零后续读者**（store 后该 slot/局部无任何 load）：消除 store，`<expr>` 直接进消费位（条件/返回值）——与 jadx 同构；
- **≥1 后续读者**：拆 `x = <expr>;` 语句在前、消费位读 `x`——语义保持（求值一次、赋值先行）。

判据是数据流事实（读者计数），与读者不变量族（丢弃分配片的 ==0 分支）同族语义。

## 决策 2：家族落点核查义务

task 1.1 须确认四形状（dance/postfix/putfield/dup-store）在 "copy … has no proved local assignment" 发出处的**同一性**：若同一门控，实现做成**形状分派**（一个机制四个分支）并在报告说明；若不同落点，各自实现但共享判据词汇。

## 决策 3：零回退与负例

- 家族前三员的既有测试（若已实现）逐字不变；移位复合/常量族不动；
- 负例：dup 值 >2 读者（store+条件+再读栈残留——手工构造）保持拒绝；
- corpus 双腿扫描：预期 diff 为空（如实记录）。

## 验证标准（可证伪）

1. 主锚：`OP2.condAssign` 恢复（`return arg0 + 1 > 0;` 或同构）、`condAssignOld` 恢复、整类拼接 `javac --release 8` exit 0、行为逐行一致（含 main 级联解锁）；
2. 零回退/负例如上；corpus 空 diff；
3. 门禁全量（基线以合并态为准）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + fingerprint 再生。

## Open Questions

1. 四形状同门控确认（task 1.1——若同，实现结构以形状分派组织）；
2. `condAssignOld` 的短路链复合形是否随主分支自然通过（实测记录；若另有阻塞先报告）。
