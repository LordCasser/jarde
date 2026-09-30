## 1. 固定基线与负例

- [x] 1.1 重放固定 Tf1：核 class/源 SHA、双行 any 表、lead/正文赋值/副本布局与主线整方法回退诊断（`Tf1.base.json`、`javap`、`java -Xverify:all`）；记录可重放基线。
- [x] 1.2 构造并冻结至少六个 verifier 有效负例：无 lead null 初始化、副本判空方向反转（`ifnonnull` 跳过清理/`ifnull` 进入清理改写）、两副本槽或调用目标不一致、正文区间外同槽赋值、副本文法增删指令、保存返回/重抛身份改写；各自 `java -Xverify:all` 通过并记录实现前后的拒绝。

## 2. 局部可空条件证书

- [x] 2.1 Guard 新增有界 prove：两行 any 表与自保护绑定行、`[aconst_null, astore s]` lead、正文同槽赋值全集与出处、两份四指令副本逐参数同形、SSA null 出处与合并值流、保存返回/`athrow` 身份、canonical 边与物理块全集恰等；固定类命中、1.2 全部负例拒绝、Tf4/Test14 固定形状互不误触。
- [x] 2.2 预算/取消原子回滚；证明失败整体保持现有回退与诊断。

## 3. 区域与源码交付

- [x] 3.1 Region：lead 呈现局部声明与 `= null`；正文经既有 finally 通道完整拥有，恰在正常清理前结束；失败回滚 visited。
- [x] 3.2 Builder：折叠两份副本为唯一 `if (cursor != null) { cursor.close(); }`；两份副本 BCI 均有来源映射；完整类 `javac --release 8` 通过。

## 4. 三方对照与验收

- [x] 4.1 探针变体（正文可注入异常、query 可返回 null、清理可注入异常）下原 class/固定 JADX Java-input/Jarde 三方 Java 8 重编，`java -Xverify:all` 正常（关闭一次、返回值正确）、正文抛错（关闭一次、原异常重抛）、query null（不关闭）、清理抛错（覆盖）逐路径一致；记录输出 SHA。
- [x] 4.2 全仓回归（`cargo test --workspace --tests --locked --no-fail-fast` 全绿）、`cargo fmt --all -- --check`、CI 同款 Clippy 新码零新增、`openspec validate --all --strict`、diff check；清理专用 Cargo target。
- [ ] 4.3 root 独立复核证书边界、副本折叠来源与三方行为，更新 CF-16 清单与巡查账本；仅标记固定 Tf1（TestFinally）JVM 切片。
