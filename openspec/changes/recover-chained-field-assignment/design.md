# Design：链式字段赋值的多赋值拆分呈现

## Context（root 已实测）

- 字节码：`iconst_5; dup; putfield c; putfield b; putfield a`——dup 在首个 putfield 前入栈、跨全部 putfield 存活。
- 局部链（dup 跨 istore）已恢复（拆双赋值）——本片是 putfield 形。
- jadx：三独立赋值，序=字节码序（源从右到左）。

## 决策 1：呈现 = 按字节码序的独立赋值组

dup 值的生产表达式求值**一次**，呈现为该表达式文本重复出现在 n 个赋值中（常量形下无副作用风险；**表达式含方法调用等副作用时**——求值一次语义必须保持：呈现为 `T tmp = <expr>; f1 = tmp; f2 = tmp;` 的 temp 形（jadx 对副作用形同法）。判据=dup 的生产点单次求值已知 + 全部消费是 putfield。）

## 决策 2：判据（三形状族的第三员）

dup 值：全部消费是 putfield → 本片；是 iinc 前旧值 → #9 片；是数组 dance → #8 片。同一"copy 无局部赋值"诊断下的形状分派，task 1.1 确认三者是否同一落点（若是，实现应做成同一机制的形状分支，报告说明）。

## 决策 3：零回退与负例

- 局部链（既有拆分）与单字段赋值逐字节不变；
- 负例：`a = (b = 5) + 1`（表达式内消费 dup）仍拒；dup 消费混合（putfield + 其它）仍拒；
- corpus 双腿扫描：预期 diff 为空（如实记录）。

## 验证标准（可证伪）

1. 主锚：`CH.chain` 恢复（`CH.c = 5; CH.b = 5; CH.a = 5;` 序）、整类 `javac --release 8` exit 0、`main` 输出一致（a/b/c=5）；副作用表达式形（合成探针）temp 呈现行为一致；
2. 零回退/负例如上；corpus 空 diff；
3. 门禁全量（基线以合并态为准）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + fingerprint 再生。

## Open Questions

1. "copy … has no proved local assignment" 发出处与 #8/#9 片的关系（同落点 vs 邻近——task 1.1 插桩）；
2. 副作用表达式 temp 形的呈现变量命名（按仓库既有 savedN/temp 约定，实现者定并测试钉死）。
