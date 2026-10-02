# 构造实参位 varargs 内联数组走查扩展重放（2026-10-02）

基线 = 主线 `338feb08`（含本批全部切片）的恢复代码；实施 = `recover-varargs-array-ctor-args` 走查扩展。所有恢复文本来自 `jarde-cli class-source --policy single-class --format text`；fixture SHA 复核与 [../results/fixture-sha256.txt](../results/fixture-sha256.txt) 一致（`W1 5813f37a…`、`W3 3549f25a…`、`W4 61011853…`），变体 SHA 见 [../variants-vca/sha256-variants-vca.txt](../variants-vca/sha256-variants-vca.txt)。

## 判据源取证（1.1）

- **既有 varargs 内联数组证明**（裸位判据源）：`crates/jarde-java/src/build.rs` 的 `ArrayInitializers::prove`/`prove_array_initializer`——元素生产判据（每元素窗口 `[cursor+2, store)` 恰为该元素值表达式 `collect_expression_bcis`+`interval_is_expression`）、区间连续封闭（`allocation` 至 `store_pos+1` 的 consumer 逐指令闭合，元素间无空洞）、值单用途（`retained`/index/元素值 `single_use_at_with_budget`）、源指令 `may_throw` 处理器与分配点相等。
- **呈现源**（`new T[]{…}` 拼写）：`render_value` 的 `NewArray` 分支（`initializers` 元素拼写 + `array_initializer_element` 兼容检查）与 `allocation_value` 别名解析；W4 裸位三方法即该呈现的冻结样例。
- **复用接口**：`report.rs` 先证 `ArrayInitializers` 再调 `init::sites`（`ConstructionFacts.arrays` 已携带该 plan）；本切片新增唯一访问器 `ArrayInitializers::inline_argument_chain_bcis`（与 `inline_char_argument_bcis` 同款形态：只陈述链 BCIs 与区间/单用途事实，消费语义由走查闭包判定），走查在构造区间扫描中调用它；空 varargs 的裸分配（无元素存储，证明不覆盖 length==0）由走查按"分配值单用途服务于该实参生产"就地判定。**无第二份判据，无第三份呈现。**
- **W3.viaArrays 基线拒绝**：`jre_new_interleaved_effect`（"the instruction at BCI 5 is an NewArray … between the allocation's copy and its constructor call"），级联整方法退化（`../results/W3.jarde.java` 在 338feb08 重放逐字一致；W1/W3/W4 三 fixture 重放与冻结输出 diff 为空）。

## 走查扩展（2.1/2.2）

- 落点：`crates/jarde-java/src/init.rs` `verify` 的构造区间效果检查——原扫描把 `anewarray` 起始的存储链当交错效果拒绝。扩展：区间扫描遇数组分配时，经 `inline_argument_chain_bcis` 复用既有证明，接受条件为（i）链完整证明（含 `children`/`local_postfix` 为空——多维与 postfix 形态不属构造实参位）；（ii）链的**唯一 consumer 是本构造实参依赖走查到达的 invoke**（`argument_dependencies`；`Arrays.asList` 等工厂）；（iii）链全部源指令位于 `(dup, constructor)` 开区间内（consumer 含于 `sources`）；空数组裸分配同判据。接受集进入区间检查与异常边界检查（新码 `jre_new_inline_array_exception_boundary`，与 concat/inline char[]/nested 同一判据）。
- **边界保持**：`String.<init>([C)V` 构造完全除外（em27 切片独占：`wrongOwner`/`extraReader`/`wrongDescriptor`/`effectful`/`extraReader` 冻结负例逐字不动，既有测试 `only_the_complete_direct_java8_string_char_array_is_embedded` 全绿）；**consumer 为构造调用本身的直接形态不做**（`new Foo(new char[]{…})`/`new Foo(1,2,3)` varargs 构造保持拒绝——em27 `wrongOwner` 冻结的正是该形态，翻转它超出本切片场景）；多维链、链中插语句、双用途、跨块保持拒绝；与嵌套构造分支互斥识别（`NewArray` vs `Allocate` 两判据结构不相交，双向测试：`nested_ctor_argument_sites.rs`/init 单测两族各自全绿）。
- **预算/取消原子性**：走查路径无预算句柄（与既有 concat/char 嵌入判定同构）；`ArrayInitializers::prove` 未改动；`inline_char_array_proof_obeys_shared_budget_and_cancellation` 及全套 stop/cancel 家族绿。

