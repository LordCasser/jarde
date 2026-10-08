# HANDOFF — jarde 主线交接入口

接续工作先读本文件，再读具体 change 的 `verification-root.md`。不要根据旧分支名或任务勾选数推断仍有未合入代码。

## 当前主线（2026-10-08，本轮收尾）

本轮先核对另一位 agent 的交接：其 29 个切片已经合入，`ddfd05f8` 与 `5adb6ba3` 的 CI 均成功。本轮沿交接队列处理了构造器函数实参，并修复验收中发现的绑定方法引用创建时机问题。

- `recover-functional-constructor-arguments`：构造参数物理依赖内接纳动态函数值，仍由既有 lambda planner 验证 bootstrap/SAM/捕获/适配；构造闭合区间与唯一消费者必须具有相同异常处理器覆盖。未增加 planner、IR 或按 JDK owner 特判。
- `preserve-bound-reference-creation-timing`：直接绑定引用也消费非空接收者证明，复用实例 this、稳定完成分配、Unknown-frame 的 String 常量来源。静态参数槽 0 不能冒充 this。
- 完整双 JDK（真实 Corretto 8 与 OpenJDK 23）验证：九种构造实参，完整 `FunctionalConstructors` + `IntBox` 重编执行、原源码/JADX/Jarde 对照；`LG.pqLambda` 独立方法回放及其他四方法逐字节不变对照；实例 this 与 String 常量整类正例。完整 LG 仍有原有编译失败，不能计为完整恢复。具体结果见两个 change 的 root 验收文档。
- 合法 Java 8 `NoCheck`/`NoStand` 原 class 在创建时成功、调用时 NPE；旧 Jarde 与 JADX 1.5.6 的 `arg0::start` 会把 NPE 提前到创建时。正式产物保守拒绝，并保留完整 capture/factory/constructor/consumer BCI。**这项以 JVM 实测为准，不能继承 JADX 的假设。**
- CI 的 export_cli 并发错误已单独修正：保留操作的首次预算停止，不能被输出回调的次生取消覆盖；真实输出预算数字与文件 I/O 优先级保持。另一个 worker panic 测试只修正了错误的 discovery EOF 调度假设，未改变生产语义。见 `preserve-export-first-stop/verification-root.md` 与共同 root 门禁。
- 上述构造族恢复不代表 `PriorityQueue<T>`/`FutureTask<T>` 返回 Signature 已恢复；泛型投影仍有独立保守边界。

## Git、工作树与磁盘

所有本轮代码、spec 和取证已合入并推送 main，当前 main 工作区干净。已删除五个此前合入的旧分支和本轮实现分支，清理 11 条目标路径已消失的工作树登记。无剩余分支占用或工作树未提交实现。

Codex 固定任务保护的历史 detached 工作树仍可能出现在 `git worktree list`；它们是干净、已包含在 main 历史内的副本，归档工具明确拒绝删除，不应绕过保护。Rust 共享 target 已在验收后清理（cargo clean 移除 11,952 个文件、19.3 GiB；当时可用空间 81 GiB）；不要把这些 detached 副本视为待合并任务。

```sh
git status -sb
git rev-parse HEAD origin/main                 # 应一致
git branch -vv                                # 只剩 main
git worktree list                             # 辅助副本均 detached
gh run list --repo LordCasser/jarde --limit 5   # 核对最新 HEAD 的 CI
df -h /System/Volumes/Data
```

