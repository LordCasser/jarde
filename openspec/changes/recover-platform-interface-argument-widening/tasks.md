# Tasks

- [x] 1. 复核 `snapshot_header_chain_widens_with` 的取 header 顺序；把"目标名命中"前移到目标 header 读取之前，并确认只影响单边（快照内源→平台目标）
      → 预审计复核：机制成立（行号漂移到 `src/facade.rs` `snapshot_header_chain_widens_with` 起 17203；旧序为 `header(&name)` → `name == target`，目标不在快照即 `continue`）。实现只交换这两步，目标名由**上一跳自己的 header** 逐字给出（源侧仍逐级读取证明），不查 classpath、不猜；`SNAPSHOT_HIERARCHY_WALK_DEPTH` 与深度检查位置不变，"第八边证明/第九边拒"语义不变（见 [verification.md](verification.md)）
- [x] 2. 对照测试：CP.byAnon `[10,20,30]`、AH/AC `[a, bb, ccc]`、AN 两-sided 正例零回退、无关系负例仍拒；fixtures 双 javac 协议
      → `tests/recover_platform_interface_argument_widening.rs` + `tests/fixtures/recover-platform-interface-argument-widening/`（`v8` = javac 23.0.1 `--release 8`、`v8-javac8` = Corretto 1.8.0_432，17 类 × 2 腿逐字节冻结）：CP.byAnon / AH.byTop / IS.byAnon（隔离形 `[10, 20, 30]`）整条调用呈现；AN 四位（return/实参/字段初始化/局部）逐字零回退；AW 命名平台接口（`java.lang.Runnable`）已证、无 header 关系（`java.lang.Throwable`）保持原拒文本；ignored replay 剥离后由 installed javac 与真 javac 8 双腿编译运行对照原类
- [x] 3. 全门禁 + 分逻辑提交（不 push）
      → 门禁与提交见 [verification.md](verification.md)
