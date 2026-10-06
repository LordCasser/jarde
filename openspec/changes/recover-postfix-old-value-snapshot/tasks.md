## 1. 取证与基线（root 已完成大半）

> **锚矩阵（root 2026-10-06 补全，源自 proposal 各追加段；MVP 分相）**：
> **A 相（本片必达）**——局部快照位（CM：`int j = i++;`/`return i++ + 10;`）、下标位三变体（`arr[idx++] = 10` 局部字段数组；**`elems[size++] = t`** AD.add 核心形=依赖链族旗舰锚，数组引用跨 putfield 舞蹈保存；静态 `src[pos++]` 三元臂）、数组存 RHS 旧值（`a[i] = i++`，现被守卫的 compilable-wrong 面，恢复即覆盖）。
> **B 相（独立后续片，本片不做）**——条件位三形（`while(xs[i++] != 0 && …)` 等，撞 local-crossing 门，机制面不同）。
> 负例：`i = i++`/`i = i--`（自赋陷阱，Non-Goal）、多消费方形。

- [x] 1.0 四形判别、jadx 双解、时间引注已知旧值存在——实测归档。（root 已完成）
- [x] 1.1 插桩定位时间引注发出处与 SSA 旧值节点表示（CM 局部形 + AD.add 依赖链形各定位一次——两形的拒绝路径可能不同：时间引注 vs saved-declaration 依赖链）；转录存证据。
  - **两形确实不同落点，且 AD 形还有第三条**：时间引注在 `build.rs` `render_value` 的 `Operation::Load` 臂（`slot_name_denotes_the_same_value` 答否）；依赖链/saved-declaration 在 `prepare_deferred_bindings` 的 `has_independent_boundary` 返回 `None`（其依赖走查停在 `Operation::Other`——即字段舞蹈的 `dup_x1`）与 `binding_refused` 渲染臂；三元臂形另撞 `conditional_arm_is_expression` 的 "the conditional arm contains an independent instruction"。三条路径的发出处、触发条件与逐锚门控实验（禁用单点改动 → 仅该形翻回拒绝）见 [results/01-instrumentation.md](results/01-instrumentation.md)。
- [x] 1.2 重验基线：主线二进制渲染 CM/CM2 与 AD/GA（两形拒、健康形恢复）；负例探针（`i = i++`）现状拒绝记录。
  - 父提交 `207760d2` 二进制（`/tmp/posv-base-target/debug/jarde-cli`）实测：`CM` 渲染与巡查归档 `jarde-CM.txt` **逐字节相同**（四形判别完整复现）；`AD`/`GA` 的拒绝**诊断逐条相同**（`the dependency chain from BCI 1 to final consumer 16 is not bounded` 等），文本差异仅在 `preserve-postfix-fallback-soundness` 守卫（晚于巡查落地）的整方法引注包装与扩展后的 `@bytecode` BCI 列表；`SA`（`i = i++`）整方法引注、`FS`（字段自赋/复合混合）引注安全，均如巡查记录。见 [results/02-anchors.md](results/02-anchors.md) 的基线一节。
- [x] 1.3 冻结 fixture：CM/CM2 + AD（复用巡查冻结件，SHA 核对）+ 静态三元形 + `a[i] = i++` 形，双腿（javac23 `--release 8` + 真 8）入 `tests/fixtures/`，README 记编译命令与 SHA。
  - 目录 `tests/fixtures/recover-postfix-old-value-snapshot/`：八类（`CM`/`CM2`/`AD`/`GA` 巡查源逐字复用，`PT` 静态三元臂、`SR` 存 RHS 与局部下标写位、`IX` 局部/静态下标四形、`NG` 负例）双腿各 9 个 class + 8 源 + `sha256.txt` + README（编译命令与逐文件 SHA）。`v8/AD.class`、`v8/CM.class`、`v8/GA.class` 与巡查 jar 内条目 **SHA 相同**（`8418a5c4…`/`96237630…`/`1d204957…`）。

## 2. 实现

