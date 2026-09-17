## 1. Specifications and dependencies

- [x] 1.1 阅读架构并建立 P0 proposal/design/specs
- [x] 1.2 固化候选库评估、版本、许可证与未覆盖边界（核对 registry 元数据、源码和准入清单，行为测试留待实施）
- [x] 1.3 建立 P1–P5 可独立验收的后续路线（六个 change strict validation 与 A01–A18 映射）

## 2. Foundation

- [x] 2.1 建立 Rust workspace、模型、预算、错误及序列化契约，以 MSRV/stable 编译和 JSON round-trip 验证
- [x] 2.2 实现不可变快照、物理 entry 枚举与有界读取，以重复项、CRC、源文件替换和超限 fixture 验证
- [x] 2.3 实现 noak Header 适配、原始字符串及属性位置，以 NUL/surrogate、属性跨度和 Header 不解码 Code 的计数验证
- [x] 2.4 实现版本能力和共享指令解码、局部失败报告，以历史 minor、switch/wide、非法 opcode 与 JVM oracle 交叉检查
- [x] 2.5 提供公共 Engine API 和 JSON CLI，验证离线无 JVM 环境及库/CLI 状态一致

## 3. Verification and delivery

- [x] 3.1 CLASS/JAR/WAR、重复项、ZIP64、压缩、CRC、快照回归
- [x] 3.2 历史版本、MUTF-8、未知属性、switch/wide、截断与属性/指令 fuzz 回归
- [x] 3.3 验证预算、取消、局部请求和无 IR 构建的边界
- [x] 3.4 完成 README、支持矩阵、CI、示例与实际测试记录
- [x] 3.5 cargo fmt/clippy/test、OpenSpec strict validation 并归档已完成 change
