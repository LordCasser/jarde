# Root 独立验收（2026-10-08，合并）

## 判据逐项

1. **栈几何复核**：`results/01-probe-instance-chain.txt` —— `dup_x1` 的三写（copy-under/moved/copy-on-top）以其**两次读所指槽位**命名；三 `aload_0` 写 V1/V2/V3 但都读槽 0 的同一入口值 V0（same-this 身份）；putfield c/b/a 的 receiver=value 配对（moved this₃/this₂ + 原 this₁）——几何与 SSA 双重钉死。
2. **门控复核**：`IDENTICAL: CH,SC,BF,BG,CA2,CF,MX,NEG / MOVED: CP`——实例形准入单独翻转，静态/复合/拼接/边界全逐字节。落点=`OPCODE_DUP_X1` 臂的**最后回退**（`instance_chain_at`），既有三证明零触碰。
3. **diff 审查**：`build.rs` `instance_chain_at`+`dup_x1_writes`+`reads_this`+`FieldCopy::moved` 渲染；`report.rs` 传 `has_receiver()`（ACC_STATIC 自有读法——static 体槽 0 是参数非 this，正确收窄）；三处既有构造点 `moved: None`。
4. **root 实测**：CP.inst 恢复为 `this.c = 5; this.b = 5; this.a = 5;`（字节码序=源求值序从右到左），0 引注；定向 3+1 ignored（双腿重编+`-Xverify:all`，`5/5/5`/`7/7/5` 一致；MX/NEG 剥离文本不可编译钉住）+ chained-field 3+1 零回退；oracle 3/3 无陈旧期望。
5. **门禁（权威口径）**：全量 exit 0、**341 targets ok、0 FAILED**；fmt OK；CI 逐字 clippy `Finished` 0；openspec **313/313**。census `(927,3957,368,2475,8)→(933,3993,368,2475,8)`（断言更新非删除，双向实测）；指纹恰九新条目。
6. **corpus**：moved=2（本片 CP 双腿）；2920 loose+739 jar 其余逐字节。flake 两见（two_exit_return scratch/export_cli 族）复跑 ×2 绿。
7. **CI**：合并推送后 run 为准（监控在案）。

## 边界裁定

四登记边界全采纳：mixed chain（需两形皆无的判据）、impure source（实例形无自有 saved 局部——invariant 3 的纯源读法）、非 this receiver（参数/局部链）、mid-dance 起始。探针边界关闭记录在案（不改已合并证据文件，正确）。
