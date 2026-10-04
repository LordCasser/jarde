# Design：识别真 javac 8 的 `getClass()` null-check 惯用法

## 决策依据（root 实测，全部可在 [取证目录](../../evidence/java-syntax-2026-10-04/qualified-outer-alloc-getclass-patrol/README.md) 复现）

**结构事实**：两种工具链对 `outer.new Inner(…)` 发射的分配序列**逐条同构**，唯一差别是 null 检查那一条调用：

```text
        javac 23 --release 8              真 javac 8 (Corretto 1.8.0_432)
 0: new           N1$Inner           0: new           N1$Inner
 3: dup                              3: dup
 4: aload_1          ← qualifier     4: aload_1          ← qualifier
 5: dup              ← copy          5: dup              ← copy
 6: invokestatic  Objects.           6: invokevirtual Object.getClass:()Class   ← 唯一差别
      requireNonNull:(O)O
 9: pop              ← 丢弃返回值     9: pop              ← 丢弃返回值
10: bipush 9                        10: bipush 9
12: invokespecial N1$Inner.<init>   12: invokespecial N1$Inner.<init>
```

即 `init.rs` 已要求的 `[qualifier, copy, check, pop]` 连续四条**结构完全成立**，只有 `check` 一条的 opcode 与符号目标不同。这是本片能做成"并列增加一种拼写"而非"新增机制"的根本原因。

**两处判据读取不同表示**（这是本片唯一的结构约束）：

| 落点 | 所属 | 读取的表示 | 现判据 |
| --- | --- | --- | --- |
| `crates/jarde-java/src/init.rs:985-998` | `jarde-java`（`new@1` 私有证明） | 已解码事实 `Operation::Invoke(call)`，经 `call.kind()/owner()/name()/descriptor()` | `kind == Static && !is_interface_reference() && owner == "java/util/Objects" && name == "requireNonNull" && descriptor == "(Ljava/lang/Object;)Ljava/lang/Object;"` |
| `src/member_inner.rs:126-131` | 根 crate（成员调用路径） | 原始 `opcode` + `CpEntryKind::MethodRef{owner,name,descriptor}`（字节串） | `opcode == 0xb8 && owner == b"java/util/Objects" && name == b"requireNonNull" && descriptor == b"(Ljava/lang/Object;)Ljava/lang/Object;"` |

`member_inner.rs` **已依赖 `jarde_java`**（`jarde_java::init::NewRecord`、`jarde_java::names::is_java_identifier`），故共享词表放在 `jarde-java` 侧对两处都可达。

## 决策 1：把"什么算 null 检查拼写"收敛到单一所有者，两处只投影表示

**不采用**"两处各自并列加一条 `getClass` 硬编码臂"——那会产生两份可漂移的判据（一处接受、另一处不接受 → 同一形在不同路径下接受面不一致，且将来加第三种拼写要改两处）。

**采用**：在 `crates/jarde-java/src/facts.rs` 定义**唯一**的 null-check 拼写词表（与既有 `STRUCTURAL_REFLECTION_METHODS`、`ACC_*` 常量同一归属惯例），两处各自把自己表示投影成同一个四元组后调用同一个谓词：

