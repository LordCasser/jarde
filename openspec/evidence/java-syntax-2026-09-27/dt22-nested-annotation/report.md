# DT-22：注解默认值与嵌套声明是两个问题

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestAnnotations` 直接断言 `float value() default 1.1f;`，`TestAnnotations2` 直接断言成员 `@interface A`。前者的物理 `AnnotationDefault` 目前已由 Jarde 读取、按原始 IEEE 位写出可重编的十六进制 float 字面量；后者的词法 owner 没有恢复。

`replay.py` 用 `javac --release 8` 编译本目录的四份源码，取完整 class 集合做 JADX/Jarde 对照。`Holder.A` 是正例，顶级 `TopDefault` 验证元素默认值本身，顶级 `Dollar$A` 则检查 `$` 字符不能单独推导成员关系。输入和 class SHA 在 `results.json`；`original-javap.txt` 保留 classfile 的 `InnerClasses` 与 `AnnotationDefault`。脚本要求固定 JADX checkout，使用 `JARDE_CLI` 指向已构建 CLI，所有输出在临时目录中编译，退出后清理。

三方完整源码的 `Runner` 都使用 `Holder.A.class`，反射输出预期为：

```text
nested=1.1
top=1.1:3
dollar=7
```

原始源码与 JADX 输出都以 Java 8 重编，`java -Xverify:all` 运行输出逐字一致。Jarde 的根级源码集合（`Holder.java`、`TopDefault.java`、`Dollar$A.java`）自身也能重编，但加原 `Runner` 后在 `Holder.A` 报唯一 `cannot find symbol: class A`。独立物理 child `Holder$A` 报告有正确的 `@interface` 声明和 `default 0x1.19999ap0f`，顶级 `TopDefault` 也保留两个 default；所以不能把差距归给浮点值解析。Jarde 当前的 `Holder` 根报告缺成员注解声明，`Holder$A` 被当作顶级源类型。

JADX 在 `ClassGen.addInnerClass` 递归写成员类型，在 `AnnotationGen.getAnnotationDefaultValue` 读取元素默认值。Jarde 已有 `member_inner::child_relation_agrees` 对双方 `InnerClasses` 做精确核对；具名成员家族的 `scan_family_root` 刻意排除 `ACC_INTERFACE|ACC_ANNOTATION`，DT-13 的 `scan_nested_enum_root` 只准 `ACC_ENUM`，两条路径都不能直接把注解当普通类或 enum 投影。DT-13 的结构化根级 writer 和物理 child 报告/derived anchor 形态可以复用，但需要把“词法成员声明”与 enum 常量组特有证明分开，避免第三套并行装配机制。

本次只定位一个**直接、唯一、非泛型、无额外成员的 Java 8 成员注解类型**。后续实现必须保持原子拒绝：任一双方关系行、选定定义、child 成员表、annotation kind/default 属性或预算不完整，根报告不写半份成员声明。嵌套注解的使用值、参数/字段注解和多 child 布局仍属 DT-23/24 或后续单独形态，不能据此宣称整个 DT-22 已追平。

重放命令：

```sh
JARDE_CLI=/absolute/path/to/jarde-cli python3 openspec/evidence/java-syntax-2026-09-27/dt22-nested-annotation/replay.py
```
