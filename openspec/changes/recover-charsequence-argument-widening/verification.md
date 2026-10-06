# 验证（change `recover-charsequence-argument-widening`，含 root 追记的 Serializable 锚）

与 `recover-comparable-argument-widening`、`recover-enum-argument-widening` **一次实现**（同一表族、
同一判定函数、同一测试入口），本文件记本片自己的锚与行；合并派发的门禁与偏差记在本片末节。

## 实现（`crates/jarde-java/src/build.rs`）

1. **新函数 `platform_interface_argument_widens(java_release, presented, required)`**（落点：紧接
   `java_lang_throwable_widens` 之后，与既有两条通道并列；Throwable 先例是独立小函数，照抄其形）。
   四张 `const` 行表（本片两张 + Comparable 一张 + 枚举族一张），同一次 `java_release == 8` 门与
   逐行命中判定；命中即调用点既有的 `cast_argument(argument, required, bci)` 呈现——**命中函数与
   两条姊妹通道同一函数同一行为，无第三种呈现**。
2. **调用点**：`invocation_argument` 的引用扩宽序列中，`java_lang_throwable_widens` 之后、
   snapshot 单边证明之前加一处 `platform_interface_argument_widens(self.profile.java_release, …)`。
3. **新函数 `platform_array_argument_widens(java_release, presented, required)` + 调用点一处**
   （数组位）。**这是 spec Scenario 1 的必需伴随，不是额外扩边**：实测只落类行时，
   `String.join` 首参通过而第 2 参（`String[]` → `java.lang.CharSequence[]`）仍在 BCI 3 拒绝，
   `join` 仍整方法拒绝；表行只在**数组位置的投影**上落地后锚才恢复。判据只把**两个非数组引用组件**
   交给上面三张表回答（rank 逐级对齐、原始组件不冒充引用组件、步数上限同 JVMS 的 255），
   既有 `array_reference_widens` 与其闭集**逐字未动**（本片不修改该函数、不修改 varargs 单元判据）。

## 行表与 javadoc 出处（逐行核对，转录见证据）

| 行 | 呈现类型 → 目标 | release 8 事实 |
| --- | --- | --- |
| 1 | `java.lang.String` → `java.lang.CharSequence` | header 自声明 |
| 2 | `java.lang.StringBuffer` → `java.lang.CharSequence` | header 自声明 |
| 3 | `java.lang.StringBuilder` → `java.lang.CharSequence` | header 自声明 |
| 4 | `java.nio.CharBuffer` → `java.lang.CharSequence` | header 自声明 |
| 5–13 | `java.lang.String` + 八装箱 → `java.io.Serializable`（root 追记的 Serializable 员） | `String`/`Character`/`Boolean` 自声明；六 `Number` 子类经 `java.lang.Number` |

转录与 rt.jar sha256 见 `openspec/evidence/java-syntax-2026-10-05/widening-row-sources/`。

## 锚实测 vs 预期（两条 javac 腿逐字一致；`essential` + source map 入口）

| 锚 | 预期 | 实测 |
| --- | --- | --- |
| `SB.join`（巡查主锚） | 整方法恢复、整类 `javac --release 8` exit 0、行为一致 | `java.lang.String.join((java.lang.CharSequence) "-", (java.lang.CharSequence[]) arg0);`，整类 refusals=0（改前该成员 3 引注，全类 3 处 `not recovered`） |
| `CS.join`（冻结形） | 同类行 | 与上逐字同形（双腿同一文本） |
| `CS.appender` | `Appendable.append(CharSequence)` 位 | `arg0.append((java.lang.CharSequence) "x");` |
| `CS.joining`/`ST.joined`（第 5 位点） | `Collectors.joining` 恢复 | `…Collectors.joining((java.lang.CharSequence) ",")`；`ST` 整类 refusals=0 |
| `CS.useBoth`/`GE.useBoth`（Serializable 锚） | 多重界调用点恢复 | `both((java.io.Serializable) "a", (java.io.Serializable) "b")`；`GE` 整类 refusals=0（`up()` 的异构数组拒绝逐字保留） |
| `CS.same` | 同型回答零变化 | `return arg0;`（不引入 cast） |
| 负例 `CSX.sealBuilder` | `StringBuilder → Serializable` 仍拒 | 拒绝文本逐字（`presents `java.lang.StringBuilder` … requires `java.io.Serializable` …`） |
| 负例 `CSX.viaSegment` | `Segment → CharSequence` 仍拒 | 拒绝文本逐字 |
| spec 负例（`Integer → CharSequence`、`String → Runnable`） | 仍拒 | 单元级实测（`platform_interface_argument_widening_reaches_exactly_the_table_rows`）；两者**无法由 javac 源产生**（源级非法），故不入 fixture |

