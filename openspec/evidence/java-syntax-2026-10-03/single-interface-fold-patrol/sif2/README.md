# 接口方法引用 owner 入锚定匹配器 —— 实现证据（change `recover-single-static-interface-fold`）

基线 worktree HEAD `2998fe58`（主线，`recover-fold-context-projection-preservation` 已合入）。本目录是
**实现片**的取证（`sif/` 是证伪期取证，保持原样不改）。三条腿的二进制：`/tmp/sif2/jarde-cli-before`
（基线源树构建）、`/tmp/sif2/jarde-cli-after`（本变更）；两者同一 `cargo build` 配置，唯一差异是
`src/facade.rs` 的一处判据位。

## 1. 判据位落点与判据复核（任务 1.1）

**判据位**：`src/facade.rs::project_static_fold_owner_texts` 内、token 锚定门的**直接覆盖段匹配器**，
即 `for (start, end, target, source_bcis) in edits` 循环中构造 `matching` 的那处 `match &entry.kind`
（改动前为 `FieldRef | MethodRef` 的 owner 命中；改动后加 `| InterfaceMethodRef`）。同一函数内另有
两处**不同的**池面——`static_fold_stored_type_anchor`（生产者锚，fcp 片引入）与
`static_fold_pool_entry_names_type`（生产者侧 CP 匹配）——本片**未改动**；`prove_static_family_target`
的接口形状要求也未改动。

**证伪期最小补丁判据复核**：`sif/README.md` §5 描述的最小补丁是「`| InterfaceMethodRef { owner, .. }`
**加上** `InvokeDynamic/MethodType/NameAndType` 的 descriptor 提及」。复核结论：**后半句不属于本片**。
`InvokeDynamic`/`Dynamic` 的返回描述符分量**已经在主线的生产者锚**
（`static_fold_pool_entry_names_type`，`af37db70`，fcp 片）里落地，且 `static_fold_stored_type_anchor`
正是消费它的首个调用点；再在直接匹配器里加 descriptor 提及会**放宽**「覆盖段含 CP 索引」的健全性
要求（把「指令的 CP 条目命名该类」变成「指令的 CP 条目提及该类」），与 design 决策 1「健全性校验
不动」冲突。故本片**只加 owner 位置**，`sif/README.md` §5 的补丁后半句不采纳——这是 4→5 个 corpus
类差别的根因（见 §5）。

**CP 种类枚举位复核**（`SIFDBG` 临时探针，`results/apply_probe.py`，已回退）：探针打印每个 token 的
覆盖 bcis、其指令的池条目种类、以及最终所走的锚路（direct/handler/producer/refused），逐类核对见
`results/anchor-probe.txt`：

| 类 | token | 覆盖段 CP 条目 | before | after |
| --- | --- | --- | --- | --- |
| `WCallI1` | `WCallI1$M` | `InterfaceMethodRef[WCallI1$M#sv]`（bci 3 `invokestatic`） | refused | direct @ bci 3 |
| `WCallC`（sif fixture） | `WCallC$M` | `MethodRef[WCallC$M#sv]`（bci 3 `invokestatic`） | direct @ bci 3 | direct @ bci 3 |
| `F1` | `F1$B` | `InterfaceMethodRef[F1$B#name]`（bci 15） | refused | direct @ bci 15 |
| `F1` | `F1$A` | `InterfaceMethodRef[F1$A#name]`（bci 8） | — | direct @ bci 8 |
| `SDAbstract` | `SDAbstract$SDB` | `InterfaceMethodRef[SDAbstract$SDB#name]`（bci 1） | refused | direct @ bci 1 |
| `SDIndirect` | `SDIndirect$SDA` | `InterfaceMethodRef[SDIndirect$SDA#name]`（bci 1） | refused | direct @ bci 1 |
| `SDDiamond` | `SDDiamond$B` | `InterfaceMethodRef[SDDiamond$B#name]`（bci 15） | refused | direct @ bci 15 |
| `pkg.SDPacked` | `pkg.SDPacked$I` | `InterfaceMethodRef[pkg/SDPacked$I#name]`（bci 1） | refused | direct @ bci 1 |
| `H3` | `H3$Sig` | `Class[H3$L7]` / `MethodRef[H3$L7#<init>]` / `MethodRef[H3#via]` / `MethodRef[java/io/PrintStream#println]` —— **无一条命名 `H3$Sig`** | refused | refused |
| `NAnchor`（本片负例） | `NAnchor$M` | 覆盖段仅 `bci=1 op=0x4c`（`astore`，**无 CP 索引**） | refused | refused |

