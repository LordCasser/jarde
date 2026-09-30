## 1. 固定基线与负例

- [x] 1.1 重放固定 Tf3：核 class/源 SHA、两行分段表（无自保护行）、lead/条件正文/间隙早返回块/三副本布局与主线回退诊断（`Tf3.base.json`、`javap`、`java -Xverify:all`）；记录可重放基线。
- [x] 1.2 构造并冻结至少七个 verifier 有效负例：间隙含额外语句、三副本目标或实参槽不一致、双保存返回身份改写、lead 非 null 初始化、出现自保护行的变体、条件跳转改写、副本文法增删；各自 `java -Xverify:all` 通过并记录实现前后的拒绝。

## 2. 分段 null-lead 证书

- [x] 2.1 Guard 新增有界 prove：两行同 handler any 表且无自保护行、`[aconst_null, astore s]` lead、间隙逐指令结构（值保存 + 副本 + areturn）、三副本逐参数同形与实参槽身份（复用 Tf2 判据）、null 出处（复用 Tf1 判据）、双保存返回身份（字面量与生产者分开证明）、canonical 边与物理块全集恰等；固定类命中、1.2 全部负例拒绝、Test13/Test5/Tf1/Tf2/Tf4 固定形状互不误触。
- [x] 2.2 条件正文（`ifnonnull`/`ifne`）与字段写在保护区间内的完整所有权；预算/取消原子回滚，失败保持现有拒绝与诊断。

## 3. 区域与源码交付

- [x] 3.1 Region：lead 呈现局部声明与 `= null`；条件正文与早返回块完整拥有（早返回呈现为正文内 `return null;`），恰在正常清理前结束；失败回滚 visited。
- [x] 3.2 Builder：三副本折叠为一份清理调用；两个返回分开呈现；三副本 BCI 均有来源映射；完整类 `javac --release 8` 通过（探针恢复类全类编译通过；固定类 class-source 的 `getInputStream()` helper 在主线基线同样回退——嵌套数组分配，非本 change 引入——故固定类全类编译受该既有限制阻塞，已在巡查账本登记）。

## 4. 三方对照与验收

- [x] 4.1 探针变体（validate 可返回 false、正文可注入异常、清理可注入异常、bytes 可预置）下原 class/固定 JADX Java-input/Jarde 三方 Java 8 重编，`java -Xverify:all` 五路径（已缓存、未缓存、提前 null、正文抛错、清理抛错覆盖）逐路径一致；记录输出 SHA。
- [x] 4.2 全仓回归（`cargo test --workspace --tests --locked --no-fail-fast` 全绿）、`cargo fmt --all -- --check`、CI 同款 Clippy 新码零新增、`openspec validate --all --strict`、diff check；构建轮次间磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [ ] 4.3 root 独立复核证书边界、三副本折叠来源与五路径行为，更新 CF-16 清单与巡查账本；仅标记固定 Tf3（TestFinally3）JVM 切片，闭合该家族后重算 CF-16 分母。
