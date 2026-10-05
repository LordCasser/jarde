# 实现与验证记录（2026-10-06，coder）

本文件记录 `recover-array-element-field-receiver` 的实现落点、锚实测与门禁结果；全部判定来自实测命令与
本工作树代码（§3 的语料扫为改动前后两枚二进制实测）。注入选案与预审计复核见 [instrumentation.md](instrumentation.md)。

## 1. 代码落点

| 文件 | 位置 | 内容 |
|---|---|---|
| `crates/jarde-java/src/build.rs` | 新 `pub(crate) fn element_receiver_type(ssa, operations, value) -> Option<String>`（`element_of_dimension` 之后、`array_of_value` 同一读法区） | `array_of_value(value, 0)` 取该值自己的数组形状（`ArrayElementLoad` 已降一维），`array_spelling` 拼 Java 类型（= `written_type` 声明局部时的同一读法），仅 `Type::Reference` 转内部名；不可证即 `None`。发布给 `crate::field`，因为数组读法属 `crate::build` |
| `crates/jarde-java/src/field.rs` | `plan`/`verify` 新参数 `element_receiver_type: &dyn Fn(ValueId) -> Option<String>`；`verify` 实例分支 | `let receiver_type = stated_type(ssa, receiver).or_else(|| element_receiver_type(receiver));`——帧命名优先，帧不命名时才问数组通道；其余判定（`UninitializedThis`、owner-cast、拒形文本）一字未动 |
| `crates/jarde-java/src/field.rs` | 模块文档 + `plan`/`verify` 文档 | 规则文本同步：接收者类型=帧命名；帧不命名的那个值（`aaload` 结果）由数组操作数自身形状命名；两者经同一比较；来源不可证则不声明 |
| `crates/jarde-java/src/report.rs` | `field::plan` 调用点（全仓唯一生产调用点） | 传入闭包 `|value| build::element_receiver_type(ssa, &operations, value)` |
| `crates/jarde-reader/src/classfile.rs` | `repository_class_fixtures_validate_without_false_target_rejections` | fixture 人口新计数 `(631, 2738, 282, 1803, 8)`（+40 class / +123 body / +28 branch target，两腿十个 fixture 家族），按既有惯例加一行来源注释 |
| `tests/fixtures/corpus-fingerprint.json` | 语料指纹 | 按该测试自身指示重录（`--ignored regenerate_corpus_fingerprint`）：新增 10 个 `.java` 源 + 40 个 `.class`（+250 行，无删改） |
| `tests/fixtures/recover-array-element-field-receiver/` | 新 fixture（源 + `v8/` + `v8-javac8/` + README） | 巡查原件形状 + `RJ`/`EM`/`RP` 三个本片控制；双腿冻结字节与 SHA-256 见 README |
| `tests/recover_array_element_field_receiver.rs` | 新对照测试目标 | 10 个非 ignored（逐字钉成员文本 + BCI 锚 + 拒形内容）+ 3 个 ignored（剥离-编译-运行重放） |

## 2. 锚实测（命令与输出）

`CLI = target/debug/jarde-cli`；协议：`class-source --format text`（文本走 stdout，plan 面走 stderr）→ 剥离
`^[[:space:]]*//` → 伴生 `X$Y` 拼写改写为嵌套简名 `Y` → 伴生自身呈现按巡查 `verify-wrong-RG.java` 的
做法并入（jarde 只在成员类型投影需要该名时自带嵌套声明，`RO` 自带、其余需并入）→ `javac` → `java
-Xverify:all`。

