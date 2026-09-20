## 1. 内容契约与公开入口

- [x] 1.1 为 RecoveryReport 添加闭合 content 分类并更新所有构造路径；从最终提交结构判定，不能使用包含 fallback 的 program.statements 计数。以全说明、纯 Java、Java+fallback、return-only、空分支和注释分隔符字符串验证分类（A13）。
- [x] 1.2 将分类纳入产物提交/停止路径和 CLI 序列化；在 output budget、取消、早停时验证 NotProduced 与实际 text/source map 一致，并对同一请求比较库/CLI 字段（A13/A14）。

## 2. 评测和文档

- [x] 2.1 扩展现有恢复评测汇总：请求/带 Code 分母、Produced、ContainsStatements、ExplanationOnly、Stopped，以及各相似度有效 pair/排除原因；用固定小样本验证数量可对账，initializer 折叠与无 Code 不混为失败，保留历史 token 启发式定义（A13/A18）。
- [x] 2.2 更新公开 API/支持说明与示例，明确 content 不证明完整恢复/编译/等价；运行恢复、R8/R9、source-map、CLI 回归及 fmt/clippy/OpenSpec strict，记录固定 SHA 与结果。只有实际验证后才同步/归档（A10/A13/A16）。
