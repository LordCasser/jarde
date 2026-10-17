# 原始 TestNestedLoops3 输入

完整原始源码及 outer/target 四个 class 原样复制自 root 已接受的 `original-baseline-root-v1`，没有删方法或修补源码。两profile均由 javac23 `--release 8` 编译；debug保留调试信息、no-debug使用`-g:none`。完整argv、工具/依赖pin、失败javac8边界和真实原JVM运行见对应OpenSpec change的results归档。

`copy-manifest.json` 固定源与每个class的bytes/SHA。target包含六个方法，测试入口为`test(I)I`；Code42B、21指令、单named row。原check的AssertJ依赖只在root隔离运行时使用，静态Rust读取不加载外部Java类。

原始文件来自本地JADX测试树，保留原文及其项目许可证约束。本夹具当前只固定红基线，不表示生产恢复已通过；补充同构类与最终产品验收另记。
