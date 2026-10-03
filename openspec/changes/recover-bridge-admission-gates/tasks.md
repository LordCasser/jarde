## 1. 取证与基线

- [x] 1.1 重放固定 BR 家族五类（SHA 核对：与 `../../evidence/java-syntax-2026-10-04/bridge-method-patrol/results/fixture-sha256.txt` 逐条一致，永久 fixture 入 `tests/fixtures/p3-bridge-projection/br-family/`）：读两门判据上下文（`src/facade.rs` 擦除返回门、`bridge.rs` 体形门）与既有投影通道；确认门 1 换层级 walk 不影响"继承需求"判定输入（直接 super/interfaces 解析独立），门 2 参数 cast 形可重建（cast 目标恰等于被转发调用描述符参数位）。取证另实锤三道额外门（MethodParameters 属性、桥 flags 0x1040、源级可见度）与两项追补（继承解析 caller 身份、Comparable 平台事实），经 root 逐项批准后一并扩展。基线四桥拒绝文本存
  `../../evidence/java-syntax-2026-10-04/bridge-method-patrol/results-gates/baseline-refusals-*.txt`。
- [x] 1.2 冻结变体/负例（tests/class_source.rs `br_family_negative_shapes_keep_their_refusals` + `the_parameter_cast_admission_walks_the_snapshot_chain_and_respects_its_edges`、p3_patterns 门 2 形测试）：源返回与桥返回无层级关系（拒绝，文本不变）、cast 目标≠源级参数类型（bridge@1 位相拒绝）、效果不纯桥（既有拒绝不变）、多层级协变（BR2 接口→父接口，正例双腿）；另钉死 named/count 不符/双 MethodParameters、flags 越界位/缺 synthetic、桥 public 源非 public、真实层级折返仍拒、提供 Comparable 走定义不短路。各变体 `java -Xverify:all` 通过并记录实现前后行为（results-gates/README.md §4–5）。**Impl 整类重编另受类级参数化接口投影缺口阻塞**（`src/class_source.rs` 泛型头机制对只有参数化接口的类静默跳过；`javac-after-raw-Comparable-loud-failure.stderr.txt` 钉死响亮失败形态），root 裁决另立专项，不并入本片。

## 2. 两门扩展

- [x] 2.1 门 1 换快照层级 walk（共享核 `snapshot_header_chain_widens_with`，深度上界 8、依赖深度观测、计费同源）；`BR$Base` 桥被投影隐藏、真实协变覆写保留（projection marker 见 results-gates/projected-BR$Base.java）。
- [x] 2.2 门 2 接受规范参数 cast 形（逐位相等判据 + 转发到同类源级目标 + void 桥形；cast 不消除）；`BR$StrBox`/`BR$Impl` 桥被隐藏；BR 家族 Node/Box/Base/StrBox 整类 `javac --release 8` 通过、`java -Xverify:all` 运行与原 class 逐路径一致（`s` 路径见 results-gates/run-*.trace.txt 与 class_source e2e same-runner 双腿）；**Impl 桥隐藏与源保留已达成，整类 javac 受 1.2 所记缺口阻塞（响亮失败，非静默），缺口另列**。
- [x] 2.3 负例保持拒绝（§1.2 全清单断言拒绝文本逐字）；既有桥投影正例与 `negative/`、`orphan/` 负例 diff 逐字不变（`bridge_admission_needs_the_same_run_shape_source_and_resolved_parent_contract` 等既有断言全绿，单类负例 usage 断言守零额外读取）；预算/取消原子性不变（OutputBytes 停止断言不变，候选计费仅按新增 cast 项计数）。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 294 目标全绿（export_cli 计时 flake 复跑通过，handoff 已知家族）；fmt、CI 完整 29 项 `-A` clippy `-D warnings`、`openspec validate --all --strict`（264 passed）全过；corpus fingerprint 再生为纯新增（零既有行改动）；双腿扫描：单类 465 类零差异、家族腿差异仅桥声明→投影标记；磁盘纪律遵守（每轮构建前 `df -h /`，最低 38Gi，未触发 clean 线）。
- [x] 3.2 BR 家族与变体对照：原 class 与 Jarde 重编两方 javac + `-Xverify:all` 逐路径一致（含经接口引用调用路径），输出与恢复源 SHA 记录于 results-gates/；固定 JADX 对照未跑（其改名路线既有决策仅作对照不作准入，且 dev 构建不在本片环境），root 复核前置部分按分工归 3.3。
- [ ] 3.3 root 独立复核两门判据、可重建性论证与三方行为，更新账本与巡查记录；`project-proved-bridge-forwards` 自身未勾项按独立债务另行登记，不在本片代办。
