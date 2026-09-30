# Throwable-wrap 实参切片（`recover-throwable-wrap-arguments`）证据 — 2026-09-30

[巡查 README](../README.md) 判别 3 的实现侧证据：`build.rs` 实参转换分派在 `platform_reference_argument_widens`（List→Iterable）回答之后新增 `java_lang_throwable_widens(presented, required)` 闭集回答，闭合异常包装重抛家族。实现落点 `crates/jarde-java/src/build.rs`；回归 `tests/p3_throwable_wrap_arguments.rs` 与 `build.rs` 单元测试 `java_lang_throwable_widening_reaches_exactly_the_table_ancestors`。

## 闭集表

45 条直接边（子类 → 直接父类，函数内 walk 展开传递闭包），逐对为 JDK 8 `java.lang` javadoc 声明的 `extends` 关系；`java_release` 不限制。机械验证：45 对从 `build.rs` 提取后对 JDK 23 运行时逐对断言 `getSuperclass()` 相等、双方包名为 `java.lang`、均为 Throwable 可赋值——全部通过；闭包抽查 22 个跨层正例可达、6 个兄弟/子包对（`IllegalStateException→Error`、`Exception→RuntimeException`、`Throwable→Exception`、`java.io.IOException→*`、`java.util.ConcurrentModificationException→*`）不可达。收录范围 = JDK 8 已存在的 java.lang 全族（含 `BootstrapMethodError`/`TypeNotPresentException` 等；不含 9+ 才有的 `StringConcatException`，子包 `java.io`/`java.util`/`java.lang.annotation`/`java.lang.reflect` 不收）。

呈现与分派其余 widening 分支同款：`cast_argument` 保留要求类型拼写（`(java.lang.Throwable) e`），防多候选重载重定向；同名、Object、数组、overload 证明、List→Iterable 各回答次序与文本逐字不变（新分支追加在 platform 之后）。

## 基线重放（tasks 1.1）

fixture SHA 与 [results/fixture-sha256.txt](../results/fixture-sha256.txt) 一致（C2 = `ae946f2d…`）。可重放命令（工作树根）：

```
target/debug/jarde-cli class-source \
  --input openspec/evidence/java-syntax-2026-09-30/cf15-crossing-patrol/fixture/C2.class \
  --class C2 --policy single-class --format text
```

实现前输出与提交的 [fixture/C2.jarde.java](../fixture/C2.jarde.java) **逐字节一致**：BCI 50 参数 1 拒绝（`IllegalStateException → Throwable … no safe reference conversion evidence`）、BCI 51/52 local3 声明级联拒绝。实现后 catch 体三语句完整（`String local2`、`RuntimeException local3 = new RuntimeException("w:"+local2, (java.lang.Throwable) local1)`、`throw local3`），整类可 `javac --release 8` 重编。

## 包装正例族 C2W（tasks 2.1/2.2）

[C2W.java](C2W.java) → 冻结 [original/C2W.class](original/C2W.class)，javac 23.0.1 `--release 8 -g:none`。实现前 6 处拒绝（[results/C2W-wrap.base.txt](results/C2W-wrap.base.txt)），实现后 **0 处拒绝、0 个 `@bytecode` 标记**（[results/C2W-wrap-recovered.txt](results/C2W-wrap-recovered.txt)）：

| 变体 | 形状 | 实现后呈现 |
| --- | --- | --- |
| V1 alias | C2.alias 同形，ISE→Throwable 参数 1 | catch 体三语句完整 |
| V2 iaeWrap | IllegalArgumentException→Throwable 参数 1 | 完整，`(java.lang.Throwable)` |
| V3 nested | 单 catch 层双层包装：ISE→Throwable（local1）+ RuntimeException→Throwable（local2） | 两语句完整，`w2→w1→ISE` 链 |
| V4 forward | 用户静态 `log(Throwable)` 收异常 | `log((java.lang.Throwable) local1);` |
| V5 multi | `note(7, e, 'x')` 多实参位次，仅中间 Throwable 位上转型 | int/char 位不动（错误上转型将不可编译） |
| V6 causeOnly | 单参构造 `new RuntimeException(e)` 源解析为 `(Throwable)`，**位次 0** 上转型 | `new RuntimeException((java.lang.Throwable) local1);` |
| V7 overloadNarrow | `pick((Throwable) e)` 原源显式窄化 + `pick(IllegalStateException)` 多候选 | `pick((java.lang.Throwable) local1);` —— 拼写保住同一重载 |
| 对照 objectTarget | `identity(Object)` | Object 回答先行不变：`identity((java.lang.Object) local1);` |

