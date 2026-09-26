# EM-04：Java 8 包声明与包注解

`p/package-info.java` 使用 `@Deprecated` 标注包，`p/Check.java` 在运行时读取 `Package` 上的注解。两份源码以 `javac --release 8 -g:none` 编译，`v8/` 固定编译结果，`SHA256SUMS` 固定字节身份。标准 `package-info.class` 的版本为 52，flags 为 `ACC_INTERFACE | ACC_ABSTRACT | ACC_SYNTHETIC`，无接口、字段和方法，有一个 `RuntimeVisibleAnnotations`。

三方完整源码与运行对照由 [replay.py](../../../../openspec/evidence/java-syntax-2026-09-27/package-info-basic/replay.py) 重放。运行样本输出 `true`，证明包注解不只是文本装饰。
