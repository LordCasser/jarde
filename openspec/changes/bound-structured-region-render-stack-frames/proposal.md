## Why

CF11真实完整类在现有Region恢复后进入AST构建，默认测试线程发生stack overflow/SIGABRT。Root LLDB实测在Builder::render_value_definition触及guard页，调用链有多层Builder::region；后者ARM64 debug单帧0x31000+0x6f0+0x20约198KiB。这是既有大递归函数的栈占用债务，单独处理，不把它混入语法owner证明。

## What Changes

- 在既有Builder实现内拆分必要的大Region变体消费分支，缩小普通嵌套调用的原生栈帧。
- 保持同一声明、作用域、表达式、预算/停止、来源和AST消费顺序，默认测试线程通过真实完整类。
- 保存原默认栈失败、8MiB仅诊断及LLDB/机器码测量，并运行受影响的循环/try/return路径和完整产品检查。

## Capabilities

### New Capabilities

无。只是内部函数栈帧重构，不新增能力或公开协议；skip_specs:true，不编造delta需求。

### Modified Capabilities

无。既有有界恢复/停止要求和语法准入保持，不降低完整来源要求。

## Impact

crates/jarde-java/src/build.rs中Builder::region的私有实现和实测验收证据。无新依赖、IR、pass、公开API、堆栈包装或提高RUST_MIN_STACK。当前CF11在制候选为实测触发输入；自身原子性与来源验收仍由原变更负责。