## 冻结与行为

- `tests/fixtures/recover-charsequence-argument-widening/`（`CS`/`CSX` 源 + 两腿 class + README 记
  sha256 与腿命令）；
- `tests/recover_platform_implementer_argument_widening.rs`：`the_charsequence_and_serializable_positions_are_presented`
  （6 文本逐字 + 全类零引注）、`the_types_outside_the_closed_rows_still_refuse`（2 负例文本 + 拒绝计数
  恰为 2）——两腿各断言一次；
- ignored replay：剥离注释后由**installed javac `--release 8`** 与**真 javac 8** 各编译一次并
  `-Xverify:all` 运行，答案 `p-q/a,b,c/b/s` 与 fixture 自身 class 逐字一致（实测通过）。

## 零回退

- 既有 java.util 表与 Throwable 通道**逐字未改**（本片不触 `platform_reference_argument_widens`、
  `java_lang_throwable_widens`、`array_reference_widens` 与任何既有表行）；
- 数组闭集、Object 回答、snapshot 层（含新落的 platform-interface 单边）未改；
- `CS.both` 本体（跨界面 cast 健康面）与 `SB`/`ST` 的其余成员逐字未动；
- 诊断文本族零变化（唯一被替换的是本片锚的整方法拒绝）。

## 边界与如实记录

- `javax.swing.text.Segment` 在 release 8 事实上实现 `CharSequence`，本片按 spec 钉的封闭四行省略该行
  （conservative：仍拒），事实与依据记在 `widening-row-sources/`；
- 平台→平台（如 `java.io` 流）与 `java.lang.Runnable`/`Comparator` 等其余接口位不在本片（前者由
  snapshot 单边证明通道覆盖，后者无锚）；
- `SB.main` 的 saved-chain 呈现形（`saved0`…）是既有域，本片不改。

## 移动的既有 pin（如实记录，共 3 处 + 语料账本）

1. **数组位谓词的第一版曾改到既有通道**（自查发现并修正）：第一版 `platform_array_argument_widens`
   把组件对同时交给 `platform_reference_argument_widens` 与 `java_lang_throwable_widens`，于是
   `p3_throwable_wrap_arguments` 的 C2WN N4（`IllegalStateException[]` → `Throwable[]`）由拒变放行
   （该测试的既有断言明写“arrays stay invariant … neither the array closure nor the java.lang
   throwable table may widen them”）。**修正**：数组位谓词只问三张新表，既有两通道的数组答案逐字保持
   ——该测试恢复全绿（未改其断言），并新增 4 条单元级拒绝（`IllegalStateException[]`→`Throwable[]`、
   `ArrayList[]`→`List[]`、`IOException[]`→`Exception[]`、`String[]`→`Runnable[]`）把该边界钉住。
2. `tests/p3_platform_collection_widening.rs`（CWN 的 table-out 计数 7 → 6）：`java.util.EnumSet` →
   `java.util.Set` 一行由**枚举片**的级联伴行移动（该行本就登记为 OB 锚的伴随，见 objects-enumset 巡查
   追加）；断言的七条里一条替换为“该拒绝句已不在文本中”，其余六条逐字保留。
