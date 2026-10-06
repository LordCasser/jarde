# 任务 1.1 / 1.3 取证：锚点重验、基线态与两处设计前提更正

工作树 `subagent-01a10fff`，起点 `bd7142f7`（主线 `recover-temporal-argument-widening` 验收提交）。
**所有行号按锚点名在本树上重新 `grep -n` 定位**（tasks.md 要求），结果如下（`src/class_source.rs` /
`src/facade.rs`，均为本树实测行号）：

| 锚点名 | 本树行号 | 内容 |
| --- | --- | --- |
| `project_generic_signature` | `class_source.rs:6334` | 类头投影唯一入口，闭包参数 `prove_direct_parent` |
| 总门 | `class_source.rs:6372`（原） | `if parsed.type_parameters.is_empty() && !parameterized_superclass { return Ok(None) }` —— 不看 interfaces |
| `direct_parent_candidate` | `class_source.rs:6367`（原） | 单 segment、实参恰为 `java/lang/String` 的父类形 |
| `direct_parent_candidate` 分支接口互斥 | `class_source.rs:6434-6441`（原） | `if !parsed.interfaces.is_empty() \|\| !physical_interfaces.is_empty() → Err` |
| `parent.binary_name.contains(&b'$')` | `class_source.rs:6444`（原） | 父类 `$` 拒绝（MVP 放宽点 1） |
| else 分支接口无实参要求 | `class_source.rs:6455-6461`（原） | 全部接口须单 segment 且无实参，否则 `Err` |
| 接口拼写循环 | `class_source.rs:6516-6519`（原） | 对 `parsed.interfaces` 逐项 `spell_ordinary_signature_type`，交 `class_declaration_with_types`（6526） |
| `prove_direct_generic_superclass_parent` | `facade.rs:13736` | 父类证明器（唯一调用点：`prepare_physical_class_source` 内闭包） |
| `parent_name.contains(&b'$')` | `facade.rs:13761` | 父类 `$` 拒绝（MVP 放宽点 2） |
| `ACC_INTERFACE → false` | `facade.rs:13765`（原） | 该证明器显式拒接口 ⇒ 接口腿不可复用 |
| 闭包注入点 | `facade.rs:6423-6439` | `prepare_physical_class_source` 内唯一注入方式 |
| `prove_class_signature_erasure` | `crates/jarde-reader/src/signature.rs:263` | 已接受 `interfaces: &[Vec<u8>]`；315–340 逐项比对 `class_internal_name(interface)` 与物理项、329 行先比 count |

**reader 侧无需扩展**（1.1 取证义务）：`ClassSignatureErasureProof.interfaces`（signature.rs:225）
已填充，`prove_class_signature_erasure` 已覆盖 interfaces 的 count 与逐项擦除；本片未改
`crates/jarde-reader`（`git diff --stat` 可核）。

## 基线态（实现前，`/tmp/pih-bin/jarde-cli-base`，sha256 `37027c690fe634149a7346749f3b0b0f7a89e21b06f1ff44acc5bf922d0bb247`）

完整逐格转录见 `results/04-baseline/before.txt`（脚本 `render-cells.sh`，自检：文本渲染必须带
`// jarde: presentation of` 头、不存在的类名不得被当作源码应答）。要点：

- `BR$Impl`：类头 `class BR$Impl extends java.lang.Object implements java.lang.Comparable`（裸），
  桥 `compareTo(Ljava/lang/Object;)I` = `admitted=false projected=false`，拒绝文本为接口边前置
  （`the erased contract comes from a generic interface the class header spells without its type arguments…`）。
- `BR$StrBox` / `Spec`：类头池形裸（`extends BR$Box` / `extends Outer$Box`）+ `class Signature projection refused:
  direct superclass does not resolve to one proved single-parameter parent definition`；`set(Ljava/lang/Object;)V`
  桥 `admitted=false projected=false`（父类边前置），协变 `get()Ljava/lang/Object;` 桥已 `admitted=true projected=true`。