```rust
/// One spelling of the null check a compiler inserts for a source-qualified
/// expression (`outer.new Inner(…)`). The check's own result is always discarded.
pub enum NullCheckSpelling {
    /// javac 9+: `invokestatic java/util/Objects.requireNonNull(Object)Object`.
    RequireNonNull,
    /// javac 8: `invokevirtual java/lang/Object.getClass()Class`.
    GetClass,
}

/// Whether one call is a null check whose result the site discards.
///
/// The kind and the symbol are matched **as a pair**, never independently: the
/// cross product (`invokestatic Object.getClass`, `invokevirtual
/// Objects.requireNonNull`) is not what any javac emits, and admitting it would
/// widen the set past the two facts this rule is evidence for.
pub fn is_discarded_null_check(
    kind: InvokeKind, owner: &[u8], name: &[u8], descriptor: &[u8],
    interface_reference: bool,
) -> bool { /* 两条成对匹配 */ }
```

- `init.rs` 臂改为：`matches!(operations.get(check.bci()), Some(Operation::Invoke(call)) if crate::facts::is_discarded_null_check(call.kind(), call.owner().as_bytes(), call.name().as_bytes(), call.descriptor().as_bytes(), call.is_interface_reference()))`。
- `member_inner.rs` 臂改为：把 `opcode` 映射到 `InvokeKind`（`0xb8 → Static`、`0xb6 → Virtual`），从 `MethodRef` 取三个字节串，调用同一谓词。
- **不变量**：`pop.opcode() == 0x57` 的检查在两处**逐字保留**且**不进谓词**——"返回值被丢弃"是调用点的事实，不是拼写的事实；两种拼写的返回值都是单槽（`Class` / `Object`），故 `pop`（非 `pop2`）对两者都正确。
- **`!interface_reference` 保留在谓词内**：两种拼写的目标都是 `MethodRef`（非 `InterfaceMethodRef`），故对两者同真；把它放进谓词可防止将来误接受接口方法的同名拼写。

**取舍**：谓词跨 crate 暴露为 `pub`。这是必要的——两处判据的所有者不同（`jarde-java` 的 `new@1` 证明 vs 根 crate 的成员调用路径），而"什么算 null 检查"必须是单一事实，否则接受面会漂移。归属放在 `facts.rs`（该 crate 已持有 `ACC_*`、`STRUCTURAL_REFLECTION_METHODS` 等同类共享词表）而非新建模块，避免制造平行词表。

## 决策 2：不得用 `java_release` / `major_version` 门控

root 实测两种产物的 class **major version 均为 52**，故版本字段**无法区分**它们；任何 `java_release == 8 → 接受 getClass` 的门都会对 javac 23 `--release 8` 的产物（同为 52）错误生效或错误失效。

更根本地，`crates/jarde-reader/src/classfile.rs:139-141` 已固化：

> "A level is a *request*, not a property of the artifact: … the answer is stated over the facts that class really carries — **never over the compiler that produced it** and never over its own `major_version`."

现行 `requireNonNull`-only 判据违反该原则（认的是编译器拼写）。本片按该原则修正为认事实的两种拼写，**不引入任何版本判据**。

## 决策 3：显式 `getClass()` 语句不得被误折叠（健全性负例）

最可能推翻本片的反例：用户源码显式写 `o.getClass();`（丢弃返回值），会不会被误判为 javac 插入的 null 检查？root 构造 [G-explicit-getclass.java](../../evidence/java-syntax-2026-10-04/qualified-outer-alloc-getclass-patrol/fixture/G-explicit-getclass.java) 实测（真 javac 8）：

```text
In explicit(G o) { o.getClass(); return o.new In(); }
   0: aload_1
   1: invokevirtual Object.getClass:()Class     ← 用户语句（在 new **之前**）
   4: pop
   5: new           G$In
   8: dup
   9: aload_1
  10: dup
  11: invokevirtual Object.getClass:()Class     ← javac null-check（在分配序列**内**）
  14: pop
  15: invokespecial G$In.<init>:(LG;)V
