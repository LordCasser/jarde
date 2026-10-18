## Why

固定 JADX `TestNestedLoops.java` 的当前完整输入已实际采集：Jarde 正确恢复 `List<String>` 形参和两层 Object foreach，但逐元素 String cast 仍在正文；同输入 JADX1.5.6 已生成 String foreach。循环识别和泛型形参证明均已有，无需新增循环机制，缺口是两种已证明结果的源级投影没有结合。

## What Changes

- 在完整泛型源码事务成立后，将精确 `List<E>` 的 Object foreach 和首句 `(E) syntheticElement` 合并为 typed foreach；原首句仍保留赋值/声明，只改为读取同一已类型化元素。
- 保留已有合成元素名字、外层变量声明、正文目标、作用域、写入时机及全部其它语句；不引入变量生命周期重建。
- 精确核物理 next/cast/store、同方法 AST 锚点、直接 iterable formal 和元素类型；缺证时保留现有可靠表示，Stop 不发布部分投影。
- 用原始完整类、原/JADX/Jarde 全源码重编、双 JVM、错误元素和循环外可观察绑定控制验收。

### Prerequisites and Non-Goals

基底 main `0738f7785ae7907f49b0c8f8b55091e6aaa4ea1b`，产品 ee013/aabf CLI；root已实际独立接受完整原始20命令和生成矩阵75命令，三份归档307成员已逐字复开，见results/README。静态旧 BCI13/42 首拒已过时，不重新实现 iterator 初始化/声明分离。

本片恢复 typed 元素头，但不承诺原来的 s1/s2 直接成为头变量、不删除冗余声明/局部复制，也不声称整条上游文本黄金或整个CF11已追平。精确原名绑定折叠、equals/remove 的 Object 包装、raw/wildcard/type-variable/nested-generic 元素、任意 Iterable 表达式、其它控制流/类型债务单列。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已有泛型源码证明和元素 cast 完整匹配时，可在保留正文绑定的条件下呈现 typed List foreach，维持求值、异常、来源和事务停止契约。

## Impact

预计限于现有 generic-call 源码事务及 jarde-java opaque AST 投影/必要测试。复用 `ForEach`、同次 AST、现有 List 源码与 iterator 证明、原子发布及计费；不新增全局 pass、IR/AST 节点、作用域 solver、依赖或 crate。JADX 的 cast-result/iterable-type 配对是算法参考，不照搬其可变类型推断或源码类型合成。