## 设计前提更正 1：拼写侧**并非**"无需扩展"（实测证伪 design 的取证义务 (a)）

design 写"`spell_ordinary_signature_type_with_member_path` 已处理类型实参……`TypeArgument::Exact(Class(Impl))`
经同一函数拼为 `BR$Impl`"，并据此判定"拼写侧确认可直接复用，无需扩展"。**本树实测证伪**：
该函数单 segment 叶子走 `simple_generic_class_name`（`class_source.rs:5061` 起），其首行判据是

```rust
if text.contains('$') || text.split('/').any(|part| !is_java_identifier(part)) {
    return Err(Error::unsupported("generic_source_shape_unproved",
        "class name has no unambiguous Java source spelling"));
}
```

即**任何含 `$` 的类名在成员/注解拼写通道一律拒绝**（同一判据已被
`openspec/evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures/boundaries.md:19` 钉为成员域的既有边界）。
后果：父类腿的 `extends BR$Box<String>`（父名含 `$`）与接口腿的 `implements java.lang.Comparable<BR$Impl>`
（实参含 `$`）**都无法经既有函数拼出**——放宽两处 `$` 拒绝后，类头投影会在拼写步以
`generic_source_shape_unproved` 拒绝（实现中途实测：`br-strbox` / `parent-spec` 两格正是此拒绝）。
故本片新增**类头专用拼写规则**（`ClassNameSpelling::BinaryPoolName` + `binary_pool_class_name`）：
类头位置的单 segment 名字按类文件自己的 binary 名拼（`/`→`.`，`$` 保留），与**裸头本来就用的**
`class_name`（`class_source.rs:1306`）逐字一致——即"投影只加类型实参、不改任何名字"，不重拼、不推断嵌套。
非类头位置逐字不动（`SelectedSourcePath` 仍是原判据）。pin 见
`src/class_source.rs` 单元测试 `class_header_spelling_keeps_pool_names_the_member_spelling_refuses`。

## 设计前提更正 2：选定环境**不携带 JRE image**，平台接口定义不可解析

design 的接口证明器判据是"接口定义在选定环境中可解析"。实测：jarde 的三种 policy
（`single_class` / `plain_jar` / explicit classpath）都只声明调用方提供的内容，
`facade.rs:31124-31146` 的桥准入注释逐字写明"this explicit environment need not carry a JRE image"，
并因此为 `java.lang.Comparable.compareTo(Object)` 单列一条**平台事实**（release 8 + ParentFirst +
ClassPath + 无 uncertainty）。因此 `BR$Impl implements java.lang.Comparable<…>` 在 jar-only 环境下
按"定义可解析"判据只能得 `Unresolved` → 裸头 → 验收锚（BR$Impl 类头呈现参数化）**无法达成**。

处置（与既有平台事实同规，非新机制）：新增
`java8_class_path_runtime()`（把桥准入那条平台事实的四个 runtime 条件提为共享 helper，两处调用，
判据逐字保留）+ `platform_header_interface_fact()`：仅当类名恰为 `java/lang/Comparable`、
`Signature` 实参恰为 1、runtime 形态为 Java 8 class-path、且环境**干净地**不提供该定义
（`state == Missing` ∧ 无 environment problems ∧ 无 unresolved deps ∧ 无 candidates，与桥准入同一
`Missing` 判据）时，按 Java SE 8 `public interface java.lang.Comparable<T>`（一个类型参数）判定 `Proved`。
证据（本机 rt.jar，Corretto 1.8.0_432）见 `results/02-q2-replay/javap-comparable.txt`。
**其余任何接口仍严格走"定义可解析 + arity 相符"**；jar 内自带的接口（如 `BridgeProbe`/`BridgeApi`）
由定义本身判定，不受该事实影响。`single_class` policy 下不解析、不收费（与桥准入"SingleClass 视为
unresolved"同姿），故单类渲染的既有态逐字不变。
