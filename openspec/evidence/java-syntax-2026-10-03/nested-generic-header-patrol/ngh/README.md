# 嵌套泛型类头投影实现证据（recover-nested-generic-class-headers，2026-10-03）

主线 `a7762a92`（worktree 实现）。固定 fixture SHA 与实现前后行为见 [results/](results/)；
本文件记录取证结论、判据复用、前后对照与遗留边界。

## 1. 未证位分解（`class_generic_source_unproved` 产生点取证）

产生点在消费层 `src/class_source.rs` 的 `ClassSourceDeclaration::project_generic_signature`
（非 `crates/jarde-reader/src/signature.rs`——reader 只提供解析与擦除证明）。原拒绝条件为一个合并位：

```
access_flags & (ACC_ANNOTATION | ACC_ENUM) != 0      // kind 位：注解/枚举头
|| !is_java_identifier(&self.name)                    // name 位：拼写
|| self.name.contains('$')                            // nesting 位：`$` 一律未证（本片链头）
|| Runtime{Visible,Invisible}TypeAnnotations 存在      // type-use 注解位
```

- **顶层路径已证的位**：name（无 `$` 的 Java 标识符）、kind（非注解/枚举，接口超类须为 Object）、
  物理父身份与擦aches（`prove_class_signature_erasure`）、直接参数化父（`Parent<String>` 单独判据）、
  type-use 注解（存在即拒）。
- **嵌套类未证的位即 nesting 位**：`this_class` 带 `$` 时物理头自身无法证明“这是外围类的成员类、
  简名是什么”——`$` 本身不构成嵌套证据（JVMS §4.7.6；顶层类名可合法含 `$`）。
- **嵌套位证明数据源**（取证结论）：**类自身 InnerClasses 自表行**（每个 classfile 的 InnerClasses
  属性都含自身行），与折叠片 `child_relation_agrees` 的子侧判据同源；不使用外围类的行
  （分离口径下外围类不必在场）。
- **scope 注入点**：`project_generic_signature` 成功 → `ClassSourceDeclaration.generic_scope`
  （serde-skip 内部交接）→ facade 将 `class_scope.type_parameters` 传入
  `project_field_signature` / `project_method_signature`（`facade.rs` 成员循环）。头投影被拒时
  scope 为空 → 成员投影在 `jvm_signature_scope_unproved`（reader `signature.rs:655` 的
  `validate_type_variables`）链式失效。注入机制既有，本片只解锁其输入。

字段层还有一处独立的 `$` 位（`project_field_signature` 的
`class_internal.contains('$')` → `field_generic_source_unproved`）：字段声明在嵌套单元内的
源位置同样依赖头位置证明，本片以“类头已投影”作为该位的证明输入（`class_header_projected`）。

## 2. 判据复用（design 决策 1）

`src/member_inner.rs` 提取单一谓词 `source_spellable_member_row(access_flags)`（折叠片
`scan_family_root` 原内联判据：接口行=隐式 static abstract+至多一个可见性位、类行无
interface/annotation/enum/synthetic 位、非 final+abstract），新增
`prove_nested_member_position`：

1. 无 `EnclosingMethod`（排除局部/匿名类）；
2. 自身行唯一（`row.class == this_class`）；
3. `outer_class` 存在（kind=member）、`inner_name` 为 UTF-8 Java 标识符、outer 为合法二进制名；
4. 名字可拼：`[outer, "$", inner_name] == this_class`；
5. 行 kind/flags 过 `source_spellable_member_row`；
6. **类 header 与行 access_flags 对应**：`row & !ACC_STATIC == header & !(ACC_SUPER|ACC_STATIC)`
   （行可加 static，头可加 super；javac 对全部成员形状实测如此，见下表）。

失败时保持链头原 code+message 逐字不变（负例前后输出逐字一致）。

实测行/头对应（javap，`--release 8 -g:none`）：

| 形状 | 行 access_flags | 头 access_flags |
| --- | --- | --- |
| `static class Box<U>`（包私有） | 0x0008 | 0x0020 |
| 非静态 `class Inner<U>` | 0x0000 | 0x0020 |
| `public abstract class Generic$A`（em01） | 0x0401 | 0x0421 |

## 3. 折叠防线评估（design 决策 3，如实记录）

防线原语义“折叠不携带子类 Signature 投影”精确化为“不携带**未投影**的 Signature”：
`generic_signature.is_some() && !(generic_type_parameters.is_some() && generic_refusal.is_none())`
时保持拒绝（头投影被拒、或投影为无类型参数的参数化父形状均含其中）。头投影成功时
`nested_static_member_source_text` 以子类自己的类型参数头文本写嵌套头（`static class Box<U>`）。

**评估结果**：头投影成功后 Signature 防线不再阻塞；但折叠的**既有 token 锚定层**保守位仍在——
外围方法体内“折叠成员类型的局部变量声明”（`Z1$Box local1 = new Z1$Box();` 的声明 token 只有
astore 段的 bci，无引用该类的指令）不被证明，折叠拒绝
`static fold token in method N is not tied to one proved class reference`。**该位与泛型无关**
（非泛型 `Solo local1 = new Solo();` 同样拒绝，见负例 M5 记录），属折叠片既有边界，本片不扩。
因此：

