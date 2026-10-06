# Root 独立验收（2026-10-06，v2，合并）

## 判据逐项

1. **落点与门控实验复核**：实现片 `results/02-locus-experiment-*` —— claim 关闭时锚渲染与父提交逐字节相同（1 拒绝、双腿）、开启时 0 拒绝——`Sites.owned` 单点翻转成立（root 追认落点裁定：无平行所有权机制，`discarded_null_check_window` 为构造尾/接收者尾共享的窗口纪律，谓词仍是 `facts::is_discarded_null_check`）。
2. **diff 审查**：`lambda.rs` 仅 +1 参数（`receiver_nonnull`）+1 合取（`!receiver_nonnull`），拒绝文本/码、`VALUE_LEVEL_REFUSALS`、`unreplayable`、`render_value` 零触碰；`init.rs` 接收者尾在构造尾**之后**按先到先得入同一 `owned` 集（构造/接收者冲突避免）；`build.rs` 的 move 链（`Load→Store→<init>→Duplicate→Allocate`，`<init>` 臂为实现片实证新链环）+ 捕获经尾读回。
3. **C1.chain 守卫交互（实现片自发现的第三道门）**：站点描述符须点名被分配类（三方捕获判据的 frame/site 半边），否则 `List<String> out = new ArrayList<>()` 声明超类型形会把整方法拒绝变成"可编译且丢 forEach 效果"的部分呈现——root 追认为必要门，单元测试钉住 C1 与父提交逐字节。
4. **root 实测（合并态自建 CLI，jar 输入+自述头断言）**：锚 `OP.sideEffect` 完整恢复（`arg0.ifPresent((Consumer) ((Object p0) -> local1.append((String) p0)))`，0 引注）；BRN 三负例（可空参数/字段读/捕获后重写）拒绝**逐字**保持 ×3；回放（ignored）通过（双腿编译+`-Xverify:all` 输出与原 class 逐字一致）。
5. **定向测试**：`recover_proved_nonnull_bound_receivers` 2 passed + 1 ignored 回放 passed；`class_source` 102/102（typed-functional `this::length`/`this::label` 零回退）；`p3_lambda_adaptation` 10+2 ignored。
6. **门禁（root 合并态）**：全量 **3097 passed / 0 failed / 59 ignored**（与实现片一致）；fmt exit 0；CI 逐字 clippy `Finished` exit 0；`openspec validate --all --strict` **303/303**。corpus 差分（实现片自检先行）：3,464 候选、移动=5 全为本形解锁（refusals 1→0）、13 未渲染逐项列明（12 为故意损坏负例 + package-info）。census `(735,3061,282,1827,8)→(739,3083,284,1833,8)` + fingerprint +6。
7. **CI**：合并推送后 run 为准（监控在案）。

## 残余边界（实现片登记，root 确认）

- 实现句柄 owner 对照需 claim 时的 bootstrap 表事实（`out::equals`/`AbstractList` 形仍部分呈现，语料 0 实例）——后续片须 root 裁定 bootstrap 事实的准入再动，不静默扩。
