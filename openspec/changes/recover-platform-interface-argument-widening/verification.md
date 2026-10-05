# 验证（change `recover-platform-interface-argument-widening`）

## 实现（1 处，判定顺序）

`src/facade.rs`：`snapshot_header_chain_widens_with` 把"走到目标名"的命中判定移到目标 header
读取之前（旧序 `header(&name)` → `name == target`；新序 `name == target` → `header(&name)`）。
目标名是**上一跳自己的 class-file header** 在 superclass/interfaces 数组里逐字给出的，这就是完整
关系；源侧从 `source` 起仍是逐级读取、逐级证明（visited/深度/预算/`dependency_depth` 记账逐字未动，
`SNAPSHOT_HIERARCHY_WALK_DEPTH` 与其检查位置未动）。文档注释同步改述。不查 classpath、不猜。

## 锚实测 vs 预期（两条 javac 腿逐字一致；`essential` + source map 入口）

| 锚 | 预期 | 实测 |
| --- | --- | --- |
| `CP.byAnon` | 整条 sort 呈现、类文本无转换拒绝 | `java.util.Collections.sort((java.util.List) local1, (java.util.Comparator) new CP$1());`，refusals=0 |
| `AH.byTop`（顶层具名 `AC`，无 `$`） | 同上 | `java.util.Collections.sort((java.util.List) local1, (java.util.Comparator) new AC());`，refusals=0 |
| `IS.byAnon`（隔离形） | `[10, 20, 30]` | 剥离文本呈现整条调用，双腿编译（installed javac `--release 8` + 真 javac 8）运行 `[10, 20, 30]` |
| `AN` own-interface 四位 | 零回退 | `returned`/`asArg`/`<clinit>`/`localVar` 文本与改动前逐字节相同（对照见下） |
| `AW.viaNamedPlatform` | 命名平台接口可证 | `runRunnable((java.lang.Runnable) new AW$Work());` |
| `AW.viaAbsent` | 无 header 关系仍拒 | 原拒文本逐字保留（`presents \`AW$MyErr\` … requires \`java.lang.Throwable\` … no safe reference conversion evidence`），全类 refusals=1 |

隔离判别实测：`CP` 整类运行 `[bo:30, al:40, al:20]/[al:20, bo:30, al:40]/[a, bb, ccc]`，
`AH` `[a, bb, ccc]`，`IS` `[10, 20, 30]`，`AN` `11/42/9/25`，逐字等于各自原类文件的运行输出。

### 改前/改后对拍（同一 fixture 字节、同一入口，逐行 diff）

同一份 `tests/fixtures/.../v8`、`v8-javac8` 字节分别由带补丁与 `git stash` 后的构建渲染，全量
diff 只有三类：

1. 三个位置由 `// jarde: not recovered … (explanation only)` + 拒绝文本 + `jarde_refused_body();`
   变成整条调用（`CP.byAnon`、`IS.byAnon`、`AW.viaNamedPlatform`）；
2. 随之两处类级注记由 `ordinary_generic_source_unproved: method body … no complete source proof`
   变为 `generic_source_shape_unproved: class name has no unambiguous Java source spelling`
   （正文已完整证明后，剩余的是类名拼写债务——`recover-snapshot-hierarchy-widening` 的同形注记）；
3. `AW` 的 fold 投影因新呈现的成员名 token 取得 source-map 锚而重写该 token（`new AW$Work()` →
   `new Work()`）；该 fold 只在请求 source map 类别时成立（既有约定，见
   `tests/member_class_static_folding.rs`）。

`AN` 四位、`CP`/`AH` 其余成员、以及两条腿的其余全部内容逐字节不动。

## 硬不变量证据

- `tests/p3_snapshot_hierarchy_widening.rs` 6/6 通过（fixtures 未触碰、测试未改）：两-sided 扩宽
  （`I1$En→I1$Greet`、`H1` 七形）逐字不动；"第八边证明/第九边拒"照旧；快照内断链（`H2$Ext`→
  `H2$Target` 经 `Helper`）与单边平台目标（`H2$MyErr`→`java.lang.Throwable`）保持原文拒绝；
- 平台闭集表、数组/Object 回答在两处调用点之前照旧（`SNAPSHOT_HIERARCHY_WALK_DEPTH` 与深度检查
  位置未动，`L8` 的第九边仍在边界外）；
- 目标名不出现在任何快照类 header 时保持现行拒绝（`AW.viaAbsent` 实测；p3 的两条负例实测）。

### 移动的既有 pin（1 处，如实记录）

