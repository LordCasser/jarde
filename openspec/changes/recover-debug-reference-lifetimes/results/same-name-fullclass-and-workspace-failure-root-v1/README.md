# 同名引用完整类与真实工作区失败

root 实际接受 16 条 CLI 请求（双 JDK × 4 完整类 × default/all），未修改生成 Java；2 条隔离全类编译和 8 条原始 / 8 条生成类运行均 exit 0，运行 stdout 逐字同原，保留每类 3 个方法身份与全部物理 source-map。ListThenMap 输出 `3`，MapThenBuilder 输出 `6:6`，StringThenQueue 输出 `1: q !`，BuilderThenArray 输出 `abc`，两 JDK 一致。没有据此新增 JADX runtime 或整单元计数。

完整工作区 root-v1 实际失败于第 52 条命令：旧测试要求同名 LVT 的错误输出逐字保持，而本改动已按预期拆分身份。该 run 没有进入第二 seed，不能算完整工作区通过。归档保留全部实际 receipt/raw、失败时的精确旧测试、private 测试修订和 v2→v3 工具适配；最终 exact cargo clean exit 0，target absent。后续必须从新源码启动全新双 seed run，不拼接这次部分通过记录。

`capture.tar.gz` 中所有成员由 root 复开并核 SHA-256，详见 `archive-review-root-v1.json`；归档不包含 60 MiB 冻结 CLI，实际 CLI bytes/SHA 保留在验收记录中。