- Z1（frozen 巡查锚，`named()` 含子类类型局部变量）：jar 口径折叠仍拒于该既有层，根单元保持
  分离文本（如实）；分离口径 `Z1$Box` 头/字段均已投影。
- NG5（无子类类型局部变量的同形家族）：折叠成功，嵌套 `static class Box<U>` + `U value` +
  构造器改名 `Box()`，重编 `--release 8` + `-Xverify:all` 运行输出 `z` 与原 class 一致。

## 4. 变体前后对照（任务 1.2，≥4 组）

全部 `--release 8 -g:none` 编译，原 class `-Xverify:all` 运行通过（NG1 `x`、NG2 `a 1`、
NG3 `7`、NG4 `y`、NG5 `z`）。前后完整输出见 [results/before/](results/before/) 与
[results/after/](results/after/)。

| 家族 | 前（分离子单元） | 后（分离子单元） | 后（jar 根） |
| --- | --- | --- | --- |
| NG1 非静态 `class Inner<U>` | `class NG1$Inner` + `class_generic_source_unproved`/`jvm_signature_scope_unproved` | `class NG1$Inner<U>`；`value` 字段同类 Fieldref（`get()` 读它）→ 诚实 `field_generic_body_unproved` 保 raw | 折叠拒于折叠片既有 token 层（§3）；根保持分离文本 |
| NG2 双参 `static class Pair<U, V>` | 同链，双字段 raw | `class NG2$Pair<U, V>` + `U first;` `V second;` | 同 NG1（main 声明 Pair 局部变量→token 层拒） |
| NG3 bound `static class Num<N extends Number>` | 同链 | `class NG3$Num<N extends java.lang.Number>`；字段同类 Fieldref 保 raw（下一链） | 同 NG1 |
| NG4 `static class Box<U>`（Z1 同形） | 三重阻断链原样 | `class NG4$Box<U>` + `U value;` | 同 NG1 |
| NG5 `static class Box<U>`（无子类局部变量） | 折叠被 Signature 防线拒 | — | **折叠成功**：`static class Box<U>` + `U value`，重编运行 `z` 一致 |
| NG4neg 行不可拼负例（Utf8 `Box`→`B0x`，同长字节补丁） | 三重阻断链 | **逐字不变**（`class_generic_source_unproved` → `jvm_signature_scope_unproved`，头/字段原样） | 根不折叠（同前） |

回归测试 `tests/nested_generic_header_projection.rs` 覆盖以上全部形状（6 测试：四变体分离
投影+反射核对、NG5 折叠+重编运行、负例链原样+对照、Z1 同形局部变量边界、预算停止保物理头、
顶层/非泛型折叠零变化）。

## 5. Z1 命中输出（frozen 巡查锚，SHA 复核通过）

- `Z1$Box.class` 分离口径（后）：

```
// jarde: class Signature `<U:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure and nested member proof
class Z1$Box<U> extends java.lang.Object {
    // jarde: field Signature `TU;` projected after descriptor erasure and no same-class Fieldref
    U value;
```

- Z1 jar 口径（后）：根头 `public class Z1<T extends java.lang.Comparable<T>>` 不变；
  `Box` 折叠拒于 §3 既有 token 层（如实）；`Z1$Box named()` 等 same-class 链标记原样。

## 6. corpus 扫描与门禁

见 [results/gates.txt](results/gates.txt)（全仓测试、fmt、clippy、openspec validate 的真实命令与结果）
与 [results/corpus-scan.txt](results/corpus-scan.txt)（双腿扫描，2138 类 × 2 腿，脚本
[results/scan_corpus.sh](results/scan_corpus.sh)）。

重放口径：分离子单元用 `cargo run --release --example class_source_file -- <class>`；
jar 口径用回归测试 `tests/nested_generic_header_projection.rs::source_of` 的同一 engine API 形状
（`PlainJar` + `SnapshotAll`，`RecoveryEvidenceRequest::essential + SourceMap`）。

## 7. 遗留缺口（如实登记）

- **同类绑定链**（`generic_call_binding_unproved`/`field_generic_body_unproved`）：独立下一片
  `recover-same-class-generic-bindings`（用户草案，本片不触碰）。NG1/NG3 的字段、Z1 的
  `List<T>` 字段与方法头按该链诚实保持 raw。
- **方法体泛型返回证明**：`U get() { return value; }`（返回字段读）在
  `ordinary_generic_source_unproved: same-run Program/SSA cannot prove the body under
  parameterized types` 拒——既有 same-run 证明集不含字段读返回，非本片链。
- **折叠 token 锚定层**：外围方法中折叠成员类型的局部变量声明（§3，与泛型无关）。
- **type-use 注解头位**：按既有整头拒绝语义不变（`Runtime*TypeAnnotations` 存在即拒），
  本片未引入独立未证位分解（该位与 nesting 位在原拒绝中已合并，语义保持）。
- **孙代**：头投影判据（自表行拼接）深度无关；折叠边界沿上片（孙代不折叠）不变。
