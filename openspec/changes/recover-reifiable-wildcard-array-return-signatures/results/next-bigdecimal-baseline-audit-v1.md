# BigDecimal `Number[]` 边界的下一片入口备忘

## 证据身份

本备忘以本次 [candidate-root-v1/manifest.json](/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-reifiable-wildcard-array-return-signatures/results/candidate-root-v1/manifest.json) 的两条 `bigdecimal-control-corrected` reports 为当前基线。Manifest SHA-256 为 `506aeb4add6aa46bfe8167c39a281864139ad77da23ef8b98afe98a9cafd26c1`，candidate CLI SHA-256 为 `196bb3e1e1bd074bb851f363938951a6d2dea81d8e895cd67e1a2560b3c2f761`。当前输入和报告拷贝均在本次 `candidate-root-v1` 目录；不以旧的缺原流 audit 作为当前结果。

| JDK leg | 本次 input jar SHA-256 | `Main.class` SHA-256 | 原始源 SHA-256 | 原始 raw stdout SHA-256 | 本次 Jarde 完整行为 |
|---|---|---|---|---|---|
| Corretto 8 | `f42437404bb92d5f4680992482828871951b2ce395a61f6f000a06226f79d0dc` | `7c302ba965d6f94528b3be6d634477bb62eb8078408c12a460d1075730571997` | `a03a516b64e5cd356c97e0b4f582cfe905bed3812b6b33f4d5efcecc41971f34` | `2cff3d4e1d402f123ac6042cd50af20e8bbe97d94384b341056fbbe4e80d7981` | render 0；source-set complete；compile 0；`-Xverify:all` run 0，但新类 stdout 空，语义不匹配，`candidate_success=false` |
| OpenJDK 23 | `f71d44a7315890d40ac5b28aafd55ffc739ab76b8590715b260a41bc5e696607` | `e4bf655f328f116957186fd9a244ad0e11cc27f8fa3c6684c3b63c11ab19dc87` | `a03a516b64e5cd356c97e0b4f582cfe905bed3812b6b33f4d5efcecc41971f34` | `2cff3d4e1d402f123ac6042cd50af20e8bbe97d94384b341056fbbe4e80d7981` | render 0；source-set complete；compile 0；`-Xverify:all` run 0，但新类 stdout 空，语义不匹配，`candidate_success=false` |

永久输入 jar 副本分别为 [javac8 jar](/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-reifiable-wildcard-array-return-signatures/results/candidate-root-v1/inputs/20-baseline-bigdecimal-control-corrected-javac8.jar) 与 [javac23 jar](/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-reifiable-wildcard-array-return-signatures/results/candidate-root-v1/inputs/21-baseline-bigdecimal-control-corrected-javac23.jar)。原始 `Main.java` 与原编译 class 的永久归档在 [BigDecimal provenance archive](/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-constructor-primitive-conversion-arguments/results/bigdecimal-historical-provenance-v1/)；两腿源相同：

```java
Number[] values = new Number[]{new BigDecimal("1.25")};
System.out.println(values.length + ":" + values[0]);
```

原程序两腿均 compile/run 0，stdout 是 `1:1.25\n`，stderr 为空。Jarde 本次生成源两腿 SHA 相同，为 `fb6b42365d313bd128a266239eb0214da1dbf747944862e4d5ee19803b2d7d5e`；输出的是 recovery envelope 与注释标记、方法体 `return;`，而非这段运算。候选 compile/run 的 exit 0 因此不能计作行为通过。

## 本次报告与物理 BCI

两条 current report 对 `main([Ljava/lang/String;)V` 给出同一结果：`quality=fallback`、`representation=mixed`、有 bytecode marker。决定性 refusal 是：BCI 16 的数组 initializer 元素呈现为 `java.math.BigDecimal`，component 是 `java.lang.Number`，该 `aastore` 没有可用于 Java initializer 的兼容 reference fact。它不是 BigDecimal `new` 或 constructor composition 失败：`new_records` 两腿都记录 `java/math/BigDecimal` 的 `head=6, dup=9, constructor=12, arguments=[10], presented=true, refusal=null`；`java/lang/StringBuilder` 的 `head=20, dup=23, constructor=24` 也 `presented=true`。

下面 BCI 来自本次永久 jar 中的原始 `Main.class` Code（main Code 长 50 bytes、`max_stack=6`），并与本次 report 的 `new_records`、diagnostics、source-map BCI 对照：

