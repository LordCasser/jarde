# 用户类层级实参上转型巡查（2026-10-02）——两闭集切片登记升级路径的首个真实触发

接口/实现域巡查（主线 `c6bebddf`）。固定 [fixture](fixture/)：`I1.java` 源、含全家族的 `fam.jar`（SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）、行为基线 orig.out（`hi:d`/`hello:d`/`static`/`hello:v`）。

## 结果矩阵

| 场景 | 主线 Jarde |
| --- | --- |
| `Greet` 接口本身（default 方法体、static 工厂、匿名实现类 `Greet$1`） | 全部完整恢复 |
| `En implements Greet` 覆写调用（`new En().hello("d")`） | 恢复 |
| **`viaInterface(new En(), "v")`：实参 `I1$En` → 形参 `I1$Greet`** | 拒绝："declared `I1$Greet` presents `I1$En` but … no safe reference conversion evidence"（BCI 39）→ 行为差（`hello:v` 缺失） |
| jar 家族输入 | 同失败——快照含 `I1$En` 字节但无证明通道消费 |

## 根因与定性

`build.rs` 转换分派的既有回答（Object 目标/同名/数组/overload 证明/平台闭集×2）都不覆盖**用户类→用户接口/父类**。这是 `recover-throwable-wrap-arguments`（用户异常类→Throwable）与 `recover-platform-collection-widening`（MyList→List）两处登记的 **"resolution 层证明升级路径" 的首个真实触发案例**：任何用户实现类传给以其接口/父类声明的形参（策略/回调/集合注入……反编译最高频形态之一）。

关键架构事实：**`I1$En.class` 的 header 自带 `implements I1$Greet`**——同快照内两类均为物理类，层级事实可从被分析工件自身的字节读取（extends 链 + interfaces 数组），不需要 classpath 猜测。JVM verifier 保证可赋值，源级安全由同快照 header 链证明。

## 处置方向

`recover-snapshot-hierarchy-widening`（大颗粒 MVP）：同快照双方均物理定义的用户类层级实参上转型——按 BCI 收集 widening 证明（呈现类型 T、要求类型 U 均在本快照有物理类时，沿 T 的 extends/implements 链有界 walk 达 U 即证），分派处消费；`cast_argument` 保留要求类型拼写。单边在快照外（含平台目标如用户异常→Throwable）不在 MVP，保持拒绝并继续登记（下一触发时扩单边）。I1 家族四路径逐字一致；两平台闭集与全部既有负例零回退。

原 class 为行为基准。

## 实现记录（`recover-snapshot-hierarchy-widening`，2026-10-02 收口）

### 通道结构取证（1.1，重放基线）

本 worktree（基线 `a7c4fb4e`）重放固定 `fam.jar`（SHA 与巡查时一致）：恢复文本与巡查冻结的 `results/I1-jar.jarde.java` 逐字一致（mod 头注），`main` 的 BCI 39 仍拒绝——`declared I1$Greet presents I1$En … no safe reference conversion evidence`。通道结构事实：

- **生产 pass**：`src/facade.rs` 的 `prove_reference_overload_calls`（挂点 `recovery_from_with_class_candidates`，`assembly_context.is_some()` 闸内，经 `reference_overload_calls_presented` 吞预算/取消错误）。该通道以 `single_reference_parameter`（仅 `(L…;)…` 单引用参描述符）+ overload 唯一性为门，因此 I1 的 `(LI1$Greet;Ljava/lang/String;)` 双参位从不进入——这正是缺口所在，而非 walk 本身。
- **header 读取面**：`selected_reference_header`（当前类走 `ReferenceClassHeader::from_ir`；其余走 `resolve_class_source_dependency_read_raw`——`resolve_symbol` 后强制 `resolved.definition.snapshot()` 必须在 `content` 内，classpath 定义不可能通过：**读取面天然限定快照自身字节**）。
- **既有 walk**：`proved_reference_widening` 已实现 visited 集合 + `observe_dependency_depth` + 逐类计费的快照 header 链 walk（含 `java/lang/Object` 短路、`name == target` 在 header 成读之后——target 缺于快照即不可达）。

### 落点（2.1）