`F1$Diamond` 一类的既有 direct/handler/producer 命中在两条腿逐字一致，确认新分支不改变它们的选择。
`H3` 是「覆盖段含 CP 索引但无一条命名目标」的既有拒绝形：其覆盖段的 `Class`/`MethodRef` 命名
`H3$L7`/`H3#via`，从不命名 `H3$Sig`，故 owner 扩展**不触碰**它——本片没有引入新的误锚面。

## 2. 实现（任务 2.1/2.2）

单判据位，`src/facade.rs`：

```rust
let names_target = match &entry.kind {
    CpEntryKind::Class { name, .. } => name.0 == target.binary,
    CpEntryKind::FieldRef { owner, .. }
    | CpEntryKind::MethodRef { owner, .. }
    | CpEntryKind::InterfaceMethodRef { owner, .. } => owner.0 == target.binary,
    _ => false,
};
```

- 与既有 Fieldref/Methodref owner **同权**（同一 `match` 臂、同一等式）；健全性要求（覆盖段必须含
  带 CP 索引的命名指令，或异常处理器类型，或生产者链）不动。
- 生产者锚（`static_fold_stored_type_anchor`）与其 CP 匹配（`static_fold_pool_entry_names_type`）
  未改动；`prove_static_family_target` 未改动；预算/取消路径未改动。
- 文档同步：两处 doc-comment/块注释更新为该匹配器实际读的种类（`Class` 名与三种 owner 位置；
  descriptor 分量只属于更宽的生产者匹配器）。
- 回归测试：`tests/member_class_static_folding.rs::interface_call_owner_anchors_the_fold_and_an_unnamed_segment_still_refuses`
  （接口调用形折叠 + 覆盖段无 CP 索引负例保持拒绝）。**可证伪性**：把 `src/facade.rs` 回退到基线、
  只保留该测试，测试 FAILED（`tests/member_class_static_folding.rs:487`）。

## 3. 变体/负例（任务 1.2；`fixtures/`，折叠文本 `fold-outputs/`，SHA `results/fold-output-sha256.txt`）

| 用例 | 形态 | before 折叠 | after 折叠 | before/after 折叠文本 SHA |
| --- | --- | --- | --- | --- |
| `WCallI1` | 单静态接口子 + `invokestatic` 指向 `InterfaceMethodRef` | 否（`prepared`/`refused`） | **是**（1 声明） | `957c4575…` → `fd20a74a…` |
| `WCMulti` | 多接口方法调用族：两个接口各一次静态调用 + 一个接口方法经 `held` 实例调用 | 否 | **是**（3 声明） | `4c38722c…` → `a988ce92…` |
| `WCSibling` | 两个 sibling 接口静态子，同一体内两个不同 `InterfaceMethodRef` owner | 否 | **是**（2 声明） | `10a7b427…` → `066d02d7…` |
| `WCDefault` | 接口静态 + default 成员同族 | 否 | **是**（2 声明） | `95e2cbaa…` → `2185ce37…` |
| `NAnchor` | **负例**：成员类型经「本运行未生产的局部值」引入 → 覆盖段 `astore` 无 CP 索引 | 否（`refused`） | **否**（`refused` 逐字同） | `7949049f…` = `7949049f…` |
| `NLocal` | **负例**：单直接静态接口子，但类级 no-capture 关系未证（局部 null 测试） | 否（`refused`） | **否**（`refused` 逐字同） | `d40f30ac…` = `d40f30ac…` |

`NAnchor`/`NLocal` 的 before/after 方差为零（`diff` 空），即负例保持拒绝是**逐字节**证据，不只是
状态字符串。`WCMulti`/`WCSibling` 是同一 owner 位置在一体内命中**两个**不同接口目标的多命中形。

## 4. 三方对照（任务 3.2；`threeway/`，脚本 `results/verify_behavior.sh`）

三条腿：原 class（`java -Xverify:all`）、固定 JADX dev（`~/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx`，
dev）、Jarde 折叠单元重编（`javac --release 8` 以原 jar 作 classpath，再 `java -Xverify:all`）。