本地最终门禁两 seed 各 3,224 项通过，ignored P3 3 项、构造整类 1 项、既有绑定引用整类 1 项通过，strict OpenSpec 320/320。生产修复与 root 验收已推送 `add621e7`；首次远端 CI run [37747803086](https://github.com/LordCasser/jarde/actions/runs/37747803086) 在收尾时运行中，最终交接文档提交的最新 run 必须按上方命令核对，**不把本地通过写成远端已绿**。本轮收尾不继续派发新语法片。

## 下一步队列（明确区分已立项与未立项）

1. **泛型字段写安全性**：`openspec/changes/prove-generic-field-write-source-types/` 已具备 proposal/design/spec/tasks，**尚未实现（0/6）**。优先完成它：字段投影为 T 时，要证明每个写位的实际已发布源码 RHS 类型可赋给 T，不能使用未发布的方法 Signature，不能把同擦除的 T/U 当成同一变量。冻结 `Hold`/`ObjectHold`/`ObjectSetter`/`TypedSetter`/`CrossSetter` 对照在 `openspec/evidence/java-syntax-2026-10-08/generic-holder-patrol/`；四个反例整类编译失败，`TypedSetter` 是应保持的正例。
2. **class-scope 泛型构造器参数恢复**：未立项，与字段写证明是两个根因。`Hold(T)` 不带方法级类型参数，不能直接套用现有仅处理方法级类型参数、空体/转发前导的 generic constructor candidate；也不能只取消 ordinary declaration 的 `<init>` gate 就宣称恢复。先关联构造参数槽、实际字段写值、字段 Signature、构造参数 Signature 的同一 class-scope binder，再立 OpenSpec。
3. **ScopeRefusalsEscape 未变异形**：合法 multi-catch 的保守拒绝，local-scope 收尾记录，先确认新主线实际状态。
4. **LoopTestValues.storeTest**：待确认是否有真实源码形，不能只围绕变异字节码扩大机制。
5. **switchBody guard 体**：先确认当前诊断与边界，再考虑切片。
6. **Class 字面量绑定引用**：真实 javac 的 `ldc; dup; getClass; pop; indy` 仍拒绝。`bound-reference-creation-timing/positive/KnownBound` 已冻结；不是非空时机错误的剩余开口，属于复制值/冗余 check 的恢复能力，**未立项**，不要混回本轮时机修正。
7. **旧 LG 的完整恢复**：`dequeOps` 把 int[] 局部变量复用成 ArrayList，`finalize` 保守拒绝，两者与当前片前的冻结 main 输出相同。完整 Jarde LG 仍不能重编；新运行的 JADX 1.5.6 LG 也因 raw Comparator lambda 的 Object 参数调用 intValue 而重编失败。历史 README 的“全解/健康”断言过强，见本轮 root 结果。未立项，不混入构造实参片。
8. 继续按 JADX 的 71 个验收单元账本推进 enum、字符串、泛型等部分已测单元。账本在 `openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md`。71 是验收单元，不是成功率；小 fixture 的闭环不能把整个单元标为全覆盖。

## 接续纪律

以 `/Users/lordcasser/workspace/testzone/jadx` 的测试与算法为起点，先冻结原输入/源码/JADX/Jarde 差异，再决定现有机制能否覆盖。每片 OpenSpec 先行，确定性实现用 Luna subagent，root 独立验收。JADX 的解析与提取算法可以参考，语义判据必须由原 class 的 JVM 行为证明。

只允许一个 Rust 构建者；使用共享 target、`CARGO_BUILD_JOBS=1`、`CARGO_INCREMENTAL=0`。20 GiB 可用空间为停建线，验收后清理 cargo 残留。不要在每个辅助工作树生成 target。

验收须断言 class-source 自述头和非空完整类，不能把空输出计为零回退。CLI status 4 仍可能附带呈现文本；以实际 bytecode 拒绝、整类重编、`-Xverify:all` 和行为对照定性，不把保守元数据当作源码完整性证明。外部 Driver 的 classpath 不得用原 jar 掩盖漏生成的依赖类。

CI 同口径：fmt、工作区 all-targets/all-features clippy（保留 CI 明列债务白名单，其他 -D warnings）、两个固定 seed 5350648285461741569/70、strict OpenSpec，以及 ignored P3 和 functional-constructor 整类 oracle。JDK 25 instruction-boundary oracle 在 CI 的 JDK 25 环境核对，本地 8/23 不冒充 25。

架构债务单独记录。审计中发现正确性问题先收紧边界；没有证明不能用 lambda/方法引用语法代替，不能按 erasure 或 rendered 字符串猜泛型类型。subagent 声称的“root ruling”需要 root 以原证据独立重裁。

---

## 之前交接（历史记录）

# HANDOFF — jarde 接续说明（2026-09-30 立；2026-10-08 会话交接终态）

本文件是给接续 agent 的入口。先确认下面的 Git 状态，再决定是否开始新工作；不要从旧分支名推断仍有未合入实现。

## 交接终态（2026-10-08 深夜，本会话结束）

**交接原因**：用户指示本会话 agent 交接。最后一在飞切片 `recover-branching-guard-body` 已完整走完闭环（实现→root 验收→合并→推送 `5adb6ba3`），无未竟工作；其 CI run 应在交接前或交接后完成——**接手者第一步核对**：`gh run list --repo LordCasser/jarde --limit 5`（若 `5adb6ba3` 非 success，按 handoff 的 flake 判定纪律处理）。

## 本会话（10-06→10-08）累计成果

**29 个切片合入 + 2 次巡查 + 4 次根因级自纠，全部 root 独立验收 + CI 绿（或监控中）**：

### 已收官的大颗粒域
- **io 域全恢复**（countLines+readAll 双方法 0 引注：io-resource-finally 锚13 + loop-test-copy-store）；
- **guard/finally 域**：lock-guard（锚12）/ resource-guard 行集 / nested-lock（多锁族）/ branching-guard（第3边界）——锁卫、资源卫、嵌套锁、可中断锁、分支体全覆盖；
- **`preserve-local-scope-across-exception-regions` 全 change 完结**（长驻里程碑：三分类测试面+2.1-2.5 原子拒绝/行为验收）；
- **value 级四族**（copy 四员/旧值 A+B 相/依赖链/多消费者重定位）与 **widening 族**（接口四表+平台事实+java.io+java.time+Number）；
- **chained-field 域**（静态/实例/复合 RHS 全清）；
- 16 个冻结行为 fixture 全部有 CI 引用。

### 会话级教训（已固化在本文档历史段与各 verification-root）
- 溯源事件 #5 起：subagent 报告引用的 "root ruling" 一律视为未授权，root 以自证重裁；
- 假零陷阱在 root 巡查上自踩（multiAwait void-only awk 空段）——渲染计数前必须断言自述头+核对非空段；
- d3 flake 根治（usage 相等排除墙钟 `74eec0ad`）；export_cli 族诊断增强在案（`07f7d427`，下次出现断言会打印全文档命名变体）；
- 磁盘纪律强化版（20Gi 停建线、ask_subagent 要占用清单、root target 验收后即清）。

## 队列（接手者按序派发，全部 spec 就绪）

1. **family-6 形态 4**：lambda→JDK ctor（`new PriorityQueue<>((a,b)->b-a)`，诊断 "allocation belongs to no shape"；SAFE 拒形但高频）——**未立项**，需先巡查取证（legacy-collections 巡查有初始证据）；
2. **Hold<T> 擦除对投影**（interface-headers 片登记：`T v;` 与拒绝态 ctor 并存致整类不可编译）——未立项；
3. **ScopeRefusalsEscape 未变异形**（合法多 catch 源保守拒绝）——local-scope 收尾片登记；
4. **循环测试位参数目标**（`LoopTestValues.storeTest` 保持拒绝——是否有真实源形待查）；
5. **多路分支 guard 体**（switchBody 拒绝面，诊断已移动为新面）；
6. 新巡查前沿：enum 域、字符串域、泛型域仍有"部分已测"单元（见 `openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md` 71 单元账本）。

## 工作模式（用户确立，接手者遵循）

巡查（构造 Java 场景→编译双腿→jadx/jarde 对照）→ 归因（门控实验先行）→ openspec 立项 → 派发 `opencode/deepseek-flash` subagent（isolated worktree，串行唯一构建者）→ root 独立验收（diff 审查+锚实测+权威口径门禁+oracle ignored 腿）→ 合并推送。**磁盘纪律与 ask_parent 溯源纪律见历史段，均为强制**。

## 核对命令

```sh
git status -sb && git rev-parse HEAD origin/main   # 应一致于 5adb6ba3 之后
gh run list --repo LordCasser/jarde --limit 5      # 5adb6ba3 的 run 结论
df -h /System/Volumes/Data | tail -1               # 交接时 47Gi
git worktree list                                  # 应只有主仓+codex 固定件
```

---

（以下为历史状态段，按时间倒序保留供追溯；最新在上。）

