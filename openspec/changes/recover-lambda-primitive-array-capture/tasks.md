## 1. 取证与基线

- [x] 1.1 根因零构建定位：`frame.rs:2299-2315` `newarray` 有意压 `Ref(RefType::Unknown)`（bootstrap loader 未声明故不锚名），`anewarray` 走 `pool_class_name` 有名；判别变量（StringBuilder ✓ / int[] ✗）实测。（root 已完成）
- [ ] 1.2 **插桩定夺方向 A/B（不得预设）**：(a) P02 class 是否含 LVT（javap 实测——javac 默认 `-g:lines,source` **不**发 LocalVariableTable，若无可写 `-g` 重编一腿对照，但**冻结 fixture 以默认编译为准**）；(b) `capture_types`（lambda.rs:705）的 `ty` 在 P02 上的实际来源（名字表/SSA 值哪一层把 Object 传进来）；(c) `RefType::Unknown` 的全消费面清单与"newarray 结果无名"依赖核对（方向 A 的前置）；(d) `RefType` 能否承载描述符形 `[I`。按 design 决策 1 的取舍标准选 A 或 B，插桩转录存证据目录。
- [ ] 1.3 重验基线：主线二进制渲染 P02 双腿，`sum` 3 引注（BCIs 15/8/9）拒绝、`map` 内联正常。

## 2. 实现

- [ ] 2.1 按 1.2 结论实现方向 A（帧层 `atype`→有名描述符类型）或方向 B（捕获门用陈述类型），`lambda.rs:765-779` 三方一致判据**逐字不动**。
- [ ] 2.2 若走 A：`RefType::Unknown` 消费面逐个核对记录（哪些曾依赖 Unknown、给名后行为），存证据目录；任何消费者的行为变化即停下报告。
- [ ] 2.3 拒绝文本与注释随取值来源变化同步（若帧类型来源语义变了，注释如实更新，不发明新拒绝码）。

## 3. 验证与验收

- [ ] 3.1 主锚：`P02_lambda.sum` 双腿恢复（0 引注、lambda 捕获内联与 `map` 同构）；渲染源集 `javac --release 8` exit 0、运行 `6` 一致。
- [ ] 3.2 零回退：`map` 与 DT-26 既有 fixture（`dt26-lambda-capture`）逐字节不变；corpus 双腿扫描仅本形差异。
- [ ] 3.3 负例：三方真不一致合成探针仍拒；`int[][]`（multianewarray）捕获维持现状并如实记录。
- [ ] 3.4 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
- [ ] 3.5 root 独立复核：1.2 插桩定夺、判据零放宽（diff 逐条）、主锚/零回退/负例实测、A 路线的消费面记录；更新 DT-26 账本行（已证差距 → 已修复，指向本片）。（留 root）
