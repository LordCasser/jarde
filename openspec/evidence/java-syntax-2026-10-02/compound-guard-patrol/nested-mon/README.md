# 嵌套 synchronized 恢复（`recover-nested-monitor-regions`）证据

落点取证、变体前后对照与三方一致性，全部 fixture 冻结于本目录（SHA 见
[results/fixtures-sha256.txt](results/fixtures-sha256.txt)），行为输出与源码 SHA 见
[results/outputs-sha256.txt](results/outputs-sha256.txt)。

## 1. 配对扫描定位（tasks 1.1）

`crates/jarde-java/src/guard.rs` 的 `monitor()` 证书：

- **enter 计数**（`jre_guard_monitor`@4 的拒绝点）：`enters = facts.order` 全量过滤
  `Operation::Monitor { enter: true }`，原实现要求 `enters == [enter]`（一线一入）；T4.sync
  两次 enter（外层@4 + 内层@12）在此被拒。现改为恰两 enter 且第二个在外层之后时进入
  `inner_monitor_pair` 配对证明。
- **每路径 exit 检查**：`exits` 全量收集后取 `[normal_exit, handler_exit]`（BCI 升序），
  normal exit 前置 `Load{slot: lock}`、`monitor_handler(row, lock)` 证明异常路径
  `astore primary; aload lock; monitorexit; aload primary; athrow`。现嵌套形态 4 个 exit，
  内层对（同槽配对：`Load{slot: inner_lock}` 于内层行内 + `monitor_handler(内层行,
  inner_lock)`）排除后余下两个仍按原证明。
- **同锁槽判据复用**：内层头部同 `dup; astore slot; monitorenter` 惯用（`header_lock_ties`
  与外层共用一个实现），内层行 ⊂ 外层行（起点严格更晚、终点不越界），锁槽不同
  （`inner_lock != lock`，javac 对嵌套生命周期不重用槽位）。
- **内层对识别位**：`innermost(next_bci(inner_enter))` 的最窄行即内层行（T4.sync：行
  13..35→38 宽 22，外层行 5..48→49 宽 43）；内层 handler 位于内层行之后（goto 形在内层
  区域内、return 形在共享 `ireturn` 之后，同外层 handler 的地位）。

## 2. 冻结变体与负例（tasks 1.2，`java -Xverify:all` 逐一通过）

| fixture | 形态 | 链接/运行验证 | 实现前（主线 342c65bf） | 实现后 |
| --- | --- | --- | --- | --- |
| `T4.class` | 巡查固定 T4（sync = 双锁 + 循环 + 返回） | 原类运行 `10`（o4.out） | `sync` 拒绝：`jre_guard_monitor`@4 | 完整恢复，重编运行 `10` |
| `NM.class` v1 | T4 形（外层类锁 + 内层 `log` 字段锁） | `10` | 拒绝（5 方法全拒） | 恢复 |
| `NM.class` v2 | 内层对外层循环体内 | `6` | 拒绝 | 恢复（内层块是 for 的循环体） |
| `NM.class` v3 | 内层体以 `return s` 结束 | `10` | 拒绝 | 恢复（return 写在外层括号内、内层块之后，行为等价） |
| `NM.class` n3 | 三层嵌套 | `5` | 拒绝 | 仍拒绝（MVP 恰一对） |
| `NM.class` sr | 同锁可重入（javac 双槽位） | `10` | 拒绝 | **恢复**（槽位配对成立，呈现重入嵌套，语义正确；登记为现状） |
| `NMissingExit.class` | 内层 normal exit 打为 `pop`（字节补丁） | 链接验证通过；运行抛 IllegalMonitorState（可捕获） | 拒绝 | 仍拒绝 |
| `NWrongExitLock.class` | 内层 exit 位改读外层锁槽（字节补丁） | 链接验证通过（运行为 JVM 死角，不作为行为基准） | 拒绝 | 仍拒绝 |

补丁负例由 `NMiss.class`（与 NM.v1 同形）单字节/单槽位补丁生成：`monitorexit@34→pop`
与 `aload_3@33→aload_1`；`-Xverify:all` 链接验证见 fixture/`LinkProbe.java`、
`RunMiss.java`。"内层 exit 跨出外层体"的位置形态（内层 exit 位于外层 exit 之后）因
StackMapTable 约束无法做成 verifier 有效类（越界行/换位均 VerifyError——见巡查记录），
对应约束以证明层检查承载（内层行 ⊂ 外层行、内层 normal exit 先于外层 normal exit）。

## 3. 三方对照（tasks 3.2）

原 class / 固定 JADX（dev，`--no-debug-info`）/ Jarde 重编三方逐路径一致
（`java -Xverify:all`）：

- T4：`nested → body[b]mid[a]`、`pick → 3:42:null`、`sync → 10` 三方一致
  （JADX 的 `nested` 亦不恢复——TWR×finally 为下一片；Jarde 的 `nested` 同）。
- NM：`v1→10, v2→6, v3→10, n3→5, sr→10` 三方一致（JADX 三层 n3 亦恢复，Jarde 按范围
  拒绝——非本片验收路径）。
- JADX 源码两处不可达/越界残迹（`main` 的越界 `return i2;`、`sr` 尾部不可达
  `return i2;`）按惯例剔除后编译运行；剔除不触及对比路径。

## 4. 呈现与既有形态

- 呈现复用 `StmtKind::Synchronized` 与 `lock_expr`：外层体一棵区域树（含内层块周围的
  循环/分支），内层对按 span 在指令行走中开合括号（`NestedPairBraces`），无新呈现函数。
- 单层（`Locked.class`）与两臂多出口（`SynchronizedMultiExit.class`）文本逐字不变
  （`crates/jarde-java/tests/nested_monitor_regions.rs` 末项固定断言）。
- 已知限制：内层体为空（`synchronized (a) { synchronized (b) {} }`）仍拒绝；内层体含
  用户 try/catch、fallback 引用块仍拒绝（`monitor_body_supported` 限定 Straight/If/Loop/
  Sequence）；monitor 与 finally/TWR 复合不做（另片）。

## 5. 环境附记

`tests/p3_boolean_short_circuit_return.rs` 的深链值渲染递归（`render_value` ×
`MAX_VALUE_DEPTH`，debug 帧约 69KB/层）在主线即处于默认 2MiB 测试线程栈约 60KB 余量内；
本片任何代码移动都会翻转代码生成布局。该测试改为在显式 16MiB 栈线程上运行同一组断言
（断言本身零改动），`render_value` 自身的深度上限仍是真正的界限。
