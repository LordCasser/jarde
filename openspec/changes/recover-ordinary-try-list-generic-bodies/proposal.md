## Why

当前产品 `597010a8cb9992eead8276b2f2a67186ada393d9` 已能恢复普通 try 内的前缀、foreach、分支和嵌套循环，完整生成类也能通过44个运行场景，但这些方法的 `List<E>` 参数仍退成 raw List。原因是现有完整 void 用途扫描对任何 handler 早拒；下一里程碑要接通已有结构证明和泛型用途证明，避免正文恢复后声明仍丢失已可证明的类型。

## What Changes

- 在同次完整方法恢复中，允许一条普通 named catch 的已认证前缀+foreach结构进入现有 void/List 用途扫描。必须绑定真实异常表行、protected范围、body/handler/continuation唯一所有权；不以“已经出现 try 文本”替代证明。
- 对现有有限 List 契约支持的精确元素类型 `E` 统一处理，至少覆盖 `String` 与 `Integer`。支持当前已恢复的分支、null写入、`remove(Object)`前缀和嵌套foreach，不按类名、方法名、变量名或debug信息识别。
- 沿用完整 Code/operands/SSA/effects/Operations 配对及全部参数用途闭合，包含 handler-owned 指令。沿用签名擦除、调用实际类型与重载证明、现有事务发布；只在已证明的 `add(E)` 调用移除合成 Object wrapper，保留真实checkcast及 `remove(Object)`边界。
- 保存双JDK、debug/none、default/all的原类/JADX/Jarde完整类比较，并建立永久正反测试、预算/取消门。原类是行为oracle；JADX实际整类编译失败保留为no-runtime，不修补其源码制造成功。

## Capabilities

### New Capabilities

无新增能力目录。

### Modified Capabilities

- `java8-recovery`：增加已认证普通 try 内精确 `List<E>` void 正文的泛型声明与调用投影要求，以及所有权、用途闭合、Stop和整类对照边界。

## Impact

主要落点是 `crates/jarde-java/src/report.rs` 的同次候选/物理扫描、`region.rs` 的现有范围与owner事实，以及 `src/facade.rs` / `src/class_source.rs` 的现有List源级证明和事务。必要时增加私有有界事实交接；不新增公共carrier、泛型solver、SSA/CFG pass、依赖或宿主classpath加载。

前置阶段：`recover-debug-reference-lifetimes` 与 `recover-protected-prefix-loop-sequences` 的产品自身CI和clean-main交付完成后应用代码；规划与实际基线归档可以提前进行。当前基线已独立核验两份27命令/54raw捕获、17物理方法、44运行行；规划完成不表示本能力已经实现。

非目标：不扩大到任意多range/multi-catch、nested try、finally/TWR/monitor，不恢复任意泛型成员或wildcard capture，不绕过raw alias、参数重赋值、未知调用或 `remove(int)`控制。纯goto文本映射缺口独立记录，不混入本项。71单元/612测试、整CF11完成计数保持既有口径，必须另按账本范围验收。