`openspec/evidence/java-syntax-2026-10-05/recover-write-accessor-field-types/pff/pff-baseline-PrivateFieldFamily$B.txt`：
该记录取自**只含 `$B` 的 standalone 快照**，而 `set(ZZ)V` 把子类 `this` 传给父类
`access$002(dt29/PrivateFieldFamily$A, Z)`——旧序要求"目标 header 在快照内"，于是整条方法被
reference-conversion 拒绝吞掉；新序下 `$B` 自己的 header 逐字写着 `extends
dt29/PrivateFieldFamily$A`，关系成立。新记录把 3 行拒绝换成

```text
        dt29.PrivateFieldFamily$A.access$002((dt29.PrivateFieldFamily$A) this, arg2);
        return;
```

（严格更好的呈现：方法不再整段 explanation-only）。同一 commit 重钉该记录并在测试文档补记原因；
同族另外两条记录（`PrivateFieldFamily`、`PrivateFieldFamily$A`）逐字节不动。`cp.jar`、`f6.jar`、
`fam.jar` 等全部 frozen jars 与源 fixture 未改。

## 门禁（本机实测，`RUST_TEST_THREADS=1 CARGO_BUILD_JOBS=1`）

| 门禁 | 结果 |
| --- | --- |
| `cargo test --workspace --tests --locked --no-fail-fast` | exit 0；307 targets / 3012 passed / 0 failed / 51 ignored（基线 306/3007/50 → +1 目标、+5 通过、+1 ignored，即本片 5 非 ignored + 1 ignored 测试） |
| `cargo test --workspace --all-targets --locked --no-fail-fast` | exit 0；314 targets / 3012 passed / 0 failed / 51 ignored（与任务书基线 313/3007/50 同形，差量同上） |
| `cargo fmt --all -- --check` | 干净（本片改动已 `cargo fmt --all`） |
| clippy（`.github/workflows/ci.yml` 46–76 逐字） | exit 0，无 warning（`-D warnings`） |
| `openspec validate --all --strict` | 300 passed, 0 failed |

已知 flake（`p4_plugins` elapsed_millis、`p3_short_circuit_transfer_gateway` scratch-dir、
`d3_artifact_binding` elapsed_millis）本轮两次全量运行均未出现，无需隔离复跑。

本轮新增的 `tests/recover_platform_interface_argument_widening.rs` 因新增 fixture 触发三处既有
"语料账本"期望，均已按各自约定更新：`tests/fixtures/corpus-fingerprint.json` 由
`cargo test --test p5_corpus_fingerprint -- --ignored regenerate_corpus_fingerprint` 重生成
（+40 文件）；`crates/jarde-reader/src/classfile.rs` 的 fixture 人口计数 `(631,2738,282,1803,8)`
→ `(665,2838,282,1803,8)`（+34 类 / +100 body，附同风格注释行）。

## 观测（非 pin）

`AD`（comparator-anon 巡查的同源 fixture，本地 `javac --release 8` 编译）整类 refusals=0：
`new java.lang.Thread((java.lang.Runnable) new AD$2(local1, arg0))` 与随后的 `start()`/`join()`
一并恢复——巡查记录的 viaThread 可恢复性缺口关闭。证据
`openspec/evidence/java-syntax-2026-10-05/comparator-anon-patrol/results/jarde-AD-after-platform-interface-widening.txt`
（该 fixture 只有 `.java`，文件头记明编译命令）。`AD` 不冻结为断言（其可编译性受孤立渲染的
checked-exception 形态影响，不在本片判据内）。

## 未做 / 边界

- 平台→平台（`java.io` 流 ctor）不在本片；
- `AD`/`IS` 的其它同族位（lambda 链、泛型 own 方法）只是随之自然呈现，未新增断言；
- 成员 fold 的"须有 source map 类别"是既有约定，本片未改。

## 证据文件

- `tests/fixtures/recover-platform-interface-argument-widening/`（源 + 两腿 class + README 含全部
  SHA-256 与 javap 事实）；
- `tests/recover_platform_interface_argument_widening.rs`（5 逐字钉 + 1 ignored 双 javac replay）；
- `openspec/evidence/java-syntax-2026-10-05/comparator-anon-patrol/results/jarde-CP-after-platform-interface-widening.txt`、
  `…/jarde-AD-after-platform-interface-widening.txt`；
  `openspec/evidence/java-syntax-2026-10-05/platform-interface-widening-patrol/results/jarde-AH-after-platform-interface-widening.txt`
  （各自入口与 refusals 计数在文件头）。
