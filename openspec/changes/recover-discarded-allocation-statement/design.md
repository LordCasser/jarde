# Design：丢弃分配语句的零读者呈现

## Context（root 已实测）

- 拒绝点：分配证明的实例读者链——丢弃形零读者，"an allocation, a copy or a cast is present that no shape verified" + "copy … has no proved local assignment"。
- 判别（DN 四形）：赋值/链式/实参（读者 ≥1）全恢复；零读者拒。**唯一变量是读者数**。
- CD.main 的 `new C();` 同因（三级委派构造的副作用形）——本片落地后级联解锁。
- jadx 呈现 `new DN.N();`（有解）。

## 决策 1：零读者 = 语句形（与 >1 拒绝并列的第三种边界）

读者计数的三个分支：`>1` 保持拒绝（多拼写位不变量）；`==1` 既有路径（表达式位）；`==0` **本片新增**——呈现为独立语句 `new N(args…);`。三分支判据在同一处收敛，不另建平行通道。

## 决策 2：构造器证明复用既有链，零新增类型/读者机器

丢弃形仍走既有构造器实参证明（`verify_member`/实参窗口）；唯一变化是读者门对 `readers.is_empty()` 的分支从"拒绝"改为"语句化呈现"。**`single_use_at` 本体零改动**（它是成员构造路径的 use-at 判据，非读者门本体）。

## 决策 3：验收与零回退

- 主锚：`DN.discarded` 恢复（`new N();` 语句 + "after" 保序）、整类 `javac --release 8` exit 0、`main` 输出一致；`CD.main` 的 `new C();` 级联解锁（quotes 3→0 预期）；
- 零回退：DN 三消费形逐字节不变；既有分配/构造测试全绿；corpus 双腿扫描差异类仅为含丢弃分配的类（语料可能 0，如实记录）；
- 负例：多读者形（既有测试）仍拒。

## 验证标准（可证伪）

1. 主锚双腿行为一致（DN/CD 各自 main 输出与原 class 逐行相同）；
2. 零回退边界如上；负例保持；
3. 门禁全量（基线以合并态为准）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + fingerprint 再生。

## Open Questions

1. 读者门零分支的精确落点（读者计数处 vs 分配证明入口——task 1.1 插桩定位拒绝发出的第一行并转录）；
2. 丢弃形若构造器含 null-check 舞蹈（真 javac8 对 `new Outer().new Inner()` 分配限定符形）——alloc-qualifier 片刚交付的 `discarded_null_check_tail` 是否已使该组合通过（实测 DN 真 8 腿确认，如实记录）。