| 用例 | 原 class | JADX Java-input | Jarde 折叠重编 | 输出逐字 |
| --- | --- | --- | --- | --- |
| `WCallI1` | `8` | 可编（`javac` 0 错误），运行不可用/输出不符 | `javac` 0 错误，`-Xverify:all` 运行 `8` | **一致**（两腿 stdout SHA `aa67a169…`） |
| `WCMulti` | `17`/`4` | 可编，运行不可用/输出不符 | 0 错误，运行 `17`/`4` | **一致**（`1cbc6a87…`） |
| `WCSibling` | `17` | 可编，运行不可用/输出不符 | 0 错误，运行 `17` | **一致**（`54183f43…`） |
| `WCDefault` | `11` | 可编，运行不可用/输出不符 | 0 错误，运行 `11` | **一致**（`25d4f2a8…`） |
| `NAnchor` | `9` | 可编，运行不可用/输出不符 | **不适用**（负例不折叠） | — |
| `NLocal` | `n` | 可编，运行不可用/输出不符 | **不适用**（负例不折叠） | — |
| `F1`（corpus 类） | `AB`/`I:A` | 不可编 | 0 错误，运行 `AB`/`I:A` | **一致**（`63c3efd5…`） |
| `SDAbstract` | 1 行 | 不可编 | 0 错误，行为一致 | **一致**（`f706a3fb…`） |
| `SDIndirect` | 1 行 | 不可编 | 0 错误，行为一致 | **一致**（`4844e901…`） |
| `SDDiamond` | 4 行 | 不可编 | 0 错误，行为一致 | **一致**（`395b38ad…`） |
| `pkg.SDPacked` | `PI` | 不可编 | 0 错误，行为一致 | **一致**（`68120a46…`） |

JADX 一栏的口径与 `handoff.md`、`sif/README.md` §6、`fcp/README.md` §6 一致：**JADX 的文本相似度或
可编性都不是本变更的验收依据**。本片 6 个新 fixture 上 JADX 文本可编但运行的输出与原始 class 不同
（`jadx-stdout!=original-stdout`），故其行为腿不可用；`sif`/`fcp` 的 `Y1`/`F1` 形上 JADX 文本根本不可
编。两条腿的**不可用原因**都记录在各自 `threeway/*/legs.txt`。

## 5. corpus 双腿扫描与 4+8 归类（任务 3.1）

**jar corpus**（`results/scan_corpus.py`）：`tests/fixtures` + `fuzz/corpus` + `openspec/evidence` 下
**106 个 jar / 391 个类**（本片自证目录 `sif2/` 不在扫描集内，与 `ngh/` 的既有惯例一致），逐类
`class-source --format text` 的 stdout SHA256，退出码无变化（5 项 EXIT 两腿一致）。

**变化 6 项 = 5 个类**（`results/corpus-diff.txt`）：

| # | 类 | before SHA | after SHA | 归类 |
| --- | --- | --- | --- | --- |
| 1 | `super-default-patrol/fixture/fam.jar:F1` | `b015b938…` | `558fcb9f…` | 新增折叠（接口调用形） |
| 2 | `…/sd/sd-negatives-orig.jar:SDAbstract` | `4193df67…` | `24caaf14…` | 新增折叠（接口调用形） |
| 3 | `…/sd/sd-negatives-orig.jar:SDIndirect` | `5aa8bf57…` | `12f6c6a1…` | 新增折叠（接口调用形） |
| 4 | `…/sd/sd-variants.jar:pkg.SDPacked` | `dd46462c…` | `183c2c12…` | 新增折叠（接口调用形） |
| 5 | `…/sd/sd-negatives-orig.jar:SDDiamond` | `8f15c13f…` | `14b2266c…` | 新增折叠（接口调用形） |
| 6 | `…/sd/sd-variants.jar:SDDiamond` | `8f15c13f…` | `14b2266c…` | 同类在第二个 jar 的副本 |