3. `tests/fixtures/corpus-fingerprint.json` 再生（新增 fixture class 文件）与
   `crates/jarde-reader/src/classfile.rs` 的 fixture 人口 `(665, 2838, 282, 1803, 8)` →
   `(687, 2932, 282, 1827, 8)`（+22 类 / +94 body，附同风格注释行）——两个 change 的 fixture 共同移动，
   记在合并派发节。

## 门禁（本机实测）

| 门禁 | 结果 |
| --- | --- |
| `cargo test --workspace --tests --locked --no-fail-fast` | exit 0；308 targets / 3021 passed / 0 failed / 52 ignored（基线 307/3012/51：+1 目标、+9 通过、+1 ignored = 本片两处单测 + 7 锚测试 + 1 ignored replay） |
| `cargo fmt --all -- --check` | exit 0 |
| clippy（`.github/workflows/ci.yml` 46–76 逐字，`-D warnings`） | exit 0，无 warning |
| `openspec validate --all --strict` | 300 passed, 0 failed |
| `git diff --check` | exit 0 |
| 语料指纹 / fixture 人口 | 已再生/重测（见上“移动的既有 pin”第 3 条） |

已知 flake（`p4_plugins`、`p3_short_circuit_transfer_gateway`、`d3_artifact_binding`、
`bulk_recovery_delivery`）本轮全量两次运行均未出现，无需隔离复跑。

## 语料扫描（实测：**非空，但全部是放宽**，如实记录）

方法与判别：语料 = `openspec/evidence/**` 的 **1987** 个 class 文件。先按 **表目标描述符**筛出候选
（descriptor 里出现 `Ljava/lang/CharSequence;`/`Ljava/lang/Comparable;`/`Ljava/io/Serializable;`/
`Ljava/util/{Collection,AbstractSet,Set};`/`Ljava/lang/Iterable;`/`[Ljava/lang/CharSequence;` 的任一）
= **52** 个类；再用**同一入口**（`class-source --policy single-class`，两个二进制：base `370f72a1` 的
`build.rs` 编出的 CLI 与 HEAD 的 CLI）逐类渲染并逐行 diff。

| 类 | 结果 |
| --- | --- |
| `…/java-syntax-2026-10-05/charsequence-arg-widening-patrol/fixture/SB.class` | 1 条拒绝 → 恢复（本片主锚） |
| `…/java-syntax-2026-10-05/recursive-generic-patrol/fixture/RG.class` | 2 条 → 恢复（comparable 锚） |
| `…/java-syntax-2026-10-02/collection-widening-patrol/widen/original/CWN.class` | 1 条（`EnumSet`→`Set`）→ 放行（枚举片 pin，已记录） |
| `…/java-syntax-2026-10-03/nested-generic-header-patrol/scg/results/three-way/SCGA.class` | 1 条 → 恢复（`same_class_generic_binding` 的移动 pin） |
| `…/java-syntax-2026-10-03/nested-generic-header-patrol/fixture/Z1.class` | 2 条 → 恢复（`main` 整段；**巡查之外的额外收益**） |
| `…/java-syntax-2026-10-05/generic-static-field-init-patrol/fixture/RG.class` | 2 条 → 恢复（`consume()` 整段；**额外收益**） |

**全 52 个候选类的 source text diff 只有上述 6 个，且新增行中出现的拒绝句为 0**（9 条拒绝被移除、
0 条新增）——与实现的“只增不减”性质一致（既有通道与表逐字未动）。故 spec 预期“空 diff”**被实测否定**
（语料确有真实 `Comparable`/`EnumSet` 调用点），但方向全部是放宽，无任何回退或新拒绝。

两个额外收益的渲染与入口记在各自巡查目录：
`nested-generic-header-patrol/results/jarde-Z1-after-platform-implementer-tables.txt`、
`generic-static-field-init-patrol/results/jarde-RG-after-platform-implementer-tables.txt`（readme 未改，
路由由文件名与文件头说明）。
