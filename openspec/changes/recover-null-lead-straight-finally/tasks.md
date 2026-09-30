## 1. 固定基线与负例

- [ ] 1.1 重放固定 Tf2：核 class/源 SHA、双行 any 表、lead/直体/两份无条件调用副本布局与主线拒绝诊断（`Tf2.base.json`、`javap`、`java -Xverify:all`）；记录可重放基线。
- [ ] 1.2 构造并冻结至少六个 verifier 有效负例：lead 非 null 常量/多指令、副本调用目标或实参槽不一致、实参来自其它槽或调用结果、保存返回改写为其它值/身份断裂、副本文法增删指令、自保护行扩围；各自 `java -Xverify:all` 通过并记录实现前后的拒绝。

## 2. finally_copy lead 准入扩展

- [ ] 2.1 在既有认领门槛并列 `[aconst_null, astore s]` lead 变体（`statement_boundary` 判据），副本增加实参槽身份证明（每个实参读 lead 槽 s 的合并值流）；固定类命中、1.2 全部负例拒绝、Tf1/Tf4/Test14/Test9 固定形状互不误触；字段赋值 lead 与无前置路径行为不变。
- [ ] 2.2 保存返回构造位置证明（区间内、无跨段移动）；预算/取消原子回滚，失败保持现有拒绝与诊断。

## 3. 区域与源码交付

- [ ] 3.1 Region：lead 呈现局部声明与 `= null`；直体正文完整拥有，恰在正常清理前结束；失败回滚 visited。
- [ ] 3.2 Builder：折叠两份副本为一份清理调用；正文末呈现 `return new Result(400);`；两份副本 BCI 均有来源映射；完整类 `javac --release 8` 通过。

## 4. 三方对照与验收

- [ ] 4.1 探针变体（正文可注入异常、getInputStream 可返回 null、清理可注入异常）下原 class/固定 JADX Java-input/Jarde 三方 Java 8 重编，`java -Xverify:all` 正常（关闭一次、返回 400）、正文抛错（关闭一次、原异常重抛）、null 流（closeQuietly 收到 null 不抛）、清理抛错（覆盖）逐路径一致；记录输出 SHA。
- [ ] 4.2 全仓回归（`cargo test --workspace --tests --locked --no-fail-fast` 全绿）、`cargo fmt --all -- --check`、CI 同款 Clippy 新码零新增、`openspec validate --all --strict`、diff check；构建轮次间磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [ ] 4.3 root 独立复核 lead 准入边界、实参槽身份与三方行为，更新 CF-16 清单与巡查账本；仅标记固定 Tf2（TestFinally2）JVM 切片。
