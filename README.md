# jarde

纯 Rust、library-first 的 JVM artifact 静态分析与按需反编译引擎规划。

当前已进入 P0 验证与交付阶段；基础公共契约、不可变 artifact 快照/顶层物理 entry 读取、classfile Header inspection、版本能力报告、按方法共享指令解码、公共 Engine 和有界 JSON CLI 已落地，容器、classfile 对抗/性质、预算、取消和局部请求边界回归已通过。P0 支持矩阵、CI/示例和归档尚未完成，查询、IR 和反编译仍未实现。

- [OpenSpec 入口](openspec/README.md)
- [P0–P5 阶段路线](openspec/roadmap.md)
- [技术栈与依赖选型](openspec/dependencies.md)
- [架构基线](JVM_Rust_Engine_Final_Architecture.md)

文档验证：`openspec validate --all --strict --no-interactive`。
