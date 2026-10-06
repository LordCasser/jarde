## 1. 取证与基线（root 已完成大半）

> **锚矩阵（root 2026-10-06 补全，源自 proposal 各追加段；MVP 分相）**：
> **A 相（本片必达）**——局部快照位（CM：`int j = i++;`/`return i++ + 10;`）、下标位三变体（`arr[idx++] = 10` 局部字段数组；**`elems[size++] = t`** AD.add 核心形=依赖链族旗舰锚，数组引用跨 putfield 舞蹈保存；静态 `src[pos++]` 三元臂）、数组存 RHS 旧值（`a[i] = i++`，现被守卫的 compilable-wrong 面，恢复即覆盖）。
> **B 相（独立后续片，本片不做）**——条件位三形（`while(xs[i++] != 0 && …)` 等，撞 local-crossing 门，机制面不同）。
> 负例：`i = i++`/`i = i--`（自赋陷阱，Non-Goal）、多消费方形。

- [x] 1.0 四形判别、jadx 双解、时间引注已知旧值存在——实测归档。（root 已完成）
- [ ] 1.1 插桩定位时间引注发出处与 SSA 旧值节点表示（CM 局部形 + AD.add 依赖链形各定位一次——两形的拒绝路径可能不同：时间引注 vs saved-declaration 依赖链）；转录存证据。
- [ ] 1.2 重验基线：主线二进制渲染 CM/CM2 与 AD/GA（两形拒、健康形恢复）；负例探针（`i = i++`）现状拒绝记录。
- [ ] 1.3 冻结 fixture：CM/CM2 + AD（复用巡查冻结件，SHA 核对）+ 静态三元形 + `a[i] = i++` 形，双腿（javac23 `--release 8` + 真 8）入 `tests/fixtures/`，README 记编译命令与 SHA。

## 2. 实现

- [ ] 2.1 按决策 1 识别后缀模式（旧值 load 先于 iinc/putfield 舞蹈 + 单消费方）呈现 `x++` 表达式形（消费位收旧值语义）；多消费方保持拒绝。局部 iinc 形与字段 dup_x1 舞蹈形共享同一快照判据（快照值=pre-update SSA 值），若插桩显示两形落点不同则分别实现并各自门控实验证实。
- [ ] 2.2 呈现按决策 2（后缀形，测试钉死）；健康三形零改动；B 相条件位不触碰（local-crossing 门零改动）。

## 3. 验证与验收

- [ ] 3.1 主锚（A 相全矩阵）：CM immUse/postfixExpr、`arr[idx++]` 写/读位、AD/GA `elems[size++] = t`（依赖链族旗舰——`null/null vs x/y` 的 compilable-wrong 面关闭）、静态三元 `src[pos++]`、`a[i] = i++`（RHS 旧值）；各形 `javac --release 8` exit 0、`-Xverify:all` 行为逐行一致（旧值语义精确）。
- [ ] 3.2 零回退：三健康形逐字节不变；负例（`i = i++`、多消费方）仍拒；corpus 双腿扫描 diff 为空。
- [ ] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
- [ ] 3.4 root 独立复核：归因非新机制、旧值语义精确（行为对比）、零回退实测；关闭 summary.md 登记行。（留 root）
