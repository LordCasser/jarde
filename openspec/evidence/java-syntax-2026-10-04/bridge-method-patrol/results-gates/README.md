# 准入门扩展取证（`recover-bridge-admission-gates`，2026-10-04）

基线：worktree HEAD `28eb5755`（源码与主线 e93d3635 相同，其差为 docs 提交）。两份 CLI：
`cli-sha256.txt`（baseline = 主线源码原样构建；changed = 本 change 构建）。输入：`../fixture/fam.jar`
（五类 SHA 见 `../results/fixture-sha256.txt`，本目录不重复；`BR.class` 与多层级变体 SHA 见
`tests/fixtures/p3-bridge-projection/br-family/v8/fixture-sha256.txt`、`br2-sha256.txt`）。

## 1. 两门取证与扩展落点

- **门 1（擦除返回门）** `src/facade.rs` `prove_class_source_bridges`：原判据
  `bridge_return == target_return || bridge_return != b"Ljava/lang/Object;" || !(L/[)`
  把桥擦除返回钉死为 `Object`。扩展为三支：参数 cast 形允许**等返回**；`Object` 桥保持
  既有 `L`/`[` 规则逐字不变；其余走 `snapshot_header_chain_widens_with`（共享核，同
  `recover-snapshot-hierarchy-widening`：深度上界 8、`observe_dependency_depth`、
  AnalysisSteps 计费、visited 防环），桥准入侧 header 来自当前类 facts + 同快照依赖读。
  链不可达/单边快照外保持现拒绝文本逐字不变；walk Err 走既有 fail-closed（清空 proofs 返回，
  新停止文本）。
- **门 2（体形门）** `crates/jarde-java/src/bridge.rs` `forward_shape`：原在 invoke 前一概拒绝
  CheckCast。扩展为规范参数 cast 形：cast 逐参数**按位序**、各读恰一个桥参数加载值（序号严格
  递增）、目标恰等于**被转发调用自身描述符**的对应参数类型（`Type::Reference` 位相比对）；
  cast 不消除（不进 `Plan::owns`，可抛 CCE 语义保留）。void 桥（无返回值转发）一并接受
  （javac `set(Object)` 桥即此形）；void 后跟 cast 仍拒（decoder 层面该体即非法栈）。
  候选侧新增 `ClassSourceBridgeCandidate::parameter_casts`（适配器，非序列化证据），
  facade 对 `bridge_parameters != target_parameters` 的候选逐位复核（分量相等或 cast 恰等），
  不符保持现拒绝文本 "the bridge and invoked method do not share a name and parameter descriptor"。

## 2. 三项既有门扩展（root 决策 A，逐条负例钉死）与两项追补（决策 A'）

取证发现两门之外另有三道门把家族挡死（javap 实证），root 批准按最小可重建性判据扩展：

1. **MethodParameters**（facade 属性门）：允许 Code 之外恰一个 `MethodParameters`，每 entry
   `name_index==0`、flags==0x1000、count==桥参数数；payload 从类原始字节按 JVMS 4.7.24 定宽
   解码。负例（tests/class_source.rs `br_family_negative_shapes_keep_their_refusals`）：named
   entry、entry flags≠0x1000、count 不符 → 均为 "the bridge declares method metadata whose
   source copying is unproved"；两个该属性 → reader 级 `classfile_duplicate_attribute`
   （成员运行拒绝，无 proof 提供）。
2. **桥 flags 子集**：恰等 0x1041 → ⊆{public,bridge,synthetic} 且 bridge|synthetic 置位
   （恰 {0x1040, 0x1041}）。负例：加 final 位、缺 synthetic 位 → 保持
   "the physical bridge has modifiers beyond public bridge synthetic that source reconstruction
   does not prove"。
3. **源级可见度相对判据**：ACC_PUBLIC 强要求 → 仅桥带 public 才要求源 public
   （桥可见度 ≤ 源可见度）；private/static/abstract/native/synthetic/bridge 源照旧拒。
   负例：桥 public + 源 patch 成包私有 → "the unique target is not a spellable, concrete public
   source method"。
4. **（A'，追补批准）继承解析的 caller 身份**：`enclosing: None` → `Some(桥成员)`——包私有父契约
   的 access 规则（JVMS 5.4.4）由此才可应用；`reloaded_current` 只豁免 `ReadReason::MemberOwner`
   （access 规则的 caller 身份读取），`ParentChain`/`HierarchyClosure` 折返照旧拒。负例
   （`the_parameter_cast_admission_walks_the_snapshot_chain_and_respects_its_edges`）：patch Box
   的 super → BR$StrBox，get 桥被拒
   "the existing resolver traversed back into this prepared class; admission is refused"，
   同类 set 桥（Box 仍声明其契约）照常准入；public 父走 ACC_PUBLIC 快路径零读取
   （BridgeProbe 正例与单类负例字节级不变，既有断言守门）。
