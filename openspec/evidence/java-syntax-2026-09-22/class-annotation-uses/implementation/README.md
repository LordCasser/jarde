# 类级注解修复后的完整执行对照

执行 `CARGO_INCREMENTAL=0 cargo build -p jarde-cli` 后，将生成的 `target/debug/jarde-cli` 复制为 `/tmp/jarde-cli-class-ann-impl`。本轮固定二进制 SHA-256 为 `b4568344e6b0d7e7e2584e31e3693f72f412a597b56fde70776db3e06e459453`。随后运行：

```sh
python3 openspec/evidence/java-syntax-2026-09-22/class-annotation-uses/implementation/run_implementation_audit.py
```

脚本用同一 CLI 重建基准和边界输出。基准的原始源码、JADX 完整源码和 Jarde 完整源码分别通过 Java 8 编译及 `java -Xverify:all`；三个 runner 输出逐行相同：`true`、`3`、`@java.lang.annotation.Retention(RUNTIME)`。CLASS-retention 组同样比较原始、JADX、Jarde 三套完整源码及执行输出；三边都是 `0`、`null`，保持运行时不可见。`invisible-original-javap.log` 和 `invisible-jarde-javap.log` 都有 `RuntimeInvisibleAnnotations`、`HiddenTag` 与 `value=5`。

同一 CLI 还读取重复条目、损坏 tag 和非法类型描述符三种冻结 class。重复与非法类型均正常返回带原属性事实和拒绝记录的源码/JSON；损坏 tag 按 CLI 的执行错误退出码 4，JSON/文本仍记录 `classfile_invalid_attribute_content`、停止原因和属性壳，未报告为成功无注解。完整命令退出码、文本、JSON、class/source SHA 和反射/JVM 日志都在 `generated/`。
