## 1. 固定基线与负例

- [x] 1.1 重放固定 Tf2：核 class/源 SHA、双行 any 表、lead/直体/两份无条件调用副本布局与主线拒绝诊断（`Tf2.base.json`、`javap`、`java -Xverify:all`）；记录可重放基线。（`results/tf2-baseline-replay.txt`：fixture SHA 与巡查账本一致、异常表 `[2,25)→32 any` + `[32,34)→32 any`、基线 7a312458 的 CLI 重放与 `Tf2.base.json` 同诊断 `jre_guard_finally_copy`@32、固定类 `-Xverify:all` 通过。）
- [x] 1.2 构造并冻结至少六个 verifier 有效负例：lead 非 null 常量/多指令、副本调用目标或实参槽不一致、实参来自其它槽或调用结果、保存返回改写为其它值/身份断裂、副本文法增删指令、自保护行扩围；各自 `java -Xverify:all` 通过并记录实现前后的拒绝。（九个负例：五个源编译 `Tf2LeadField/Tf2LeadExtra/Tf2SlotMismatch/Tf2ArgOtherSlot/Tf2CleanupExtra` + 四个同长补丁 `Tf2TargetMismatch/Tf2ReturnIdentity/Tf2RethrowIdentity/Tf2SelfRowWidened`（`negatives/patch-tf2.py`）；`verify-Tf2*.txt` 全过，`baseline-refusals-tf2.txt`（实现前全拒）与 `recovery-tf2.txt`（实现后仍全拒）成对。）

## 2. finally_copy lead 准入扩展

- [x] 2.1 在既有认领门槛并列 `[aconst_null, astore s]` lead 变体（`statement_boundary` 判据），副本增加实参槽身份证明（每个实参读 lead 槽 s 的合并值流）；固定类命中、1.2 全部负例拒绝、Tf1/Tf4/Test14/Test9 固定形状互不误触；字段赋值 lead 与无前置路径行为不变。（`completed_null_local_lead` 第三 lead 答案；`cleanup_sequence`/`prove_finally_copy` 的 `admit_loads` 只由该 lead 授予，其余两 lead 的副本文法逐字节不变；`null_lead_copies_read_the_lead` 复用 Tf1 的 `local_null_handler_provenance` 形态做槽身份与唯一性定义普查；guard 单测 `null_lead_straight_*` 与 e2e `p3_null_lead_straight_finally` 全绿，家族形状测试锁 Tf1/Tf4 各自证书。）
- [x] 2.2 保存返回构造位置证明（区间内、无跨段移动）；预算/取消原子回滚，失败保持现有拒绝与诊断。（保存返回 `astore_3`@24 在 `proof.protected` 区间内、`SavedReturn` 完成式正文呈现；`null_lead_*` 检查的每个 `charge` 在证毕前返回 `StopReason`，presented/refused 与基线同码同锚；guard 测试断言 `Budget`/`Cancelled` 传播。）

## 3. 区域与源码交付

- [x] 3.1 Region：lead 呈现局部声明与 `= null`；直体正文完整拥有，恰在正常清理前结束；失败回滚 visited。（`build.rs` 声明规划器的 `null_lead_straight` lead 准入 + 同族 `nullable_type` 类型判定：`java.io.InputStream local2; local2 = null;` 分离式（既有裁决），正文 [2,25) 由 Guard 区域完整拥有，`regions[0].blocks == [0, 32]`。）
- [x] 3.2 Builder：折叠两份副本为一份清理调用；正文末呈现 `return new Result(400);`；两份副本 BCI 均有来源映射；完整类 `javac --release 8` 通过。（`finally { this.closeQuietly(local2); }` 恰一份；构造 `new Tf2$Result(400)` 留在正文内，完成式按既有 saved-return 呈现 `Tf2$Result local3 = new Tf2$Result(400); return local3;`（design 决策 2 的呈现层裁决——改走内联会改变 FinallyNormal 的既有输出）；26 个物理 BCI 全有来源；探针侧完整类 `javac --release 8` 通过（`probe/jarde-tf2.javac.stderr` 空），固定类正文引用的 `Tf2$Result` 是独立 class 文件，按巡查惯例在探针侧完成整类编译。）

## 4. 三方对照与验收

- [x] 4.1 探针变体（正文可注入异常、getInputStream 可返回 null、清理可注入异常）下原 class/固定 JADX Java-input/Jarde 三方 Java 8 重编，`java -Xverify:all` 正常（关闭一次、返回 400）、正文抛错（关闭一次、原异常重抛）、null 流（closeQuietly 收到 null 不抛）、清理抛错（覆盖）逐路径一致；记录输出 SHA。（`probe/src-tf2` 四路径三方 `run-*-tf2.txt` 逐字节相同（SHA `45300452…`），三侧 verify 输出为空，`behavior-tf2-sha256.txt` 记录；JADX Java-input 仍仅作参照。）
- [x] 4.2 全仓回归（`cargo test --workspace --tests --locked --no-fail-fast` 全绿）、`cargo fmt --all -- --check`、CI 同款 Clippy 新码零新增、`openspec validate --all --strict`、diff check；构建轮次间磁盘低于 15Gi 先 `cargo clean`，完成即清。（落地提交全绿：workspace `cargo test` 0 失败、fmt/clippy 干净、validate 全过；见落地记录与提交说明。）
- [ ] 4.3 root 独立复核 lead 准入边界、实参槽身份与三方行为，更新 CF-16 清单与巡查账本；仅标记固定 Tf2（TestFinally2）JVM 切片。
