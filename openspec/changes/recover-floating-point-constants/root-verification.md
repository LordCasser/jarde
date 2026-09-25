# Fixture verification

root 独立编译三个 source-only 输入，确认 `FloatingConstants.class` 与永久 1722-byte
fixture 完全相同，29 个 Code 方法；SHA-256
`7d557979522ebda315a93715c37ba50b480f6780eb8c4be86ae6a14e540b9652`。
原 class 经 `java -Xverify:all` 的 36 行与 JADX 完整输出一致。

`floating-constants/final-fixture-before/` 保存独立 javap、三方原样输出、编译日志与脚本。
CLI `c050b502ba3fc23f33937ac607b1da7b51447e02e1a6d41b1d060c2b93b5b8d9` 有 56 处
`@bytecode`，完整 javac 失败。运行前后校验 CLI hash 相同。此处是待实施输入的验收，
不是功能已实现。root 已运行 Rust 红测试：3 个预期失败、1 ignored，实际进入正面结构化恢复断言；日志为同目录 `floating-red.log`。永久语料现已冻结为 94 class / 579 Code / 75 handler / 236 target / 8 subroutine、232 个指纹文件，13 个新增输入外既有条目不变。内存 NaN 变体的 Rust 回归仍待实施。