- **pass**：新增 `prove_snapshot_hierarchy_widenings`（`src/facade.rs`，同闸挂点）：扫 `0xb6/0xb7/0xb8/0xb9` 调用位，按描述符取引用参数（`reference_parameter_sites` 给出该参数在被耗操作数**值序**中的序数——接收者占 0、`J/D` 一值两槽；SSA site 的 Stack 读按深度升序即操作数序，**不用绝对槽位**：`getstatic System.out` 等残余栈前缀会使操作数从槽 1 起）。呈现 T 取 SSA `RefType::Named`（`L…;` 与裸内部名同归一，数组/Unknown 不证）；T==U、目标 `java/lang/Object` 不产证明（分派既有回答先行）。命中产出 `ProvedSnapshotHierarchyWidening{bci, source, target}`（点分拼写）。
- **walk**：`proved_reference_widening` 的链 walk 抽出为共享 `snapshot_header_chain_widens(…, max_depth)`（overload 通道传 `u64::MAX`，行为不变）；本通道传 **8**：第 8 边（L7→…→L0→Sig）可证，第 9 边拒绝（H3 钉死）。
- **分派**：`build.rs` `invocation_argument` 在 `java_lang_throwable_widens` 之后、Err 之前查询该集合并走 `cast_argument`——同名/数组/Object/overload 证明/平台对全部先行，顺序不变。

### 变体与负例（1.2，前后行为）

固定于 [fixture/hier/](fixture/hier/)（javac 23.0.1 `--release 8 -g:none`，SHA 见 [results/hier/fixture-sha256.txt](results/hier/fixture-sha256.txt)）。`H1` 前 7 拒（BCI 12/30/46/64/82/100/123）→ 后 0 拒且 7 位全部 `(H1$Greet/H1$Other) new …` 要求类型拼写；`H3` 前 2 拒 → 后 1 拒（恰第 9 边）；`H2` 前后均 2 拒且文本逐字一致：

| 形状 | 前 | 后 |
| --- | --- | --- |
| V1 两级继承 TwoLevel→Mid→Greet | 拒 | `(H1$Greet) new H1$TwoLevel()` |
| V2 多实现 Multi implements Greet,Other（双向位） | 拒×2 | `(H1$Greet)`/`(H1$Other) new H1$Multi()` |
| V3 匿名类 H1$1 implements Greet | 拒 | `(H1$Greet) new H1$1()` |
| V4 超接口跳 ViaSub implements Sub extends Greet | 拒 | `(H1$Greet) new H1$ViaSub()` |
| V5 前置 String 参后第 1 位 | 拒 | `lead("lead", (H1$Greet) new H1$Multi())` |
| V6 实例调用（接收者占位偏移） | 拒 | `new H1$Caller().call((H1$Greet) new H1$TwoLevel())` |
| V7 深度第 8 边 L7→Sig | 拒 | `(H3$Sig) new H3$L7()` |
| N-A final 传 Object（既有分支） | `(java.lang.Object) new H1$Fin()` | 同前，逐字不变 |
| N-B 快照内无关联（链经 Helper，Helper 缺于快照；`H2$Ext`/`H2$Target` 均在快照） | 拒 | 拒，文本逐字不变（`H2` 原件携 `Helper.class` 于 classpath 照常 `-Xverify:all` 运行） |
| N-C 单边快照外（`H2$MyErr`→`java.lang.Throwable`） | 拒 | 拒，文本逐字不变 |

### 三方对照与门禁（3.1/3.2）

三方运行（原 class / 固定 JADX dev / Jarde 重编，`java -Xverify:all`）逐路径一致，SHA 见 [results/hier/threeway-sha256.md](results/hier/threeway-sha256.md)：I1 四路径 `40a24bdd…`（= `fixture/orig.out`）、H1 八路径 `f4bdad2c…`、H2 `80a211c3…`（原文/JADX）、H3 `842e854d…`（原文/JADX）。门禁：`cargo test --workspace --tests --locked --no-fail-fast` **2798 全绿**（基线 2790+8：集成 6 + facade 单元 2）、`cargo fmt --all -- --check` 通过、clippy 按 `ci.yml` 完整 29 项 `-A` + `-D warnings` 通过、`openspec validate --all --strict` 239 项全过。

### 遗留（继续登记）

单边快照外（T 或 U 不在快照——含用户类→平台目标 Throwable/List、平台中间类断链）不在 MVP，保持拒绝；下一真实案例触发时扩单边通道。本片证据目录 `results/hier/`（`*.before/after.jarde.java`、`jadx-*.java`、`*.out`、SHA 清单）。
