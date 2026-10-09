## Why

EM-18 的 CharSequence/Collection/Throwable 与自有子类数组在元素为 inline `new` 时，早于类型兼容检查被现有构造器与数组两个独立证明拒绝；同一冻结输入经源码参考版 JADX 完整重编/运行成功。仅扩大任一白名单会绕过另一侧的实例、消费点及效果证明，需要准确组合这两份现有证明。

## What Changes

- 在既有数组与构造器证明内组合“构造元素、准确 aastore、fresh initializer”三者的闭合事实，共同成功才呈现完整初始化器。
- 复用现有构造参数次序、唯一 instance consumer、handler 与数组存储次序证明，绑定store BCI及stored ValueId，candidate结构共同提交后普通构造census不重复登记；不新增 pass、全局 alias 分析或 AST 类型。
- 冻结直接构造器参数副作用、extra consumer、失败后半元素、旧数组与现有 inline-array constructor arguments 的对抗回归。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：fresh reference-array 初始化器可组合已证明的内联构造元素，且保持效果/异常语义及完整来源记录。

## Impact

现有 `init.rs` 的构造器验证、`build.rs` 的 ArrayInitializers 与 expression closure、`report.rs` 的证明编排，以及既有 AST/render route。不添加 crate、外部依赖或类型层级服务。

## Prerequisites and Non-Goals

赋值兼容依赖 `recover-heterogeneous-array-init` 的已验证 predicate 与准确站点证明；本项不得以构造器成功替代兼容检查。只针对同 block、闭合 fresh initializer 中的完整构造元素，nested composition 需沿用既有深度预算。

结构证明与Builder呈现成功是独立平面；后续类型拒绝须拒绝完整正文并保留真实来源。direct boxed family的五个primitive argument conversions作为独立未覆盖控制保留，不混入本项引用元素组合。

不改 main 的 concat/arraylength/field 恢复，不解析缺失依赖，不重排元素，不无条件允许一般 aastore，不将 raw marker 生成的可编译空体计作语义成功。