```

**位置可区分**：用户语句在 `new` 之前，javac 的检查是分配序列内紧邻 `invokespecial` 的连续四条。既有判据的连续性 + 锚定 + 区间检查（`init.rs` 的 `block.get(index+2..index+6)` 与 `pop.bci() >= at`；`member_inner.rs` 的 `first != qualifier_copy.bci || ordinary.iter().any(|bci| bci <= pop.bci || bci >= call_bci)`）**已足以排除**，本片**不放宽任何位置约束**。该负例必须冻结进验收：呈现中用户的 `o.getClass();` 语句须**保留为语句**（不得消失），且整类须可编译、行为一致。

## 决策 4：拒绝文本与注释须随判据同步

`init.rs:997` 的 `"…lacks the contiguous local load, dup, exact requireNonNull(Object), pop check"` 与 `member_inner.rs:131` 的 `"member call has no exact early requireNonNull check"` 在判据扩为两种拼写后**会与实际判据不符**（拒绝的其实是"两种拼写都不是"）。须改为不提单一拼写的表述，例如 `"…lacks the contiguous local load, dup, discarded null check, pop"` / `"member call has no exact early discarded null check"`。**同步修正 `member_inner.rs:37` 的注释**（现写"explicit `requireNonNull; pop` pair"）。

**注意**：拒绝文本变更会影响以文本断言的既有测试。实现者须先 `grep` 这两条文本在 `tests/` 与 `crates/**/tests` 中的断言点，逐一核对是"断言旧文本"（须更新为新文本，**不得削弱断言强度**）还是仅出现在证据转录（历史证据不改写）。

## 验证标准（可证伪）

1. **主锚（真 javac 8）**：以 Corretto 1.8.0_432 编译冻结的 `N1.java`，`N1` 族渲染的引注数从 **18 → 0**（或与 javac 23 腿同形），且呈现含 `return arg1.new Inner(9).total();` 与 `new N1().new Inner(3).total()`；渲染源集 `javac --release 8` exit 0，`java -Xverify:all` 输出 **`10`/`7`/`13`** 与原 class 逐行一致。
2. **javac 9+ 零回退**：既有 8 个 `requireNonNull` 形 fixture 类（`p3-lambda-adaptation/v8/BoundNullLambdaAdaptationProbe`、`proved-java-structure/anonymous-member-base/{AnonymousMemberBase,$1,$2}`、`recover-generic-enclosing-member-call-sites/matrix/{UseGenericObject,UseGenericTyped,UsePlain,UsePlainRaw}`）渲染**逐字节不变**；两片（`recover-inner-class-instance-folding` 7/7、`recover-proved-member-inner-construction` 9/9）全部测试通过，含其 `assert!(report.text.contains("new N1().new Inner(3).total()"))` 与 `contains("return arg1.new Inner(9).total();")`。
3. **负例仍拒/不误折叠**：显式 `getClass()` 语句形（决策 3）呈现保留该语句且不误折叠；`pop` 缺失形、非连续形、区间越界形各自仍按既有码拒绝。
4. **corpus 双腿扫描**：差异类**只应是此前因 `getClass` 形被拒的类**；出现任何 `requireNonNull` 形差异即回退失败，停下报告。root 精确普查已钉死基线：`requireNonNull` 形 8 类、`getClass` 形 **0 类**（故 `tests/fixtures` 内预期**零差异**，正例须新冻结真 javac 8 fixture）。
5. **门禁**：`cargo test --workspace --tests --locked --no-fail-fast`（基线 **299 目标 / 2959 passed / 0 failed**）、fmt、CI-exact clippy（`ci.yml` 46–76 逐字，含 `--all-features`）、`openspec validate --all --strict`（**273 项**）、`git diff --check`、新增 fixture 后**再生 corpus fingerprint**。

## Open Questions

1. **`member_inner.rs` 的 opcode→`InvokeKind` 映射放哪**：该片已有原始 opcode 判定惯例（`0xbb`/`0x59`/`0x57`/`0xb7`/`0xb8` 字面量）。为两条拼写引入映射时，是在调用点就地 `match opcode { 0xb8 => Static, 0xb6 => Virtual, _ => return refuse(…) }`，还是在 `facts.rs` 提供 `InvokeKind::from_opcode`？**倾向前者**（就地、最小面），除非 `facts.rs` 已有同类映射可复用——实现者须先查证，有则复用，不新建第二套。
2. **是否顺带覆盖 `Objects.requireNonNull` 的其它重载**（`(Object,String)Object`、`(T)T` 泛型形）：本片**不做**（Non-Goal），但实现者若取证发现真 javac 8/9+ 对本构造会发射其它重载，须停手报告而非自行放宽。
