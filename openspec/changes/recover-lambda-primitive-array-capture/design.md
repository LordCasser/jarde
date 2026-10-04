# Design：lambda 捕获原生数组的类型来源

## 决策依据（root 已实测/读码）

1. **判别变量**（P02 探针内同方法级对照）：`StringBuilder` 捕获内联 ✓ / `int[]` 捕获拒 ✗。
2. **根因**：`frame.rs:2299-2315`——`newarray` 压 `Ref(RefType::Unknown)`（注释：bootstrap loader 未声明故不锚名）；`anewarray` 用 `pool_class_name` 得有名类型。帧层**有意**只锚池可命名类型。
3. **拒绝点**：`lambda.rs:765-779` 三方一致判据 `frame_type != site_type || site_type != implementation_type`——帧 Unknown（拼作 `Object`）≠ 站点/实现的 `int[]`。判据本身正确（防静默捕获转换），错的是喂给它的帧类型对原生数组天然无名。

## 决策 1：方向 A vs B（实现片 task 1.1 插桩后定夺，不得预设）

**方向 A（帧层给名）**：`newarray` 按 `atype` 码（4..=11 → `[Z [C [F [D [B [S [I [J`）压**有名**描述符类型。前置问题（插桩/读码回答）：`RefType` 是否已有"描述符形"可承载 `[I` 而不破坏其池锚定语义？`RefType::Unknown` 的其它消费者（引用一致性、`checkcast` 合并、phi 合并）是否依赖"newarray 结果无名"这一事实（若有代码以 Unknown 分支做保守处理，给名会改变它们的行为——**须逐一排查 `RefType::Unknown` 的消费面**）。
**方向 B（捕获门用陈述类型）**：捕获操作数若为局部装载，且该局部的 LVT 陈述类型为 `[I`，则以陈述类型作 `frame_type` 参与判据。前置问题：`capture_types`（`lambda.rs:705`）的 `ty` 现从何处来（名字表？SSA 值？）——插桩确认它在 P02 上为何是 `Object`（帧 Unknown 传导 or 名字表本身无此局部的类型陈述）。若名字表已有 `[I` 而传导丢了，B 是窄修；若名字表也只有 Object，B 需要新增"LVT 陈述类型"读取。
**root 倾向 B**（不动帧层的保守语义、影响面小、与 `nested_member_reference_spelling` 的"陈述事实"先例同构），**但以插桩为准**——A/B 的取舍标准写进 task 1.1：若 `RefType::Unknown` 消费面排查显示无依赖（A 安全）且 B 的类型陈述不可得，则 A；否则 B。

## 决策 2：三方一致判据逐字保留

`lambda.rs:765-779` 的判据结构与拒绝文本不改。改变的只是 `frame_type` 的**取值来源**对原生数组不再天然 Unknown。真不一致（站点 `Object` 实现 `String` 之类的合成探针）仍拒——负例冻结。

## 决策 3：双腿 + 零回退

P02 双腿（真 javac 8 / javac 23）fixture 冻结；`StringBuilder` 捕获（现正常）与 DT-26 既有验收锚（`dt26-lambda-capture` 的 fixture）逐字节零回退；`map` 方法渲染不变。

## 验证标准（可证伪）

1. **主锚**：`P02_lambda.sum`（`int[] t={0}; l.forEach(i -> t[0]+=i)`）恢复为源级 lambda 捕获（与 `map` 同构的内联形），`sum` 源码区 0 引注（现 3 条：BCIs 15/8/9）；渲染源集双腿 `javac --release 8` exit 0，运行 `6` 与原 class 一致。
2. **零回退**：`P02_lambda.map`（StringBuilder 捕获）与 DT-26 既有 fixture 逐字节不变；全 corpus 双腿扫描仅 lambda-原生数组捕获形差异，出现其它差异停下报告。
3. **负例**：站点/实现/帧三方真不一致的合成探针仍拒（判据未放宽）；`multianewarray` 捕获（`int[][]`）**维持现状**（拒绝或恢复皆如实记录，不在本片范围）。
4. **门禁**：全量测试（基线以合并态为准）、fmt、CI-exact clippy、openspec strict、`git diff --check`、新 fixture 后再生 fingerprint。
5. **若走方向 A**：`RefType::Unknown` 消费面逐一排查的记录存证据目录（哪些消费者曾依赖 Unknown、给名后各自行为核对）。

## Open Questions

1. `capture_types` 的 `ty` 在 P02 上的实际来源与传导链（task 1.1 插桩）。
2. `RefType` 能否承载描述符形 `[I`（方向 A 的类型格问题）。
3. LVT 在 P02 class（javac 默认 `-g:lines,source` 不发 LVT！）是否可用——**注意**：javac 默认**不**生成 LocalVariableTable，若 P02 无 LVT，方向 B 的"陈述类型"必须来自别处（如 newarray 的 atype 码本身——那其实就回到 A 的信息源）。这是 A/B 取舍的**关键未知**，task 1.1 必须先回答。
