# 类头参数化投影的语料普查（2026-10-04，root）——两片合并的验收边界与跨片耦合

为待派发的合并片 `recover-parameterized-class-headers`（= `recover-parameterized-interface-headers` + 姊妹片"父类嵌套名参数化投影"，二者改**同一函数** `class_source.rs::project_generic_signature` 的相邻分支，串行必 rebase 冲突，故合并派发）确定**可证伪的验收边界**。方法：对 `tests/fixtures/` 全部 `.class` 用 `javap -p` 读已解码的声明行（不自行解析 `Signature`，避免括号内 `;` 造成的切分错误）。

**结论：全语料 482 类中，两条腿合计只有 4 个类在范围内——而这 4 个恰好全是 `recover-bridge-admission-gates` / `recover-bridge-superclass-header-precondition` 两片验收时钉死的类。故本片的验收必须包含对桥派发行为的复验，不是纯呈现润色。**

转录见 [census-output.txt](census-output.txt)（有效轮）与 [census-first-run-invalid.txt](census-first-run-invalid.txt)（无效轮，保留作为脚手架自检教训的证据）。

## 普查结果

| 腿 | 判据 | 命中数 | 类 |
| --- | --- | --- | --- |
| 父类腿 | `extends` 的父类 binary 名含 `$` **且**带类型实参 | **2** | `BR$StrBox extends BR$Box<java.lang.String>`（`p3-bridge-projection/br-family/v8/`）；`Spec extends Outer$Box<java.lang.String>`（`p3-bridge-projection/bridge-superclass-precondition/v8/`） |
| 接口腿 | `implements` 的接口带类型实参 | **2** | `BR$Impl implements java.lang.Comparable<BR$Impl>`（`br-family/v8/`）；`BridgeProbe implements BridgeApi<java.lang.String>`（`p3-bridge-projection/positive/v8/`） |
| 父类含 `$` 但**无**类型实参 | — | 6 | `BR2$Mid`、`MixedInstanceControls$DerivedBox`、`AnonymousCaptureCases$1`、`AnonymousMemberBase$1`、`AnonymousMemberBase$2`、`NestedSuper$Sub` —— **不在本片范围**（无类型实参可投影） |

有效性指标：`TOTAL_CLASSES=482`、`ERRS=0`、`NO_DECL_LINE=1`、`DECL_NOMATCH=1`（唯一未匹配项是 `package-info.class`，其声明为 `interface p.package-info {`，名字含连字符且不是类，属正确排除）。

**验收判据（可证伪）**：合并片落地后，corpus 双腿扫描的差异类**应恰为上述 4 个**（`BR$StrBox`、`Spec`、`BR$Impl`、`BridgeProbe`）。出现第 5 个差异类即为判据过宽（误纳了非"$ 父类带实参"/"接口带实参"形），**停下报告**；上述 4 个中若有任一无变化，说明该腿未生效，也须报告。

## 跨片耦合（本片最重要的架构约束，root 读码确认）

`recover-bridge-superclass-header-precondition`（已验收，merge `cc4b6f11`）的守卫判据是：

```text
use_kind == InvokeVirtual && parameter_cast_form && !class_header_projected
    && 类 Signature 的 superclass 段带实参  →  拒绝隐藏桥（保持可见）
```

其中 `class_header_projected` = `class_scope.is_some()`（取自同一装配趟的类头投影结果）。**本片让父类头携带类型实参 → `class_scope` 置位 → `class_header_projected` 转真 → 该守卫的前置不再成立 → 桥恢复隐藏。** 这是**设计意图**（bridge-superclass 的 design 明写"姊妹片落地后本片前置自动失效，无需回退"），但它的正确性依赖一条不变量：

> **类头携带了该契约的类型实参 ⟹ javac 会从该头重新生成这个桥 ⟹ 擦除派发仍路由到子类覆写体。**

对 `Spec`/`BR$StrBox` 而言，这意味着它们的 `set(Object)` 参数收窄桥会从"可见"**变回"隐藏"**。若该不变量在任一形上不成立，就会**复活 `cc4b6f11` 刚修掉的静默错值**（擦除派发路由到父类体，`BOX.set(Object) ran` 而非 `SPEC.set(String) ran`）。

**故本片的验收必须包含（不可省略，且不得以"corpus 渲染差异符合预期"代替）**：

1. `Spec` 与 `BR$StrBox` 的**擦除派发端到端复验**：以投影后的完整源集 `javac --release 8` 重编，经父类擦除引用调用 `b.set("x")`，`java -Xverify:all` 运行必须打印 **`SPEC.set(String) ran`**（`Spec`）/ 与 `BR$Base` 基线一致（`BR$StrBox`），**不得**是父类体的输出。判据与 `openspec/evidence/java-syntax-2026-10-04/bridge-superclass-rawheader-misdispatch/README.md` 的决定性表相同。
2. 重编产物 `javap -c` 须显示 javac **确实重新生成了**该参数收窄桥（`ACC_BRIDGE` 方法存在且转发到子类覆写体）——这是上面不变量的直接证据，不能只靠运行输出推断。
3. `BR$Impl`（接口腿）与 `BridgeProbe` 同样须复验桥派发：接口边的守卫判据是 `bridge_interface_contract_generic`（`cc4b6f11` 未改），本片让类头呈现 `implements java.lang.Comparable<BR$Impl>` 后会使其拒绝分支转为准入分支，故协变返回桥的隐藏必须仍行为正确（`BR$Base.next` / `Comparable.compareTo` 派发一致）。
4. 既有测试**逐字更新而非削弱**：`tests/class_source.rs` 中 `BR$StrBox` 的断言在 `cc4b6f11` 被更新为"桥可见"，本片落地后须再更新为"桥隐藏 + 类头参数化"。**不得为了让旧断言变绿而放宽任何一道桥准入前置**——两片合起来的效果是"头参数化 + 桥隐藏 + 派发正确"，任一环节缺失都必须表现为响亮失败。

## 脚手架自检教训（本普查自身）

首轮普查（[census-first-run-invalid.txt](census-first-run-invalid.txt)）报出**完全相同**的 2+2 命中数，但有 **65 个 `DECL_NOMATCH`**：正则的类名段 `([\w$]+)` 不接受带包前缀的点号名（`cf04.Negative`、`probe.Empty`、`matrix.Outer$A$Generic<V>`），致使全部带包类被静默跳过。**首轮结论是低估的假象**（4 个自检类恰在默认包，故自检通过却漏掉了这一 bug）。修正为 `([\w$.]+)` 并容忍嵌套类型实参与行尾 ` {` 后，`DECL_NOMATCH` 降至 1（仅 `package-info`，属正确排除），命中数不变——**说明 2+2 是真实边界，而非扫描遗漏**。已把"自检样本必须覆盖语料形态多样性 + `NO_MATCH` 必须为零才可采信"固化进 `handoff.md`。