- [x] 2.1 按决策 1 识别后缀模式（旧值 load 先于 iinc/putfield 舞蹈 + 单消费方）呈现 `x++` 表达式形（消费位收旧值语义）；多消费方保持拒绝。局部 iinc 形与字段 dup_x1 舞蹈形共享同一快照判据（快照值=pre-update SSA 值），若插桩显示两形落点不同则分别实现并各自门控实验证实。
  - 快照值就是 pre-update SSA 值（load 的栈输出 / 舞蹈留在接收者下方的拷贝），证明取其"单消费方 + 更新后 + 区间仅表达式自身管道 + 区间内不读更新值"；局部形与字段形分两段实现、各自门控（见 1.1 表）。`i = i++`/`i = i--` 由"消费者写回快照自身槽位"排除；字段版 `f = f++` 同理按成员身份排除；多消费方（`i += i++ + 1`、`i + i++`）由"其它读必须是增量前语句的读或同一语句指令的读"排除。实现见 `crates/jarde-java/src/build.rs` 的 `prove_snapshots`/`prove_local_snapshots`/`prove_field_snapshots`/`snapshot_consumer`/`snapshot_expression`。
- [x] 2.2 呈现按决策 2（后缀形，测试钉死）；健康三形零改动；B 相条件位不触碰（local-crossing 门零改动）。
  - 前缀/拆语句/复合三形逐字节不变（`tests/recover_postfix_old_value_snapshot.rs` 的 `the_healthy_shapes_stay_byte_identical_on_both_legs`）；B 相 `NG.condShape` 仍以 `local 0 crosses a quoted fallback region` 整方法拒（该门与 `region.rs` 未改动），`AC.ioLoop`/`condAssign` 文本与基线逐字节相同。

## 3. 验证与验收

- [x] 3.1 主锚（A 相全矩阵）：CM immUse/postfixExpr、`arr[idx++]` 写/读位、AD/GA `elems[size++] = t`（依赖链族旗舰——`null/null vs x/y` 的 compilable-wrong 面关闭）、静态三元 `src[pos++]`、`a[i] = i++`（RHS 旧值）；各形 `javac --release 8` exit 0、`-Xverify:all` 行为逐行一致（旧值语义精确）。
  - 全部锚点 0 条自身诊断、整方法呈现；双腿编译 exit 0、`-Xverify:all` 运行输出与原 class 逐行相同（`CM 2/30/2/29/5`、`CM2 5/6/15/16`、`AD x/y`、`PT a/b/null/3`、`SR 102/104001`、`IX 8/0/0/20/0`）。`GA` 整类因既存 `GA$Cfg` 池形嵌套名不编译（父提交同），其锚以隔离探针双腿 `x/y` 验证。见 [results/02-anchors.md](results/02-anchors.md) 与 `results/behavior.sh`。
- [x] 3.2 零回退：三健康形逐字节不变；负例（`i = i++`、多消费方）仍拒；corpus 双腿扫描 diff 为空。
  - 三健康形逐字节不变 ✓；负例仍拒且 `NG` 去注释文本不可编译（安全形）✓；`FS` 渲染与父提交逐字节相同 ✓。corpus 双腿扫描（父提交二进制 vs 本片二进制，单类姿态全语料 + 每 jar 条目）移动 **29 类**，逐类分类见 [results/03-corpus-delta.md](results/03-corpus-delta.md)：14 为本片新 fixture、8 为巡查冻结锚（CM/AD/GA/AC/SD/SA/CH/LocalRewrite）、3 为机制同族的消费位（`PC.viaArg`/`PC.viaChain`/`IT$IntRange.next` 的**调用实参位**）——后者非矩阵锚点，行为已逐一双腿实测精确（`1/n5:6`、`3/4/5`），作为范围说明上报。
- [x] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
  - `cargo test --workspace --all-targets --all-features --locked --no-fail-fast`：323 个 target 全绿、exit 0（`p4_plugins` 计时族复跑一次绿，判定为既有 flake 家族）；`cargo fmt --all -- --check` 干净；CI 逐字 clippy（`sed -n '46,76p' .github/workflows/ci.yml …`）exit 0；`openspec validate --all --strict` 303/303；`git diff --check` 干净；reader fixture census 重测 `(783, 3275, 286, 2023, 8)`、corpus fingerprint 再生（+26 文件，纯增）。命令与尾部见 [results/04-gates.md](results/04-gates.md)。
- [ ] 3.4 root 独立复核：归因非新机制、旧值语义精确（行为对比）、零回退实测；关闭 summary.md 登记行。（留 root）