| BCI | 原指令/角色 | 当前证据含义 |
|---:|---|---|
| 1 | `anewarray java/lang/Number` | 创建目标 `Number[]` |
| 4 | `dup` | 保留数组引用供初始化与赋给局部 |
| 6 / 9 / 12 | `new BigDecimal` / `dup` / `<init>(String)V` | 构造器已作为 `new` source 成功呈现；参数字符串来自 BCI 10 |
| 15 | `aastore` | BigDecimal→Number 的 initializer 赋值事实缺失；report 将 refusal anchor 标在 BCI 16 |
| 16 | `astore_1` | 需要提交 `values` 局部的数组值；前一 initializer 拒绝后，这个声明无法被完整恢复 |
| 17 | `getstatic System.out` | 当前 `jre_field_not_emitted` 指出 field identity proof 通过，但最终 body 没有对应 Java field 操作；`jre_field_accesses` 为 0 presented / 1 refused |
| 20 / 23 / 24 | `new StringBuilder` / `dup` / `<init>()V` | concat builder `new` site 自身 presented |
| 28 | `arraylength` | 位于从 BCI 20 起的 `append` concat chain 内部；当前 `jre_concat_interleaved_effect` 明确拒绝跨过它重排 append |
| 29 / 34 / 40 / 43 | `append(int)` / `append(String)` / `append(Object)` / `toString()` | builder 链后续步骤；诊断为 1 candidate、0 presented、1 refused |
| 46 | `invokevirtual PrintStream.println(String)` | 对 `local1` 的 statement 仍报告 `P3 2b.2` 未绑定；这是数组写入被拒后的下游局部读，不是 BigDecimal constructor 拒绝 |

当前两个关键拒绝必须分开跟踪。赋值拒绝来自 reference compatibility：现有 release-8 boxed-number closed table只列 Byte、Short、Integer、Long、Float、Double 到 Number，明确将 BigDecimal 放在表外（[build.rs:29902](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:29902)、[build.rs:33969](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:33969)）。同一赋值门还可消费 facade 从所选 snapshot 建立、精确绑定 store BCI/source/target 的 hierarchy widening proof（[facade.rs:25169](/Users/lordcasser/workspace/projects/jarde/src/facade.rs:25169)、[build.rs:26673](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:26673)）。本次输入只有 Main；报告的 `runtime_resolution` 为 `not_requested`，故这两条报告没有证明所选运行时事实是否能给出 `BigDecimal extends Number` 的精确证明。

另一个拒绝是 concat 顺序：`StringBuilder` append 链跨过 BCI 28 的 `arraylength`，而将整链写作一个表达式会把 append 调用移到 arraylength 另一侧；concat 规则明确拒绝这种 interleaved effect（[concat.rs:35](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/concat.rs:35)）。即使之后补齐 BigDecimal→Number，不能据此宣称完整 `main` 已恢复。`System.out` BCI 17 未进入最终文本及 BCI 46 的 `local1` 未绑定也要在完整方法验收中重新核对；后者是拒绝数组声明的派生症状，不宜另立为该类型事实的根因。

## JADX 对照的完整语义证据

永久 archive 中的两份 JADX 源都是 `defpackage.Main`，SHA-256 同为 `25c4d66035b18c5aefd7339a77cbbd996f00121920ff4bba894f4c5e8243be62`；所编 class 分别是 Corretto 8 `38322df4a1d0721d075ecbcbfe3edd54ef3500793ef5764618ea8c1d4c89ac03` 和 OpenJDK 23 `811d2c9ad07521b96fca39b1de3981af699cc704ac66709be8319572c7131c07`。成功的 FQCN-aware recheck 由 [recheck manifest extract](/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-constructor-primitive-conversion-arguments/results/bigdecimal-historical-provenance-v1/provenance-manifests/jadx-runtime-recheck-v2-bigdecimal-extract.json) 固定；其父 addendum-02 manifest SHA 为 `6276e7c987ade43de10340abad9bb3788f2d6f180d2fa51e499da0bff03fc02c`。两腿用 `-Xverify:all` 运行 `defpackage.Main` 均 exit 0，stdout/stderr 与原始源流哈希完全相同。此前把入口写成未限定 `Main` 的失败运行保留在 archive 中，但不计入这 2/2 完整语义比较。

## 下一片的窄复测清单

1. 固定上述两个 jar/class/source 身份与原始 `1:1.25\n` 双流；先确认所选 runtime/snapshot 能否给出精确 BigDecimal→Number 继承事实，并要求证明绑定 BCI 15 的 `aastore`。缺事实时维持 refusal；不要把它扩展为任意类层次服务或无来源 subtype 表。
2. 若该赋值事实被证明，检查同一个 `new BigDecimal` site、一次 constructor 调用、数组写入与 `values` 局部能否一并恢复；重查数组长度、`System.out` 读和 `println` 的来源及顺序。不能因 element 单点通过而放过剩余 `jre_concat_interleaved_effect` 或 field refusal。
3. 将 concat/arraylength 问题保持为独立边界：若本片只补类型赋值，完整 main 的拒绝仍应如实保留；要认定完整程序成功，需单独证明 BCI 28 的 arraylength 与 append 次数/顺序均未改变。
4. 每个最终声称的成功腿都要求无拒绝/bytecode marker、全源集新建空 classpath/sourcepath 编译、目标 JDK `-Xverify:all` 运行，并逐字节比较原始 stdout/stderr。当前 candidate-root-v1 的编译/run 0 与空 stdout 已展示为何 exit-only 不足。

本备忘只登记现有事实和下次验收入口；没有改产品、测试、fixture、OpenSpec 任务，也没有运行 Cargo、Git、Java 或 Jarde CLI。
