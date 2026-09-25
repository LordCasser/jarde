# 类声明注解使用：Java 8 完整类对照

`Tagged.java` 使用无元素的 `@Deprecated`；`RetentionTagged.java` 是合法 `@interface`，其类声明带有值为 `RUNTIME` 的 `@Retention`。`TagRunner.java` 直接反射这两项，并调用一条普通方法作为非注解对照。全部源码由 `javac 23.0.1 --release 8 -g:none` 编译。`run_audit.py` 使用冻结的 Jarde CLI（SHA-256 `7f9dd55bb020d0308504a9484ad44b157b271f08b1b96c667e538f935c6a38d5`）重新生成原始、JADX 1.5.6、Jarde 三套**完整**类源码并分别编译、在 `java -Xverify:all` 下执行。脚本只用冻结 CLI，不调用 Cargo。运行方式：

```sh
python3 openspec/evidence/java-syntax-2026-09-22/class-annotation-uses/run_audit.py
```

固定结果（`generated/summary.json` 保存源码/class 哈希和实际输出）：

| 输入/输出 | `@Deprecated` 反射 | 普通方法 | `@Retention` 反射 |
| --- | --- | --- | --- |
| 原始 class | `true` | `3` | `@java.lang.annotation.Retention(RUNTIME)` |
| JADX 完整源码 | `true` | `3` | `@java.lang.annotation.Retention(RUNTIME)` |
| Jarde 完整源码 | `false` | `3` | `null` |

三套 class 均通过 Java 8 源码编译和 JVM 全校验，因此这是**可编译而静默失真**，不是方法体或注解类型头的编译失败。`javap-Tagged.log` 和 `javap-RetentionTagged.log` 显示真实的 `RuntimeVisibleAnnotations`，JADX 源码保留两个注解使用，Jarde 源码都省略。Jarde 的 `RetentionTagged` 头已正确写成 `@interface`；缺口在类级注解内容读取和源码装配。`Deprecated` 的其它 class 属性不能替代 `RuntimeVisibleAnnotations` 来推断 `@Deprecated`，也不能从 `@interface` 头反推 `@Retention`。

`generated/` 留有原 class、完整 `javap -v -c -p`、JADX 源码、Jarde 全文和 `--evidence all` JSON、编译/运行日志及退出码。主代理把整个目录复制到 `/tmp/jarde-class-ann-use-root-66_fskq3/case` 后独立重放：`summary.json`、两份 Jarde 类源码和三套运行输出逐字节相同；`javap` 日志的 Classfile 绝对路径随复制目录改变，其余关键事实与 class 哈希相同。未把此缺口混入当前浮点默认值实现。