**与证伪期 `corpus-interface-anchor-G2.txt` 的 4 类差异（本片的真实发现）**：G2 补丁（§1 的
「owner + descriptor 提及」版）变 4 类，本片变 5 类，多出的正是 `SDDiamond`。根因不是本片多放了
什么，而是**基线已经换了**：G2 取证在 `63640c67` 一带，而基线 `2998fe58` 已含 fcp 片的
**生产者锚**（`static_fold_stored_type_anchor`）。`SDDiamond` 的折叠同时需要两枚 token——`SDDiamond$Use`
（由生产者锚处置，`bci 7 astore` → 生产者 `bci 4 MethodRef[SDDiamond$Use#<init>]`）与
`SDDiamond$B`（由本片 owner 扩展处置）。**实测**：把 `static_fold_stored_type_anchor` 强制返回
`None`（实验，已回退）后，本变更对 `SDDiamond` 仍为 `refused`，而 `SDAbstract`/`SDIndirect`/
`pkg.SDPacked` 仍为 `projected`——即 `SDDiamond` 是 **owner 扩展 ∧ fcp 生产者锚**的合取，两个已
合入的切片各管一半。这也说明：本变更的 5 类与 fcp 的 8 类不重叠于同一类（fcp 的 8 类见
`../fcp/results/corpus-fcp-changed-classes.txt`，无一是 `super-default-patrol` 的类）。

**既有家族逐字不变**：`M1`/`M1$*`/`M1User`/`M2`/`M2$Solo`/`FV1…FV5`（member-class-folding-patrol）
两腿 SHA 全同；折叠+mixed+instance（`inner-class-folding-patrol`）、lambda 内联
（`lambda-inline-patrol`）、nested-spelling（`nested-name-spelling-patrol`）、fcp 的 8 类全部零回退。

**独立 `.class` corpus**（`results/scan_classes.sh`）：`tests/fixtures`+`fuzz/corpus`+`openspec/evidence`
下 **2163 个独立 class 文件**（`class_source_file` example，两腿同一源树的 example 二进制），
`results/classfiles-diff.txt` **为空**——本变更对独立 class 输入零文本变化（这是预期的：折叠只在
同类 jar 内已解析到子定义时发生；独立 `.class` 无兄弟定义）。

## 6. 门禁（任务 3.1）

- `cargo test --workspace --tests --locked --no-fail-fast`：**2903 passed / 0 failed / 46 ignored**
  （293 个 suite；日志 `results/tests.log`）。基线同命令 2902（本片新增 1 项回归测试）。
- `cargo fmt --all -- --check`：通过。
- clippy，CI `.github/workflows/ci.yml` 实有 **29 项 `-A` 清单**
  （`--workspace --all-targets --all-features --locked -- -D warnings`）：零警告
  （`results/clippy.log`）。
- `openspec validate --all --strict`：**261 passed, 0 failed**（基线同命令同为 261：本片未增删
  spec/change 条目）。
- 磁盘纪律：每轮构建/测试前后 `df -h /`（本片 44Gi → 26Gi 区间，未破 12Gi 下限）；报告前已
  `cargo clean`（见 §8）。

## 7. 遗留与边界

- **`H3` 保持拒绝**（登记，非缺陷）：覆盖段含 CP 索引但无一条命名 `H3$Sig`（覆盖段命名 `H3$L7` /
  `H3#via` / `println`）。它需要的不是 owner 扩展而是该 token 的**健全**锚（现五类
  `Field/ClassDefinition/MethodPoint/MethodSignature/ConstructorParameter` 之外的新机制），属另一片。
- **`sif` fixture `Y1` 的折叠不由本片达成**：其阻塞在 fcp 已处置的根重投影门与 `viaLambda` 局部
  声明 token（后者由生产者锚处置）——本片不变。`sif/README.md` §9 的 Y1 遗留按 fcp 片的记录口径。
- **JADX 行为腿**在本片 6 个新 fixture 上不可用（文本可编、输出不符），见 §4；本片不据此主张
  任何「追平 JADX」的结论。
- 待 root 复核：tasks.md 3.3 保持未勾选。

## 8. 产物

- `fixtures/`：`WCallI1`/`WCMulti`/`WCSibling`/`WCDefault`（正例）+ `NAnchor`/`NLocal`（负例）源码。
- `fold-outputs/`：6 用例 before/after 的 `class-source` 文本。
- `results/`：`scan_corpus.py`（jar corpus 双腿）、`scan_classes.sh`（独立 class 双腿）、
  `verify_behavior.sh`（三方对照）、`apply_probe.py`（SIFDBG 探针，已回退）、`corpus-{before,after}.txt`
  （391 项）、`corpus-diff.txt`、`classfiles-before.txt`（2163 项）、`classfiles-diff.txt`（空）、
  `anchor-probe.txt`、`fold-output-sha256.txt`、`tests.log`、`clippy.log`。
- `threeway/`：每用例的 `legs.txt`（各腿退出码/输出 SHA/判定）、`*.jarde.java` + SHA。