| # | 锚 | 输入/命令 | 结果（实测） |
|---|---|---|---|
| 1 | **RG 主锚（本片目标）** | `v8/RG.class`（+`RG$Item`），两腿 | `<clinit>` 呈现 `for (RG$Item local4 : local1) { RG.BY_LABEL.put((java.lang.Object) local4.label, (java.lang.Object) local4); }`；类文本零 `@bytecode`；BCI 86/91/94/96 是恢复语句的来源锚 |
| 2 | **改动前的错码读数** | 改动前二进制（`git stash` 后重建）渲染同一 fixture，同法剥离编译运行 | 编译 **exit 0**、运行 **`null/null/null`** —— 查找表静默为空（compilable-wrong，判别成立） |
| 3 | RG 剥离文本（两腿各自渲染） | `javac --release 8`（javac 23.0.1）与真 javac 8 各编译一次 | 两腿均 **exit 0**；运行 `of("beta")` ☓非 null、`of("alpha")` ☓非 null、`of("?")` = `null`（与原 class 同形；`Item` 无 `toString`，身份哈希随编译不同，故以"是否找到成员"为判据） |
| 4 | RH（`total`=5，两腿） | 剥离文本编译运行 | `5/xy`（`total`=2+3、`first`="xy"），与原 class 逐字相同 |
| 5 | RK（元素→局部 / 元素直接） | 同上 | `q/q`，与原 class 逐字相同 |
| 6 | RL（当前类数组 / 伴生类数组） | 同上 | `s/i`，与原 class 逐字相同 |
| 7 | EM enum 数据点 | 文本钉 `EM.<clinit>` 的 `EM.BY_LABEL.put((java.lang.Object) local3.label, (java.lang.Object) local3);`（两腿仅常量池形状不同：`$VALUES = $values();` vs `new EM[]{…}`） | 循环体恢复；剥离文本仍不可编译——常量渲染为 `public static final EM A;`（enum 常量池形债，本片范围外；测试以"必须保持不可编译"记录该债） |
| 8 | RM 零回退（元素方法调用） | 成员文本逐字比对改动前后 | `loopCall`/`elemCall` **逐字节相同**；剥离文本运行 `5/2`，与原 class 相同 |
| 9 | RJ 零回退（直接参数 / 局部别名） | 同上 | `direct`/`viaLocal` **逐字节相同** |
| 10 | RO 零回退（调用结果接收者） | 同上 | `viaCall`/`viaCallLocal` **逐字节相同**；运行 `m/m` |
| 11 | RP 不可证仍拒 | `RP.merged`（条件表达式两数组）、`RP.branchy`（两分支合并） | 两形均保持原拒形且成员文本逐字节不变；诊断面实测：`jre_field_shape` = "the receiver of the field access at BCI 13/14 has the type this run does not state…"（合并值的数组形状未证，不猜组件类型） |
| 12 | RN 写侧（自然修复，如实报告） | `writeElem`/`writeLoop` | 读/写共用同一接收者比较，故写侧同样恢复：`arg0[0].tag = arg1;`、`local5.tag = arg1;`；剥离文本运行 `X/2`，与原 class 相同。本片**未**放宽任何写侧判据（无新增分支） |
| 13 | 其余四夹具文本级 diff（两腿） | 改动前后渲染对照（剔除 `elapsed_millis`） | RJ/RM/RO/RP **零变化**；RG/RH/RK/RL/RN/EM 的变化**只**是引注块 → 恢复语句（逐块核对） |

上表 1、4–11 的文本读数由 10 个非 ignored 测试固化；表 2–10、12 的编译-运行读数由 3 个 ignored 重放
测试（`cargo test --test recover_array_element_field_receiver --locked -- --ignored`）固化。**判别实测**：
把当时的三文件补丁 `git stash` 后重建，同一测试目标 6 个恢复锚测试 FAILED、4 个对照测试 ok，ignored 重放
在 `of("beta")` = `"null"` 处 FAILED，即新检查确能捕获旧行为。

## 3. 全量 fixture 语料回归扫（`tests/fixtures/**` 全部 631 个 `.class`）

协议：改动前代码（`git checkout HEAD~3 -- crates/` 后重建二进制）与改动后二进制各渲染一遍
（`class-source --policy single-class`，只比较 **stdout** 的呈现文本与退出码——plan 面走 stderr，含
`elapsed_millis`，按本仓库既有做法不计入比较）。

| 量 | 结果 |
|---|---|
| 退出码 | **631/631 相同**（零新拒、零新失败） |
| 呈现文本 | **619/631 逐字节相同** |
| 12 处不同的定性 | 全部（12/12）是本片**新增**的 fixtures：`RG`/`RH`/`RK`/`RL`/`RN`/`EM` × 两腿 = 12；**本片 fixture 之外零变化**——本片新增的四个对照（`RJ`/`RM`/`RO`/`RP`）也在 619 之内，逐字节相同 |

