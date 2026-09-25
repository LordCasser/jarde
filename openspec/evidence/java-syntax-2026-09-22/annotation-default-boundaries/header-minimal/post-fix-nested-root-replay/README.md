# 嵌套注解默认值修后独立重放

root 在 `header-minimal/` 原 fixture 的临时复制上，以最后构建的 CLI `/tmp/jarde-cli-nested-root-final`（SHA-256 `7f9dd55bb020d0308504a9484ad44b157b271f08b1b96c667e538f935c6a38d5`）重新执行 `run_audit.py`。只替换临时脚本中旧 CLI 的 SHA pin；原 Java 源、重新编译的 class SHA 清单与既有 fixture 逐字节相同。CLI 哈希在执行前后相同。

Basic 与 Nested 两组的原 class、JADX 和 Jarde **完整类**均经 `javac --release 8 -g:none` 编译并由 `java -Xverify:all` 执行。Basic 三方输出 `5`、`source-only`、`[2, 4]`；Nested 三方输出 `6`、`2`。Jarde 的 `Nested.java` 写出 `default @Inner(value = 6)` 和数组 `{ @Inner(value = 2), @Inner(value = 3) }`，没有修改生成正文或 runner。前一阶段 `post-fix-root-replay/` 的相同 Nested 类可编译但反射 NPE，清楚标明本项修复的单因子结果。`summary.json`、每类完整源码/JSON、class SHA 及 javac/java 日志保存在此目录。
