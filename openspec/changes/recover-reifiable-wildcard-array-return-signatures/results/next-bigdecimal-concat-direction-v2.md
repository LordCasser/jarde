# BigDecimal / Number 两个拒绝的职责拆分（基于 selected-headers 六腿）

## 判断

下一步应先闭合 `BigDecimal` 元素写入 `Number[]` 的精确赋值事实；当前证据不支持把 concat 优化拒绝当作完整方法未恢复，也不支持先改 concat whitelist。

`bigdecimal-selected-headers-v1/manifest.json` 的六条诊断腿显示：两条原始输入（Corretto 8 与 OpenJDK 23 编译的 Main，均未带 selected header）还是 `quality=fallback`；另外四条在输入中显式选入真实 Corretto 8 `BigDecimal.class`（其中两条还额外选入 `Number.class`）后均为 `quality=structured`、`representation=java`，正文完整呈现数组赋值、`System.out.println` 与原有 StringBuilder 调用链。四条报告仍带 `jre_concat_interleaved_effect`，且每条 `jre_concat_chains` 仍为 0 presented / 1 refused。也就是说，本次正文成功与否随 `aastore` 的赋值证据变化，而不随 concat 优化成功变化。

这些是显式提供 header 后的 render 诊断，不是普通 Main-only jar 已被修复。Manifest purpose 明确为 diagnostic render、无 compile/runtime/whole-program acceptance；正文 `syntax_status=unchecked`，并保留“presentation is not claimed to compile”标记。root 另行进行的生成源码编译/运行必须独立报告，不能倒灌成这六条诊断自身的验收。

## 六腿中的可核事实

报告身份见 [manifest.json](/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-reifiable-wildcard-array-return-signatures/results/bigdecimal-selected-headers-v1/manifest.json)，SHA-256 为 `d5c1fd65a29c0c48670c1915fc1a0b9a83d81afa7d5dc493564566c64476d3e5`。CLI SHA 为 `196bb3e1e1bd074bb851f363938951a6d2dea81d8e895cd67e1a2560b3c2f761`；附加 header 来自 `/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/jre/lib/rt.jar`，rt.jar SHA-256 为 `b27515a608ee447566b688e2bbb2257b1f0d8eceb96c307b87eb28d90a6630f4`。

两条 `original-only` 腿的 Main class hash 仍分别是 Corretto 8 `7c302ba9…571997`、OpenJDK 23 `e4bf655f…19dc87`，原输入 jar hash 与 v1 baseline 相同。两条都因 BCI 15 `aastore` 缺少 BigDecimal→Number compatible reference fact 而拒绝初始化器，后续 BCI 46 `local1` 无法绑定是该失败的派生结果。对应 new-site 记录仍显示 BigDecimal `new/dup/<init>` BCI `6/9/12` 与 StringBuilder `20/23/24` 均已 presented；所以这里不是构造表达式自身失败。

另外四腿都从同一个 Corretto 8 `BigDecimal.class` header（SHA-256 `3a2a2aad71f1f3c888b9c550e27a40be9cf41a80e4f72506700848ec8dc31c08`）得到相同正文：

```java
java.lang.Number[] local1 = new java.lang.Number[]{new java.math.BigDecimal("1.25")};
java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(local1.length).append(":").append((java.lang.Object) local1[0]).toString());
```

`BigDecimal-and-Number` 还传入 `Number.class`（SHA-256 `fcd410a44ef259d973ebc682f4d86d30940560081fb3b168eb2cd3ede692ad0b`），但 source text 与 quality 和对应 `BigDecimal-only` 腿完全相同。现有 header walk 在读 `BigDecimal` 后，若其 header 的 superclass/interface 名字直接命中目标 `Number`，便按“命名到 target”完成关系证明；不要求再读取目标 header。因此这份证据支持 BigDecimal header 单独足以提供此关系，额外 Number header 没有改变呈现结果。

两类失败要继续分开：