## 4. 门禁

| 门禁 | 命令 | 结果 |
|---|---|---|
| 全量测试 | `cargo test --workspace --tests --locked --no-fail-fast` | **306 个测试二进制 / 3007 passed / 0 failed / 50 ignored**（改动前实测同命令基线：本片新文件移开 + 补丁 `git stash` 后重建，**305 / 2997 / 0 / 47**；差 = 本片新增 1 个测试目标 + 10 个测试 + 3 个 ignored。`p4_plugins`、`p3_short_circuit_transfer_gateway` 本次实测均全绿，未触发已知 flake） |
| 格式 | `cargo fmt --all -- --check` | exit 0 |
| clippy | `.github/workflows/ci.yml` 46–76 逐字（`cargo clippy --workspace --all-targets --all-features --locked -- <30 项 A + -D warnings>`） | exit 0（无告警） |
| OpenSpec | `openspec validate --all --strict` | **300 passed, 0 failed**（含本 change） |
| 新测试目标 | `cargo test --test recover_array_element_field_receiver --locked` | 10 passed / 0 failed / 3 ignored；`-- --ignored` 3 passed / 0 failed |

## 5. 提交（不 push）

| # | 提交 | 内容 |
|---|---|---|
| 1 | `feat(field): prove an element read's receiver type from its array` | `build.rs` 的 `element_receiver_type` + `field.rs` 注入点与文档 + `report.rs` 调用点 |
| 2 | `test(receiver): freeze the array-element receiver anchors on both javac legs` | fixtures（源 + 双腿字节 + README）、`tests/recover_array_element_field_receiver.rs`、reader fixture 人口计数、corpus 指纹重录 |
| 3 | `docs(change): record the injection ruling, anchors and gates` | `openspec/changes/recover-array-element-field-receiver/` 的 instrumentation/verification/tasks |

## 6. 与预审计/巡查的偏差（如实记录）

1. **写侧随同一比较自然恢复（RN）**：proposal 的硬不变量写"写侧不放宽"。本片**没有**新增或放宽任何写侧
   判据——`verify` 的接收者比较对读/写是同一次（`owner_cast` 那条也在同一处），所以元素字段写
   （`xs[0].tag = v`、for-each `x.tag = v`）随读侧一起被证明。巡查记录写侧原为"整方法拒 / 吞空后缺
   return"，故这不是新增可编译错码面；行为锚实测 `X/2` 与原 class 一致（§2 行 12）。若 root 认为写侧应
   保持拒绝，需要的是显式按 `FieldAccess` 分叉的**新判据**，不是本次注入的修正。
2. **fixture 入口（`main`）与巡查原件有差异**：巡查把 `new Item[]{new Item(…)}` 直接写在实参位，那是内联
   数组初始化的分配元素通道（本片范围外的现存拒绝，两腿同样拒），会让成员带 `jarde_refused_body` 标记而
   不可编译。故本片 fixtures 的 `main` 把数组取到局部、元素经静态字段读入；**被本片改变/对照的方法本身
   逐字取自巡查原件**，打印值是巡查的值（`5`、`q/q`、`s/i`、`X/2`）。逐条差异写在 fixtures README。
3. **`RJ`/`EM`/`RP` 三个 fixture 是本片新写的**：巡查矩阵把 `direct(Item)`/`viaLocal`、enum 查找表
   `EM.Code` 与"不可证数组"列为判别行，但未提交对应 `.java`/`.class`。本片按矩阵描述补写并冻结（`EM` 取
   `values()` + `BY_LABEL` 形，读数为 `B/A/null` 的同一构造；`RP` 是本片的负面对照，无巡查原件）。
4. **EM 剥离文本仍不可编译**：enum 常量渲染为 `public static final EM A;`（常量池形债），巡查已记录
   "不另计锚"。本片只在文本面锚定 `EM.<clinit>` 的循环体，并以 ignored 测试记录"该文本必须保持不可编译"。
5. **语料扫的既有面变化为 0**：与 preserve-monitor-exit-evaluation-order 不同（那一轮有 1 处既有 fixture
   同形变化），本片 631 个 fixture class 中变更的 12 个全部是本片新增文件（§3）。
