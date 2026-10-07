# EM-04：Java 8 包声明与包注解

`p/package-info.java` 使用 `@Deprecated` 标注包，`p/Check.java` 在运行时读取 `Package` 上的注解。两份源码以 `javac --release 8 -g:none` 编译，`v8/` 固定编译结果，`SHA256SUMS` 固定字节身份。标准 `package-info.class` 的版本为 52，flags 为 `ACC_INTERFACE | ACC_ABSTRACT | ACC_SYNTHETIC`，无接口、字段和方法，有一个 `RuntimeVisibleAnnotations`。

三方完整源码与运行对照由 [replay.py](../../../../openspec/evidence/java-syntax-2026-09-27/package-info-basic/replay.py) 重放。运行样本输出 `true`，证明包注解不只是文本装饰。

## 行为基线（CI 守卫实测，2026-10-07）

`java -Xverify:all -cp v8 p.Check` 运行冻结 class（JDK 23.0.1）的输出：

```text
true
```

CI 守卫见 [`tests/fixture_behavior_guards.rs`](../../../fixture_behavior_guards.rs)：呈现腿在默认套件
（`p.package-info` 的整文本是 `@java.lang.Deprecated` 后接 `package p;`），行为腿标 `#[ignore]`
（`cargo test --test fixture_behavior_guards --locked -- --ignored`）并重编两份渲染文本后对照运行输出。
取证与分类见 [`openspec/changes/recover-fixture-behavior-guard-coverage/results/`](../../../../openspec/changes/recover-fixture-behavior-guard-coverage/results/README.md)。
