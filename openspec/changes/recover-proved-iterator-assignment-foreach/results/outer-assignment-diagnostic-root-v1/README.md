# 固定外层赋值同 IR 诊断

root 在产品 993e9f8c0f413bb7c31538e6f7c2c92a742e7af4 执行实际 runner-v2，两命令 test/clean 均 exit0；独立 Luna accept-v3 由 root 实际执行 exit0。三请求共12 scoped events，每组 outer13/path[1] 均到 proof_complete_diagnostic_only。

实际 AST Assign 的目标是 s1；next17 Value32 → CheckCast22 Value33 → Store25/local4 Value34。所有既有 iterator proof 及来源/删除计划均通过；成功在名称分配、AST改写、列表删除之前强制 None，公开 API 仍断言一个内层 foreach/一个外层 while。该诊断不证明永久改写或 runtime 对照通过，不推定元素 LocalVariable 身份。

4732 输入原字节恢复，默认 Rust 栈，5GiB/target1GiB 一秒守卫未中止，target峰值327621731B，finally clean。原第一次诊断编译 E0308/get(&u32) 失败保留在 compile-failure，源码亦恢复/clean；第二次只增加解引用一个字节，不能把第一次计为通过。

actual/execution.json SHA256 581ee2a06b144ef9ba3cc622bc213344d58b25312aa8d898fcf83411488ef811。archive-manifest逐项核对原/private raw与仓库副本的字节和SHA；receipt原绝对路径保持历史事实。architecture-review-luna仅静态审查，真实门结论来自actual日志。旧完整三方类基线复用前片固定证据；新候选/新控制尚未验收。