## 变体前后（1.2；`variants-vca/V*.before.jarde.java` vs `V*.after.jarde.java`，均为 `java -Xverify:all` 通过的 javac --release 8 产物）

| 场景 | 主线 | 实施 |
| --- | --- | --- |
| W3.viaArrays `new ArrayList<Integer>(Arrays.asList(1,2,3))` | 整方法退化 | `return new java.util.ArrayList((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.Integer[]{…})).size();` |
| W3.viaArraysEmpty `Arrays.asList(new Integer[0])`（空 varargs） | 退化 | 完整恢复（`new java.lang.Integer[0]` 拼写复用） |
| W1.use（sumExt/addSuper/nameOf 全链 + `6.0:7:W1`） | 退化 | 全方法恢复，零引述 |
| V1.viaEmptyCall `Arrays.asList()`（零长 varargs 调用 → `new String[0]`） | 退化 | 完整恢复 |
| V1.viaBoxedMix（同组件混合装箱：常量自动装箱 + 显式 `valueOf` 混合） | 退化 | 完整恢复 |
| V1.viaHashSet `new HashSet<String>(Arrays.asList("a","b"))` | 退化 | 完整恢复 |
| V1.viaCallElements（元素为调用生产 `valueOf(7)`/`box(8)`） | 退化 | 完整恢复 |
| V3.viaMixed 跨类型 LUB `Arrays.asList(1, 2L, 3.0)`（`Number[]` 组件） | 退化（`jre_new_interleaved_effect`） | **仍拒绝**；诊断翻为 `array@1` 元素兼容性边界（"array component is `java.lang.Number`"）——与裸位同形（`M.bareMixed` 探针同码拒绝）：构造位已接受，剩余缺口是数组元素层的既有边界，非本切片修改面 |
| V2.midStatement（元素窗口内 `putstatic` 语句） | 退化 | **保持拒绝**（`jre_new_interleaved_effect`，消息与主线一致） |
| V2.doubleUse（链尾 `dup; putstatic` 数组逃逸双用途） | 退化 | **保持拒绝**（同上） |

既有位逐字不变：W4 裸位三方法与冻结 `../results/W4.jarde.java` diff 为空；嵌套构造位 X1–X4、varargs 切片 `VarargsCalls`（`tests/fixtures/proved-varargs-calls/v8/`）、collection-widening `G1` 与主线基线二进制重放逐字一致（`varargs_ctor_argument_sites.rs` 逐字钉死 W4 三方法与 X2.nested）。

## 三方对照（3.2；`threeway/`，固定 JADX dev）

原 class / 固定 JADX / Jarde class-source 三条腿各自 `javac --release 8` 重编后 `java -Xverify:all` 运行，stdout SHA 逐腿一致，stderr 全净：

| 类 | stdout SHA（一致腿） | 内容 |
| --- | --- | --- |
| W1 | `d6999843f38f6b09577f8b09a9c2b076b051d7d2c1fa794271a2d9d38bb47b87`（三腿） | `6.0:7:W1` |
| W3 | `50b29863ae3d6f7eb930fef69429ec9c55b157dc429d598350542f81ca5bc070`（三腿） | `3:0:W3` |
| V1 | `b563e60edb39ae080f23cf5808de7837ff618da9e0a094724876ef3abf1c2f52`（三腿） | `0:3:2:2` |
| V2（负例） | `67345218527a2fb92112750fc675f8693763b75b3d0d822186fc257e8fc88bba`（原/JADX 两腿） | `2:2:2:1` |
| V3（边界） | `1121cfccd5913f0a63fec40a6ffd44ea64f9dc135c66634ba001d10bcf4302a2`（原/JADX 两腿） | `3` |

V2/V3 的 Jarde 腿文本含引述不可编译（负例/边界按设计保持拒绝，与嵌套切片 X4 同口径）。

## 门禁（3.1）

- `cargo test --workspace --tests --locked --no-fail-fast`：**2832 通过、0 失败**（283 个测试二进制全绿；中途树态混入变体拆分的最后一次运行后，已在最终树上复跑确认）。
- `cargo fmt --all -- --check`：通过。
- CI 同款 clippy（`ci.yml` 全量 30 项 `-A` 清单 + `-D warnings`，`--workspace --all-targets --all-features --locked`）：零告警。
- `openspec validate --all --strict`：通过（245 项，0 失败）。