三方对照（tasks 3.2）：原冻结 class 运行、JADX 输出重编运行（固定 dev CLI `jadx-cli/build/install/jadx/bin/jadx`，[results/jadx-C2W.java](results/jadx-C2W.java)）、Jarde 恢复重编运行，三者 `java -Xverify:all` 输出逐字节一致（SHA 同为 `006a40…`，见 [results/wrap-sha256.txt](results/wrap-sha256.txt)）；注入异常路径的包装消息、cause instanceof、双层包装链逐路径一致。C2 同样三方一致（SHA 同为 `07a675…`，[results/jadx-C2-run.txt](results/jadx-C2-run.txt)）。

## 负例族 C2WN（tasks 1.2）

[C2WN.java](C2WN.java) → 冻结 [original/C2WN.class](original/C2WN.class)（含两个用户嵌套异常类），原类 `java -Xverify:all` 全路径运行通过（[results/C2WN-original-run.txt](results/C2WN-original-run.txt)）。实现前 8 处拒绝、实现后 **6 处拒绝逐字保留**（同一 BCI、同一对类型，[results/C2WN-wrap-recovered.txt](results/C2WN-wrap-recovered.txt)）：

| # | 形状 | 拒绝（实现前后一致） |
| --- | --- | --- |
| N1 | 用户 `MyFailure extends Exception` → Throwable（参数 1） | BCI 45 表外拒绝 |
| N1b | 用户 `MyError extends Error` → `log(Throwable)`（参数 0） | BCI 19 表外拒绝 |
| N2 | `java.io.IOException` → Throwable：真 Throwable 但非 java.lang，闭集不收，升级路径另启 | BCI 45 拒绝 |
| N5 | 单参构造参数 0（cause 位）：`new RuntimeException(e)` 收用户类型 | BCI 23 位次 0 拒绝 |
| N4 | 数组异常类型 `[IllegalStateException → [Throwable`（参数呈现为数组形参，非局部级联） | BCI 31 分派拒绝（数组形状不变式） |
| N4b | 用户 `[MyFailure → [Throwable` | BCI 31 拒绝 |
| — | 伴随形状 `overloadNarrow`/`rawMix` | 实现前拒绝、实现后恢复（归正例族口径） |

负例形状到 verifier 现实的映射说明：**原始/引用混形**在 verifier 有效字节码里只能以装箱后引用呈现（`rawMix` 的 `Boolean→Object` 走既有 Object 回答、逐字不变），原始拼写的拒绝钉在单元测试负例（`int/boolean` 对）与 V5 多位次形状（错误上转型不可编译）两侧；**`String → Throwable` 类非祖先 java.lang 对**无法通过 javac 产出（校验器保证呈现类型可赋值于要求类型），钉在单元测试负例与 N2 的“真子类但表外”fixture 两侧。JADX 对 C2WN 的输出仅作结构参照（[results/jadx-C2WN.java](results/jadx-C2WN.java)）：嵌套类非限定名不可独立编译，故 C2WN 无 jadx 运行列，运行锚为原类。

## 升级路径登记（本片不实现）

1. **非 java.lang 呈现/要求类型**（N1/N1b/N2/N4b）：未来以 resolution 环境的类层级查询建 `reference_overload_calls` 同款逐 BCI 证明；触发条件 = 首个真实用户类上转型案例。
2. **词典嵌套 try-in-try 双层包装**：`local 1 crosses a quoted fallback region`（Slice A `recover-named-row-crossing-locals` 域）；本片 V3 用单 catch 层双包装恢复同一 cause 链语义。
3. C2WN `main` 的内联 `new IllegalStateException[]{…}` 数组创建呈现缺口（`the array instruction … produces a value nothing in this body reads`）属数组初始化呈现域，与实参分派无关；负例类不重编，不影响验收。

## 测试与门禁

- `crates/jarde-java/src/build.rs`：`java_lang_throwable_widening_reaches_exactly_the_table_ancestors`（26 正例跨层可达 + 17 负例含子包/用户类/兄弟/向下/原始拼写/数组形状/平台对隔离；同名 `(Throwable,Throwable)` 不属上转型谓词）。
- `tests/p3_throwable_wrap_arguments.rs`：C2 命中与整类文本钉死、C2/C2W 三方运行对照、C2WN 六拒绝逐字钉死、预算 8 字节停止与取消原子性（`Incomplete`、无发布）。
- 既有转换回答回归：`array_invocation_widening.rs`（Object/数组/overload/List→Iterable）与 `p3_iterable_foreach.rs` 全量套件内保持绿。
