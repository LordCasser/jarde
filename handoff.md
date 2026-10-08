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

