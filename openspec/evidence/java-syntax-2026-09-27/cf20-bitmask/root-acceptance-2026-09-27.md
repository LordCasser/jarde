# CF-20 主线独立复核

root 以主线组合 CLI `/tmp/jarde-root-cli-cf12-cf13` 从归档原 class 独立生成完整 Jarde 源码，SHA-256 `7a5574ec9946852885e080f06cb1f522d1a2b9d2a6f0a3b62499463eb741eccb` 与审计归档逐字节相同。root 重新以 `javac --release 8 -g:none` 编译原始、固定 JADX、Jarde 三份完整源码，均通过；再用 `java -Xverify:all` 跑每份类的同一个 `main`，六行逐行一致：0/1/`Integer.MIN_VALUE` 不命中掩码，2/3/`Integer.MIN_VALUE+2` 命中，每个谓词的 `observe` 均只调用一次。Jarde 源码保留 `(observe(arg0) & 2) != 0` 与 `== 0` 两个整数位运算比较。

因此固定 CF-20 首片未见恢复差距。JADX 原测试只检查一次 ` & ` 文本；这里的运行结论来自另构的 Java 8 完整类对照，不扩大为所有位运算 lowering 的追平声明。
