## Why

DT-29 的父字段写入和单个引用转换已各自恢复，但固定 JADX `TestFieldCast` 的完整类族仍不能由 Jarde 生成可重编源码：`C/D.set`、`D` 泛型方法头及根类 `run/bits` 的证据在不同层失去闭合。继续逐字段修补无法证明类族可用；本里程碑以原始、固定 JADX、Jarde 的全部物理类作为一个验收对象。

## What Changes

- 将选中类层级、物理 CP owner、逐 BCI SSA 接收者和方法目标绑定成同一有限证书，覆盖 `C/D.set` 的 `B→A` 字段与 accessor、根类 `run` 三次 `B→A` 私有 `bits(A)` 调用。
- 在准确 `Signature` 与物理擦除一致、正文参数使用获证时，恢复 `D.set` 的 `<T extends B>` void 方法头及反射可见的泛型元数据。
- 恢复 `bits` 中四组跨基本块布尔条件值进入同一 `StringBuilder` 的保序拼接；未证明的别名、额外效果和边继续拒绝。
- 每包先用独立完整类族负例验证边界，最后对固定 `FieldCast` 九个物理类进行一次整体 Java 8 重编、`java -Xverify:all` 和来源验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：扩展有证明的父字段/方法引用转换、泛型方法声明和跨块条件拼接，使固定 DT-29 完整类族保持字段绑定、方法绑定、求值顺序和源码可重编性。

## Impact

涉及 `src/facade.rs` 的选中类层级与逐点证书、`src/class_source.rs` 的泛型声明投影，以及 `crates/jarde-java/src/{field,build,concat}.rs` 的证书消费和表达式恢复；保留核心与 CLI 分层。固定 JADX checkout `2fb1b16386941660fda07e9017285aec40fcb37f` 和输入位于 `openspec/evidence/java-syntax-2026-09-27/dt29-reference-cast-audit/combined/`。先决条件是已合入的同包直接父字段首片。非目标包括一般任意层级推断、任意跨块拼接和与 DT-29 无关的 Region/finally 债务。
