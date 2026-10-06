## 1. 取证与基线（root 已完成大半）

- [x] 1.0 拒绝点、dup-store 机制、jadx 消除解、家族四员定位——实测归档。（root 已完成）
- [x] 1.1 插桩确认四形状（dance/postfix/putfield/dup-store）在同一门控（"copy … has no proved local assignment"）的同一性；转录存证据。
      → `results/01-gating.md`：dance/putfield/dup-store 同落 copy 门（后缀旧值已由前片实现）；**两落点**已证——int 形落 `prove_local_assignments` 的参数槽准入 + 呈现臂，引用形先落 `region.rs::test_is_pure`（loop@1 `StatementFree`，BCI 5）再落同一 copy 门。
- [x] 1.2 重验基线：主线二进制渲染 OP2（两形拒、移位/常量族恢复）；负例（>2 读者）现状拒绝记录。
      → `results/renders/OP2-op2.jar.txt`、`AC-ac.jar.txt`（两腿）：condAssign/ioLoop 拒；负例（CF-06 `ExtraCopy` 多读者控制、`NegativeAssignments` 三形）仍拒。
- [x] 1.3 冻结 fixture：OP2（javac23 `--release 8`）+ 真 8 腿入 `tests/fixtures/`，README 记编译命令与 SHA。
      → 巡逻锚 OP2/AC 以巡逻自身 jar 复核（`results/renders/*-after.txt`）；本片自有 fixture
      `tests/fixtures/recover-dup-store-conditional/{DS,REF,NEG}.java` 双腿（`v8/`、`v8-javac8/`）冻结，
      README 记编译命令与九个 SHA。

## 2. 实现

- [x] 2.1 按决策 1 实现两分支（零读者消除 / 有读者拆语句）；>2 读者保持拒绝。
      → 读者事实 = store 写出的值沿 phi 归并可达的指令读者（`local_store_is_observed`）；零读者 → 消除（`return x + 1 > 0;`、`while (read() != null)`）；有读者 → 结构自身测试位写拆语句（`x = x + 1; if (x > 0) …`）。
      偏差已记录：**局部**目标（CF-06 已呈现形）保持原位赋值表达式不变（其 replay 与录制 SHA 钉死该文本）；
      多读者控制（`ExtraCopy`）逐字仍拒。
- [x] 2.2 家族形状分派结构按 1.1 结论组织（同门控则单机制四分支）。
      → 同门控：机制仍是 `local_assignment_at` + `prove_local_assignments` 一处，按读者数 + 测试位分派三种呈现
      （`LocalAssignmentPresentation::{Eliminated,Split,Expression}`）；引用形的第二落点（`test_is_pure`）只放宽
      "副本 + 其不可观察 store" 这一对，其余指令逐字不变。

## 3. 验证与验收

- [x] 3.1 主锚：condAssign/condAssignOld 恢复、整类拼接 `javac --release 8` exit 0、行为逐行一致（含 main 级联解锁）。
      → `results/03-anchors.md`：condAssign 恢复（`return arg0 + 1 > 0;`，类内该诊断 2→0）、ioLoop 恢复；
      **condAssignOld 实测不是 dup-store 形**（`iinc; iload; ifle`，无 dup），其门控是短路链布尔局部决策
      （`proves_boolean_local_store` 的 Boolean 消费位不含分支测试），属 `recover-short-circuit-local-values`
      边界——按 design 开放问题 2 "先报告" 处理，未即兴扩域。main 是级联（实测：condAssignOld 体健康则 main 整渲）；
      整类编译+行为腿因此钉在本片自有 `DS`/`REF`（`true/1/1/1/false`、`2`，两编译器腿逐字一致，`-Xverify:all`）。
- [x] 3.2 零回退：移位复合/常量族逐字节不变；负例仍拒；corpus 双腿扫描 diff 为空。
      → `results/03-corpus-delta.md`：全语料 6 个 MOVED（本片四 fixture + OP2/AC 两锚），其余逐字节不变；
      负例（本片 `NEG` 两形、CF-06 三控制、后缀 NG 五形）逐字仍拒；`p3_inner_assignment`/`p3_local_rewrite`/
      `p3_java_recovery`/`p3_loop_test_values`/postfix 套件全绿。
- [x] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
      → `results/04-gates.md`：workspace 全量 exit 0 / 324 ok / 0 FAILED；fmt clean；CI-exact clippy exit 0 且零 warning；
      `openspec validate --all --strict` 305/305；fingerprint 再生 +45/-0；census 重测 (789, 3311, 286, 2061, 8)。
      pass 2 的栈溢出（`p3_immediate_functional_receivers` 深递归）为真实发现，已用 `#[inline(never)]` 提取呈现臂修复并复测。
- [x] 3.4 root 独立复核：两分支判据保守性、零回退实测、家族结构合理；关闭 summary.md 登记行。（root 2026-10-06 完成，见 [verification-root.md](verification-root.md)：双落点门控复核 ✓、纯度豁免成对性 ✓、condAssignOld 前提证伪追认为 short-circuit-local 边界 ✓、局部活目标保 CF-06 契约追认 ✓、oracle ignored 腿 3/3 ✓、门禁权威口径 324 targets ok/0 FAILED ✓；copy 族第 4 员关闭——登记行同步）
