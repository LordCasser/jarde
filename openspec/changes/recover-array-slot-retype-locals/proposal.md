## Why

[EM-17/18 数组巡查](../../evidence/java-syntax-2026-10-01/em17-slot-reuse-patrol/README.md)发现：同一局部槽被**不同元素类型**的数组定义先后写入（`int[] a` 后 `boolean[] f` 复用槽 2，javac 常规降低）时，主线把槽声明拼为首个定义的类型，后续 `local2 = new boolean[3];` 在 `int[] local2;` 之下**不可编译**（A2.fillCalc 固定复现；A1 动态维度/混合初始化器对照完整恢复、行为一致）。根因与 `recover-caught-value-argument-typing` 同族：槽类型决策单所有者假设被复用打破——但数组场景需要**分段声明**而非仅改实参呈现。

## What Changes

- 同槽多数组定义且元素类型不同、各定义的读取段在字节码上不交叠时，按定义分段呈现：每段以自身值类型声明（源码上就是两个作用域局部）；呈现层实现（分段子句或新名），证明层零改动。
- 类型相同的多定义、单一定义、跨段交叠读取（真实别名）不受影响；交叠读取保持现有拒绝/呈现。
- 顺带消除 A2 形态的拷贝别名呈现（`local0 = …; local2 = local0;`）当且仅当它是分段呈现的自然产物；不为此单独造机制。
- 以 A2 固定类、A1 对照、多段（3+ 类型交替）与交叠别名负例验收；整类可 `javac --release 8` 且行为一致。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：不同元素类型的数组槽复用按定义分段声明，恢复文本可重编。

## Impact

仅 `crates/jarde-java` 私有 build.rs/声明呈现及测试；串行排在 caught-value-argument-typing 之后（同文件触点）。不新增证明机制；既有数组呈现（recover-nested-array-initializers 等已验收证书）零回退。