- `aastore` 类型门：无 header 时 initializer element 是 `java.math.BigDecimal`，component 是 `java.lang.Number`，正文 fallback；显式 selected header 后正文完整。该事实应精确绑定到 BCI 15 和 source/target 类型对。
- concat 优化门：无论上述两种质量状态，BCI 28 `arraylength` 仍使候选 concat 以 `jre_concat_interleaved_effect` 拒绝。但它只是链式简化没被采用；正文可照字节码保留普通调用链，不等于必然丢失方法。

## 普通构造与调用链为什么可以恢复

Builder 将 concat chain 作为一种可选表达式替换：`render_value` 遇到 `invoke` 时，只有成功计划中存在 `chains.value_at(bci)` 才调用 `concat_expr`；否则继续 ordinary `invoke_expr` / `call_expr` 路径（[build.rs](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:24989)）。普通调用渲染器按物理 operands 递归渲染 receiver 与参数，再按方法 descriptor 检查参数（[build.rs](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:25406)、[build.rs](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:25577)）。因此 concat chain verifier 拒绝时，调用仍可以逐层写为 `new StringBuilder().append(...).append(...).toString()`，而无需移动 BCI 28 的 arraylength。

构造表达式也有独立普通路径：`render_value` 由 `init::Site` 将 allocation value 送到 `new_expr`（[build.rs](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:24663)）；`new_expr` 依据准确 site constructor 与 SSA operands 渲染 `new BigDecimal("1.25")`（[build.rs](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:26252)）。数组 initializer 则在每个配对 store BCI 调用 `array_initializer_element`；该门只接受 exact/Object/null、已有闭集关系或同一 snapshot 对精确 store/source/target 的 widening proof（[build.rs](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:26668)）。六腿的差异正落在这个赋值门，而普通 new/call site 在两类结果里均保留。

更关键的是，facade 已有从真实 `aastore` 收集 SSA reference source/target，并把 snapshot hierarchy chain proof 产出为带 `bci/source/target` 的事实的路径（[facade.rs](/Users/lordcasser/workspace/projects/jarde/src/facade.rs:25175)）。hierarchy walk 只展开所选 snapshot 中的类定义，下一层父类/interface 名称命中目标即可；未读到的分支结束，不查其他 classpath（[facade.rs](/Users/lordcasser/workspace/projects/jarde/src/facade.rs:24203)）。这和六腿所用的 BigDecimal header 是吻合的。closed release-8 helper 当前只收六个 boxed primitive wrapper 到 Number，不包含 BigDecimal（[build.rs](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:29393)）。因此下一改动宜只补这个确切事实入口：优先验证现有 snapshot proof 在正常请求的实际供应条件；若产品策略还需要无 selected-header 时的 release-8 platform closed fact，则只考虑 `java.math.BigDecimal -> java.lang.Number` 这一对，不泛化成任意类继承表。

## 建议的最小下一片与边界

先只处理赋值事实，不改 `concat.rs`。让普通 Main-only request 在既有规则下仍可诚实拒绝；在显式 runtime snapshot 或经批准的 Java 8 platform closed fact 提供证据时，只让精确 BCI 15 的 BigDecimal→Number initializer assignment 通过。保留当前构造/consumer SSA identity、唯一 reader、array ownership、handler/interval 和原子提交证明。然后单独核对两 JDK 生成正文：无 bytecode marker，正文完整，来源覆盖 BCI 1/4、6/9/12/15/16、17、20/23/24、27–46；concat refusal 可以仍作为 warning 存在，但不能导致完整 statement fallback。

此次 selected-header 正文会保留显式 StringBuilder 方法调用。不要把 `jre_concat_interleaved_effect` 作为恢复阻塞，也不要为了去掉这个 warning 改动其 `ArrayLength` whitelist。只有后续产品目标明确要求 `+` 形式/concat warning 清零时，才单独推进 v1 已记录的 `ArrayLength` concat 方向，并为该优化单独验收 append 顺序、NPE 求值位置、source map 与 interleaved statement 负例。

当前 evidence 仍没有证明生成正文编译或运行；root 后续的全空 CP/sourcepath compile 与目标 JDK `-Xverify:all` 双流比较是下一道独立验证。本文只读 manifest、Jarde 静态调用路径与现有 facade/helper；没有改产品、测试、fixture 或 OpenSpec 任务，没有运行 Cargo、Git、Java 或 Jarde CLI。