5. **（B，追补批准）Comparable 平台事实**：仅一条三元事实——owner==`java/lang/Comparable` ∧
   descriptor==`(Ljava/lang/Object;)I` ∧ InvokeInterface，守卫与
   `proved_reference_widening` 的 ArrayList/List 平台契约完全同款
   （java_release==8、ParentFirst、ClassPath、external_override/runtime_transformation 均
   None）；环境**已提供** Comparable 定义时走该定义不短路。负例：提供改名后（comparetO）的
   定义 → "a direct parent or interface needed for the erased method is unresolved"；
   接口边 patch 成 `java/lang/Object` → 同拒（三元事实不覆盖）。

## 3. 基线拒绝 → 修后投影（`baseline-refusals-*.txt`、`projected-*.java` + `.sha256`）

| 类 | 桥 | 基线拒绝（baseline CLI） | 修后 |
| --- | --- | --- | --- |
| BR$Base | `next()LBR$Node;` | 门 1：the source return type is not a proved covariant subtype of the erased Object return | admitted+projected（marker 落盘） |
| BR$Impl | `compareTo(Ljava/lang/Object;)I` | bridge@1 did not prove a pure single forward | admitted+projected |
| BR$StrBox | `set(Ljava/lang/Object;)V` | flags 门：modifiers beyond public bridge synthetic | admitted+projected |
| BR$StrBox | `get()Ljava/lang/Object;` | flags 门（同上） | admitted+projected |

## 4. javac / java 前后对照

- `javac-before-name-clash.stderr.txt`：基线恢复源 javac `--release 8` 失败——
  `已在类 BR$Base中定义了方法 next()`、`已在类 BR$StrBox中定义了方法 get()`（与巡查存档
  `../results/` 的报错一致）。
- 修后 Node/Box/Base/StrBox 四类源 + `PairRunner` javac exit 0；
  `run-after.trace.txt` = `BR$Base`/`s`（runner 经接口引用调用 `n.next()` 走再生桥）；
  `run-original.trace.txt` = 原 class `BR.main` 输出 `Base`/`s`/`0`。两 trace 的两处差异均为
  已知解释项：runner 用 `getName()`（恢复源为顶级二进制名，`getSimpleName` 依赖内部类元数据）；
  `0` 行属 Impl 路径（见 §6）。`br_family_recovered_source_recompiles_and_runs_like_the_original`
  以 same-runner 双腿断言 `BR$Base`/`s` 逐字一致。
- 多层级协变（接口→父接口）：`BR2`（Base2 implements Mid extends Node2）正例，
  `the_multilevel_covariant_bridge_walks_two_interface_edges` 双腿 javac+`-Xverify:all` 一致，
  walk 深度 2。

## 5. corpus 双腿扫描（差异应仅桥家族形态）

- 单类腿：tests/fixtures 全部 465 个 `.class`（含本次新增 11 个），baseline vs changed CLI，
  single-class policy，class-source 文本 SHA 对比——**零差异**。
- 家族腿：fam.jar 六类 plain-jar——BR / BR$Node / BR$Box 逐字相同；BR$Base / BR$StrBox /
  BR$Impl 差异**仅为**桥成员声明（6 行）被替换为 `// jarde: projected bridge …` 标记行，
  无其它行变化。

## 6. Impl 整类重编缺口（root 决策 B，另立专项）

桥门全部通过（Impl 桥已投影、源级 `compareTo(BR$Impl)` 保留），但恢复源
`class BR$Impl implements java.lang.Comparable { public int compareTo(BR$Impl) }` javac 报
"未覆盖 Comparable 中的抽象方法 compareTo(Object)"——类级参数化**接口**投影缺口：泛型头机制在
`src/class_source.rs`（`type_parameters.is_empty() && !parameterized_superclass → Ok(None)`）
对只有参数化接口的类静默跳过（预留拒绝文案 "parameterized or nested parent needs a separate
inherited-member proof"，既有 interfaces 拼写代码相邻）。`javac-after-raw-Comparable-loud-failure
.stderr.txt` 钉死当前响亮失败形态。该缺口由 root 另立专项（编号待其落 spec 后回填），不并入
本片。隐藏桥本身在任何情况下不劣于基线：基线是 name clash（响亮失败），现在是更接近真相的
响亮失败。

## 7. 验证命令与结果（2026-10-04）

- `cargo test --workspace --tests --locked --no-fail-fast`：见 §8（最终轮全绿）。
- `cargo fmt --all -- --check`：通过（修过 bridge.rs / p3_patterns.rs 两处格式后）。
- `cargo clippy --workspace --all-targets --all-features --locked --` + CI 29 项 `-A` 清单
  （`.github/workflows/ci.yml`）`-D warnings`：通过（`chunks_exact_to_as_chunks` 不在清单，
  按其建议改写为步长索引）。
- `openspec validate --all --strict`：见 tasks.md 3.1。
- 新增测试：`tests/class_source.rs` 4 个（家族准入、家族 e2e、多层级、负例+A/B 钉死）；
  `crates/jarde-java/tests/p3_patterns.rs` 门 2 形测试 5 个（规范形、void 形、组合形、
  错型、非参数值）；`crates/jarde-java/src/bridge.rs` 候选/预算单元不变。

## 8. 复现命令

```
cargo test -p jarde --test class_source --locked
cargo test -p jarde-java --test p3_patterns --locked
cargo test -p jarde-reader --lib classfile::tests::repository_class_fixtures --locked
```
