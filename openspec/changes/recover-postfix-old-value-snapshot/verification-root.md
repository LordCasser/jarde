# Root 独立验收（2026-10-06，A 相，合并）

## 判据逐项

1. **三路径插桩复核**：`results/01-instrumentation.md` —— A（时间引注，`render_value` Load 臂）/ B（依赖链+saved-declaration，`prepare_deferred_bindings` 停在 `dup_x1`）/ C（条件臂独立指令）三条路径各有落点与**逐锚门控**（快照证明单独→CM/CM2/SD；+吸收感知→AD/GA；+条件臂→AC/PT；关掉条件臂改动则 AC/PT 回拒而 AD 保持）。字段舞蹈形**零新机制**（复用 `prove_postfix_field` 身份纪律，消费者从 ireturn 泛化为单消费者）。
2. **diff 审查**：build.rs +907 行核心——`SnapshotValue`/`prove_{local,field}_snapshots`/`snapshot_consumer` + 渲染钩子 + 三个走查的吸收感知（`absorbed` 参数化）+ **事后记账检查**（`collect_quoted_bcis`：声称的快照既未渲染也未引注→整方法引注——方向保守的失败关闭，追认）。判据块与拒绝文本不动。
3. **root 实测（合并态自建 CLI，jar+自述头）**：`AD.add` → `this.elems[this.size++] = arg1;` 原源码形态、0 依赖链引注（`null/null vs x/y` compilable-wrong 面关闭）；定向 `recover_postfix_old_value_snapshot` 7 passed + p3_local_rewrite 4+1 ignored + 回放双腿编译运行一致（7.13s ignored 腿）。
4. **守卫更新审查**：`p3_local_rewrite` 两断言更新为恢复态（改名+新期望，非删除，控制组未动）✓；census/fingerprint 再生纯新增 ✓；计费 pin 未动（证明只在身份检查后计费）✓。
5. **门禁（root 合并态，权威口径）**：全量 exit 0、**323 targets 全 ok、3108 passed / 0 failed / 62 ignored、零 FAILED 行**（注：早先两轮报 "failed=1" 系 root 快记 awk 把 `test result…` 开头的测试名行误计——教训记录：总数判定以 exit 码 + `test result: FAILED` 行 + ok 计数为准，不用宽匹配 awk）；fmt 0；CI 逐字 clippy `Finished` 0；openspec **304/304**。corpus 差分 29 类全分类：14 本片 fixture + 8 巡查锚 + **3 个矩阵外恢复**（`PC.viaArg/viaChain` 实参位、`IT$IntRange.next`、增强 for 字段下标形——**追认为机制内外沿**，与 ladder 同级 switch 同规：读法只证明快照自身，消费位泛化是同一判据的正确外推；三形已行为验证与原一致）。
6. **负例**：`i = i++`/`i = i--`/`f = f++`/`i += i++ + 1` 与 B 相条件形（`AC.ioLoop`/`condAssign`）逐字回拒；Phase B 门未触碰。
7. **CI**：合并推送后 run 为准（监控在案）。

## 残余

- Phase B（条件位三形）显式延后——local-crossing 门零改动，独立后续片；
- GA 整类编译被既有 `GA$Cfg` 池拼写债挡（父提交已在），锚经隔离探针验证；
- 矩阵外三形已追认；若后续巡查发现该泛化产生任何行为差文本，按守卫四族处置。
